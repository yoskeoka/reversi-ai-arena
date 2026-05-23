use std::collections::BTreeMap;

use aiarena_protocol::gamemaster::{
    self, ActionDecision, DecisionMode, FailureReason, MatchResult, MatchStatus, Placement,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::{BOARD_SIZE, MatchState};
use crate::{
    Action, GAME_ID, GAME_VERSION, InitState, LegalActionHint, PlayerColor, RULESET_VERSION,
    ReversiActionStatus, ReversiDecisionStep, ReversiGameMasterInitState,
};

pub const DECISION_DEADLINE_MS: u64 = 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotState {
    pub board: [[crate::Disc; BOARD_SIZE]; BOARD_SIZE],
    pub current_player: Option<PlayerColor>,
    pub turn: i32,
    pub consecutive_passes: u8,
    pub completed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner_by_forfeit: Option<PlayerColor>,
}

impl From<&MatchState> for SnapshotState {
    fn from(value: &MatchState) -> Self {
        Self {
            board: value.board,
            current_player: value.current_player,
            turn: value.turn,
            consecutive_passes: value.consecutive_passes,
            completed: value.completed,
            winner_by_forfeit: value.winner_by_forfeit,
        }
    }
}

impl From<SnapshotState> for MatchState {
    fn from(value: SnapshotState) -> Self {
        Self {
            board: value.board,
            current_player: value.current_player,
            turn: value.turn,
            consecutive_passes: value.consecutive_passes,
            completed: value.completed,
            winner_by_forfeit: value.winner_by_forfeit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReversiGameMaster {
    pub match_id: String,
    pub players: Vec<gamemaster::Player>,
    pub player_colors: BTreeMap<String, PlayerColor>,
    pub state: MatchState,
    pub last_action_statuses: BTreeMap<String, ReversiActionStatus>,
    pub stderr_bytes: BTreeMap<String, i32>,
}

impl ReversiGameMaster {
    pub fn initialize(
        match_id: impl Into<String>,
        players: Vec<gamemaster::Player>,
        resume_snapshot: Option<SnapshotState>,
    ) -> Result<(Self, ReversiGameMasterInitState), String> {
        if players.len() != 2 {
            return Err("reversi requires exactly two players".to_string());
        }
        let mut player_colors = BTreeMap::new();
        player_colors.insert(players[0].player_id.clone(), PlayerColor::Black);
        player_colors.insert(players[1].player_id.clone(), PlayerColor::White);

        let state = resume_snapshot
            .map(MatchState::from)
            .unwrap_or_else(MatchState::new_standard);
        let mut per_player = BTreeMap::new();
        for player in &players {
            let color = *player_colors
                .get(&player.player_id)
                .ok_or_else(|| format!("missing color for {}", player.player_id))?;
            per_player.insert(
                player.player_id.clone(),
                InitState {
                    you_are: color,
                    board_size: BOARD_SIZE as u8,
                    opening: MatchState::opening(),
                    ruleset: RULESET_VERSION.to_string(),
                },
            );
        }

        let mut last_action_statuses = BTreeMap::new();
        let mut stderr_bytes = BTreeMap::new();
        for player in &players {
            last_action_statuses.insert(
                player.player_id.clone(),
                ReversiActionStatus {
                    player_id: player.player_id.clone(),
                    action_status: ActionDecision::NoAction,
                    failure_reason: None,
                    action: None,
                },
            );
            stderr_bytes.insert(player.player_id.clone(), 0);
        }

        Ok((
            Self {
                match_id: match_id.into(),
                players,
                player_colors,
                state,
                last_action_statuses,
                stderr_bytes,
            },
            ReversiGameMasterInitState { per_player },
        ))
    }

    pub fn next_decision_step(&self) -> Option<ReversiDecisionStep> {
        let current = self.state.current_player?;
        let player_id = self.player_id_for_color(current)?;
        let visible_state = self.state.visible_state_for(current);
        let request = gamemaster::DecisionRequest {
            player_id,
            visible_state: Some(visible_state.clone()),
            legal_action_hint: Some(LegalActionHint {
                legal_actions: visible_state.legal_actions.clone(),
            }),
            deadline_ms: DECISION_DEADLINE_MS,
        };
        Some(ReversiDecisionStep {
            turn: self.state.turn,
            mode: DecisionMode::Sequential,
            requests: vec![request],
        })
    }

    pub fn normalize_action(
        &self,
        request: &gamemaster::DecisionRequest<crate::VisibleState, crate::LegalActionHint>,
        status: &gamemaster::ActionStatus<Value>,
    ) -> ReversiActionStatus {
        let mut normalized = ReversiActionStatus {
            player_id: status.player_id.clone(),
            action_status: status.action_status,
            failure_reason: status.failure_reason,
            action: None,
        };
        if normalized.player_id.is_empty() {
            normalized.player_id = request.player_id.clone();
        }
        if normalized.action_status != ActionDecision::Accepted {
            return normalized;
        }
        let Some(raw_payload) = status.action.clone() else {
            normalized.action_status = ActionDecision::NoAction;
            normalized.failure_reason = Some(FailureReason::InvalidIllegalAction);
            return normalized;
        };
        let action = match decode_turn_action(raw_payload) {
            Ok(action) => action,
            Err(_) => {
                normalized.action_status = ActionDecision::NoAction;
                normalized.failure_reason = Some(FailureReason::InvalidProtocolMalformed);
                normalized.action = None;
                return normalized;
            }
        };
        let Some(color) = self.player_colors.get(&request.player_id).copied() else {
            normalized.action_status = ActionDecision::NoAction;
            normalized.failure_reason = Some(FailureReason::InvalidProtocolMalformed);
            normalized.action = None;
            return normalized;
        };
        if !self.state.is_legal_action(color, &action) {
            normalized.action_status = ActionDecision::NoAction;
            normalized.failure_reason = Some(FailureReason::InvalidIllegalAction);
            normalized.action = None;
        } else {
            normalized.action = Some(action);
        }
        normalized
    }

    pub fn apply_decision_results(
        &mut self,
        statuses: &[ReversiActionStatus],
    ) -> Result<(), String> {
        let current = self
            .state
            .current_player
            .ok_or_else(|| "match is already completed".to_string())?;
        let current_player_id = self
            .player_id_for_color(current)
            .ok_or_else(|| "missing player for current color".to_string())?;
        let status = statuses
            .iter()
            .find(|candidate| candidate.player_id == current_player_id)
            .ok_or_else(|| format!("missing action status for {}", current_player_id))?
            .clone();
        self.last_action_statuses
            .insert(current_player_id.clone(), status.clone());

        if status.action_status != ActionDecision::Accepted {
            self.state.forfeit(current);
            return Ok(());
        }

        let action = status
            .action
            .ok_or_else(|| "accepted action is missing payload".to_string())?;
        if self.state.apply_valid_action(&action).is_err() {
            self.state.forfeit(current);
        }
        Ok(())
    }

    pub fn current_snapshot(
        &self,
    ) -> gamemaster::Snapshot<SnapshotState, crate::VisibleState, crate::Action> {
        let mut per_player = BTreeMap::new();
        for player in &self.players {
            let color = self.player_colors[&player.player_id];
            per_player.insert(
                player.player_id.clone(),
                gamemaster::PlayerSnapshot {
                    visible_state: Some(self.state.visible_state_for(color)),
                    last_action_status: self.last_action_statuses[&player.player_id].clone(),
                    stderr_bytes: *self.stderr_bytes.get(&player.player_id).unwrap_or(&0),
                },
            );
        }
        gamemaster::Snapshot {
            match_id: self.match_id.clone(),
            game_id: Some(GAME_ID.to_string()),
            game_version: Some(GAME_VERSION.to_string()),
            ruleset_version: Some(RULESET_VERSION.to_string()),
            turn: self.state.turn,
            status: self.status(),
            game_state: Some(SnapshotState::from(&self.state)),
            per_player,
        }
    }

    pub fn current_exported_snapshot(&self) -> crate::ReversiExportedSnapshot {
        let mut players = Vec::new();
        for player in &self.players {
            players.push(gamemaster::ExportedPlayerSnapshot {
                player_id: player.player_id.clone(),
                last_action_status: self.last_action_statuses[&player.player_id].clone(),
            });
        }
        crate::ReversiExportedSnapshot {
            match_id: self.match_id.clone(),
            game_id: Some(GAME_ID.to_string()),
            game_version: Some(GAME_VERSION.to_string()),
            ruleset_version: Some(RULESET_VERSION.to_string()),
            turn: self.state.turn,
            status: self.status(),
            public_state: Some(self.state.public_state()),
            players,
        }
    }

    pub fn current_result(&self) -> MatchResult {
        let summary = self.state.summary();
        let placements = if summary.winners.len() == 2 {
            vec![
                Placement {
                    player_id: self.players[0].player_id.clone(),
                    place: 1,
                },
                Placement {
                    player_id: self.players[1].player_id.clone(),
                    place: 1,
                },
            ]
        } else {
            let winner = summary.winners[0];
            self.players
                .iter()
                .map(|player| Placement {
                    player_id: player.player_id.clone(),
                    place: if self.player_colors[&player.player_id] == winner {
                        1
                    } else {
                        2
                    },
                })
                .collect()
        };
        MatchResult { placements }
    }

    pub fn status(&self) -> MatchStatus {
        if self.state.completed {
            MatchStatus::Completed
        } else {
            MatchStatus::Running
        }
    }

    fn player_id_for_color(&self, color: PlayerColor) -> Option<String> {
        self.player_colors
            .iter()
            .find_map(|(player_id, candidate)| (*candidate == color).then(|| player_id.clone()))
    }
}

fn decode_turn_action(raw_payload: Value) -> Result<Action, serde_json::Error> {
    if raw_payload.get("kind").is_some() {
        return serde_json::from_value(raw_payload);
    }
    #[derive(Deserialize)]
    struct WrappedTurnResult {
        action: Action,
    }
    serde_json::from_value::<WrappedTurnResult>(raw_payload).map(|wrapped| wrapped.action)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn players() -> Vec<gamemaster::Player> {
        vec![
            gamemaster::Player {
                player_id: "p1".to_string(),
                ai_id: "a1".to_string(),
            },
            gamemaster::Player {
                player_id: "p2".to_string(),
                ai_id: "a2".to_string(),
            },
        ]
    }

    #[test]
    fn game_master_requests_black_first() {
        let (master, _) = ReversiGameMaster::initialize("m1", players(), None).expect("init");
        let step = master.next_decision_step().expect("step");
        assert_eq!(step.requests.len(), 1);
        assert_eq!(step.requests[0].player_id, "p1");
    }

    #[test]
    fn normalize_turn_rejects_illegal_pass() {
        let (master, _) = ReversiGameMaster::initialize("m1", players(), None).expect("init");
        let step = master.next_decision_step().expect("step");
        let normalized = master.normalize_action(
            &step.requests[0],
            &gamemaster::ActionStatus {
                player_id: "p1".to_string(),
                action_status: ActionDecision::Accepted,
                failure_reason: None,
                action: Some(serde_json::json!({
                    "action": {
                        "kind": "pass"
                    }
                })),
            },
        );
        assert_eq!(normalized.action_status, ActionDecision::NoAction);
        assert_eq!(
            normalized.failure_reason,
            Some(FailureReason::InvalidIllegalAction)
        );
    }
}

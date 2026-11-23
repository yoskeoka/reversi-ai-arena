use std::collections::BTreeMap;

use aiarena_protocol::gamemaster::{
    self, ActionDecision, DecisionMode, FailureReason, MatchResult, MatchStatus, Placement,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::{BOARD_SIZE, MatchState};
use crate::{
    Action, GAME_ID, GAME_VERSION, InitState, LegalActionHint, PlayerColor, ReversiActionStatus,
    ReversiDecisionStep, ReversiGameMasterInitState,
};

use crate::opening::{OpeningMoves, Ruleset, initial_state};

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
    #[serde(default)]
    pub public_replay_turns: Vec<crate::PublicReplayTurn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening_moves: Option<OpeningMoves>,
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
            public_replay_turns: Vec::new(),
            opening_moves: None,
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
    ruleset: Ruleset,
    opening_moves: Option<OpeningMoves>,
    pub last_action_statuses: BTreeMap<String, ReversiActionStatus>,
    pub stderr_bytes: BTreeMap<String, i32>,
    pub public_replay_turns: Vec<crate::PublicReplayTurn>,
}

impl ReversiGameMaster {
    pub fn initialize(
        match_id: impl Into<String>,
        players: Vec<gamemaster::Player>,
        resume_snapshot: Option<
            gamemaster::Snapshot<SnapshotState, crate::VisibleState, crate::Action>,
        >,
    ) -> Result<(Self, ReversiGameMasterInitState), String> {
        Self::initialize_for(
            match_id,
            players,
            resume_snapshot,
            Ruleset::Standard,
            || Ok(None),
        )
    }

    pub fn initialize_for(
        match_id: impl Into<String>,
        players: Vec<gamemaster::Player>,
        resume_snapshot: Option<
            gamemaster::Snapshot<SnapshotState, crate::VisibleState, crate::Action>,
        >,
        ruleset: Ruleset,
        select: impl FnOnce() -> Result<Option<OpeningMoves>, String>,
    ) -> Result<(Self, ReversiGameMasterInitState), String> {
        if players.len() != 2 {
            return Err("reversi requires exactly two players".to_string());
        }
        let match_id = match_id.into();
        let mut player_colors = BTreeMap::new();
        player_colors.insert(players[0].player_id.clone(), PlayerColor::Black);
        player_colors.insert(players[1].player_id.clone(), PlayerColor::White);

        let saved = match resume_snapshot.as_ref() {
            Some(snapshot) => {
                if snapshot
                    .ruleset_version
                    .as_deref()
                    .unwrap_or(Ruleset::Standard.id())
                    != ruleset.id()
                {
                    return Err("snapshot ruleset does not match startup ruleset".to_string());
                }
                match snapshot.game_state.clone() {
                    Some(state) => Some(state),
                    None if ruleset.is_xot() => {
                        return Err("XOT snapshot is missing game state".to_string());
                    }
                    None => None,
                }
            }
            None => None,
        };
        let opening_moves = match &saved {
            Some(snapshot) => snapshot.opening_moves,
            None => select()?,
        };
        if ruleset.is_xot() != opening_moves.is_some() {
            return Err("opening sequence does not match ruleset".to_string());
        }
        let initial = initial_state(opening_moves.as_ref())?;
        let (state, public_replay_turns) = match saved {
            Some(mut snapshot) => {
                let turns = std::mem::take(&mut snapshot.public_replay_turns);
                let state = MatchState::from(snapshot);
                if ruleset.is_xot() {
                    let mut replayed = initial.clone();
                    for turn in &turns {
                        if replayed.current_player != Some(turn.color)
                            || player_colors.get(&turn.player_id) != Some(&turn.color)
                        {
                            return Err("XOT snapshot contains an out-of-turn action".into());
                        }
                        replayed.apply_valid_action(&turn.action)?;
                    }
                    if let Some(winner) = state.winner_by_forfeit {
                        let loser = winner.opponent();
                        if replayed.current_player != Some(loser) {
                            return Err("XOT snapshot contains an invalid forfeit".into());
                        }
                        replayed.forfeit(loser);
                    }
                    if replayed != state {
                        return Err("XOT snapshot state disagrees with its transcript".into());
                    }
                }
                (state, turns)
            }
            None => (initial.clone(), Vec::new()),
        };
        let placements = initial.placements();
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
                    opening: placements.clone(),
                    ruleset: ruleset.id().to_string(),
                    initial_turn: ruleset.is_xot().then_some(initial.turn as u32),
                },
            );
        }

        let mut last_action_statuses = BTreeMap::new();
        let mut stderr_bytes = BTreeMap::new();
        for player in &players {
            if let Some(snapshot) = resume_snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.per_player.get(&player.player_id))
            {
                last_action_statuses.insert(
                    player.player_id.clone(),
                    snapshot.last_action_status.clone(),
                );
                stderr_bytes.insert(player.player_id.clone(), snapshot.stderr_bytes);
                continue;
            }
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
                match_id,
                players,
                player_colors,
                state,
                ruleset,
                opening_moves,
                last_action_statuses,
                stderr_bytes,
                public_replay_turns,
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
        } else {
            self.public_replay_turns.push(crate::PublicReplayTurn {
                player_id: current_player_id,
                color: current,
                action,
            });
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
            ruleset_version: Some(self.ruleset.id().to_string()),
            turn: self.state.turn,
            status: self.status(),
            game_state: Some(SnapshotState {
                public_replay_turns: self.public_replay_turns.clone(),
                opening_moves: self.opening_moves,
                ..SnapshotState::from(&self.state)
            }),
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
            ruleset_version: Some(self.ruleset.id().to_string()),
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

    pub fn current_public_replay(&self) -> Result<gamemaster::PublicReplay, String> {
        if !self.state.completed {
            return Err("public replay is available only after completion".to_string());
        }
        Ok(gamemaster::PublicReplay {
            format: "reversi/replay".to_string(),
            version: "1".to_string(),
            payload: serde_json::to_value(crate::PublicReplayPayload {
                board_size: BOARD_SIZE as u8,
                opening: MatchState::opening(),
                ruleset: self.ruleset.id().to_string(),
                opening_moves: self.opening_moves,
                turns: self.public_replay_turns.clone(),
            })
            .map_err(|err| format!("encode public replay: {err}"))?,
        })
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

    #[test]
    fn completed_match_exposes_versioned_public_replay() {
        let (mut master, _) = ReversiGameMaster::initialize("m1", players(), None).expect("init");
        master
            .apply_decision_results(&[ReversiActionStatus {
                player_id: "p1".to_string(),
                action_status: ActionDecision::NoAction,
                failure_reason: Some(FailureReason::InvalidTimeout),
                action: None,
            }])
            .expect("forfeit completes match");
        let replay = master.current_public_replay().expect("public replay");
        assert_eq!(replay.format, "reversi/replay");
        assert_eq!(replay.version, "1");
        assert_eq!(replay.payload["turns"], serde_json::json!([]));
    }

    #[test]
    fn resume_restores_public_replay_turns() {
        let (mut master, _) = ReversiGameMaster::initialize("m1", players(), None).expect("init");
        master
            .apply_decision_results(&[ReversiActionStatus {
                player_id: "p1".to_string(),
                action_status: ActionDecision::Accepted,
                failure_reason: None,
                action: Some(crate::Action {
                    kind: crate::ActionKind::Place,
                    position: Some(crate::Position { row: 2, col: 3 }),
                }),
            }])
            .expect("apply accepted action");
        let snapshot = master.current_snapshot();
        let (resumed, _) =
            ReversiGameMaster::initialize("m1", players(), Some(snapshot)).expect("resume");
        assert_eq!(resumed.public_replay_turns.len(), 1);
    }

    #[test]
    fn xot_first_request_and_first_turn_forfeit_share_the_opening() {
        let moves = crate::opening::select_opening(b"f5d6c4d3c2b3b4b5\n", b"test").unwrap();
        for ruleset in [Ruleset::XotLarge, Ruleset::XotSmall] {
            let (mut master, initialized) =
                ReversiGameMaster::initialize_for("xot", players(), None, ruleset, || {
                    Ok(Some(moves))
                })
                .unwrap();
            let step = master.next_decision_step().unwrap();
            assert_eq!(step.turn, 9);
            assert_eq!(step.requests[0].player_id, "p1");
            let visible = step.requests[0].visible_state.as_ref().unwrap();
            assert_eq!(visible.board, master.state.public_state().board);
            for player in initialized.per_player.values() {
                assert_eq!(player.opening.len(), 12);
                assert_eq!(player.opening, master.state.placements());
                assert_eq!(player.initial_turn, Some(9));
                assert_eq!(player.ruleset, ruleset.id());
            }
            master
                .apply_decision_results(&[ReversiActionStatus {
                    player_id: "p1".into(),
                    action_status: ActionDecision::NoAction,
                    failure_reason: Some(FailureReason::InvalidTimeout),
                    action: None,
                }])
                .unwrap();
            let replay = master.current_public_replay().unwrap();
            assert_eq!(replay.payload["opening_moves"], serde_json::json!(moves));
            assert_eq!(replay.payload["turns"], serde_json::json!([]));
            assert_eq!(master.current_exported_snapshot().turn, 9);
            assert_eq!(master.state.summary().winners, [PlayerColor::White]);
        }
    }

    #[test]
    fn xot_resume_preserves_accepted_turns_and_never_selects_again() {
        let ruleset = Ruleset::XotSmall;
        let script = include_str!("../../../testdata/reversi/scripted-games/end-with-1-empty-cell-forced-pass-for-both.txt").trim();
        let first_eight = format!("{}\n", &script[..16]);
        let moves = crate::opening::select_opening(first_eight.as_bytes(), b"test").unwrap();
        let (mut master, initialized) =
            ReversiGameMaster::initialize_for("xot", players(), None, ruleset, || Ok(Some(moves)))
                .unwrap();
        let mut actions = &script[16..];
        while let Some(step) = master.next_decision_step() {
            let (action, remaining) = if let Some(rest) = actions.strip_prefix("pass") {
                (
                    Action {
                        kind: crate::ActionKind::Pass,
                        position: None,
                    },
                    rest,
                )
            } else {
                let (coordinate, rest) = actions.split_at(2);
                (
                    Action {
                        kind: crate::ActionKind::Place,
                        position: Some(crate::Position {
                            row: coordinate.as_bytes()[1] - b'1',
                            col: coordinate.as_bytes()[0] - b'a',
                        }),
                    },
                    rest,
                )
            };
            actions = remaining;
            master
                .apply_decision_results(&[ReversiActionStatus {
                    player_id: step.requests[0].player_id.clone(),
                    action_status: ActionDecision::Accepted,
                    failure_reason: None,
                    action: Some(action),
                }])
                .unwrap();
        }
        assert!(actions.is_empty());
        assert_eq!(master.state.consecutive_passes, 2);
        assert_eq!(master.state.winner_by_forfeit, None);
        let snapshot = master.current_snapshot();
        let (resumed, resumed_init) = ReversiGameMaster::initialize_for(
            "xot",
            players(),
            Some(snapshot.clone()),
            ruleset,
            || panic!("must not select another opening"),
        )
        .unwrap();
        assert_eq!(resumed_init, initialized);
        assert_eq!(resumed.state, master.state);
        let replay = resumed.current_public_replay().unwrap();
        assert_eq!(
            replay.payload,
            master.current_public_replay().unwrap().payload
        );
        assert!(
            resumed
                .public_replay_turns
                .iter()
                .any(|turn| turn.action.kind == crate::ActionKind::Pass)
        );
        assert!(
            ReversiGameMaster::initialize_for(
                "xot",
                players(),
                Some(snapshot),
                Ruleset::XotLarge,
                || panic!("must reject the mismatched ruleset")
            )
            .is_err()
        );
    }
}

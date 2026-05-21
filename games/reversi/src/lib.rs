//! Reversi game-surface DTOs shared by the game master and player surfaces.

use aiarena_protocol::{GameMetadata, gamemaster, player};
use serde::{Deserialize, Serialize};

pub const GAME_ID: &str = "reversi";
pub const GAME_VERSION: &str = "1.0.0";
pub const RULESET_VERSION: &str = "standard";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disc {
    Empty,
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerColor {
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscPlacement {
    pub position: Position,
    pub disc: Disc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreSummary {
    pub black: u8,
    pub white: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitState {
    pub you_are: PlayerColor,
    pub board_size: u8,
    pub opening: Vec<DiscPlacement>,
    pub ruleset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisibleState {
    pub turn: u32,
    pub board: Vec<Vec<Disc>>,
    pub current_player: Option<PlayerColor>,
    pub legal_actions: Vec<Position>,
    pub scores: ScoreSummary,
    pub pass_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegalActionHint {
    pub legal_actions: Vec<Position>,
    pub pass_allowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Place,
    Pass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub kind: ActionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicState {
    pub turn: u32,
    pub board: Vec<Vec<Disc>>,
    pub current_player: Option<PlayerColor>,
    pub scores: ScoreSummary,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSummary {
    pub black: u8,
    pub white: u8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub winners: Vec<PlayerColor>,
}

pub type ReversiGameMasterInitState = gamemaster::InitState<InitState>;
pub type ReversiDecisionStep = gamemaster::DecisionStep<VisibleState, LegalActionHint>;
pub type ReversiActionStatus = gamemaster::ActionStatus<Action>;
pub type ReversiExportedSnapshot = gamemaster::ExportedSnapshot<PublicState, Action>;
pub type ReversiPlayerInitParams = player::InitParams<InitState>;
pub type ReversiPlayerTurnParams = player::TurnParams<VisibleState, LegalActionHint>;
pub type ReversiPlayerTurnResult = player::TurnResult<Action>;
pub type ReversiPlayerGameOverParams = player::GameOverParams<VisibleState, GameSummary>;

pub fn game_metadata() -> GameMetadata {
    GameMetadata {
        game_id: GAME_ID.to_string(),
        game_version: GAME_VERSION.to_string(),
        ruleset_version: RULESET_VERSION.to_string(),
    }
}

pub fn sample_visible_state() -> VisibleState {
    VisibleState {
        turn: 1,
        board: vec![
            vec![Disc::Empty, Disc::Empty, Disc::Empty, Disc::Empty],
            vec![Disc::Empty, Disc::White, Disc::Black, Disc::Empty],
            vec![Disc::Empty, Disc::Black, Disc::White, Disc::Empty],
            vec![Disc::Empty, Disc::Empty, Disc::Empty, Disc::Empty],
        ],
        current_player: Some(PlayerColor::Black),
        legal_actions: vec![Position { row: 0, col: 1 }, Position { row: 1, col: 0 }],
        scores: ScoreSummary { black: 2, white: 2 },
        pass_required: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_turn_payload_round_trips_through_protocol_shape() {
        let params = ReversiPlayerTurnParams {
            turn: 3,
            visible_state: sample_visible_state(),
            legal_action_hint: LegalActionHint {
                legal_actions: vec![Position { row: 2, col: 3 }],
                pass_allowed: false,
            },
            deadline_ms: 1_500,
        };

        let json = serde_json::to_string(&params).expect("serialize");
        let decoded: ReversiPlayerTurnParams = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.turn, 3);
        assert_eq!(decoded.visible_state.scores.black, 2);
        assert_eq!(decoded.legal_action_hint.legal_actions.len(), 1);
    }

    #[test]
    fn game_master_action_status_uses_reversi_action_payload() {
        let status = ReversiActionStatus {
            player_id: "p1".to_string(),
            action_status: gamemaster::ActionDecision::Accepted,
            failure_reason: None,
            action: Some(Action {
                kind: ActionKind::Place,
                position: Some(Position { row: 2, col: 3 }),
            }),
        };

        let json = serde_json::to_value(status).expect("serialize");
        assert_eq!(json["action"]["kind"], "place");
        assert_eq!(json["action"]["position"]["row"], 2);
    }
}

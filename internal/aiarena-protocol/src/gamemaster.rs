use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::common::GameMetadata;

pub const METHOD_METADATA: &str = "metadata";
pub const METHOD_INITIALIZE_MATCH: &str = "initialize_match";
pub const METHOD_NEXT_DECISION_STEP: &str = "next_decision_step";
pub const METHOD_NORMALIZE_ACTION: &str = "normalize_action";
pub const METHOD_APPLY_DECISION_RESULTS: &str = "apply_decision_results";
pub const METHOD_CURRENT_SNAPSHOT: &str = "current_snapshot";
pub const METHOD_CURRENT_EXPORTED_SNAPSHOT: &str = "current_exported_snapshot";
pub const METHOD_CURRENT_RESULT: &str = "current_result";
pub const METHOD_SHUTDOWN: &str = "shutdown";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub player_id: String,
    pub ai_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionMode {
    Sequential,
    Simultaneous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: serde::de::DeserializeOwned"
))]
pub struct InitState<T = Value> {
    pub per_player: BTreeMap<String, T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "State: Serialize, Hint: Serialize",
    deserialize = "State: serde::de::DeserializeOwned, Hint: serde::de::DeserializeOwned"
))]
pub struct DecisionRequest<State = Value, Hint = Value> {
    pub player_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_state: Option<State>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_action_hint: Option<Hint>,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "State: Serialize, Hint: Serialize",
    deserialize = "State: serde::de::DeserializeOwned, Hint: serde::de::DeserializeOwned"
))]
pub struct DecisionStep<State = Value, Hint = Value> {
    pub turn: i32,
    pub mode: DecisionMode,
    pub requests: Vec<DecisionRequest<State, Hint>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    Starting,
    Initializing,
    Running,
    Finishing,
    Completed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionDecision {
    Accepted,
    NoAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureReason {
    InvalidTimeout,
    InvalidProtocolMalformed,
    InvalidProtocolMismatchedId,
    InvalidProtocolLateResponse,
    InvalidIllegalAction,
    RuntimeStopped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Action: Serialize",
    deserialize = "Action: serde::de::DeserializeOwned"
))]
pub struct ActionStatus<Action = Value> {
    pub player_id: String,
    pub action_status: ActionDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<FailureReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placement {
    pub player_id: String,
    pub place: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResult {
    pub placements: Vec<Placement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "State: Serialize, Action: Serialize",
    deserialize = "State: serde::de::DeserializeOwned, Action: serde::de::DeserializeOwned"
))]
pub struct PlayerSnapshot<State = Value, Action = Value> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_state: Option<State>,
    pub last_action_status: ActionStatus<Action>,
    pub stderr_bytes: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "GameState: Serialize, State: Serialize, Action: Serialize",
    deserialize = "GameState: serde::de::DeserializeOwned, State: serde::de::DeserializeOwned, Action: serde::de::DeserializeOwned"
))]
pub struct Snapshot<GameState = Value, State = Value, Action = Value> {
    pub match_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruleset_version: Option<String>,
    pub turn: i32,
    pub status: MatchStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_state: Option<GameState>,
    pub per_player: BTreeMap<String, PlayerSnapshot<State, Action>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Action: Serialize",
    deserialize = "Action: serde::de::DeserializeOwned"
))]
pub struct ExportedPlayerSnapshot<Action = Value> {
    pub player_id: String,
    pub last_action_status: ActionStatus<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "PublicState: Serialize, Action: Serialize",
    deserialize = "PublicState: serde::de::DeserializeOwned, Action: serde::de::DeserializeOwned"
))]
pub struct ExportedSnapshot<PublicState = Value, Action = Value> {
    pub match_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruleset_version: Option<String>,
    pub turn: i32,
    pub status: MatchStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_state: Option<PublicState>,
    pub players: Vec<ExportedPlayerSnapshot<Action>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "GameState: Serialize, State: Serialize, Action: Serialize",
    deserialize = "GameState: serde::de::DeserializeOwned, State: serde::de::DeserializeOwned, Action: serde::de::DeserializeOwned"
))]
pub struct InitializeMatchParams<GameState = Value, State = Value, Action = Value> {
    pub players: Vec<Player>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rng_seed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_snapshot: Option<Snapshot<GameState, State, Action>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: serde::de::DeserializeOwned"
))]
pub struct InitializeMatchResult<T = Value> {
    pub init_state: InitState<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "State: Serialize, Hint: Serialize, Action: Serialize",
    deserialize = "State: serde::de::DeserializeOwned, Hint: serde::de::DeserializeOwned, Action: serde::de::DeserializeOwned"
))]
pub struct NormalizeActionParams<State = Value, Hint = Value, Action = Value> {
    pub request: DecisionRequest<State, Hint>,
    pub action_status: ActionStatus<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "State: Serialize, Hint: Serialize, Action: Serialize",
    deserialize = "State: serde::de::DeserializeOwned, Hint: serde::de::DeserializeOwned, Action: serde::de::DeserializeOwned"
))]
pub struct ApplyDecisionResultsParams<State = Value, Hint = Value, Action = Value> {
    pub step: DecisionStep<State, Hint>,
    pub action_statuses: Vec<ActionStatus<Action>>,
}

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::common::{GameMetadata, Transport};

pub const METHOD_INIT: &str = "init";
pub const METHOD_TURN: &str = "turn";
pub const METHOD_GAME_OVER: &str = "game_over";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "State: Serialize",
    deserialize = "State: serde::de::DeserializeOwned"
))]
pub struct InitParams<State = Value> {
    pub match_id: String,
    pub player_id: String,
    pub game_id: String,
    pub game_version: String,
    pub ruleset_version: String,
    pub deadline_ms: u64,
    pub state: State,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "State: Serialize, Hint: Serialize",
    deserialize = "State: serde::de::DeserializeOwned, Hint: serde::de::DeserializeOwned"
))]
pub struct TurnParams<State = Value, Hint = Value> {
    pub turn: i32,
    pub visible_state: State,
    pub legal_action_hint: Hint,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "State: Serialize, Summary: Serialize",
    deserialize = "State: serde::de::DeserializeOwned, Summary: serde::de::DeserializeOwned"
))]
pub struct GameOverParams<State = Value, Summary = Value> {
    pub match_id: String,
    pub final_visible_state: State,
    pub summary: Summary,
    pub shutdown_after_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitResult {
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Action: Serialize",
    deserialize = "Action: serde::de::DeserializeOwned"
))]
pub struct TurnResult<Action = Value> {
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameOverResult {
    pub ack: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidecarManifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_id: Option<String>,
    pub protocol: SidecarProtocol,
    pub runtime: Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidecarProtocol {
    pub transport: Transport,
    pub game_id: String,
    pub game_version: String,
    pub ruleset_version: String,
}

impl SidecarProtocol {
    pub fn metadata(&self) -> GameMetadata {
        GameMetadata {
            game_id: self.game_id.clone(),
            game_version: self.game_version.clone(),
            ruleset_version: self.ruleset_version.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Runtime {
    LocalSubprocess {
        command: Vec<String>,
    },
    WasmWasi {
        module: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        memory_limit_pages: Option<u32>,
    },
}

impl InitResult {
    pub const fn ready() -> Self {
        Self { ready: true }
    }
}

impl GameOverResult {
    pub const fn ack() -> Self {
        Self { ack: true }
    }
}

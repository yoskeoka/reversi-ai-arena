//! Rust reference-player protocol helpers for the Reversi player surface.

use aiarena_protocol::{
    Request, Response, Transport,
    player::{GameOverResult, InitResult, Runtime, SidecarManifest, SidecarProtocol},
};
use reversi_game::{
    Action, ActionKind, GAME_ID, GAME_VERSION, LegalActionHint, RULESET_VERSION,
    ReversiPlayerGameOverParams, ReversiPlayerInitParams, ReversiPlayerTurnParams,
    ReversiPlayerTurnResult,
};

/// Returns a stable placeholder name for the mainline Rust player surface.
pub fn player_name() -> &'static str {
    "rust-reference"
}

/// Builds the current sidecar manifest DTO for the Rust reference player.
pub fn sidecar_manifest() -> SidecarManifest {
    SidecarManifest {
        ai_id: Some(player_name().to_string()),
        protocol: SidecarProtocol {
            transport: Transport::StdioJsonrpcNdjson,
            game_id: GAME_ID.to_string(),
            game_version: GAME_VERSION.to_string(),
            ruleset_version: RULESET_VERSION.to_string(),
        },
        runtime: Runtime::WasmWasi {
            module: "./reversi-rust-reference-player.wasm".to_string(),
            args: vec!["./reversi-rust-reference-player.wasm".to_string()],
            memory_limit_pages: Some(64),
        },
    }
}

/// Decodes one typed `init` request from the JSON-RPC transport layer.
pub fn decode_init_request(
    request: &Request,
) -> Result<ReversiPlayerInitParams, aiarena_protocol::DecodeError> {
    request.parse_params()
}

/// Decodes one typed `turn` request from the JSON-RPC transport layer.
pub fn decode_turn_request(
    request: &Request,
) -> Result<ReversiPlayerTurnParams, aiarena_protocol::DecodeError> {
    request.parse_params()
}

/// Decodes one typed `game_over` request from the JSON-RPC transport layer.
pub fn decode_game_over_request(
    request: &Request,
) -> Result<ReversiPlayerGameOverParams, aiarena_protocol::DecodeError> {
    request.parse_params()
}

/// Encodes a successful `init` response.
pub fn init_ready_response(id: &str) -> Result<Response, serde_json::Error> {
    Response::success(id, &InitResult::ready())
}

/// Encodes a successful `turn` response with a typed Reversi action payload.
pub fn turn_action_response(id: &str, action: Action) -> Result<Response, serde_json::Error> {
    Response::success(id, &ReversiPlayerTurnResult { action })
}

/// Encodes a successful `game_over` acknowledgement.
pub fn game_over_ack_response(id: &str) -> Result<Response, serde_json::Error> {
    Response::success(id, &GameOverResult::ack())
}

/// Returns a deterministic placeholder action for the current reference player.
pub fn choose_placeholder_action(
    _state: &reversi_game::VisibleState,
    hint: &LegalActionHint,
) -> Action {
    match hint.legal_actions.first().copied() {
        Some(position) => Action {
            kind: ActionKind::Place,
            position: Some(position),
        },
        None => Action {
            kind: ActionKind::Pass,
            position: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiarena_protocol::{
        match_response_id,
        player::{METHOD_GAME_OVER, METHOD_INIT, METHOD_TURN},
    };
    use reversi_game::{InitState, Position};
    use reversi_game::{PlayerColor, ScoreSummary, VisibleState};

    #[test]
    fn manifest_uses_expected_transport_contract() {
        let manifest = sidecar_manifest();
        assert_eq!(manifest.protocol.transport.as_str(), "stdio-jsonrpc-ndjson");
        assert_eq!(manifest.protocol.game_id, "reversi");
    }

    #[test]
    fn typed_turn_request_decodes_from_transport_request() {
        let params = ReversiPlayerTurnParams {
            turn: 7,
            visible_state: VisibleState {
                turn: 7,
                board: vec![vec![reversi_game::Disc::Empty; 2]; 2],
                current_player: Some(PlayerColor::Black),
                legal_actions: vec![Position { row: 0, col: 0 }],
                scores: ScoreSummary { black: 2, white: 2 },
            },
            legal_action_hint: LegalActionHint {
                legal_actions: vec![Position { row: 0, col: 0 }],
            },
            deadline_ms: 500,
        };
        let request = Request::new("turn-7", METHOD_TURN, &params).expect("request");
        let decoded = decode_turn_request(&request).expect("decode");
        assert_eq!(decoded.turn, 7);
        assert_eq!(decoded.legal_action_hint.legal_actions[0].row, 0);
    }

    #[test]
    fn placeholder_action_prefers_first_legal_move() {
        let action = choose_placeholder_action(
            &reversi_game::sample_visible_state(),
            &LegalActionHint {
                legal_actions: vec![Position { row: 2, col: 3 }],
            },
        );
        assert_eq!(action.kind, ActionKind::Place);
        assert_eq!(action.position.expect("position").col, 3);
    }

    #[test]
    fn placeholder_action_passes_only_when_no_legal_action_exists() {
        let action = choose_placeholder_action(
            &reversi_game::sample_visible_state(),
            &LegalActionHint {
                legal_actions: Vec::new(),
            },
        );
        assert_eq!(action.kind, ActionKind::Pass);
        assert_eq!(action.position, None);
    }

    #[test]
    fn init_and_game_over_responses_keep_request_id() {
        let init = init_ready_response("init-1").expect("init");
        match_response_id("init-1", &init).expect("id");
        let game_over = game_over_ack_response("over-1").expect("game_over");
        match_response_id("over-1", &game_over).expect("id");
    }

    #[test]
    fn method_constants_match_public_contract() {
        assert_eq!(METHOD_INIT, "init");
        assert_eq!(METHOD_TURN, "turn");
        assert_eq!(METHOD_GAME_OVER, "game_over");
    }

    #[test]
    fn init_payload_reuses_reversi_dto() {
        let request = Request::new(
            "init-1",
            METHOD_INIT,
            &ReversiPlayerInitParams {
                match_id: "m1".to_string(),
                player_id: "p1".to_string(),
                game_id: GAME_ID.to_string(),
                game_version: GAME_VERSION.to_string(),
                ruleset_version: RULESET_VERSION.to_string(),
                deadline_ms: 1_000,
                state: InitState {
                    you_are: PlayerColor::Black,
                    board_size: 8,
                    opening: Vec::new(),
                    ruleset: RULESET_VERSION.to_string(),
                },
            },
        )
        .expect("request");

        let decoded = decode_init_request(&request).expect("decode");
        assert_eq!(decoded.state.board_size, 8);
    }
}

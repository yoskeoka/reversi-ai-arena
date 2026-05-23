//! Rust reference-player protocol helpers and heuristic move selection.

use aiarena_protocol::{
    Request, Response, Transport,
    player::{GameOverResult, InitResult, Runtime, SidecarManifest, SidecarProtocol},
};
use reversi_game::{
    Action, ActionKind, Disc, GAME_ID, GAME_VERSION, LegalActionHint, PlayerColor, Position,
    RULESET_VERSION, ReversiPlayerGameOverParams, ReversiPlayerInitParams, ReversiPlayerTurnParams,
    ReversiPlayerTurnResult, VisibleState, engine::BOARD_SIZE, engine::MatchState,
};

type BoardCoordinate = (usize, usize);
type CornerAdjacency = (BoardCoordinate, &'static [BoardCoordinate]);

const CORNERS: &[(usize, usize)] = &[(0, 0), (0, 7), (7, 0), (7, 7)];
const CORNER_ADJACENT: &[CornerAdjacency] = &[
    ((0, 0), &[(0, 1), (1, 0), (1, 1)]),
    ((0, 7), &[(0, 6), (1, 7), (1, 6)]),
    ((7, 0), &[(6, 0), (7, 1), (6, 1)]),
    ((7, 7), &[(6, 7), (7, 6), (6, 6)]),
];
const EDGE_START: usize = 1;
const EDGE_END: usize = BOARD_SIZE - 1;

#[derive(Debug, Clone)]
struct BelievedTurn {
    current_player: PlayerColor,
    legal_actions: Vec<Position>,
}

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

/// Chooses a deterministic action from the current visible board and legal hints.
pub fn choose_action(state: &VisibleState, hint: &LegalActionHint) -> Action {
    let believed_turn = infer_believed_turn(state, hint);
    if believed_turn.legal_actions.is_empty() {
        return Action {
            kind: ActionKind::Pass,
            position: None,
        };
    }

    let search_depth: u8 = if count_empty_cells(&state.board) <= 12 {
        3
    } else {
        2
    };

    let mut ordered_actions = believed_turn.legal_actions.clone();
    ordered_actions.sort_by_key(|position| (position.row, position.col));

    let mut best_position = ordered_actions[0];
    let mut best_score = i32::MIN;
    for position in ordered_actions {
        let candidate_state = simulate_action(state, believed_turn.current_player, position);
        let score = minimax(
            &candidate_state,
            search_depth.saturating_sub(1),
            believed_turn.current_player,
        );
        if score > best_score
            || (score == best_score
                && (position.row, position.col) < (best_position.row, best_position.col))
        {
            best_score = score;
            best_position = position;
        }
    }

    Action {
        kind: ActionKind::Place,
        position: Some(best_position),
    }
}

fn infer_believed_turn(state: &VisibleState, hint: &LegalActionHint) -> BelievedTurn {
    let candidates = [
        build_turn_candidate(state, hint, PlayerColor::Black),
        build_turn_candidate(state, hint, PlayerColor::White),
    ];
    let mut preferred_index = 0usize;
    if candidates[1].score > candidates[0].score
        || (candidates[1].score == candidates[0].score
            && state.current_player == Some(PlayerColor::White))
    {
        preferred_index = 1;
    }

    BelievedTurn {
        current_player: candidates[preferred_index].current_player,
        legal_actions: candidates[preferred_index].legal_actions.clone(),
    }
}

#[derive(Debug, Clone)]
struct TurnCandidate {
    current_player: PlayerColor,
    legal_actions: Vec<Position>,
    score: i32,
}

fn build_turn_candidate(
    state: &VisibleState,
    hint: &LegalActionHint,
    current_player: PlayerColor,
) -> TurnCandidate {
    let legal_actions = legal_actions_for_color(state, current_player);
    let mut score = 0i32;
    if state.current_player == Some(current_player) {
        score += 40;
    }
    score += overlap_score(&legal_actions, &state.legal_actions) * 20;
    score += overlap_score(&legal_actions, &hint.legal_actions) * 20;
    score -= mismatch_score(&legal_actions, &state.legal_actions) * 5;
    score -= mismatch_score(&legal_actions, &hint.legal_actions) * 5;
    if !legal_actions.is_empty() {
        score += 3;
    }

    TurnCandidate {
        current_player,
        legal_actions,
        score,
    }
}

fn simulate_action(
    state: &VisibleState,
    current_player: PlayerColor,
    position: Position,
) -> MatchState {
    let mut match_state = match_state_from_visible_state(state);
    match_state.current_player = Some(current_player);
    match_state.completed = false;
    let _ = match_state.apply_valid_action(&Action {
        kind: ActionKind::Place,
        position: Some(position),
    });
    match_state
}

fn minimax(state: &MatchState, depth: u8, root_player: PlayerColor) -> i32 {
    if depth == 0 || state.completed || state.current_player.is_none() {
        return evaluate_state(state, root_player);
    }

    let current_player = state.current_player.expect("current player");
    let legal_actions = state.legal_actions_for(current_player);
    if legal_actions.is_empty() {
        let mut passed = state.clone();
        passed
            .apply_valid_action(&Action {
                kind: ActionKind::Pass,
                position: None,
            })
            .expect("forced pass");
        return minimax(&passed, depth.saturating_sub(1), root_player);
    }

    let mut ordered_actions = legal_actions;
    ordered_actions.sort_by_key(|position| (position.row, position.col));

    let mut best = if current_player == root_player {
        i32::MIN
    } else {
        i32::MAX
    };
    for position in ordered_actions {
        let mut next = state.clone();
        next.apply_valid_action(&Action {
            kind: ActionKind::Place,
            position: Some(position),
        })
        .expect("legal action");
        let score = minimax(&next, depth.saturating_sub(1), root_player);
        if current_player == root_player {
            best = best.max(score);
        } else {
            best = best.min(score);
        }
    }
    best
}

fn evaluate_state(state: &MatchState, root_player: PlayerColor) -> i32 {
    let opponent = root_player.opponent();
    let own_disc = root_player.disc();
    let opponent_disc = opponent.disc();

    let corner_diff = count_corners(state, own_disc) - count_corners(state, opponent_disc);
    let mobility_diff = state.legal_actions_for(root_player).len() as i32
        - state.legal_actions_for(opponent).len() as i32;
    let edge_diff = count_edges(state, own_disc) - count_edges(state, opponent_disc);
    let disc_diff = count_discs(state, own_disc) - count_discs(state, opponent_disc);
    let corner_risk =
        corner_adjacent_penalty(state, own_disc) - corner_adjacent_penalty(state, opponent_disc);
    let parity = if count_empty_on_state(state).is_multiple_of(2) {
        -1
    } else {
        1
    };
    let endgame_weight = if count_empty_on_state(state) <= 10 {
        3
    } else {
        1
    };

    corner_diff * 100 + mobility_diff * 15 + edge_diff * 8 + disc_diff * endgame_weight
        - corner_risk * 12
        + parity * 4
}

fn count_corners(state: &MatchState, disc: Disc) -> i32 {
    CORNERS
        .iter()
        .filter(|&&(row, col)| state.board[row][col] == disc)
        .count() as i32
}

fn count_edges(state: &MatchState, disc: Disc) -> i32 {
    let mut count = 0i32;
    for index in EDGE_START..EDGE_END {
        if state.board[0][index] == disc {
            count += 1;
        }
        if state.board[BOARD_SIZE - 1][index] == disc {
            count += 1;
        }
        if state.board[index][0] == disc {
            count += 1;
        }
        if state.board[index][BOARD_SIZE - 1] == disc {
            count += 1;
        }
    }
    count
}

fn count_discs(state: &MatchState, disc: Disc) -> i32 {
    state
        .board
        .iter()
        .flatten()
        .filter(|candidate| **candidate == disc)
        .count() as i32
}

fn corner_adjacent_penalty(state: &MatchState, disc: Disc) -> i32 {
    let mut penalty = 0i32;
    for &((corner_row, corner_col), adjacent) in CORNER_ADJACENT {
        if state.board[corner_row][corner_col] != Disc::Empty {
            continue;
        }
        penalty += adjacent
            .iter()
            .filter(|&&(row, col)| state.board[row][col] == disc)
            .count() as i32;
    }
    penalty
}

fn count_empty_cells(board: &[Vec<Disc>]) -> usize {
    board
        .iter()
        .flatten()
        .filter(|disc| **disc == Disc::Empty)
        .count()
}

fn count_empty_on_state(state: &MatchState) -> usize {
    state
        .board
        .iter()
        .flatten()
        .filter(|disc| **disc == Disc::Empty)
        .count()
}

fn legal_actions_for_color(state: &VisibleState, current_player: PlayerColor) -> Vec<Position> {
    let mut match_state = match_state_from_visible_state(state);
    match_state.current_player = Some(current_player);
    match_state.completed = false;
    match_state.legal_actions_for(current_player)
}

fn overlap_score(left: &[Position], right: &[Position]) -> i32 {
    left.iter()
        .filter(|position| right.contains(position))
        .count() as i32
}

fn mismatch_score(left: &[Position], right: &[Position]) -> i32 {
    right
        .iter()
        .filter(|position| !left.contains(position))
        .count() as i32
}

fn match_state_from_visible_state(state: &VisibleState) -> MatchState {
    let mut board = [[Disc::Empty; BOARD_SIZE]; BOARD_SIZE];
    for (row_index, row) in state.board.iter().enumerate().take(BOARD_SIZE) {
        for (col_index, disc) in row.iter().enumerate().take(BOARD_SIZE) {
            board[row_index][col_index] = *disc;
        }
    }

    MatchState {
        board,
        current_player: state.current_player,
        turn: state.turn as i32,
        consecutive_passes: 0,
        completed: state.current_player.is_none(),
        winner_by_forfeit: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiarena_protocol::{
        match_response_id,
        player::{METHOD_GAME_OVER, METHOD_INIT, METHOD_TURN},
    };
    use reversi_game::{InitState, ScoreSummary};

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
                board: vec![vec![reversi_game::Disc::Empty; 8]; 8],
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
    fn choose_action_prefers_corner_over_interior_move() {
        let state = strategic_corner_test_state();
        let hint = LegalActionHint {
            legal_actions: match_state_from_visible_state(&state)
                .legal_actions_for(PlayerColor::Black),
        };
        let action = choose_action(&state, &hint);
        assert_eq!(action.kind, ActionKind::Place);
        assert_eq!(action.position, Some(Position { row: 0, col: 0 }));
    }

    #[test]
    fn choose_action_passes_only_when_no_believed_legal_action_exists() {
        let mut state = reversi_game::sample_visible_state();
        state.board = vec![vec![Disc::Black; 8]; 8];
        state.current_player = None;
        state.legal_actions = Vec::new();
        state.scores = ScoreSummary {
            black: 64,
            white: 0,
        };
        let action = choose_action(
            &state,
            &LegalActionHint {
                legal_actions: Vec::new(),
            },
        );
        assert_eq!(action.kind, ActionKind::Pass);
        assert_eq!(action.position, None);
    }

    #[test]
    fn choose_action_recovers_when_hint_is_empty_but_board_shows_legal_moves() {
        let mut state = strategic_corner_test_state();
        state.current_player = None;
        state.legal_actions = vec![Position { row: 0, col: 0 }, Position { row: 3, col: 5 }];

        let action = choose_action(
            &state,
            &LegalActionHint {
                legal_actions: Vec::new(),
            },
        );

        assert_eq!(action.kind, ActionKind::Place);
        assert_eq!(action.position, Some(Position { row: 0, col: 0 }));
    }

    #[test]
    fn choose_action_ignores_inconsistent_hint_and_uses_believed_legal_moves() {
        let mut state = strategic_corner_test_state();
        state.current_player = Some(PlayerColor::Black);
        state.legal_actions = vec![Position { row: 0, col: 0 }, Position { row: 3, col: 5 }];

        let action = choose_action(
            &state,
            &LegalActionHint {
                legal_actions: vec![Position { row: 7, col: 7 }],
            },
        );

        assert_eq!(action.kind, ActionKind::Place);
        assert_eq!(action.position, Some(Position { row: 0, col: 0 }));
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

    fn strategic_corner_test_state() -> VisibleState {
        let mut board = vec![vec![Disc::Empty; 8]; 8];
        board[0][1] = Disc::White;
        board[0][2] = Disc::Black;
        board[1][0] = Disc::White;
        board[1][1] = Disc::White;
        board[1][2] = Disc::White;
        board[2][0] = Disc::Black;
        board[2][2] = Disc::Black;
        board[2][3] = Disc::White;
        board[2][4] = Disc::Black;

        VisibleState {
            turn: 12,
            board,
            current_player: Some(PlayerColor::Black),
            legal_actions: Vec::new(),
            scores: ScoreSummary { black: 4, white: 4 },
        }
    }
}

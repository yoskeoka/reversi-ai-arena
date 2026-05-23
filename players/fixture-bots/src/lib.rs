use std::collections::VecDeque;
use std::io::{self, BufReader};

use aiarena_protocol::{
    Decoder, Encoder, Response,
    player::{GameOverResult, InitResult, METHOD_GAME_OVER, METHOD_INIT, METHOD_TURN},
};
use reversi_game::{
    Action, ActionKind, Position, ReversiPlayerTurnParams, ReversiPlayerTurnResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptToken {
    Move(Position),
    Pass,
}

#[derive(Debug)]
pub struct ScriptedStrategy {
    remaining: VecDeque<ScriptToken>,
}

impl ScriptedStrategy {
    pub fn from_moves(moves: &str) -> Result<Self, String> {
        Ok(Self {
            remaining: parse_script_tokens(moves)?.into(),
        })
    }

    pub fn next_action(&mut self, params: &ReversiPlayerTurnParams) -> Action {
        match self.remaining.pop_front() {
            Some(ScriptToken::Move(position)) => Action {
                kind: ActionKind::Place,
                position: Some(position),
            },
            Some(ScriptToken::Pass) => Action {
                kind: ActionKind::Pass,
                position: None,
            },
            None => choose_first_legal(params),
        }
    }
}

pub fn run_fixture_loop<F>(mut choose_action: F) -> Result<(), String>
where
    F: FnMut(&ReversiPlayerTurnParams) -> Action,
{
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut decoder = Decoder::new(BufReader::new(stdin.lock()));
    let mut encoder = Encoder::new(stdout.lock());

    while let Some(request) = decoder.decode_request().map_err(|err| err.to_string())? {
        let id = request.id.clone().unwrap_or_default();
        let response = match request.method.as_str() {
            METHOD_INIT => Response::success(id, &InitResult::ready()),
            METHOD_TURN => {
                let params: ReversiPlayerTurnParams =
                    request.parse_params().map_err(|err| err.to_string())?;
                let action = choose_action(&params);
                Response::success(id, &ReversiPlayerTurnResult { action })
            }
            METHOD_GAME_OVER => Response::success(id, &GameOverResult::ack()),
            _ => return Err(format!("unsupported method {}", request.method)),
        }
        .map_err(|err| err.to_string())?;
        encoder.encode(&response).map_err(|err| err.to_string())?;
    }

    Ok(())
}

pub fn choose_first_legal(params: &ReversiPlayerTurnParams) -> Action {
    match params.legal_action_hint.legal_actions.first().copied() {
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

pub fn parse_script_tokens(moves: &str) -> Result<Vec<ScriptToken>, String> {
    let bytes = moves.as_bytes();
    let mut index = 0;
    let mut tokens = Vec::new();
    while index < bytes.len() {
        if bytes[index..].starts_with(b"pass") {
            tokens.push(ScriptToken::Pass);
            index += 4;
            continue;
        }
        if index + 2 > bytes.len() {
            return Err(format!("trailing move fragment at byte {}", index));
        }
        let token = std::str::from_utf8(&bytes[index..index + 2])
            .map_err(|_| format!("non-ascii move token at byte {}", index))?;
        tokens.push(ScriptToken::Move(parse_position(token)?));
        index += 2;
    }
    Ok(tokens)
}

fn parse_position(token: &str) -> Result<Position, String> {
    let bytes = token.as_bytes();
    if bytes.len() != 2 {
        return Err(format!("invalid move token {}", token));
    }
    let col = match bytes[0] {
        b'a'..=b'h' => bytes[0] - b'a',
        _ => return Err(format!("invalid file {}", token)),
    };
    let row = match bytes[1] {
        b'1'..=b'8' => bytes[1] - b'1',
        _ => return Err(format!("invalid rank {}", token)),
    };
    Ok(Position { row, col })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_line_parses_into_positions() {
        let tokens = parse_script_tokens("c4e3pass").expect("tokens");
        assert_eq!(
            tokens,
            vec![
                ScriptToken::Move(Position { row: 3, col: 2 }),
                ScriptToken::Move(Position { row: 2, col: 4 }),
                ScriptToken::Pass,
            ]
        );
    }
}

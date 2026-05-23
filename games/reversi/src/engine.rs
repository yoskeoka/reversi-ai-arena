use serde::{Deserialize, Serialize};

use crate::{
    Action, ActionKind, Disc, DiscPlacement, GameSummary, PlayerColor, Position, PublicState,
    ScoreSummary, VisibleState,
};

pub const BOARD_SIZE: usize = 8;

const DIRECTIONS: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchState {
    pub board: [[Disc; BOARD_SIZE]; BOARD_SIZE],
    pub current_player: Option<PlayerColor>,
    pub turn: i32,
    pub consecutive_passes: u8,
    pub completed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner_by_forfeit: Option<PlayerColor>,
}

impl MatchState {
    pub fn new_standard() -> Self {
        let mut board = [[Disc::Empty; BOARD_SIZE]; BOARD_SIZE];
        board[3][3] = Disc::White;
        board[3][4] = Disc::Black;
        board[4][3] = Disc::Black;
        board[4][4] = Disc::White;
        Self {
            board,
            current_player: Some(PlayerColor::Black),
            turn: 1,
            consecutive_passes: 0,
            completed: false,
            winner_by_forfeit: None,
        }
    }

    pub fn opening() -> Vec<DiscPlacement> {
        vec![
            DiscPlacement {
                position: Position { row: 3, col: 3 },
                disc: Disc::White,
            },
            DiscPlacement {
                position: Position { row: 3, col: 4 },
                disc: Disc::Black,
            },
            DiscPlacement {
                position: Position { row: 4, col: 3 },
                disc: Disc::Black,
            },
            DiscPlacement {
                position: Position { row: 4, col: 4 },
                disc: Disc::White,
            },
        ]
    }

    pub fn legal_actions_for(&self, color: PlayerColor) -> Vec<Position> {
        if self.completed {
            return Vec::new();
        }
        let mut positions = Vec::new();
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                let position = Position {
                    row: row as u8,
                    col: col as u8,
                };
                if !self.flips_for(color, position).is_empty() {
                    positions.push(position);
                }
            }
        }
        positions
    }

    pub fn visible_state_for(&self, color: PlayerColor) -> VisibleState {
        let legal_actions = if self.current_player == Some(color) && !self.completed {
            self.legal_actions_for(color)
        } else {
            Vec::new()
        };
        VisibleState {
            turn: self.turn as u32,
            board: self.board_vec(),
            current_player: self.current_player,
            legal_actions,
            scores: self.scores(),
        }
    }

    pub fn public_state(&self) -> PublicState {
        PublicState {
            turn: self.turn as u32,
            board: self.board_vec(),
            current_player: self.current_player,
            scores: self.scores(),
            completed: self.completed,
        }
    }

    pub fn summary(&self) -> GameSummary {
        let scores = self.scores();
        if let Some(winner) = self.winner_by_forfeit {
            return match winner {
                PlayerColor::Black => GameSummary {
                    black: scores.black,
                    white: scores.white,
                    winners: vec![PlayerColor::Black],
                },
                PlayerColor::White => GameSummary {
                    black: scores.black,
                    white: scores.white,
                    winners: vec![PlayerColor::White],
                },
            };
        }
        let winners = if scores.black > scores.white {
            vec![PlayerColor::Black]
        } else if scores.white > scores.black {
            vec![PlayerColor::White]
        } else {
            vec![PlayerColor::Black, PlayerColor::White]
        };
        GameSummary {
            black: scores.black,
            white: scores.white,
            winners,
        }
    }

    pub fn apply_valid_action(&mut self, action: &Action) -> Result<(), &'static str> {
        let color = self.current_player.ok_or("match is completed")?;
        let legal_actions = self.legal_actions_for(color);
        if legal_actions.is_empty() {
            if action.kind != ActionKind::Pass || action.position.is_some() {
                return Err("forced pass turn requires pass");
            }
            self.consecutive_passes += 1;
        } else {
            if action.kind != ActionKind::Place {
                return Err("legal move turn requires placement");
            }
            let position = action.position.ok_or("placement requires position")?;
            let flips = self.flips_for(color, position);
            if flips.is_empty() {
                return Err("placement is not legal");
            }
            self.set_disc(position, color.disc());
            for flip in flips {
                self.set_disc(flip, color.disc());
            }
            self.consecutive_passes = 0;
        }

        let next_player = color.opponent();
        self.current_player = Some(next_player);
        self.turn += 1;

        if self.is_board_full() || self.consecutive_passes >= 2 {
            self.completed = true;
            self.current_player = None;
        }

        Ok(())
    }

    pub fn forfeit(&mut self, loser: PlayerColor) {
        self.completed = true;
        self.current_player = None;
        self.winner_by_forfeit = Some(loser.opponent());
    }

    pub fn is_legal_action(&self, color: PlayerColor, action: &Action) -> bool {
        let legal_actions = self.legal_actions_for(color);
        if legal_actions.is_empty() {
            return action.kind == ActionKind::Pass && action.position.is_none();
        }
        if action.kind != ActionKind::Place {
            return false;
        }
        match action.position {
            Some(position) => legal_actions.contains(&position),
            None => false,
        }
    }

    fn board_vec(&self) -> Vec<Vec<Disc>> {
        self.board.iter().map(|row| row.to_vec()).collect()
    }

    fn scores(&self) -> ScoreSummary {
        let mut black = 0u8;
        let mut white = 0u8;
        for row in &self.board {
            for disc in row {
                match disc {
                    Disc::Black => black += 1,
                    Disc::White => white += 1,
                    Disc::Empty => {}
                }
            }
        }
        ScoreSummary { black, white }
    }

    fn flips_for(&self, color: PlayerColor, position: Position) -> Vec<Position> {
        if !self.in_bounds(position) || self.disc_at(position) != Disc::Empty {
            return Vec::new();
        }
        let mut flips = Vec::new();
        for (dr, dc) in DIRECTIONS {
            let mut line = Vec::new();
            let mut row = position.row as i8 + dr;
            let mut col = position.col as i8 + dc;
            while (0..BOARD_SIZE as i8).contains(&row) && (0..BOARD_SIZE as i8).contains(&col) {
                let candidate = Position {
                    row: row as u8,
                    col: col as u8,
                };
                let disc = self.disc_at(candidate);
                if disc == color.opponent().disc() {
                    line.push(candidate);
                } else if disc == color.disc() {
                    if !line.is_empty() {
                        flips.extend(line);
                    }
                    break;
                } else {
                    break;
                }
                row += dr;
                col += dc;
            }
        }
        flips
    }

    fn is_board_full(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|disc| *disc != Disc::Empty))
    }

    fn disc_at(&self, position: Position) -> Disc {
        self.board[position.row as usize][position.col as usize]
    }

    fn set_disc(&mut self, position: Position, disc: Disc) {
        self.board[position.row as usize][position.col as usize] = disc;
    }

    fn in_bounds(&self, position: Position) -> bool {
        (position.row as usize) < BOARD_SIZE && (position.col as usize) < BOARD_SIZE
    }
}

impl PlayerColor {
    pub const fn opponent(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }

    pub const fn disc(self) -> Disc {
        match self {
            Self::Black => Disc::Black,
            Self::White => Disc::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_opening_legal_actions_match_reversi() {
        let state = MatchState::new_standard();
        let legal = state.legal_actions_for(PlayerColor::Black);
        assert_eq!(
            legal,
            vec![
                Position { row: 2, col: 3 },
                Position { row: 3, col: 2 },
                Position { row: 4, col: 5 },
                Position { row: 5, col: 4 },
            ]
        );
    }

    #[test]
    fn applying_c4_flips_one_disc() {
        let mut state = MatchState::new_standard();
        state
            .apply_valid_action(&Action {
                kind: ActionKind::Place,
                position: Some(Position { row: 3, col: 2 }),
            })
            .expect("apply");
        assert_eq!(state.board[3][2], Disc::Black);
        assert_eq!(state.board[3][3], Disc::Black);
        assert_eq!(state.current_player, Some(PlayerColor::White));
    }

    #[test]
    fn illegal_pass_is_rejected_when_legal_moves_exist() {
        let state = MatchState::new_standard();
        assert!(!state.is_legal_action(
            PlayerColor::Black,
            &Action {
                kind: ActionKind::Pass,
                position: None,
            }
        ));
    }
}

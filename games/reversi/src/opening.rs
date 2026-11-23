use std::collections::HashSet;
use std::str::FromStr;

use sha2::{Digest, Sha256};

use crate::{Action, ActionKind, PlayerColor, Position, RULESET_VERSION, engine::MatchState};

pub const XOT_LEN: usize = 8;
const ROW_BYTES: usize = XOT_LEN * 2 + 1;
pub type OpeningMoves = [Position; XOT_LEN];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ruleset {
    Standard,
    XotLarge,
    XotSmall,
}

impl Ruleset {
    pub const ALL: [Self; 3] = [Self::Standard, Self::XotLarge, Self::XotSmall];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Standard => RULESET_VERSION,
            Self::XotLarge => "standard-xot-large",
            Self::XotSmall => "standard-xot-small",
        }
    }

    pub const fn is_xot(self) -> bool {
        !matches!(self, Self::Standard)
    }
}

impl FromStr for Ruleset {
    type Err = String;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|ruleset| ruleset.id() == id)
            .ok_or_else(|| format!("unsupported Reversi ruleset {id:?}"))
    }
}

pub fn initial_state(opening: Option<&OpeningMoves>) -> Result<MatchState, String> {
    let mut state = MatchState::new_standard();
    if let Some(moves) = opening {
        for (index, &position) in moves.iter().enumerate() {
            state
                .apply_valid_action(&Action {
                    kind: ActionKind::Place,
                    position: Some(position),
                })
                .map_err(|err| format!("XOT move {}: {err}", index + 1))?;
        }
        if state.completed || state.current_player != Some(PlayerColor::Black) {
            return Err("XOT opening must leave Black to move".to_string());
        }
    }
    Ok(state)
}

fn parse_row(row: &[u8]) -> Result<OpeningMoves, String> {
    if row.len() != ROW_BYTES || row[ROW_BYTES - 1] != b'\n' {
        return Err("expected eight coordinates and LF".into());
    }
    let mut moves = [Position { row: 0, col: 0 }; XOT_LEN];
    for (position, coordinate) in moves.iter_mut().zip(row.chunks_exact(2)) {
        let [file, rank] = [coordinate[0], coordinate[1]];
        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return Err("invalid XOT coordinate".into());
        }
        *position = Position {
            row: rank - b'1',
            col: file - b'a',
        };
    }
    Ok(moves)
}

pub fn validate_corpus(data: &[u8], rows: usize, expected_sha256: &str) -> Result<(), String> {
    let digest = format!("{:x}", Sha256::digest(data));
    if digest != expected_sha256 {
        return Err(format!(
            "SHA-256 mismatch: expected {expected_sha256}, got {digest}"
        ));
    }
    if data.len() != rows * ROW_BYTES {
        return Err(format!("expected {rows} XOT rows of {ROW_BYTES} bytes"));
    }
    let mut seen = HashSet::with_capacity(rows);
    for (index, row) in data.chunks_exact(ROW_BYTES).enumerate() {
        let result = if seen.insert(row) {
            parse_row(row).and_then(|moves| initial_state(Some(&moves)).map(|_| ()))
        } else {
            Err("duplicate opening".into())
        };
        result.map_err(|err| format!("row {}: {err}", index + 1))?;
    }
    Ok(())
}

pub fn select_opening(data: &[u8], seed: &[u8]) -> Result<OpeningMoves, String> {
    let rows = data.len() / ROW_BYTES;
    if rows == 0 || !data.len().is_multiple_of(ROW_BYTES) {
        return Err("XOT opening data is missing or malformed".to_string());
    }
    let row = select_index(rows as u64, seed) as usize;
    parse_row(&data[row * ROW_BYTES..(row + 1) * ROW_BYTES])
}

fn select_index(rows: u64, seed: &[u8]) -> u64 {
    let threshold = rows.wrapping_neg() % rows;
    for counter in 0..=u64::MAX {
        let mut digest = Sha256::new();
        digest.update(seed);
        digest.update(counter.to_be_bytes());
        let hash = digest.finalize();
        let sample = u64::from_be_bytes(hash[..8].try_into().expect("SHA-256 gives 32 bytes"));
        if sample >= threshold {
            return sample % rows;
        }
    }
    unreachable!("all SHA-256 samples rejected")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_selection_is_repeatable_and_varies_with_seed() {
        assert_eq!(select_index(10_784, b"test"), select_index(10_784, b"test"));
        let selected: HashSet<_> = (0_u32..32)
            .map(|n| select_index(10_784, &n.to_be_bytes()))
            .collect();
        assert!(selected.len() > 1);
    }

    #[test]
    fn corpus_validation_checks_digest_format_and_engine_legality() {
        let row = b"f5d6c4d3c2b3b4b5\n";
        let digest = |data: &[u8]| format!("{:x}", Sha256::digest(data));
        validate_corpus(row, 1, &digest(row)).unwrap();
        assert!(validate_corpus(row, 1, "different").is_err());
        for invalid in [b"X5d6c4d3c2b3b4b5\n", b"a1d6c4d3c2b3b4b5\n"] {
            assert!(validate_corpus(invalid, 1, &digest(invalid)).is_err());
        }
        let duplicate = row.repeat(2);
        assert!(validate_corpus(&duplicate, 2, &digest(&duplicate)).is_err());
    }
}

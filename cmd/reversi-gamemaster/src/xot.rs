use reversi_game::opening::{OpeningMoves, Ruleset};

#[cfg(feature = "xot")]
pub fn opening(
    ruleset: Ruleset,
    rng_seed: Option<&str>,
    player_ids: &[String],
) -> Result<Option<OpeningMoves>, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let data: &[u8] = match ruleset {
        Ruleset::Standard => return Ok(None),
        Ruleset::XotLarge => include_bytes!(concat!(env!("OUT_DIR"), "/openingslarge.txt")),
        Ruleset::XotSmall => include_bytes!(concat!(env!("OUT_DIR"), "/openingssmall.txt")),
    };
    let derived;
    let seed = match rng_seed {
        Some(seed) => seed.as_bytes(),
        None => {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|err| format!("read XOT match clock: {err}"))?;
            if time.is_zero() {
                return Err("XOT match clock is unavailable".to_string());
            }
            derived = serde_json::to_vec(&(time.as_nanos(), player_ids))
                .map_err(|err| format!("encode XOT match seed: {err}"))?;
            &derived
        }
    };
    reversi_game::opening::select_opening(data, seed).map(Some)
}

#[cfg(not(feature = "xot"))]
pub fn opening(
    ruleset: Ruleset,
    _rng_seed: Option<&str>,
    _player_ids: &[String],
) -> Result<Option<OpeningMoves>, String> {
    match ruleset {
        Ruleset::Standard => Ok(None),
        _ => Err("game master was built without XOT data".into()),
    }
}

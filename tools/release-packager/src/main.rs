use reversi_game::{GAME_ID, GAME_VERSION, RULESET_VERSION};
use serde_json::{Value, json};

fn game_manifest() -> Value {
    json!({
        "schema_version": "arena-bundle/v1",
        "artifact_kind": "game",
        "game_id": GAME_ID,
        "game_version": GAME_VERSION,
        "rulesets": [{
            "ruleset_version": RULESET_VERSION,
            "player_count": 2,
            "max_active_bots_per_owner": 3
        }],
        "runtime": { "kind": "wasm-wasi", "module": "reversi-gamemaster.wasm" }
    })
}

fn ai_manifest() -> Value {
    let mut manifest = serde_json::to_value(reversi_rust_reference_player::sidecar_manifest())
        .expect("reference player manifest is serializable");
    let object = manifest
        .as_object_mut()
        .expect("reference player manifest is an object");
    let protocol = object
        .remove("protocol")
        .and_then(|value| value.as_object().cloned())
        .expect("reference player manifest has protocol");
    object.insert(
        "game_id".to_string(),
        protocol
            .get("game_id")
            .cloned()
            .expect("reference player protocol has game_id"),
    );
    object.insert(
        "game_version".to_string(),
        protocol
            .get("game_version")
            .cloned()
            .expect("reference player protocol has game_version"),
    );
    object.insert("schema_version".to_string(), json!("arena-bundle/v1"));
    object.insert("artifact_kind".to_string(), json!("ai"));
    object.insert(
        "rulesets".to_string(),
        json!([{ "ruleset_version": RULESET_VERSION }]),
    );
    object
        .get_mut("runtime")
        .and_then(Value::as_object_mut)
        .expect("reference player manifest has runtime")
        .extend([
            ("module".to_string(), json!("rust-reference-ai.wasm")),
            ("args".to_string(), json!(["./rust-reference-ai.wasm"])),
        ]);
    manifest
}

fn main() {
    let kind = std::env::args().nth(1).unwrap_or_default();
    let manifest = match kind.as_str() {
        "game-manifest" => game_manifest(),
        "ai-manifest" => ai_manifest(),
        _ => {
            eprintln!("usage: reversi-release-packager <game-manifest|ai-manifest>");
            std::process::exit(2);
        }
    };
    println!(
        "{}",
        serde_json::to_string(&manifest).expect("serialize manifest")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_manifest_matches_reversi_policy() {
        assert_eq!(game_manifest()["game_id"], GAME_ID);
        assert_eq!(game_manifest()["game_version"], GAME_VERSION);
        assert_eq!(
            game_manifest()["rulesets"][0]["ruleset_version"],
            RULESET_VERSION
        );
        assert_eq!(game_manifest()["rulesets"][0]["player_count"], 2);
        assert_eq!(
            game_manifest()["rulesets"][0]["max_active_bots_per_owner"],
            3
        );
    }

    #[test]
    fn ai_manifest_reuses_reference_identity_and_has_bundle_fields() {
        let manifest = ai_manifest();
        assert_eq!(
            manifest["ai_id"],
            reversi_rust_reference_player::player_name()
        );
        assert_eq!(manifest["game_id"], GAME_ID);
        assert_eq!(manifest["game_version"], GAME_VERSION);
        assert_eq!(manifest["runtime"]["module"], "rust-reference-ai.wasm");
        assert_eq!(
            manifest["runtime"]["args"],
            json!(["./rust-reference-ai.wasm"])
        );
        assert_eq!(manifest["schema_version"], "arena-bundle/v1");
        assert_eq!(manifest["artifact_kind"], "ai");
    }
}

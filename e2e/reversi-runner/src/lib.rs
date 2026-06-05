#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::OnceLock;

    use reversi_fixture_bots::{ScriptToken, parse_script_tokens};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct ResultSummary {
        status: String,
        game_id: String,
        game_version: String,
        ruleset_version: String,
        #[allow(dead_code)]
        turn: i32,
        placements: Vec<Placement>,
    }

    #[derive(Debug, Deserialize)]
    struct Placement {
        player_id: String,
        place: i32,
    }

    #[derive(Debug, Deserialize)]
    struct ExportedSnapshot {
        public_state: Option<PublicState>,
        players: Vec<ExportedPlayerSnapshot>,
    }

    #[derive(Debug, Deserialize)]
    struct PublicState {
        completed: bool,
        scores: ScoreSummary,
    }

    #[derive(Debug, Deserialize)]
    struct ScoreSummary {
        black: u8,
        white: u8,
    }

    #[derive(Debug, Deserialize)]
    struct ExportedPlayerSnapshot {
        player_id: String,
        last_action_status: ActionStatus,
    }

    #[derive(Debug, Deserialize)]
    struct ActionStatus {
        action_status: String,
        failure_reason: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct HistoryEvent {
        turn: i32,
        kind: String,
        #[allow(dead_code)]
        player_id: Option<String>,
        payload: Option<HistoryPayload>,
    }

    #[derive(Debug, Deserialize)]
    struct HistoryPayload {
        action_status: Option<String>,
        action: Option<HistoryAction>,
    }

    #[derive(Debug, Deserialize)]
    struct HistoryAction {
        kind: String,
    }

    struct ScriptedCase {
        file_name: &'static str,
        match_id: &'static str,
        expected_winner: Option<&'static str>,
        min_pass_count: usize,
        expected_disc_total: Option<u16>,
        require_final_double_pass: bool,
    }

    #[test]
    #[ignore = "requires pinned tagged arena-runner install"]
    fn tagged_runner_completes_with_first_legal_fixtures() {
        let repo = repo_root();
        let runner = runner_bin();
        let fixtures = build_fixtures(&repo);
        let fixture_dir = prepare_fixture_dir(&repo);
        let output_dir = fixture_dir.join("artifacts-first-legal");
        let manifest_path = write_game_master_manifest(&fixture_dir, &repo);

        run_runner(
            &runner,
            &repo,
            &[
                "--game-master-manifest",
                &repo_relative(&repo, &manifest_path),
                "--match-id",
                "reversi-first-legal",
                "--output-dir",
                &repo_relative(&repo, &output_dir),
                "--log-output",
                "none",
                "--player",
                &format!(
                    "p1={}",
                    repo_relative(&repo, &fixtures.legal_move_first_entry)
                ),
                "--player",
                &format!(
                    "p2={}",
                    repo_relative(&repo, &fixtures.legal_move_first_entry)
                ),
            ],
        );

        let summary = read_summary(output_dir.join("reversi-first-legal/result-summary.json"));
        assert_eq!(summary.status, "completed");
        assert_eq!(summary.game_id, "reversi");
        assert_eq!(summary.game_version, "1.0.0");
        assert_eq!(summary.ruleset_version, "standard");
    }

    #[test]
    #[ignore = "requires pinned tagged arena-runner install"]
    fn tagged_runner_completes_with_rust_wasm_fixture() {
        let repo = repo_root();
        let runner = runner_bin();
        let fixtures = build_fixtures(&repo);
        let fixture_dir = prepare_fixture_dir(&repo);
        let output_dir = fixture_dir.join("artifacts-rust-wasm");
        let manifest_path = write_game_master_manifest(&fixture_dir, &repo);

        run_runner(
            &runner,
            &repo,
            &[
                "--game-master-manifest",
                &repo_relative(&repo, &manifest_path),
                "--match-id",
                "reversi-rust-wasm",
                "--output-dir",
                &repo_relative(&repo, &output_dir),
                "--log-output",
                "none",
                "--player",
                &format!("p1={}", repo_relative(&repo, &fixtures.rust_wasm_entry)),
                "--player",
                &format!(
                    "p2={}",
                    repo_relative(&repo, &fixtures.legal_move_first_entry)
                ),
            ],
        );

        let match_dir = output_dir.join("reversi-rust-wasm");
        let summary = read_summary(match_dir.join("result-summary.json"));
        assert_eq!(summary.status, "completed");

        let exported = read_exported_snapshot(match_dir.join("exported-snapshot.json"));
        let public_state = exported.public_state.expect("public state");
        assert!(public_state.completed);
    }

    #[test]
    #[ignore = "requires pinned tagged arena-runner install"]
    fn tagged_runner_replays_canonical_scripted_suite() {
        let repo = repo_root();
        let runner = runner_bin();
        let fixtures = build_fixtures(&repo);
        let cases = scripted_cases();

        for case in cases {
            let fixture_dir = prepare_fixture_dir(&repo);
            let output_dir = fixture_dir.join(format!("artifacts-{}", case.match_id));
            let manifest_path = write_game_master_manifest(&fixture_dir, &repo);
            let line = load_script_line(&repo, case.file_name);
            let (black_moves, white_moves) = split_moves(&line);
            let p1_entry = write_scripted_player_manifest(
                &fixture_dir,
                &fixtures.scripted_player_entry,
                "black",
                &black_moves,
            );
            let p2_entry = write_scripted_player_manifest(
                &fixture_dir,
                &fixtures.scripted_player_entry,
                "white",
                &white_moves,
            );

            run_runner(
                &runner,
                &repo,
                &[
                    "--game-master-manifest",
                    &repo_relative(&repo, &manifest_path),
                    "--match-id",
                    case.match_id,
                    "--output-dir",
                    &repo_relative(&repo, &output_dir),
                    "--log-output",
                    "none",
                    "--player",
                    &format!("p1={}", repo_relative(&repo, &p1_entry)),
                    "--player",
                    &format!("p2={}", repo_relative(&repo, &p2_entry)),
                ],
            );

            let match_dir = output_dir.join(case.match_id);
            let summary = read_summary(match_dir.join("result-summary.json"));
            assert_eq!(summary.status, "completed");
            if let Some(expected_winner) = case.expected_winner {
                let winner = summary
                    .placements
                    .iter()
                    .find(|placement| placement.place == 1)
                    .expect("winner placement");
                assert_eq!(winner.player_id, expected_winner);
            }
            let exported = read_exported_snapshot(match_dir.join("exported-snapshot.json"));
            let public_state = exported.public_state.expect("public state");
            assert!(public_state.completed);
            if let Some(expected_disc_total) = case.expected_disc_total {
                assert_eq!(
                    public_state.scores.black as u16 + public_state.scores.white as u16,
                    expected_disc_total
                );
            }

            let history = read_history(match_dir.join("history.json"));
            let accepted_passes = accepted_passes(&history);
            assert!(
                accepted_passes.len() >= case.min_pass_count,
                "expected at least {} accepted passes in {}, got {}",
                case.min_pass_count,
                case.match_id,
                accepted_passes.len()
            );
            if case.require_final_double_pass {
                let accepted_actions = accepted_actions(&history);
                let tail = accepted_actions
                    .get(accepted_actions.len().saturating_sub(2)..)
                    .expect("last two accepted actions");
                assert_eq!(tail.len(), 2, "expected two accepted actions at the end");
                assert_eq!(
                    tail[0]
                        .payload
                        .as_ref()
                        .and_then(|payload| payload.action.as_ref())
                        .map(|action| action.kind.as_str()),
                    Some("pass")
                );
                assert_eq!(
                    tail[1]
                        .payload
                        .as_ref()
                        .and_then(|payload| payload.action.as_ref())
                        .map(|action| action.kind.as_str()),
                    Some("pass")
                );
                assert_eq!(tail[1].turn, tail[0].turn + 1);
            }
        }
    }

    #[test]
    #[ignore = "requires pinned tagged arena-runner install"]
    fn illegal_pass_with_legal_move_causes_immediate_loss() {
        let repo = repo_root();
        let runner = runner_bin();
        let fixtures = build_fixtures(&repo);
        let fixture_dir = prepare_fixture_dir(&repo);
        let output_dir = fixture_dir.join("artifacts-illegal-pass");
        let manifest_path = write_game_master_manifest(&fixture_dir, &repo);
        let invalid_entry = write_scripted_player_manifest(
            &fixture_dir,
            &fixtures.scripted_player_entry,
            "invalid",
            "pass",
        );

        run_runner(
            &runner,
            &repo,
            &[
                "--game-master-manifest",
                &repo_relative(&repo, &manifest_path),
                "--match-id",
                "reversi-illegal-pass",
                "--output-dir",
                &repo_relative(&repo, &output_dir),
                "--log-output",
                "none",
                "--player",
                &format!("p1={}", repo_relative(&repo, &invalid_entry)),
                "--player",
                &format!(
                    "p2={}",
                    repo_relative(&repo, &fixtures.legal_move_first_entry)
                ),
            ],
        );

        let match_dir = output_dir.join("reversi-illegal-pass");
        let summary = read_summary(match_dir.join("result-summary.json"));
        assert_eq!(summary.status, "completed");
        let winner = summary
            .placements
            .iter()
            .find(|placement| placement.place == 1)
            .expect("winner placement");
        assert_eq!(winner.player_id, "p2");

        let exported = read_exported_snapshot(match_dir.join("exported-snapshot.json"));
        let invalid_status = exported
            .players
            .iter()
            .find(|player| player.player_id == "p1")
            .expect("p1 snapshot");
        assert_eq!(invalid_status.last_action_status.action_status, "no_action");
        assert_eq!(
            invalid_status.last_action_status.failure_reason.as_deref(),
            Some("invalid-illegal-action")
        );
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("e2e dir")
            .parent()
            .expect("repo root")
            .to_path_buf()
    }

    fn runner_bin() -> PathBuf {
        env::var_os("ARENA_RUNNER_BIN")
            .map(PathBuf::from)
            .expect("ARENA_RUNNER_BIN must be set by make runner-e2e")
    }

    struct FixtureBuilds {
        legal_move_first_entry: PathBuf,
        scripted_player_entry: PathBuf,
        rust_wasm_entry: PathBuf,
    }

    fn prepare_fixture_dir(repo: &Path) -> PathBuf {
        build_fixtures(repo);
        let root = repo.join(format!(
            ".tmp-reversi-runner-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix time")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create fixture dir");
        root
    }

    fn build_fixtures(repo: &Path) -> &'static FixtureBuilds {
        static BUILD_ONCE: OnceLock<FixtureBuilds> = OnceLock::new();
        BUILD_ONCE.get_or_init(|| {
            build_binary(repo, "reversi-gamemaster");
            build_binary(repo, "reversi-legal-move-first");
            build_binary(repo, "reversi-scripted-player");
            let legal_move_first_entry = repo.join("target/debug/reversi-legal-move-first");
            let scripted_player_entry = repo.join("target/debug/reversi-scripted-player");
            let rust_wasm_entry = build_rust_wasm_fixture(repo);

            FixtureBuilds {
                legal_move_first_entry,
                scripted_player_entry,
                rust_wasm_entry,
            }
        })
    }

    fn build_binary(repo: &Path, bin: &str) {
        let output = cargo_command(repo)
            .current_dir(repo)
            .args(["build", "--bin", bin])
            .output()
            .expect("run cargo build");
        if !output.status.success() {
            panic!(
                "cargo build {} failed:\n{}\n{}",
                bin,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn build_rust_wasm_fixture(repo: &Path) -> PathBuf {
        let output = cargo_command(repo)
            .current_dir(repo)
            .args([
                "build",
                "--target",
                "wasm32-wasip1",
                "-p",
                "reversi-rust-reference-player",
                "--bin",
                "reversi-rust-reference-player",
            ])
            .output()
            .expect("run cargo build for wasm player");
        if !output.status.success() {
            panic!(
                "cargo build wasm player failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let cache_dir = repo.join("target/reversi-fixtures/rust-reference-player");
        fs::create_dir_all(&cache_dir).expect("create wasm fixture cache dir");
        let module_path = cache_dir.join("reversi-rust-reference-player.wasm");
        let built_module =
            repo.join("target/wasm32-wasip1/debug/reversi-rust-reference-player.wasm");
        fs::copy(&built_module, &module_path).expect("copy wasm module into cache");

        let entry = cache_dir.join("reversi-rust-reference-player");
        fs::write(&entry, b"cached wasm sidecar entry").expect("write wasm entry placeholder");
        let manifest = serde_json::json!({
            "ai_id": "rust-reference",
            "protocol": {
                "transport": "stdio-jsonrpc-ndjson",
                "game_id": "reversi",
                "game_version": "1.0.0",
                "ruleset_version": "standard"
            },
            "runtime": {
                "kind": "wasm-wasi",
                "module": "./reversi-rust-reference-player.wasm",
                "args": ["./reversi-rust-reference-player.wasm"],
                "memory_limit_pages": 64
            }
        });
        write_json(&entry.with_extension("arena.json"), &manifest);
        entry
    }

    fn cargo_command(repo: &Path) -> Command {
        let mut command = Command::new("cargo");
        command.current_dir(repo);
        command.env(
            "CARGO_HOME",
            env::var("CARGO_HOME")
                .unwrap_or_else(|_| "/tmp/reversi-ai-arena-cargo-home".to_string()),
        );
        command
    }

    fn write_game_master_manifest(fixture_dir: &Path, repo: &Path) -> PathBuf {
        let manifest_path = fixture_dir.join("reversi-gamemaster-manifest.json");
        let manifest = serde_json::json!({
            "metadata": {
                "game_id": "reversi",
                "game_version": "1.0.0",
                "ruleset_version": "standard"
            },
            "runtime": {
                "kind": "local-subprocess",
                "command": [repo.join("target/debug/reversi-gamemaster").display().to_string()]
            }
        });
        write_json(&manifest_path, &manifest);
        manifest_path
    }

    fn write_scripted_player_manifest(
        fixture_dir: &Path,
        scripted_player_entry: &Path,
        name: &str,
        moves: &str,
    ) -> PathBuf {
        let entry = fixture_dir.join(format!("reversi-scripted-player-{}", name));
        fs::write(&entry, b"manifest-backed entry").expect("write entry placeholder");
        let manifest = serde_json::json!({
            "ai_id": format!("scripted-{}", name),
            "protocol": {
                "transport": "stdio-jsonrpc-ndjson",
                "game_id": "reversi",
                "game_version": "1.0.0",
                "ruleset_version": "standard"
            },
            "runtime": {
                "kind": "local-subprocess",
                "command": [scripted_player_entry.display().to_string(), "--moves", moves]
            }
        });
        write_json(&entry.with_extension("arena.json"), &manifest);
        entry
    }

    fn run_runner(runner: &Path, repo: &Path, args: &[&str]) {
        let output = Command::new(runner)
            .current_dir(repo)
            .args(args)
            .output()
            .expect("run arena-runner");
        if !output.status.success() {
            panic!(
                "arena-runner failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn scripted_cases() -> &'static [ScriptedCase] {
        &[
            ScriptedCase {
                file_name: "end-with-1-empty-cell-forced-pass-for-both.txt",
                match_id: "reversi-scripted-terminal-double-pass",
                expected_winner: None,
                min_pass_count: 2,
                expected_disc_total: Some(63),
                require_final_double_pass: true,
            },
            ScriptedCase {
                file_name: "fastest-black-win.txt",
                match_id: "reversi-scripted-fastest-black-win",
                expected_winner: Some("p1"),
                min_pass_count: 0,
                expected_disc_total: None,
                require_final_double_pass: false,
            },
            ScriptedCase {
                file_name: "short-white-win.txt",
                match_id: "reversi-scripted-short-white-win",
                expected_winner: Some("p2"),
                min_pass_count: 0,
                expected_disc_total: None,
                require_final_double_pass: false,
            },
            ScriptedCase {
                file_name: "multiple-passes-middle-and-empty-end.txt",
                match_id: "reversi-scripted-multi-pass-empty-end",
                expected_winner: None,
                min_pass_count: 3,
                expected_disc_total: None,
                require_final_double_pass: true,
            },
        ]
    }

    fn load_script_line(repo: &Path, file_name: &str) -> String {
        fs::read_to_string(repo.join("testdata/reversi/scripted-games").join(file_name))
            .expect("read line")
            .trim()
            .to_string()
    }

    fn split_moves(line: &str) -> (String, String) {
        let mut black = String::new();
        let mut white = String::new();
        let tokens = parse_script_tokens(line)
            .unwrap_or_else(|err| panic!("scripted line is invalid: {line}: {err}"));
        for (turn, token) in tokens.into_iter().enumerate() {
            if turn.is_multiple_of(2) {
                push_script_token(&mut black, token);
            } else {
                push_script_token(&mut white, token);
            }
        }
        (black, white)
    }

    fn push_script_token(target: &mut String, token: ScriptToken) {
        match token {
            ScriptToken::Move(position) => {
                target.push((b'a' + position.col) as char);
                target.push((b'1' + position.row) as char);
            }
            ScriptToken::Pass => target.push_str("pass"),
        }
    }

    fn read_summary(path: PathBuf) -> ResultSummary {
        serde_json::from_slice(&fs::read(path).expect("read summary")).expect("decode summary")
    }

    fn read_exported_snapshot(path: PathBuf) -> ExportedSnapshot {
        serde_json::from_slice(&fs::read(path).expect("read exported snapshot"))
            .expect("decode exported snapshot")
    }

    fn read_history(path: PathBuf) -> Vec<HistoryEvent> {
        serde_json::from_slice(&fs::read(path).expect("read history")).expect("decode history")
    }

    fn accepted_passes(history: &[HistoryEvent]) -> Vec<&HistoryEvent> {
        history
            .iter()
            .filter(|event| {
                event.kind == "turn_result"
                    && event
                        .payload
                        .as_ref()
                        .and_then(|payload| payload.action_status.as_deref())
                        == Some("accepted")
                    && event
                        .payload
                        .as_ref()
                        .and_then(|payload| payload.action.as_ref())
                        .map(|action| action.kind.as_str())
                        == Some("pass")
            })
            .collect()
    }

    fn accepted_actions(history: &[HistoryEvent]) -> Vec<&HistoryEvent> {
        history
            .iter()
            .filter(|event| {
                event.kind == "turn_result"
                    && event
                        .payload
                        .as_ref()
                        .and_then(|payload| payload.action_status.as_deref())
                        == Some("accepted")
            })
            .collect()
    }

    fn repo_relative(repo: &Path, path: &Path) -> String {
        let relative = path.strip_prefix(repo).expect("path under repo");
        format!("./{}", relative.display())
    }

    fn write_json(path: &Path, value: &serde_json::Value) {
        fs::write(path, serde_json::to_vec_pretty(value).expect("json")).expect("write json");
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::OnceLock;

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

    #[test]
    #[ignore = "requires pinned tagged arena-runner install"]
    fn tagged_runner_completes_with_first_legal_fixtures() {
        let repo = repo_root();
        let runner = runner_bin();
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
                    repo_relative(&repo, &repo.join("target/debug/reversi-legal-move-first"))
                ),
                "--player",
                &format!(
                    "p2={}",
                    repo_relative(&repo, &repo.join("target/debug/reversi-legal-move-first"))
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
    fn tagged_runner_replays_both_scripted_completion_lines() {
        let repo = repo_root();
        let runner = runner_bin();
        let lines = load_script_lines(&repo);

        for (index, line) in lines.iter().enumerate() {
            let fixture_dir = prepare_fixture_dir(&repo);
            let output_dir = fixture_dir.join(format!("artifacts-scripted-{}", index + 1));
            let manifest_path = write_game_master_manifest(&fixture_dir, &repo);
            let (black_moves, white_moves) = split_moves(line);
            let p1_entry =
                write_scripted_player_manifest(&fixture_dir, &repo, "black", &black_moves);
            let p2_entry =
                write_scripted_player_manifest(&fixture_dir, &repo, "white", &white_moves);

            run_runner(
                &runner,
                &repo,
                &[
                    "--game-master-manifest",
                    &repo_relative(&repo, &manifest_path),
                    "--match-id",
                    &format!("reversi-scripted-{}", index + 1),
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

            let match_dir = output_dir.join(format!("reversi-scripted-{}", index + 1));
            let summary = read_summary(match_dir.join("result-summary.json"));
            assert_eq!(summary.status, "completed");
            let exported = read_exported_snapshot(match_dir.join("exported-snapshot.json"));
            let public_state = exported.public_state.expect("public state");
            assert!(public_state.completed);
            assert_eq!(
                public_state.scores.black as u16 + public_state.scores.white as u16,
                64
            );
        }
    }

    #[test]
    #[ignore = "requires pinned tagged arena-runner install"]
    fn illegal_pass_with_legal_move_causes_immediate_loss() {
        let repo = repo_root();
        let runner = runner_bin();
        let fixture_dir = prepare_fixture_dir(&repo);
        let output_dir = fixture_dir.join("artifacts-illegal-pass");
        let manifest_path = write_game_master_manifest(&fixture_dir, &repo);
        let invalid_entry = write_scripted_player_manifest(&fixture_dir, &repo, "invalid", "pass");

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
                    repo_relative(&repo, &repo.join("target/debug/reversi-legal-move-first"))
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

    fn prepare_fixture_dir(repo: &Path) -> PathBuf {
        static BUILD_ONCE: OnceLock<()> = OnceLock::new();
        BUILD_ONCE.get_or_init(|| {
            build_binary(repo, "reversi-gamemaster");
            build_binary(repo, "reversi-legal-move-first");
            build_binary(repo, "reversi-scripted-player");
        });
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

    fn build_binary(repo: &Path, bin: &str) {
        let output = Command::new("cargo")
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
        repo: &Path,
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
                "command": [repo.join("target/debug/reversi-scripted-player").display().to_string(), "--moves", moves]
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

    fn load_script_lines(repo: &Path) -> Vec<String> {
        [
            repo.join("testdata/reversi/scripted-games/line-1.txt"),
            repo.join("testdata/reversi/scripted-games/line-2.txt"),
        ]
        .into_iter()
        .map(|path| {
            fs::read_to_string(path)
                .expect("read line")
                .trim()
                .to_string()
        })
        .collect()
    }

    fn split_moves(line: &str) -> (String, String) {
        let mut black = String::new();
        let mut white = String::new();
        let bytes = line.as_bytes();
        let mut index = 0usize;
        let mut turn = 0usize;
        while index < bytes.len() {
            let token = &line[index..index + 2];
            if turn.is_multiple_of(2) {
                black.push_str(token);
            } else {
                white.push_str(token);
            }
            turn += 1;
            index += 2;
        }
        (black, white)
    }

    fn read_summary(path: PathBuf) -> ResultSummary {
        serde_json::from_slice(&fs::read(path).expect("read summary")).expect("decode summary")
    }

    fn read_exported_snapshot(path: PathBuf) -> ExportedSnapshot {
        serde_json::from_slice(&fs::read(path).expect("read exported snapshot"))
            .expect("decode exported snapshot")
    }

    fn repo_relative(repo: &Path, path: &Path) -> String {
        let relative = path.strip_prefix(repo).expect("path under repo");
        format!("./{}", relative.display())
    }

    fn write_json(path: &Path, value: &serde_json::Value) {
        fs::write(path, serde_json::to_vec_pretty(value).expect("json")).expect("write json");
    }
}

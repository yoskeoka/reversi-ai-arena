# Reversi AI Arena

`reversi-ai-arena` hosts the Reversi-specific game master, AI players, replay
visualizer, fixtures, and public examples that integrate with `ai-arena`.

Current verification entrypoints:

- `make verify-rust`
- `make verify-release-artifacts` with the pinned runner and prepared XOT inputs
- `make verify-workflows`

## Run The Public Replay Visualizer

The viewer is a separate Vite application. It starts with its built-in terminal
fixture, so no `ai-arena` service is needed for the first local view:

```sh
cd visualizer
npm ci
npm run dev
```

Open `http://127.0.0.1:4173`.

For development connections to local, staging, or production `ai-arena`, see
[`DEVELOPMENT.md`](DEVELOPMENT.md). See
[`visualizer/README.md`](visualizer/README.md) for visualizer-specific notes.

## Build And Validate Official Artifacts

The official Reversi game master and reference AI are separate WASM/WASI
`arena-bundle/v1` ZIPs. Build deterministic development artifacts and validate
the exact bytes with the pinned upstream contract:

```sh
tools/release-packager/fetch-xot.sh .tmp/xot
XOT_OPENINGS_DIR="$PWD/.tmp/xot" AI_ARENA_DIR=/path/to/ai-arena make verify-release-artifacts
```

Use an ai-arena v0.3.0 checkout at `212839f537274d8874a2e29afc2bb292cd842dd0`.
Verification builds `dist/reversi-game-dev.arena.zip`,
`dist/reversi-rust-reference-ai-dev.arena.zip`, and `dist/SHA256SUMS`,
then exercises these exact ZIPs through the upstream WASI runner and
registration services. It requires `jq`. Register the same game bundle for
`standard`, `standard-xot-large` (the default XOT choice), or `standard-xot-small`.
For tagged operator packaging and XOT data provenance, see the
[XOT reference](docs/references/xot.md).

## Run A Native Development Match Through `arena-runner`

The native manifest overlay below is only a local development convenience. It
does not replace bundle validation or staging with the official WASM artifacts.

1. Install the pinned runner:

   ```sh
   go install github.com/yoskeoka/ai-arena/cmd/arena-runner@v0.2.0
   ```

2. Build the local Reversi artifacts:

   ```sh
   cargo build --bin reversi-gamemaster
   cargo build --target wasm32-wasip1 -p reversi-rust-reference-player --bin reversi-rust-reference-player
   ```

3. Write a local game-master manifest:

   ```sh
   mkdir -p ./.tmp
   printf '%s\n' \
     '{' \
     '  "metadata": {' \
     '    "game_id": "reversi",' \
     '    "game_version": "1.2.0",' \
     '    "ruleset_version": "standard"' \
     '  },' \
     '  "runtime": {' \
     '    "kind": "local-subprocess",' \
     "    \"command\": [\"$PWD/target/debug/reversi-gamemaster\"]" \
     '  }' \
     '}' > ./.tmp/reversi-gamemaster-manifest.json
   ```

   For now, use an absolute path in `runtime.command[0]`. Runner-side support
   for manifest-relative game-master command resolution is tracked as a
   follow-up plan in `ai-arena`.

4. Write a local Rust reference-player sidecar entry:

   ```sh
   mkdir -p ./.tmp/rust-reference-player
   : > ./.tmp/rust-reference-player/reversi-rust-reference-player
   cp ./target/wasm32-wasip1/debug/reversi-rust-reference-player.wasm ./.tmp/rust-reference-player/
   ```

   ```json
   {
     "ai_id": "rust-reference",
     "protocol": {
       "transport": "stdio-jsonrpc-ndjson",
       "game_id": "reversi",
       "game_version": "1.2.0",
       "ruleset_version": "standard"
     },
     "runtime": {
       "kind": "wasm-wasi",
       "module": "./reversi-rust-reference-player.wasm",
       "args": ["./reversi-rust-reference-player.wasm"],
       "memory_limit_pages": 64
     }
   }
   ```

   Save it at `./.tmp/rust-reference-player/reversi-rust-reference-player.arena.json`.

   For official release identity, tag and bundle versioning policy, see
   [submittable artifacts](docs/specs/submittable-artifacts.md).

5. Launch a local match from the repository root:

   ```sh
   arena-runner \
     --game-master-manifest ./.tmp/reversi-gamemaster-manifest.json \
     --match-id local-dev \
     --output-dir ./.tmp/arena-runner-output \
     --log-output none \
     --player p1=./.tmp/rust-reference-player/reversi-rust-reference-player \
     --player p2=./.tmp/rust-reference-player/reversi-rust-reference-player
   ```

This produces the standard runner artifact set under
`./.tmp/arena-runner-output/local-dev/`.

## Export A Kifu From Runner Artifacts

Build or run the helper from the repository root:

```sh
cargo run --bin reversi-kifu-export -- ./.tmp/arena-runner-output/local-dev
```

The default output is the compact shared notation that omits explicit `pass`
turns.

To keep forced-pass turns in the exported transcript:

```sh
cargo run --bin reversi-kifu-export -- --include-pass ./.tmp/arena-runner-output/local-dev
```

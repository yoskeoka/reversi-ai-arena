# Reversi AI Arena

`reversi-ai-arena` hosts the Reversi-specific game master, AI players, replay
visualizer, fixtures, and public examples that integrate with `ai-arena`.

Current verification entrypoints:

- `make verify-rust`
- `make verify-workflows`

## Run A Local Match Through `arena-runner`

The Phase 1 and Phase 2 integration path uses the tagged external runner plus a
Reversi-owned game-master manifest overlay.

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
     '    "game_version": "1.0.0",' \
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
       "game_version": "1.0.0",
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

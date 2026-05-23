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
   cargo build --bin reversi-gamemaster --bin reversi-legal-move-first
   ```

3. Write a local game-master manifest:

   ```json
   {
     "metadata": {
       "game_id": "reversi",
       "game_version": "1.0.0",
       "ruleset_version": "standard"
     },
     "runtime": {
       "kind": "local-subprocess",
       "command": ["../target/debug/reversi-gamemaster"]
     }
   }
   ```

4. Launch a local match from the repository root:

   ```sh
   arena-runner \
     --game-master-manifest ./tmp/reversi-gamemaster-manifest.json \
     --match-id local-dev \
     --output-dir ./tmp/arena-runner-output \
     --log-output none \
     --player p1=./target/debug/reversi-legal-move-first \
     --player p2=./target/debug/reversi-legal-move-first
   ```

This produces the standard runner artifact set under
`./tmp/arena-runner-output/local-dev/`.

## Export A Kifu From Runner Artifacts

Build or run the helper from the repository root:

```sh
cargo run --bin reversi-kifu-export -- ./tmp/arena-runner-output/local-dev
```

The default output is the compact shared notation that omits explicit `pass`
turns.

To keep forced-pass turns in the exported transcript:

```sh
cargo run --bin reversi-kifu-export -- --include-pass ./tmp/arena-runner-output/local-dev
```

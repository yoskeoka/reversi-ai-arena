# Language and Toolchain Policy

## Mainline Language Split

- Rust is the mainline implementation language for the Reversi game master and
  primary AI-player path.
- Go is an optional lightweight lane for protocol samples, fixture helpers, and
  a small reference bot.
- Browser-facing replay tooling lives under `visualizer/` and uses TypeScript
  plus Vite with Phaser as the rendering dependency.

## Toolchain Boundaries

- The Rust workspace is rooted at the repository top level and currently owns
  crates under `games/reversi/`, `players/rust-reference/`, and
  `internal/aiarena-protocol/`.
- `internal/aiarena-protocol/` is the neutral Rust compatibility crate for the
  public `ai-arena` transport contract. It may be shared by the game master and
  player surfaces, but it must stay free of Reversi rules logic.
- The repository Rust toolchain must include `rustfmt`, `clippy`, and the
  `wasm32-wasip1` target so local and CI verification can exercise the same
  baseline without ad hoc setup.
- Go support code must stay visibly secondary to the Rust mainline and should
  live under dedicated `players/` or verification-oriented directories.
- Browser tooling must remain isolated to `visualizer/` and must not drive
  repository-wide framework choices.

## Verification Expectations

- Run local Clippy from the repository root for changes to Rust sources,
  manifests, build scripts, and consumed inputs. Select all affected crates.
  If shared dependency or toolchain changes cannot be scoped, verify a
  representative set locally and record the remaining CI coverage. Changes
  with no Rust impact can skip Clippy.
- Check library and binary targets for ordinary changes, all Cargo targets for
  changes to tests, examples, benchmarks, or related conditional code, and
  `wasm32-wasip1` for WASI code. Explicitly select affected non-default
  features. Run applicable formatting, tests, WASM builds, and runner E2E.
- Go verification may use targeted `go test` commands only for the directories
  that actually contain Go support code.
- Visualizer verification may use targeted `npm` scripts inside `visualizer/`
  once dependencies and implementation are introduced.
- Changes confined to `visualizer/**` use visualizer-local checks. Changes
  to any tracked Rust workspace member, root Rust-toolchain files, or the Rust
  workflow definition run the full Rust CI gate. This includes
  `internal/aiarena-protocol/**`, which affects game-master and player transport.
- The full gate uses `make verify-rust`, also available locally, for formatting,
  workspace Clippy with all targets and warnings denied, workspace tests,
  both WASI builds, and runner E2E.
- Go support code does not require a dedicated CI lane yet.

The concrete command contract for this policy is defined in
[quality-gates.md](./quality-gates.md).

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
  crates under `games/reversi/` and `players/rust-reference/`.
- The repository Rust toolchain must include `rustfmt`, `clippy`, and the
  `wasm32-unknown-unknown` target so local and CI verification can exercise the
  same baseline without ad hoc setup.
- Go support code must stay visibly secondary to the Rust mainline and should
  live under dedicated `players/` or verification-oriented directories.
- Browser tooling must remain isolated to `visualizer/` and must not drive
  repository-wide framework choices.

## Verification Expectations

- Rust changes should be verifiable from the repository root through
  repository-local commands that wrap Cargo-based checks.
- The Rust baseline for this phase is:
  - formatting check across the workspace
  - Clippy with warnings denied across workspace targets
  - workspace Rust tests
  - WASM build verification for `players/rust-reference`
- Go verification may use targeted `go test` commands only for the directories
  that actually contain Go support code.
- Visualizer verification may use targeted `npm` scripts inside `visualizer/`
  once dependencies and implementation are introduced.
- GitHub Actions should keep CI scoped to the owning surface:
  - `visualizer/**` changes run visualizer-local checks only
  - `games/reversi/**` changes run the Reversi game-surface build only
  - `players/rust-reference/**` changes run the Rust player-surface test and
    WASM build verification path
  - `cmd/**`, root Rust-toolchain files, or the Rust workflow definition run the
    full Rust verification path because those changes can affect multiple Rust
    surfaces
- Go support code does not require a dedicated CI lane yet.

The concrete command contract for this policy is defined in
[quality-gates.md](./quality-gates.md).

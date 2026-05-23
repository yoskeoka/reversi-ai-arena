# Command Surfaces

This directory owns runnable entrypoints such as future game-master launchers,
developer tooling wrappers, and replay helpers.

Packages that implement core rules or player logic must remain outside `cmd/`.

Phase 1 adds `reversi-gamemaster` here as the thin stdio JSON-RPC launcher for
the Reversi-owned game-master implementation.

Phase 2 adds `reversi-kifu-export` here as the thin CLI that reads
`arena-runner` artifacts and prints a shareable move transcript.

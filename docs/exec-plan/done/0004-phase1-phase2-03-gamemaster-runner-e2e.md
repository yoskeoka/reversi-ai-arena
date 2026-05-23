# Phase 1/2 03 Game Master Runner E2E
**Execution**: Use `/execute-task` to implement this plan.

## Objective

Deliver Phase 1 by implementing the Rust Reversi game master, wiring it to the
Rust protocol crate, and verifying that it can be launched by a tagged
`arena-runner` host in local and CI end-to-end tests with minimal fixture
players that are sufficient to drive the game to completion.

This plan turns `reversi-ai-arena` into a continuously verifiable registered
game path rather than a local-only rules prototype.

## Confirmed Direction

- The game master is built in Rust.
- `arena-runner` must be consumed as a versioned external host, installed from
  `go install github.com/yoskeoka/ai-arena/cmd/arena-runner@v0.2.0`, then
  executed as the pinned binary on `PATH` during local and CI verification, not
  via a sibling checkout path.
- The game master binary is built in this repository and then launched through
  the tagged host.
- `dungeon-game-ai-arena` e2e structure is the direct reference for the runner
  consumption and CI ownership model.
- Phase 1 includes the minimum player assets needed to make `arena-runner`
  execute real Reversi matches:
  - a legal-move fixture bot that always selects the first legal move when the
    game master exposes legal choices
  - a deterministic scripted fixture bot that follows a fixed move list
- Reversi pass handling is fully determined by the legal-move set:
  - when `legal_actions` is empty, the current player must emit `pass`
  - when `legal_actions` is non-empty, `pass` is illegal
- Even in forced-pass turns, the game master should still issue a turn request
  and require an explicit `pass` response so move-by-move logs stay shared
  across players and each turn consumes an equivalent decision window.
- If a player has one or more legal moves and responds with timeout or illegal
  action, the game master should treat that event as an immediate loss for that
  player.
- These Phase 1 players are verification fixtures, not the main competitive AI
  implementation. Building a stronger WASM AI remains the responsibility of
  Phase 2.

## Code Changes

- Implement the Reversi rules/core match progression in `games/reversi/`.
- Add a runnable Rust game-master binary under `cmd/`, which remains the owning
  entrypoint surface in the repository-structure policy.
- Add local/e2e test helpers and golden-style assertions under:
  - `e2e/`
  - `testdata/`
- Add the minimum player fixtures needed by runner e2e under `players/` and/or
  `testdata/`, including:
  - a legal-move-first fixture bot
  - a scripted fixture bot driven by fixed move sequences
- Add repo-local commands and CI wiring for:
  - building the game master
  - installing or locating the pinned tagged `arena-runner`
  - executing end-to-end matches against produced artifacts

## Spec Changes

- Add a Reversi game-master spec covering:
  - game identity and metadata
  - board state lifecycle and pass handling
  - forced-pass turns as explicit player responses rather than silent skips
  - immediate-loss handling for timeout or illegal action when legal moves
    exist
  - exported snapshot/result expectations
  - the binary entrypoint and its runner-facing contract
- Add a tagged-runner consumption spec for this repository that fixes:
  - the `ai-arena` host version policy, starting with
    `github.com/yoskeoka/ai-arena/cmd/arena-runner@v0.2.0`
  - the single local and CI invocation strategy: install the pinned binary, then
    invoke that installed `arena-runner`
  - the minimum fixture-player contract required for Phase 1 runner e2e
  - what artifacts must exist after a successful match
- Add or extend a verification-assets spec that records the deterministic Phase
  1 fixture game lines used to prove end-to-end completion:
  - `c4e3f6e6f5c5f4g6f7g5d6d3f3c3h4h5g4h3e2f2d2d1g3e1b6c2b4e8f8g8b3h2f1g1b1a3h7a5a4b5a6a7c6e7d7d8g7a2c8b8c7c1h1h8h6g2b2b7a8a1`
  - `c4e3f4c5d6f3e6c3d3e2b6f5b4f6c2e7d2c7f1c6f2a6d7c8f8d8g5g6e8h4b8f7g8b5g3g4h3a3h5b3b7h6h7e1d1b1c1g1g2a8a7h8g7h2a4h1a5b2a2a1`
- Update [docs/project-plan.md](../../project-plan.md) during execution to mark
  Phase 1 complete only after the tagged-runner verification path is green.

## Design Decisions

- Keep the runner dependency explicit and versioned in the consumer repo.
- Verify the real external-consumer launch path early instead of relying only on
  in-process or unit-test coverage.
- Treat game-master build, runner install, and e2e artifact validation as one
  acceptance path rather than separate optional checks.
- Keep the Phase 1 player surface intentionally small: enough to validate the
  end-to-end game-master path, but not so ambitious that it duplicates the real
  AI-player work planned for Phase 2.

## Sub-tasks

- [x] Define the Phase 1 contract in specs before implementing the game master.
- [x] [parallel] Implement the Reversi core rule/state modules inside
      `games/reversi/`.
- [x] [parallel] Add the runnable game-master binary and protocol-crate
      integration.
- [x] [parallel] Add the minimum Phase 1 fixture players:
      - a legal-move-first bot
      - a scripted bot that can replay fixed move sequences
- [x] [depends on: rule/state modules, game-master binary] Add tagged-runner e2e
      tests and deterministic test fixtures.
- [x] [depends on: minimum Phase 1 fixture players] Add deterministic verification
      coverage for the two fixed completion lines listed in this plan.
- [x] [depends on: tagged-runner e2e] Add CI wiring that installs the pinned
      `arena-runner` host and runs the same end-to-end path.
- [x] [depends on: green e2e path] Update `docs/project-plan.md` to mark
      Phase 1 complete.

## Parallelism

- Rules implementation and binary wiring can proceed in parallel after the
  Phase 1 spec and protocol crate are available.
- Minimum fixture-player implementation can proceed in parallel with the game
  master once the transport contract is fixed.
- E2E verification depends on the game-master binary plus deterministic fixture
  players and fixed game lines.

## Verification

- Cargo unit tests cover core Reversi rules and state transitions.
- The game-master binary builds from the repo root.
- The legal-move-first fixture bot can finish a match by consuming legal moves
  from the game master.
- Forced-pass turns still flow through explicit player `turn` requests and
  `pass` responses in the resulting records.
- A player that has legal moves but times out or emits an illegal action loses
  immediately, and that outcome is covered by deterministic verification.
- The scripted fixture bot can replay the two fixed game lines listed in this
  plan and reach terminal state consistently.
- The tagged `arena-runner` host can complete a Reversi match against local
  players and produce the expected artifacts.
- CI runs the same pinned-host end-to-end flow.

## Out of Scope

- Mainline Rust WASM AI-player logic beyond the minimum Phase 1 fixture players
- Reference-bot fixture caching policy beyond what Phase 1 e2e strictly needs
- Replay visualizer and watcher flows

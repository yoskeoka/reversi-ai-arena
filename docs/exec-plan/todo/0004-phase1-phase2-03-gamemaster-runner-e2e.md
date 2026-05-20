# Phase 1/2 03 Game Master Runner E2E
**Execution**: Use `/execute-task` to implement this plan.

## Objective

Deliver Phase 1 by implementing the Rust Reversi game master, wiring it to the
Rust protocol crate, and verifying that it can be launched by a tagged
`arena-runner` host in local and CI end-to-end tests.

This plan turns `reversi-ai-arena` into a continuously verifiable registered
game path rather than a local-only rules prototype.

## Confirmed Direction

- The game master is built in Rust.
- `arena-runner` must be consumed as a versioned external host, installed from
  `go install ...@v0.1.x` or otherwise pinned to that exact tag family during
  verification, not via a sibling checkout path.
- The game master binary is built in this repository and then launched through
  the tagged host.
- `dungeon-game-ai-arena` e2e structure is the direct reference for the runner
  consumption and CI ownership model.

## Code Changes

- Implement the Reversi rules/core match progression in `games/reversi/`.
- Add a runnable Rust game-master binary under `cmd/` or another thin entrypoint
  surface that matches the repository-structure policy.
- Add local/e2e test helpers and golden-style assertions under:
  - `e2e/`
  - `testdata/`
- Add repo-local commands and CI wiring for:
  - building the game master
  - installing or locating the pinned tagged `arena-runner`
  - executing end-to-end matches against produced artifacts

## Spec Changes

- Add a Reversi game-master spec covering:
  - game identity and metadata
  - board state lifecycle and pass handling
  - exported snapshot/result expectations
  - the binary entrypoint and its runner-facing contract
- Add a tagged-runner consumption spec for this repository that fixes:
  - the `ai-arena` host version policy
  - local and CI invocation strategy
  - what artifacts must exist after a successful match
- Update [docs/project-plan.md](../../project-plan.md) during execution to mark
  Phase 1 complete only after the tagged-runner verification path is green.

## Design Decisions

- Keep the runner dependency explicit and versioned in the consumer repo.
- Verify the real external-consumer launch path early instead of relying only on
  in-process or unit-test coverage.
- Treat game-master build, runner install, and e2e artifact validation as one
  acceptance path rather than separate optional checks.

## Sub-tasks

- [ ] Define the Phase 1 contract in specs before implementing the game master.
- [ ] [parallel] Implement the Reversi core rule/state modules inside
      `games/reversi/`.
- [ ] [parallel] Add the runnable game-master binary and protocol-crate
      integration.
- [ ] [depends on: rule/state modules, game-master binary] Add tagged-runner e2e
      tests and deterministic test fixtures.
- [ ] [depends on: tagged-runner e2e] Add CI wiring that installs the pinned
      `arena-runner` host and runs the same end-to-end path.
- [ ] [depends on: green e2e path] Update `docs/project-plan.md` to mark
      Phase 1 complete.

## Parallelism

- Rules implementation and binary wiring can proceed in parallel after the
  Phase 1 spec and protocol crate are available.
- E2E verification depends on both the game-master binary and deterministic
  fixture shape.

## Verification

- Cargo unit tests cover core Reversi rules and state transitions.
- The game-master binary builds from the repo root.
- The tagged `arena-runner` host can complete a Reversi match against local
  players and produce the expected artifacts.
- CI runs the same pinned-host end-to-end flow.

## Out of Scope

- Mainline Rust WASM AI-player logic
- Reference-bot fixture caching policy beyond what Phase 1 e2e strictly needs
- Replay visualizer and watcher flows

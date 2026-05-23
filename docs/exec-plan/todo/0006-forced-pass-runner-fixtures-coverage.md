# Forced-Pass Runner Fixtures Coverage
**Execution**: Use `/execute-task` to implement this plan.

Addresses: `docs/issues/0002-add-forced-pass-runner-fixtures.md`

## Objective

Close the remaining Phase 1 runner-verification gap around explicit forced-pass
turns and terminal double-pass completion by adding deterministic fixtures and
tagged-runner assertions that exercise those paths through the real manifest
overlay flow.

This plan strengthens the repository's replay-safe verification assets without
expanding scope into new runtime modes or non-deterministic scenario setup.

## Confirmed Direction

- Reuse the existing scripted-fixture lane before introducing any new fixture
  transport or custom snapshot-injection path.
- Keep forced-pass coverage inside the tagged `arena-runner` verification path
  so local and CI continue to prove the same external-consumer contract.
- Prefer checked-in deterministic move lines under `testdata/` for canonical
  pass scenarios, with test-generated temporary manifests only for launch
  wiring.
- Treat terminal double-pass completion as an artifact-level contract: the
  runner result must complete normally, preserve the pass turns in history, and
  allow terminal empty cells when the game ends by consecutive passes rather
  than full-board fill.

## Code Changes

- Extend deterministic scripted Reversi fixtures under `testdata/reversi/`
  with at least:
  - one line that reaches a forced-pass turn
  - one line that ends with consecutive forced passes before the board is full
- Update tagged-runner e2e coverage in `e2e/reversi-runner/src/lib.rs` to:
  - replay the new pass-specific fixtures
  - assert the forced-pass player must emit `pass`
  - assert normal completion for terminal double-pass endings
  - verify artifact details beyond `completed`, including preserved pass turns
    and non-full-board terminal state when applicable
- Update fixture helpers only if the current scripted-player manifest or move
  parser cannot express explicit `pass` turns cleanly.

## Spec Changes

- Update `docs/specs/verification-assets.md` to record the canonical
  forced-pass scripted lines and what each line proves.
- Update `docs/specs/tagged-runner-consumption.md` to make pass-specific runner
  assertions explicit for the scripted-fixture lane.
- Update `docs/specs/reversi-game-master.md` to clarify the verification bar
  for:
  - explicit forced-pass turn requests
  - terminal double-pass completion with remaining empty cells
  - artifact/history expectations for those turns

## Design Decisions

- Do not add a synthetic snapshot/bootstrap test path just to force pass
  positions; existing scripted fixtures are the preferred durable asset shape
  for this repository.
- Keep this work as Phase 1 verification debt payoff rather than folding it
  into unrelated AI-player or visualizer plans.

## Sub-tasks

- [ ] Define the forced-pass fixture contract in specs before changing tests.
- [ ] [parallel] Curate deterministic scripted lines from the tracked kifu
      candidates and commit them under `testdata/reversi/`.
- [ ] [parallel] Audit the current scripted-player helper to confirm explicit
      `pass` tokens are already supported, or note the smallest helper change
      required.
- [ ] [depends on: forced-pass fixture contract, scripted lines] Extend
      `e2e/reversi-runner` coverage for forced-pass and terminal double-pass
      scenarios.
- [ ] [depends on: e2e coverage] Verify the assertions distinguish:
      - illegal `pass` when legal moves exist
      - required `pass` when legal moves are empty
      - normal terminal completion with empty cells remaining after consecutive
        passes

## Parallelism

- Scripted-line curation and helper-capability audit can run in parallel after
  the spec contract is fixed.
- Runner-test changes depend on the canonical forced-pass fixture lines being
  chosen, but not on any unrelated AI-player or visualizer work.

## Verification

- Unit or helper-level coverage proves the scripted-fixture lane can express
  explicit `pass` turns.
- Tagged-runner e2e completes at least one forced-pass scripted match through
  the manifest-backed game master.
- Tagged-runner artifacts show the forced-pass turn in replay-safe history.
- Tagged-runner e2e proves terminal completion after consecutive forced passes
  even when the final board still contains empty cells.
- Existing illegal-pass immediate-loss coverage remains intact and clearly
  separated from the new required-pass success path.

## Out of Scope

- New runtime modes or runner CLI flags
- Snapshot-resume or injected-start-state test flows
- Mainline AI-player behavior changes
- Replay visualizer work

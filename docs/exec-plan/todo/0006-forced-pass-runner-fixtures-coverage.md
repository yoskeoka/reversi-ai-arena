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
- The canonical scripted-fixture suite for this plan comes from
  `docs/issues/0002-add-forced-pass-runner-fixtures.md`:
  - `end with 1 empty cell(forced-pass for both)` for the terminal
    double-pass case with one empty square remaining
  - `fastest black win` as a short deterministic completion line in the same
    scripted-fixture batch
  - `short white win` as the corresponding short deterministic white-win line
  - `multiple passes in the middle and ends with some empty cells` for a
    non-terminal forced pass before the final consecutive-pass ending
- Treat the full four-case issue suite as the acceptance set for this plan.
  Once all four scripted cases are added to runner coverage, this repository
  counts the forced-pass fixture gap as closed.
- Treat terminal double-pass completion as an artifact-level contract: the
  runner result must complete normally, preserve the pass turns in history, and
  allow terminal empty cells when the game ends by consecutive passes rather
  than full-board fill.

## Canonical Kifu

- `end with 1 empty cell(forced-pass for both)`
  `f5f6e6f4e3c5c4d6b5d3c3e2f2c2d2b3b4f3c1e1g3g4h4h5c6h3g5f1c7a4a5h6d7g6a3e7f8d1f7b1g2b2h2h1g1b8c8e8d8g8a1a2h7h8g7b7b6a6a7`
- `fastest black win`
  `f5d6c5f4e7f6g5e6e3`
- `short white win`
  `f5f6e6d6e7f7d7f4c5c7c6b6`
- `multiple passes in the middle and ends with some empty cells`
  `d3c5f6f5e6e3d6f7b6e7f3c6d7c8g5f4g7g6e8c7d8h8b5f2h5h4f1g4h7h6b8g3g2h3h2c4b3g8f8a8b4c3h1a3g1e2e1b7c2d1a7d2a6c1a4a5a2`

## Code Changes

- Extend deterministic scripted Reversi fixtures under
  `testdata/reversi/scripted-games/` with all four issue-tracked cases:
  - the issue-tracked line `end with 1 empty cell(forced-pass for both)`
  - the issue-tracked line `fastest black win`
  - the issue-tracked line `short white win`
  - the issue-tracked line `multiple passes in the middle and ends with some
    empty cells`
- Update tagged-runner e2e coverage in `e2e/reversi-runner/src/lib.rs` to:
  - replay the new pass-specific fixtures
  - assert the forced-pass player must emit `pass`
  - assert normal completion for terminal double-pass endings
  - verify artifact details beyond `completed`, including preserved pass turns
    and non-full-board terminal state when applicable
- Update the scripted-fixture parsing/scheduling helpers so canonical fixture
  lines can encode explicit `pass` tokens without assuming fixed two-character
  move chunks.

## Spec Changes

- Update `docs/specs/verification-assets.md` to record the canonical
  four-case scripted suite and what each line proves.
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
      candidates and commit them under
      `testdata/reversi/scripted-games/`, using all four issue-tracked cases
      as the required initial suite.
- [ ] [parallel] Audit the current scripted-player and `e2e/reversi-runner`
      token-splitting helpers, then define the smallest format/parser change
      needed so deterministic line fixtures can include explicit `pass`.
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
- Tagged-runner e2e covers the issue's `fastest black win` and `short white
  win` cases as part of the required scripted-fixture batch for this plan.
- Tagged-runner e2e covers the issue's `multiple passes in the middle and ends
  with some empty cells` case as the success path for explicit forced-pass
  turns before terminal state.
- Tagged-runner artifacts show the forced-pass turn in replay-safe history.
- Tagged-runner e2e proves terminal completion for the issue's
  `end with 1 empty cell(forced-pass for both)` case, including the remaining
  empty square after consecutive forced passes.
- The forced-pass fixture gap is considered closed once all four issue-tracked
  scripted cases are committed and covered by tagged-runner verification.
- Existing illegal-pass immediate-loss coverage remains intact and clearly
  separated from the new required-pass success path.

## Out of Scope

- New runtime modes or runner CLI flags
- Snapshot-resume or injected-start-state test flows
- Mainline AI-player behavior changes
- Replay visualizer work

# Visualizer replay summary
> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

Addresses: https://github.com/yoskeoka/ai-arena/issues/362

## Objective and completion boundary

Make a completed Reversi replay easy to identify and navigate. Discovery shows each supported completed match with a short match identifier and its completed timestamp, newest first. The replay summary identifies Black and White by public bot name and short immutable revision, removes the redundant terminal-replay label, provides direct initial/final state controls, and explicitly says that the match has ended when the final frame is selected.

The viewer continues to use only anonymous public spectator endpoints with credentials omitted. It neither exposes run IDs nor accesses private records, artifacts, or operator endpoints.

## References and current behavior

- `visualizer/src/api/public-api.ts:11-29` models the current public list and sorts supported matches lexically by `match_id`.
- `visualizer/src/main.ts:27-51` renders a full-ID match option, generic colour/score summary, the permanent `Completed terminal replay` label, and previous/play/next controls.
- `visualizer/src/replay/model.ts:36-55` includes the opening as frame zero and computes a separate terminal frame; UI navigation must add that terminal frame to its selectable sequence rather than treating `frames` as the final state.
- `visualizer/e2e/replay.spec.ts:3-38` asserts the existing terminal label and playback controls.
- `docs/specs/visualizer-architecture.md:25-39` defines public-only discovery and semantic controls.
- ai-arena `typespec/namespaces/public/api.tsp:15-20` currently lacks completion and player identity metadata. Dependency issue [ai-arena#362](https://github.com/yoskeoka/ai-arena/issues/362) defines the required versioned public-contract extension.

## Change map

- (MODIFY) `docs/specs/visualizer-architecture.md`: define public completed-match identity metadata, descending completion ordering, short-identifier presentation, colour/player summary, and initial/final/terminal control behavior.
- (MODIFY) `visualizer/src/api/public-api.ts`: model and strictly validate the new public completion and Black/White participant metadata on list and detail responses; sort supported records by immutable completion timestamp descending and expose shared short-ID/label helpers.
- (MODIFY) `visualizer/src/main.ts` and `visualizer/src/style.css`: render formatted dropdown options and accessible replay summary/navigation controls; use the explicit terminal final state rather than a permanently rendered terminal label.
- (MODIFY) `visualizer/src/replay/model.ts` and `visualizer/src/replay/model.test.ts`: expose one immutable selectable-frame sequence containing the opening, every accepted turn, and the final validated terminal state exactly once.
- (MODIFY) `visualizer/src/api/public-api.test.ts` and `visualizer/e2e/replay.spec.ts`: cover metadata decoding/order for list and detail loads, compact labels, initial/final jumps, terminal wording, and no regression to private/run controls.
- (MODIFY) `visualizer/README.md` and `DEVELOPMENT.md`: document the visible completed-match labels and that live public API compatibility requires ai-arena#362.
- (NO CHANGE) `visualizer/src/renderer/board.ts` and Reversi rules: this delivery changes replay controls and metadata presentation only.

## Black-box specification changes

1. Every completed public match list and detail response includes `completed_at` as an RFC 3339 UTC timestamp and `participants` with required `black` and `white` objects. Each object has required non-empty `bot_name` and `ai_submission_id` strings. State responses inherit the same fields through the public match model. Completion time is fixed on transition to a completed terminal selected run; mutable storage update time is not an equivalent. Missing, empty, or invalid required metadata makes the record unsupported and the viewer shows an explainable error for a selected deep link.
2. The completed-match selector displays `match-` plus the first eight hexadecimal characters when its ID is `match-` followed by a canonical UUID (for example, `match-a2471327`). For another ID, it displays the escaped full ID without truncation. A revision uses the first eight hexadecimal characters only when it is a canonical UUID; otherwise it displays its escaped full value. Option values and deep links always use full match IDs. The selector orders candidates newest completion first, with a deterministic full-ID tie-breaker.
3. The replay summary renders colour, bot name, and revision identifier, for example `black (hoge1:e651021e) 39`. It never replaces missing required participant data with a colour-only fallback.
4. Playback includes `Initial state` before `Previous turn` and `Final state` after `Next turn`. Initial selects the opening frame; final selects the validated terminal frame. When that terminal frame is selected, the status says the match has ended and does not present a next-player concept.
5. The phrase `Completed terminal replay` is not displayed.

## Work

1. Implement and land ai-arena#362 first: extend its TypeSpec/public projection and generated contract, persist an immutable terminal completion time, and expose colour-associated bot display names and submission revisions only through the anonymous public match resource. Verify the current selected-run consistency and public-only boundary there.
2. Update this repository's visualizer architecture specification before TypeScript. State the exact public metadata required, display truncation semantics, timestamp ordering, and semantic control/terminal behavior.
3. Extend the browser API client types and validation for the landed fields on both list and `/matches/{id}` detail responses. Reject or explain malformed metadata instead of silently sorting on mutable timestamps; retain full IDs for requests and URL parameters. Implement deterministic newest-first ordering.
4. Refactor the shell's match-option formatter and replay summary. Escape all displayed public strings. Shorten only canonical UUID forms. Preserve the complete ID as the option value, and display Black/White bot/revision labels alongside each score.
5. Update the replay model to expose one selectable sequence that begins at the opening and ends at the separately validated terminal state. Add initial/final jump buttons around the existing step controls. Ensure playback reaching that final state updates the same terminal message, prevents an invented next player, and retains keyboard-accessible semantic buttons.
6. Update API/unit/browser fixtures to include the expanded public fields. Assert timestamp ordering, list/detail rejection, non-UUID presentation, displayed text, terminal-frame navigation, and removal of the obsolete terminal label. Keep request assertions proving the visualizer never calls non-public routes.
7. Update short local/staging usage guidance and compatibility note. Do not add a client-side workaround for API bases that have not yet deployed ai-arena#362.

## Dependencies and parallelism

ai-arena#362 is a hard upstream contract dependency for timestamps and participant metadata. Its PR must merge and deploy to the selected API base before end-to-end staging acceptance; visualizer UI work may use contract fixtures after the TypeSpec shape is reviewed, but must not claim live staging support before that deployment.

After the API shape is settled, client decoding/tests and shell/CSS work can proceed in parallel. Browser tests follow both. Ruleset selection (#41), live polling/streaming, replay rule semantics, run promotion, and private artifact access remain out of scope.

## Verification

- ai-arena: generated public contract and route tests prove immutable `completed_at`, colour-associated public participant metadata, no private fields, selected-run consistency, and terminal-time persistence.
- Visualizer API tests: accept valid expanded records in both list and detail loads; reject/filter missing or malformed data; newest-first order with deterministic tie-break; full IDs remain request values while canonical UUID labels use short IDs.
- Visualizer UI tests: display completion timestamp and bot/revision score labels; initial/previous/play/next/final controls select the correct frames; final state announces completion without a next player; obsolete terminal label is absent.
- Playwright: mocked staging contract verifies selector order, deep-link full match ID, no operator/artifact/run-control request, and keyboard-accessible state jumps.
- Run `npm run typecheck`, `npm run test`, `npm run build`, relevant Playwright coverage, and the repository workflow lint. Perform the required live staging check only after ai-arena#362 is deployed.

## Non-goals

- Inferring bot names, revisions, or completion time from private data or mutable timestamps.
- Run-ID selection, live watching, dynamic ruleset/game-major selection, analysis overlays, or Reversi rule changes.
- Changing the public API from this repository; that contract is owned and delivered through ai-arena#362.

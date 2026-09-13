# replay-discovery-controls
> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

Addresses: N/A

## Objective and completion boundary

Make the externally hosted Reversi replay visualizer usable without manually constructing a `match` URL: the user selects an ai-arena base URL and then a completed public Reversi match. Loading that match continues to use ai-arena's automatic official/current selected run; the viewer never presents a run-ID selector.

The base URL selector contains `local`, `stg`, and `prod`. A valid `?api=` URL, even if not a built-in option, is represented as the selected custom value on initial load. A `?match=` remains a deep-link default, with its associated API base. Ruleset selection is explicitly deferred to related issue #41; this delivery supports the current standard ruleset only.

## References and current behavior

- `visualizer/src/main.ts:1-20` chooses fixture mode unless both `api` and `match` URL parameters are present.
- `visualizer/src/api/public-api.ts:3-14` loads one public match, state, and replay with credential-free requests, and validates selected-run consistency.
- ai-arena `typespec/namespaces/public/api.tsp:10-27,54-56` already lists public matches with `match_id`, `selected_run_id`, game metadata, and lifecycle. No new ai-arena API is required.
- `docs/specs/visualizer-architecture.md:28-39` establishes public-only inputs, terminal replay validation, and separate external hosting.
- `visualizer/README.md:8-18` is the canonical local/replay entrypoint documentation.

## Change map

- (MODIFY) `docs/specs/visualizer-architecture.md`: define public match discovery, configured base URL profiles, custom deep-link base behavior, and fixed current-ruleset boundary.
- (MODIFY) `visualizer/src/api/public-api.ts`: model and fetch the public match list using `credentials: "omit"`; retain exact-match loading and public replay validation.
- (MODIFY) `visualizer/src/main.ts` and `visualizer/src/style.css`: replace query-only startup with semantic base/profile and completed-match controls, loading/error states, and deep-link synchronization.
- (MODIFY) `visualizer/src/**/*.test.ts` and `visualizer/e2e/*`: verify discovery filtering, profile/default precedence, custom `api` selection, match deep links, and no run selector/private request.
- (MODIFY) `visualizer/README.md`: document fixture mode, the three base profiles, custom `?api=`, and selecting a completed match.
- (NO CHANGE) replay model/rules engine: ruleset choice is deferred to #41; game ID and major remain fixed.

## Work

1. Update the visualizer behavioral spec before code. State that discovery reads only `GET /api/v1-alpha/public/matches`; it filters the returned metadata to the currently supported Reversi game/version/ruleset and completed lifecycle.
2. Centralize base-profile configuration. Provide local, staging, and production public origins; normalize values; preserve any valid `api` query URL as a selected custom option rather than silently replacing it.
3. Add the public list client and deterministic sorting/error handling. A completed candidate exposes its `match_id`; its `selected_run_id` is informational only and must not become an input control.
4. Render accessible dropdowns for base URL and match. On a selection, update the query string using `history.replaceState`/equivalent, load the chosen match through the existing public resources, and preserve deep links. Fixture mode remains available only when no API/match selection is active.
5. Keep the current replay payload's `ruleset: "standard"` validation. Do not imply multi-ruleset support in the UI; link or document #41 as deferred work.
6. Update local-run instructions with the shortest complete sequence and a deep-link example for a real match.

## Dependencies and parallelism

ai-arena's public match-list endpoint is already deployed contract input. The ai-arena companion plan `0131-ranking-completed-match-discovery` improves authenticated operator history but is independent: the visualizer must never call it. Ruleset selection is related deferred work under #41 and is not an execution dependency.

## Verification

- API-client tests: list request uses only the anonymous public route and omits credentials; filter accepts supported completed Reversi records and rejects queued, other-game, incompatible-major, and non-standard ruleset records.
- UI tests: local/stg/prod profile selection, custom `?api=` selected state, `?match=` deep-link preload, match switch query update, and accessible error/empty states.
- Playwright: select staging-equivalent mocked base then a match, replay it end-to-end, and inspect requests to prove no private artifact, cookie/token, operator route, or run-ID selection occurs.
- `npm run typecheck`, `npm run test`, `npm run build`, and applicable workflow lint from `visualizer/`/repository root.

## Non-goals

- ai-arena API additions, ranking UI changes, run promotion/selection, private artifacts, platform-hosted visualizers, event streaming, and dynamic ruleset/game-major selection.

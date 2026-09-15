# public-match-discovery-filters
> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

Addresses: `docs/issues/0004-visualizer-ruleset-selection.md`

## Objective and completion boundary

Make public replay discovery request a bounded, server-filtered Reversi scope instead of downloading a mixed global list and filtering it after a single arbitrary page. The visualizer always sends `game_id=reversi` and `game_version_major=1`, requests the platform defaults `page=1`, `limit=20`, `sort=completed_at`, and `sort_order=desc`, and adds `ruleset_version` only when the visitor selects one.

The visualizer exposes an optional ruleset filter through a URL parameter and an accessible selector. With no parameter it requests every Reversi major-1 ruleset; selecting a ruleset updates the URL, re-requests page one, and requires that ruleset in the replay payload. It preserves existing `api` and `match` deep links, never adds a run selector or credential/private request, and continues to reject a selected response missing the immutable completion timestamp or participant provenance. A Reversi match returned by staging must become selectable and replayable after the companion ai-arena API release and staging deployment.

## References and current behavior

- `visualizer/src/api/public-api.ts:4-35` models an items-only public list and calls `/api/v1-alpha/public/matches` without query parameters.
- `visualizer/src/api/public-api.ts:19-20,58-64` locally filters only `reversi` major 1 and hard-coded `standard`, then independently sorts completed records. A global first page dominated by another game can therefore appear empty.
- `visualizer/src/main.ts:15-26,28-42` reads `api` / `match`, renders semantic selectors, and keeps deep links with `history.replaceState`.
- `docs/specs/visualizer-architecture.md:22-40` defines public-only discovery, completion metadata requirements, fixed `standard` ruleset, and client-side ordering.
- `docs/issues/0004-visualizer-ruleset-selection.md` deliberately defers configurable ruleset selection; this plan resolves and deletes it during implementation.
- ai-arena plan `0132-public-match-list-filters` owns the public API's `game_id`, `game_version_major`, `ruleset_version`, page/limit, and completed-at sort contract. The visualizer must not use unauthenticated operator endpoints as a substitute.

## Change map

- (MODIFY) `docs/specs/visualizer-architecture.md`: replace the fixed-standard/client-sorted discovery rule with server-scoped paging, URL-configurable optional ruleset, and page-one latest-completed behavior.
- (MODIFY) `visualizer/src/api/public-api.ts`: model page metadata/options, build encoded public list queries, retain credentials omission and response validation, and derive the supported selected ruleset from configuration rather than a constant.
- (MODIFY) `visualizer/src/main.ts` and `visualizer/src/style.css`: add accessible ruleset controls, synchronize `ruleset` with `api`/`match`, reset stale matches on scope changes, and render a useful empty/error page.
- (MODIFY) `visualizer/src/api/public-api.test.ts`, UI tests, and `visualizer/e2e/replay.spec.ts`: cover exact query construction, no client reliance on mixed-list ordering, optional ruleset omission/selection, URL update, selection reset, and public-only requests.
- (MODIFY) `visualizer/README.md`: document page-one discovery scope, optional `ruleset` deep link, and the ai-arena deployment dependency.
- (DELETE) `docs/issues/0004-visualizer-ruleset-selection.md`: remove it with this plan after the configurable filter is delivered.
- (NO CHANGE) Phaser rendering/replay rules, public match detail/state/replay paths, API base profiles, run selection, private artifacts, and event streaming.

## Work

1. Update the visualizer behavioral spec before TypeScript. State the fixed Reversi game/major query, optional ruleset query, page-one size 20, completed-at descending request, URL semantics, and server-owned filtering/sorting.
2. Add typed public-list query/options and response page metadata. Serialize only supported query values using `URL` / `URLSearchParams`; keep `credentials: "omit"`; validate that returned items match the requested scope before exposing them.
3. Replace the hard-coded `standard` candidate predicate with a configured ruleset predicate. Omitted ruleset permits all Reversi major-1 results; a specified ruleset requires both list and replay payload identity to agree. Preserve strict `completed_at` / participant validation.
4. Add a semantic ruleset selector and `?ruleset=` deep-link handling. Changing base or ruleset clears `match`, reloads first-page discovery, and never changes a custom API base to a profile. Keep fixture mode only when neither external selection is active.
5. Extend unit and end-to-end fixtures with mixed games, major versions, multiple rulesets, more than one page worth of global records, valid page metadata, and one stage-equivalent Reversi replay. Prove the network request carries the scope rather than relying on local filtering.
6. Update local documentation. After ai-arena release/deployment, run the local visualizer against stg and capture the latest completed standard Reversi match selection and replay as the manual acceptance check.

## Dependencies and parallelism

The implementation depends on ai-arena `0132-public-match-list-filters` being merged and available in staging. Unit/UI work against contract fixtures can proceed in parallel, but the visualizer must not merge a call to query parameters before that public API is released. Deploy acceptance is serial: ai-arena current SHA and a public replay-capable completed Reversi record first, then the local visualizer.

## Verification

- TypeScript API tests verify encoded `game_id=reversi`, `game_version_major=1`, `page=1`, `limit=20`, `sort=completed_at`, `sort_order=desc`, optional `ruleset_version`, and omitted credentials.
- UI/e2e tests verify a mixed global corpus cannot hide scoped Reversi results, URL deep links, ruleset changes clear stale match selection, selected responses remain metadata-safe, and no private/operator/run-selection request occurs.
- Run `npm run typecheck`, `npm run test`, `npm run build`, `npm run test:e2e`, and applicable workflow verification with a writable npm cache.
- Manual staging proof uses `?api=https%3A%2F%2Fai-arena-staging-p4ml.onrender.com` (and optional `ruleset=standard`) to select and render a completed Reversi match returned by the filtered first page.

## Non-goals

- Client-driven global pagination, arbitrary game/major selection, multi-column sort UI, cursor pagination, current live-match discovery, replay analysis tools, event streaming, private artifacts, and service deployment implementation.

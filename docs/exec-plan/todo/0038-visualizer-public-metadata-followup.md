# Visualizer public metadata follow-up
> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

Addresses: https://github.com/yoskeoka/ai-arena/pull/366

## Objective and completion boundary

After ai-arena #366 has merged and the separate runtime metadata implementation has deployed to the selected API base, make the Reversi visualizer consume that metadata defensively. #366 supplies the participant-order contract only. The browser accepts only valid completed standard-Reversi records, orders them by actual completion instants, compares equivalent metadata structurally, and retains full match IDs in deep links.

This plan consumes ai-arena's generic public metadata contract. It does not add Reversi rules, colour assignment, schema fields, or routes to ai-arena.

## References

- `visualizer/src/api/public-api.ts:20-58` validates discovery and detail/state metadata.
- `visualizer/src/api/public-api.test.ts` covers list validation and labels but lacks detail/state rejection cases.
- `visualizer/e2e/replay.spec.ts:14-39` verifies public-only discovery but must assert the full deep-link ID.
- `docs/specs/visualizer-architecture.md` owns the public-only and malformed-metadata boundary.

## Change map

- (MODIFY) `docs/specs/visualizer-architecture.md`: state strict RFC 3339 instant validation and structural metadata equality.
- (MODIFY) `visualizer/src/api/public-api.ts`: guard unknown list items, validate calendar timestamps, sort by parsed instants, and compare participant fields without JSON serialization ordering.
- (MODIFY) `visualizer/src/api/public-api.test.ts`: cover valid and rejected detail/state metadata plus malformed list items and fractional-second ordering.
- (MODIFY) `visualizer/e2e/replay.spec.ts`: assert that the `match` query parameter remains the exact full ID.

## Black-box specification changes

1. Invalid RFC 3339 UTC calendar timestamps and malformed list items are unsupported without preventing valid matches from being discovered.
2. Completion ordering compares the complete RFC 3339 fractional-second text after the shared UTC second, then full match ID; it does not reduce precision to JavaScript milliseconds.
3. Detail and state metadata must have equal values, independent of JSON property order. Their participant sequence remains Black then White under the deployed ai-arena contract.

## Work and verification

1. Update the architecture specification before TypeScript.
2. Implement the bounded decoder and comparison changes without private fallback paths.
3. From `visualizer/`, run `npm run typecheck`, `npm run test`, `npm run build`, and `npm run test:e2e`. From the repository root, run `make verify-workflows`.

## Non-goals

- Changing ai-arena TypeSpec, storage, game rules, or participant ordering.
- Reading operator endpoints, run controls, private artifacts, or credentials.

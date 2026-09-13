# Visualizer ruleset selection

## Summary

The initial Reversi replay visualizer is limited to the currently supported
standard ruleset. Before another ruleset can be replayed, the discovery flow
must offer a visible ruleset selector.

## Boundary

- Keep the Reversi game ID and supported major fixed.
- Derive selectable rulesets from public match discovery metadata.
- Preserve ai-arena's automatic selected-run behavior: a viewer selects a
  match, never a run ID.
- Do not introduce credentials, private artifacts, or operator API requests.

## Acceptance

A viewer can select an available ruleset, then select a public match in that
ruleset and replay the automatically selected official/current run.

## Deferred reason

This is not required for the current standard-ruleset replay-discovery
controls. It needs its own spec-first execution plan once a non-standard
ruleset is available.

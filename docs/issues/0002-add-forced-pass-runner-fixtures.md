# Add forced-pass runner fixtures and terminal pass coverage

## Summary

Phase 1 runner verification covers successful completion lines and immediate
loss on an illegal pass when legal moves exist, but it does not yet cover the
opposite edge: turns where `pass` is the only legal action and terminal double
pass completion.

## Details

- The current deterministic scripted fixtures only contain full-placement game
  lines.
- We do not yet have a dedicated fixture line or other reusable verification
  asset that exercises:
  - an explicit forced-pass turn
  - a second consecutive forced pass that ends the match
- The game rules and runtime behavior support explicit `pass`, but the repo
  still lacks a deterministic replay asset that proves this path through the
  tagged `arena-runner` host.

## Proposed Solution

Add deterministic verification assets and tests for the pass-specific edge
cases:

1. introduce at least one scripted fixture or equivalent reproducible scenario
   that reaches a forced-pass turn
2. add tagged-runner coverage that asserts the player must return `pass` and
   the resulting artifacts record that turn correctly
3. add coverage for terminal double-pass completion once a deterministic
   scenario is available

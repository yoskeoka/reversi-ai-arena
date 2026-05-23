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

## Reference kifu

### end with 1 empty cell(forced-pass for both)
f5f6e6f4e3c5c4d6b5d3c3e2f2c2d2b3b4f3c1e1g3g4h4h5c6h3g5f1c7a4a5h6d7g6a3e7f8d1f7b1g2b2h2h1g1b8c8e8d8g8a1a2h7h8g7b7b6a6a7

### fastest black win
f5d6c5f4e7f6g5e6e3

### short white win
f5f6e6d6e7f7d7f4c5c7c6b6

## multiple passes in the middle and ends with some empty cells
d3c5f6f5e6e3d6f7b6e7f3c6d7c8g5f4g7g6e8c7d8h8b5f2h5h4f1g4h7h6b8g3g2h3h2c4b3g8f8a8b4c3h1a3g1e2e1b7c2d1a7d2a6c1a4a5a2

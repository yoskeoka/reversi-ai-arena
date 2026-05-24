# Review `--include-pass` kifu output format

## Summary

`reversi-kifu-export --include-pass` currently keeps `pass` turns inline in the
compact move string, for example:

`d3c5b6f3f5c6e3a6d6e6d7e8c8c4b3e7f7f8c7b4a4a3a2f4c3b2g4d2d1f2f1e1e2c1b1h4g3g5f6g7h3c2h5h6d8g6b7h2a7a8a1b8h8g8h7b5h1g1g2passa5`

This is machine-readable and technically preserves the pass turn, but it is
hard to visually scan because the delimiter and grouping around `pass` are not
obvious in the single compact token stream.

## Details

- The current compact notation is good for dense copy/paste and round-tripping.
- Adding literal `pass` in the middle preserves information, but the result is
  easy to miss when reading a long line by eye.
- We should review whether `--include-pass` needs a more legible representation
  while preserving deterministic export and import expectations.

## Proposed Direction

Evaluate alternatives that keep pass turns explicit without making the compact
output ambiguous. Candidate directions:

1. add a delimiter around `pass`, such as `...g2-pass-a5`
2. switch only `--include-pass` to a tokenized representation with separators
3. keep the current compact string as the default but add another explicit
   human-readable mode

Any change should define the canonical contract for both export readability and
future parser compatibility before implementation.

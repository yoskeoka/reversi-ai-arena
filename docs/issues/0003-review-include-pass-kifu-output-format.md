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

Do not invent a new house style first. Instead, survey game-record formats that
existing Othello/Reversi software and databases already use, then decide
whether `reversi-kifu-export --include-pass` should align with one of them or
offer a compatible readable variant.

The initial comparison set should include at least:

1. WTHOR-style tournament record workflows and transcripts
2. SGF-based Reversi/Othello game records
3. GGF and other text formats already accepted by established Othello tools
4. compact move-string conventions used by current analysis tools

Before implementation, document which existing format families were reviewed,
how they express pass turns, and why the chosen output should match,
approximate, or intentionally differ from them.

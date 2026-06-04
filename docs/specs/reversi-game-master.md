# Reversi Game Master

## Purpose

This specification defines the Reversi-owned game-master contract that
`reversi-ai-arena` exposes to the tagged `ai-arena` runner.

It fixes:

- the Reversi match metadata tuple
- board-state progression and legal-move computation
- forced-pass behavior
- invalid-action and timeout handling
- exported snapshot and result expectations
- the runnable entrypoint surface owned by this repository

## Match Metadata

- `game_id`: `reversi`
- `game_version`: `1.0.0`
- `ruleset_version`: `standard`

Phase 1 owns one stable ruleset only. Future opening-policy or runtime-policy
variants may add new ruleset identifiers, but they must not change the core
Reversi board rules.

## Entrypoint

- The runnable entrypoint is the repository-owned `reversi-gamemaster`
  executable under `cmd/`.
- The executable speaks the public game-master JSON-RPC over
  `stdio-jsonrpc-ndjson`.
- The game-master implementation lives in the Reversi-owned Rust surface under
  `games/reversi/`; the `cmd/` entrypoint is a thin transport adapter.

## Initial State

Phase 1 starts from the standard 8x8 opening:

- two black discs and two white discs in the center
- Black moves first
- `board_size = 8`
- `ruleset = standard`

The per-player init payload must identify:

- which color the player controls
- the board size
- the opening placements
- the ruleset identifier

## Turn Model

- Reversi uses sequential decision steps.
- Each step targets exactly one current player.
- The visible state must include:
  - current turn number
  - full public board
  - current player color while the match is active
  - `legal_actions`
  - current disc counts

## Legal Actions And Pass

- `legal_actions` is the single authority for pass behavior.
- If `legal_actions` is non-empty, the player must return a placement and
  `pass` is illegal.
- If `legal_actions` is empty, the player must still receive a turn request and
  must return an explicit `pass`.
- The game master must not silently skip forced-pass turns.

This keeps public move logs, runner artifacts, and per-turn deadlines aligned
across normal turns and forced-pass turns.

## Action Validation

- A placement is legal only when it targets an empty square and flips one or
  more opponent discs along a valid direction.
- `pass` is legal only when `legal_actions` is empty.
- If the player response is malformed, mismatched, late, timed out, or
  semantically illegal, the game master must treat it as a failed turn rather
  than repairing or guessing the move.

## Loss Handling

- If a player has one or more legal moves and fails to provide one valid legal
  placement, that player loses immediately.
- An explicit loss includes at least:
  - timeout
  - malformed or unusable protocol response
  - `pass` while legal placements exist
  - a placement that is not in the current legal set
- Forced-pass turns also require an explicit valid response. The game master may
  treat failure to emit the required `pass` as an immediate loss because the
  player failed the turn request.

When a player loses immediately, the remaining player wins and the match
becomes terminal without inventing extra recovery turns.

## Terminal Conditions

The match becomes terminal when any of the following holds:

- the board is full
- both players have completed consecutive forced-pass turns
- one player loses immediately due to a failed turn

At terminal state:

- `current_player` becomes `null`
- `completed = true` in the exported public state
- placements reflect the winner or tie outcome

## Snapshot Contract

The internal snapshot must preserve enough state to resume the match without
recomputing from logs alone, including:

- current board
- current player
- current turn number
- consecutive forced-pass count
- last accepted or failed action status per player
- terminal winner or failure state when already completed

The exported snapshot must preserve only public replay-safe data:

- public board
- current or terminal player turn state
- current scores
- whether the match is completed
- last action status per player

## Result Contract

- Final placements are determined from disc counts unless the match ended by an
  immediate-loss failure.
- If the match ends by immediate loss, the surviving player takes first place.
- If the disc counts are equal at a normal terminal end, both players share
  first place.

## Verification Expectations

Phase 1 verification must prove at least:

- legal-move generation matches the standard opening and later flips
- explicit forced-pass turns are requested and recorded
- required-pass success cases complete normally through the tagged runner
- illegal `pass` with legal moves causes an immediate loss
- timeout or other unusable response with legal moves causes an immediate loss
- the canonical four-case scripted suite reaches terminal results consistently
- the terminal double-pass fixture can finish with empty cells still remaining
  after both forced-pass turns are accepted
- the tagged runner can launch the manifest-backed game master and produce
  standard artifacts
- runner artifacts preserve accepted `pass` actions in replay-safe history

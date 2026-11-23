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
- `game_version` is `1.2.0`.
- `ruleset_version` is `standard`, `standard-xot-large`, or `standard-xot-small`.

The competition's ruleset stays fixed for the match. Large is the default
XOT choice. A direct game-master launch with no ruleset uses standard.

The v0.3.0 WASI runner supplies `AI_ARENA_RULESET_VERSION` before reading
metadata. Empty or unknown identifiers fail startup.

`game_version` is the source-controlled release identity. A tagged release
uses the exact `v${game_version}` Git tag, and the same value must appear in
game-master runtime metadata and both released bundle manifests.

## Entrypoint

- The runnable entrypoint is the repository-owned `reversi-gamemaster`
  executable under `cmd/`.
- The executable speaks the public game-master JSON-RPC over
  `stdio-jsonrpc-ndjson`.
- The game-master implementation lives in the Reversi-owned Rust surface under
  `games/reversi/`; the `cmd/` entrypoint is a thin transport adapter.
- The official delivery is `reversi-gamemaster.wasm`, built in release mode for
  `wasm32-wasip1` and packaged according to
  [submittable artifacts](submittable-artifacts.md). Native execution is a
  development-only convenience.

## Initial State

Standard starts at turn 1 with the four center discs and Black to move.
Both XOT rulesets select one row from the corresponding official list and
apply all eight placements from the same center position. The first AI
decision is Black at turn 9. Automatic placements have no AI calls, deadlines,
or decision billing.

A supplied runner `rng_seed` is a UTF-8 string. The game hashes the seed with
SHA-256 and a big-endian counter starting at zero, using the first 64 bits and
rejection sampling to give every row equal odds.

For a fresh XOT match without a runner seed, the game derives a seed from
the WASI wall clock and player IDs. Standard ignores the seed.

The per-player init payload must identify:

- which color the player controls
- the board size
- all occupied squares and their colors at the start of AI play
- the first AI turn for XOT (`initial_turn: 9`), omitted for standard
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

- selected XOT opening sequence and ruleset, with no reselection on resume
- current board
- current player
- current turn number
- consecutive forced-pass count
- last accepted or failed action status per player
- accepted public replay turns, so a resumed completed match publishes a full
  transcript rather than only post-resume turns
- terminal winner or failure state when already completed

Historical standard snapshots without an opening sequence remain readable.
Resuming requires the startup ruleset to agree with the stored ruleset (an
absent legacy ruleset means standard).

The exported snapshot must preserve only public replay-safe data:

- public board
- current or terminal player turn state
- current scores
- whether the match is completed
- last action status per player

## Public Replay Contract

For a completed match, the game master publishes `current_public_replay` with
format `reversi/replay`, version `1`, and a JSON payload. The payload contains
the four standard center placements as `opening`, the selected `ruleset`, and
only the ordered accepted AI placements and passes in `turns`. XOT adds
`opening_moves`, an array of exactly eight board coordinates played alternately
from the center position. Its initial frame shows the turn 9 board.

Standard keeps its historical format and omits `opening_moves`. Only
completed matches publish replays. The payload excludes private state,
runner history, stderr, and artifact locators.

The consumer checks its replayed final state against the published snapshot.
A forfeit before the first AI move leaves the AI transcript empty. The
exported state identifies the winning player.

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

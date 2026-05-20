# Architectural Decision Records (ADR)

## 2026-05-20: Use product-surface-first top-level layout for Reversi

### Context
Phase 0 of `reversi-ai-arena` exists to fix repository boundaries, language
choices, and visualizer architecture before implementation begins. The repo
needs a top-level layout that keeps `ai-arena` public-contract boundaries
visible, supports separate delivery of the game master, AI players, and
visualizer, and avoids making implementation language the primary organizing
principle.

### Decision
Adopt a product-surface-first top-level layout:

- `cmd/` for user-facing and developer-facing entrypoints
- `games/reversi/` for the Reversi game master and core rules implementation
- `players/` for AI-player implementations and reference bots
- `visualizer/` for replay and future watcher clients
- `e2e/`, `testdata/`, `tools/`, and `docs/` for verification assets, fixtures,
  tooling, and documentation

Language-specific code may exist under these surfaces when needed, but the
surface ownership is the primary boundary.

### Consequences
Positive:

- The repository structure matches the project milestones and review scope.
- The boundary between `ai-arena` platform contracts and Reversi-owned code is
  easier to explain in specs and public guidance.
- Later phases can evolve `players/` and `visualizer/` independently without
  re-framing the entire repo.

Negative:

- Build tooling may need to aggregate multiple language ecosystems from
  different surface roots.
- Some shared logic placement decisions inside `games/reversi/` and `players/`
  still need to be specified during execution.

---

## 2026-05-20: Use a lightweight Vite plus TypeScript shell around Phaser

### Context
Phase 0 also needs to lock the visualizer packaging boundary before replay work
starts. The board renderer should use Phaser, but the rest of the spectator UI
should stay lightweight and avoid committing the repository to React or another
large framework.

### Decision
Use `visualizer/` as a lightweight Vite plus TypeScript browser shell. Phaser
is the board-rendering dependency inside that shell, while playback controls,
artifact loading, and layout stay in minimal web-standard code.

### Consequences
Positive:

- Phaser is fixed as the board-rendering layer without expanding it into a full
  application framework.
- Later replay work can grow inside `visualizer/` without revisiting the root
  packaging choice.
- The repository avoids early coupling to React or a larger UI framework.

Negative:

- The repo now carries a third toolchain surface in addition to Rust and small
  Go support lanes.
- A future richer spectator UI may need more structure inside the shell if the
  control surface grows substantially.

---

## [YYYY-MM-DD] Title of Decision

### Context
[Describe the issue or problem.]

### Decision
[Describe the decision made.]

### Consequences
[Describe the positive and negative consequences.]

---

# Project Plan: Reversi AI Arena

## Goal

`reversi-ai-arena` is a game repository for `ai-arena` that develops a Reversi
competition line with a low-complexity game master, a practical spectator
experience, and AI-player development that is constrained by WASM runtime
budgets.

This repository is not only for adding another playable game to `ai-arena`.
It is also a dogfooding vehicle for strengthening the platform, especially the
parts around game registration, replay, spectator tooling, and external game
master development.

## This Repo's Role

`ai-arena` remains the platform and public-contract repository. This repository
owns the Reversi-specific implementation, verification assets, and game-facing
tools that are needed to operate Reversi as a registered game.

- `ai-arena` side:
  - game master protocol and shared public contracts
  - AI runtime and WASM execution model
  - registry, runner, replay, and future spectator platform APIs
- `reversi-ai-arena` side:
  - Reversi game master
  - Reversi AI-player implementations and reference bots
  - Reversi replay visualizer and future watcher client
  - Reversi-specific fixtures, golden outputs, and CI verification assets
  - public guidance for implementing an external game master against `ai-arena`

## Significance

- Reversi has simple core rules, so the game master and visualizer are easier to
  build than richer games while still being valuable as a registered `ai-arena`
  game.
- The game is a good educational target for AI engineering under constrained
  WASM resources, including search, evaluation, opening choice, and memory
  budgeting.
- Reversi is a strong dogfooding target for platform work around replay,
  exported snapshots, visualization, and spectator-facing APIs.
- The project creates reusable Phaser knowledge that can help future game
  visualizer development across other repositories.
- Reversi already has a well-known competitive foundation, so the repository can
  focus on implementation quality, platform integration, and spectator tooling
  rather than inventing game value from scratch.

## Requirements

### Game Identity

- The game is standard Reversi with stable core rules. Pass handling and end
  conditions do not vary across rulesets.
- The game should be registered and verified as an `ai-arena` game through the
  existing platform contracts rather than a special-case path.
- Game compatibility should treat the Reversi rule identity as stable and use
  rulesets only for bounded policy variation such as opening setup and runtime
  budgets.

### Ruleset Boundaries

- Rulesets may vary in opening policy, such as the standard four-stone opening
  or XOT-style randomized opening progression.
- Rulesets may also vary in AI-facing runtime constraints such as per-turn time,
  WASM size limits, and memory limits.
- Rulesets should not redefine the core board rules, legal-move logic, pass
  semantics, or end conditions.

### Game Master

- The mainline Reversi game master should be implemented in Rust.
- The game master must run through `arena-runner`, complete matches locally, and
  remain continuously verified in CI.
- Match outputs must preserve deterministic replay inputs where applicable so
  runner-produced artifacts can be consumed directly by replay tooling.

### AI Players

- The primary AI-player implementation should be Rust.
- A lightweight Go reference bot may exist for protocol samples, fixtures, and
  comparison lanes, but Rust is the main competitive implementation path.
- A future follow-up may evaluate whether Edax source code or CLI behavior can
  be adapted into a WASM lane, but that is not part of the initial mainline.

### Visualizer

- The visualizer should separate the game-rendering layer from the surrounding
  spectator controls and settings UI.
- The board-rendering layer should use Phaser.
- The surrounding UI should stay lightweight and web-standard oriented rather
  than adopting React or another large UI framework that conflicts with
  canvas-driven rendering.
- A small CSS framework may be adopted later, but it is not fixed in the
  initial project plan.
- The initial visualizer must be able to read runner-exported JSON artifacts and
  replay completed matches without requiring internal-only state.

### Snapshot and Spectator Model

- Replay and spectator clients should be built around public exported state, not
  private internal match state.
- The repository should assume a clear boundary between internal snapshots used
  for engine/debug purposes and exported snapshots used for replay and public
  spectators.
- Real-time watching depends on future spectator-facing platform APIs and should
  be designed as a later integration phase rather than baked into the initial
  replay tool.

### Relationship to `reversi-adventure`

- This repository may reuse knowledge and implementation ideas from
  `reversi-adventure`, especially around core Reversi logic and AI techniques.
- It does not require shared libraries or a live multi-repo code-sharing model.
  Copying and adapting code is acceptable when that keeps repository boundaries
  simple.

### Documentation and Examples

- This repository should eventually provide public-facing documentation that
  shows how to build an external game master for `ai-arena`, using Reversi as a
  concrete example.
- Internal docs, public docs, and code comments should all be written in
  English in this repository.

## Non-Goals

- Building a human-first standalone Reversi product outside the `ai-arena`
  ecosystem
- Requiring shared libraries with `reversi-adventure`
- Exposing private engine/debug-only state to the spectator path
- Treating the initial visualizer as a full analysis workstation with AI
  evaluation overlays
- Locking in a CSS framework before the spectator UI requirements are clearer

## Milestones

- [ ] Phase 0: Fix the repository boundaries, language choices, and visualizer
      architecture for Reversi as an `ai-arena` game.
- [ ] Phase 1: Implement the Rust game master, verify it through
      `arena-runner`, and keep that path green in CI.
- [ ] Phase 2: Implement the Rust AI-player mainline and add a lightweight Go
      reference bot for samples and verification support.
- [ ] Phase 3: Build a replay visualizer that reads exported runner artifacts
      and replays completed matches with Phaser-rendered game screens.
- [ ] Phase 4: Integrate a real-time watcher after the platform exposes the
      required spectator-facing game-state API.
- [ ] Phase 5: Publish external game-master guidance and sample documentation
      for `ai-arena`, using Reversi as the main example.
- [ ] Phase 6: Explore advanced analysis lanes such as stronger Rust AI tuning,
      optional evaluation overlays, and possible Edax-derived experiments.

## Milestone Intent

- Phase 0 exists to avoid drifting into duplicated engine work or an accidental
  UI-framework commitment before the architecture is fixed.
- Phase 1 establishes Reversi as a continuously verified registered game rather
  than only a local prototype.
- Phase 2 treats constrained AI implementation itself as part of the value of
  the project, not just a support artifact for the game master.
- Phase 3 focuses on artifact-driven replay first because that is useful even
  before the platform's live spectator APIs are ready.
- Phase 4 is explicitly gated on platform capability and should consume public
  spectator APIs rather than introducing a Reversi-specific bypass.
- Phase 5 turns Reversi into a practical external-game-master example for
  future `ai-arena` adopters and game authors.
- Phase 6 keeps higher-cost analysis features and third-party engine experiments
  out of the initial delivery path.

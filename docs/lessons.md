# Lessons Learned

- **Mistake**: I modeled Reversi pass behavior with optional boolean flags (`pass_allowed` / `pass_required`) instead of deriving it from the legal-move set.
- **Pattern**: I introduced transport DTO fields that weakened a domain rule which was already fully determined by the game state.
- **Rule**: When a game rule is fully determined by canonical state such as `legal_actions`, encode that rule directly from the state and do not add parallel permissive flags unless the user or spec explicitly requires them.
- **Applied**: Reversi payload DTOs, future game-master turn validation, and player transport payload design in `games/reversi/**` and related specs.

- **Mistake**: I treated the PR review as if every suggested test should be added immediately, even after the user narrowed scope to keep missing forced-pass fixtures as follow-up work.
- **Pattern**: I collapsed distinct evidence levels, mixing protocol/runtime fixes with missing deterministic fixture assets.
- **Rule**: When review feedback combines fixable code defects and missing verification assets, fix the code defects in-branch and file a `docs/issues/` follow-up for the asset gap if the user keeps that scope out of the current PR.
- **Applied**: PR review triage, deterministic fixture expansion, and follow-up issue handling in `docs/issues/` during execute-task work.

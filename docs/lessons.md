# Lessons Learned

- **Mistake**: I modeled Reversi pass behavior with optional boolean flags (`pass_allowed` / `pass_required`) instead of deriving it from the legal-move set.
- **Pattern**: I introduced transport DTO fields that weakened a domain rule which was already fully determined by the game state.
- **Rule**: When a game rule is fully determined by canonical state such as `legal_actions`, encode that rule directly from the state and do not add parallel permissive flags unless the user or spec explicitly requires them.
- **Applied**: Reversi payload DTOs, future game-master turn validation, and player transport payload design in `games/reversi/**` and related specs.

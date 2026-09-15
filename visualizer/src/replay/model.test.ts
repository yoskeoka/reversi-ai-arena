import { describe, expect, it } from "vitest";
import { apply, buildReplay } from "./model";

const opening = [{ position: { row: 3, col: 3 }, disc: "white" }, { position: { row: 3, col: 4 }, disc: "black" }, { position: { row: 4, col: 3 }, disc: "black" }, { position: { row: 4, col: 4 }, disc: "white" }];
const terminal = (board: unknown) => ({ status: "completed", public_state: { completed: true, current_player: null, scores: { black: 4, white: 1 }, board } });

describe("public Reversi replay", () => {
  it("replays an accepted placement without mutating its prior frame", () => {
    const initial = Array.from({ length: 8 }, () => Array(8).fill("empty"));
    opening.forEach(({ position, disc }) => { initial[position.row][position.col] = disc; });
    const board = apply(initial, "black", { kind: "place", position: { row: 2, col: 3 } });
    const model = buildReplay({ board_size: 8, opening, ruleset: "standard", turns: [{ player_id: "p1", color: "black", action: { kind: "place", position: { row: 2, col: 3 } } }] }, terminal(board));
    expect(model.frames).toHaveLength(3);
    expect(model.frames[0].board[2][3]).toBe("empty");
    expect(model.final.scores).toEqual({ black: 4, white: 1 });
    expect(model.frames.at(-1)).toBe(model.final);
    expect(model.final.currentPlayer).toBeNull();
    expect(Object.isFrozen(model.frames[0].board)).toBe(true);
    expect(Object.isFrozen(model.frames[0].board[0])).toBe(true);
  });
  it("rejects incompatible replay data", () => {
    expect(() => buildReplay({ board_size: 4, opening: [], ruleset: "standard", turns: [] }, {})).toThrow("unsupported");
  });
  it("rejects a non-standard opening", () => {
    expect(() => buildReplay({ board_size: 8, opening: [], ruleset: "standard", turns: [] }, {})).toThrow("opening");
  });
});

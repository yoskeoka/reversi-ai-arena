export type Disc = "empty" | "black" | "white";
export type Color = "black" | "white";
export type Action = { kind: "place" | "pass"; position?: { row: number; col: number } };
export type ReplayPayload = {
  board_size: number;
  opening: { position: { row: number; col: number }; disc: Disc }[];
  ruleset: string;
  turns: { player_id: string; color: Color; action: Action }[];
};
export type ExportedState = {
  status: "completed";
  public_state: { board: Disc[][]; current_player: Color | null; scores: { black: number; white: number }; completed: boolean };
};
export type ReplayFrame = { board: Disc[][]; currentPlayer: Color | null; scores: { black: number; white: number }; turn: number; lastAction?: Action };
export type ReplayModel = { frames: readonly ReplayFrame[]; turns: ReadonlyArray<ReplayPayload["turns"][number]>; final: ReplayFrame };

const directions = [-1, 0, 1].flatMap((row) => [-1, 0, 1].map((col) => [row, col] as const)).filter(([row, col]) => row !== 0 || col !== 0);
const other = (color: Color): Color => color === "black" ? "white" : "black";
const clone = (board: Disc[][]) => board.map((row) => [...row]);
const standardOpening = [{ position: { row: 3, col: 3 }, disc: "white" }, { position: { row: 3, col: 4 }, disc: "black" }, { position: { row: 4, col: 3 }, disc: "black" }, { position: { row: 4, col: 4 }, disc: "white" }] as const;

export function decodeReplay(value: unknown): ReplayPayload {
  if (!isRecord(value)) throw new Error("public replay payload must be an object");
  const payload = value as Partial<ReplayPayload>;
  if (payload.board_size !== 8 || payload.ruleset !== "standard" || !Array.isArray(payload.opening) || !Array.isArray(payload.turns)) {
    throw new Error("unsupported Reversi replay initial-position parameters");
  }
  if (payload.opening.length !== standardOpening.length || !payload.opening.every((placement, index) => placement?.disc === standardOpening[index].disc && placement.position?.row === standardOpening[index].position.row && placement.position?.col === standardOpening[index].position.col)) {
    throw new Error("unsupported Reversi replay opening");
  }
  return payload as ReplayPayload;
}

export function buildReplay(payloadValue: unknown, exportedValue: unknown): ReplayModel {
  const payload = decodeReplay(payloadValue);
  const exported = exportedValue as ExportedState;
  if (!isRecord(exported) || exported.status !== "completed" || !isRecord(exported.public_state) || exported.public_state.completed !== true) {
    throw new Error("a completed final exported state is required");
  }
  const board = Array.from({ length: payload.board_size }, () => Array<Disc>(payload.board_size).fill("empty"));
  for (const placement of payload.opening) {
    if (!placement?.position || !inBounds(placement.position.row, placement.position.col) || !isDisc(placement.disc) || placement.disc === "empty") throw new Error("malformed opening");
    board[placement.position.row][placement.position.col] = placement.disc;
  }
  let current: Color | null = "black";
  const frames: ReplayFrame[] = [frame(board, current, 1)];
  for (const turn of payload.turns) {
    if (!turn || turn.color !== current || !turn.player_id || !turn.action || !isAction(turn.action)) throw new Error("malformed or out-of-turn public replay entry");
    const next = apply(board, turn.color, turn.action);
    board.splice(0, board.length, ...next);
    current = other(turn.color);
    frames.push(frame(board, current, frames.length + 1, turn.action));
  }
  const final = frame(board, null, frames.at(-1)?.turn ?? 1, frames.at(-1)?.lastAction);
  const publicState = exported.public_state;
  if (!sameBoard(final.board, publicState.board) || final.scores.black !== publicState.scores.black || final.scores.white !== publicState.scores.white || publicState.current_player !== null) {
    throw new Error("public replay conflicts with final exported state");
  }
  return Object.freeze({ frames: Object.freeze(frames.map(freezeFrame)), turns: Object.freeze(payload.turns.map(freezeTurn)), final: freezeFrame(final) });
}

export function apply(board: Disc[][], color: Color, action: Action): Disc[][] {
  const next = clone(board);
  if (action.kind === "pass") {
    if (legalMoves(board, color).length) throw new Error("pass is illegal while placements exist");
    return next;
  }
  if (!action.position || !inBounds(action.position.row, action.position.col) || board[action.position.row][action.position.col] !== "empty") throw new Error("malformed placement");
  const flips = directions.flatMap(([dr, dc]) => captures(board, color, action.position!.row, action.position!.col, dr, dc));
  if (!flips.length) throw new Error("placement is illegal");
  next[action.position.row][action.position.col] = color;
  for (const [row, col] of flips) next[row][col] = color;
  return next;
}

function legalMoves(board: Disc[][], color: Color) { return board.flatMap((row, r) => row.flatMap((disc, c) => disc === "empty" && directions.some(([dr, dc]) => captures(board, color, r, c, dr, dc).length) ? [[r, c]] : [])); }
function captures(board: Disc[][], color: Color, row: number, col: number, dr: number, dc: number) { const found: [number, number][] = []; for (let r = row + dr, c = col + dc; inBounds(r, c) && board[r][c] === other(color); r += dr, c += dc) found.push([r, c]); return found.length && inBounds(row + dr * (found.length + 1), col + dc * (found.length + 1)) && board[row + dr * (found.length + 1)][col + dc * (found.length + 1)] === color ? found : []; }
function frame(board: Disc[][], currentPlayer: Color | null, turn: number, lastAction?: Action): ReplayFrame { const scores = { black: 0, white: 0 }; board.flat().forEach((disc) => { if (disc !== "empty") scores[disc]++; }); return { board: clone(board), currentPlayer, scores, turn, lastAction }; }
function freezeFrame(value: ReplayFrame): ReplayFrame { return Object.freeze({ ...value, board: Object.freeze(value.board.map((row) => Object.freeze([...row]))) as unknown as Disc[][], scores: Object.freeze({ ...value.scores }), lastAction: value.lastAction && freezeAction(value.lastAction) }); }
function freezeTurn(value: ReplayPayload["turns"][number]): ReplayPayload["turns"][number] { return Object.freeze({ ...value, action: freezeAction(value.action) }); }
function freezeAction(value: Action): Action { return Object.freeze({ ...value, position: value.position && Object.freeze({ ...value.position }) }); }
function isRecord(value: unknown): value is Record<string, unknown> { return typeof value === "object" && value !== null; }
function isDisc(value: unknown): value is Disc { return value === "empty" || value === "black" || value === "white"; }
function isAction(value: unknown): value is Action { return isRecord(value) && (value.kind === "pass" || value.kind === "place"); }
function inBounds(row: number, col: number) { return Number.isInteger(row) && Number.isInteger(col) && row >= 0 && row < 8 && col >= 0 && col < 8; }
function sameBoard(left: Disc[][], right: unknown): boolean { return Array.isArray(right) && left.length === right.length && left.every((row, i) => Array.isArray(right[i]) && row.every((disc, j) => disc === right[i][j])); }

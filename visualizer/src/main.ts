import "./style.css";
import { loadReplay } from "./api/public-api";
import { mountBoard } from "./renderer/board";
import { buildReplay, type ReplayModel } from "./replay/model";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("missing #app root");
const params = new URLSearchParams(location.search);
const fixture = { board_size: 8, ruleset: "standard", opening: [{ position: { row: 3, col: 3 }, disc: "white" }, { position: { row: 3, col: 4 }, disc: "black" }, { position: { row: 4, col: 3 }, disc: "black" }, { position: { row: 4, col: 4 }, disc: "white" }], turns: [] };
const finalState = { status: "completed", public_state: { completed: true, current_player: null, scores: { black: 2, white: 2 }, board: Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row === 3 && col === 3) || (row === 4 && col === 4) ? "white" : (row === 3 && col === 4) || (row === 4 && col === 3) ? "black" : "empty")) } };

void start();
async function start() {
  try {
    const model = params.has("api") && params.has("match") ? await loadReplay(params.get("api")!, params.get("match")!) : buildReplay(fixture, finalState);
    render(model);
  } catch (error) { app!.innerHTML = `<main class="shell"><p role="alert">Replay unavailable: ${escape(error instanceof Error ? error.message : "unknown error")}</p></main>`; }
}
function render(model: ReplayModel) {
  let index = 0, playing: number | undefined;
  app!.innerHTML = `<main class="shell"><header><p class="eyebrow">Public Reversi replay</p><h1>Reversi replay visualizer</h1></header><section aria-label="Reversi board"><div id="board"></div></section><section class="summary" aria-live="polite"><p id="turn"></p><p id="score"></p><p id="result">Completed terminal replay</p></section><nav aria-label="Playback controls"><button id="previous" type="button">Previous turn</button><button id="play" type="button">Play</button><button id="next" type="button">Next turn</button></nav></main>`;
  const updateBoard = mountBoard(document.querySelector<HTMLElement>("#board")!, model.frames[0]);
  const update = () => { const current = model.frames[index]; updateBoard(current); document.querySelector("#turn")!.textContent = `Turn ${current.turn}: ${current.currentPlayer ?? "terminal"}`; document.querySelector("#score")!.textContent = `Score — black ${current.scores.black}, white ${current.scores.white}`; };
  document.querySelector<HTMLButtonElement>("#previous")!.onclick = () => { index = Math.max(0, index - 1); update(); };
  document.querySelector<HTMLButtonElement>("#next")!.onclick = () => { index = Math.min(model.frames.length - 1, index + 1); update(); };
  document.querySelector<HTMLButtonElement>("#play")!.onclick = (event) => { const button = event.currentTarget as HTMLButtonElement; if (playing) { clearInterval(playing); playing = undefined; button.textContent = "Play"; } else { playing = window.setInterval(() => { index = Math.min(model.frames.length - 1, index + 1); update(); if (index === model.frames.length - 1) { clearInterval(playing); playing = undefined; button.textContent = "Play"; } }, 700); button.textContent = "Pause"; } };
  update();
}
function escape(value: string) { const node = document.createElement("span"); node.textContent = value; return node.innerHTML; }

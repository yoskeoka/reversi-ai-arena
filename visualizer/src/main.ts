import "./style.css";
import { baseProfiles, listCompletedMatches, loadReplay, normalizeBaseUrl, type PublicMatch } from "./api/public-api";
import { mountBoard } from "./renderer/board";
import { buildReplay, type ReplayModel } from "./replay/model";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("missing #app root");
const fixture = { board_size: 8, ruleset: "standard", opening: [{ position: { row: 3, col: 3 }, disc: "white" }, { position: { row: 3, col: 4 }, disc: "black" }, { position: { row: 4, col: 3 }, disc: "black" }, { position: { row: 4, col: 4 }, disc: "white" }], turns: [] };
const finalState = { status: "completed", public_state: { completed: true, current_player: null, scores: { black: 2, white: 2 }, board: Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row === 3 && col === 3) || (row === 4 && col === 4) ? "white" : (row === 3 && col === 4) || (row === 4 && col === 3) ? "black" : "empty")) } };

void start();
async function start() {
  const params = new URLSearchParams(location.search);
  const base = params.get("api") && normalizeBaseUrl(params.get("api")!) || undefined;
  const matchId = params.get("match") || undefined;
  if (!base && !matchId) return render(buildReplay(fixture, finalState));
  if (!base) return renderDiscovery(undefined, matchId, [], "A match link requires a valid public API base URL.");
  try { renderDiscovery(base, matchId, await listCompletedMatches(base)); if (matchId) render(await loadReplay(base, matchId), base, matchId); }
  catch (error) { renderDiscovery(base, matchId, [], errorMessage(error)); }
}

function renderDiscovery(base?: string, selectedMatch?: string, matches: PublicMatch[] = [], error?: string) {
  const profile = baseProfiles.find((item) => item.baseUrl === base)?.id ?? (base ? "custom" : "");
  const options = baseProfiles.map((item) => `<option value="${item.id}"${profile === item.id ? " selected" : ""}>${item.label}</option>`).join("");
  const matchOptions = matches.map((match) => `<option value="${escape(match.match_id)}"${match.match_id === selectedMatch ? " selected" : ""}>${escape(match.match_id)}</option>`).join("");
  app!.innerHTML = `<main class="shell"><header><p class="eyebrow">Public Reversi replay</p><h1>Reversi replay visualizer</h1></header><section class="discovery" aria-label="Replay discovery controls"><label for="api-base">Public API base</label><select id="api-base"><option value="">Select an environment</option>${options}${profile === "custom" ? `<option value="custom" selected>Custom: ${escape(base!)}</option>` : ""}</select><label for="match">Completed Reversi match</label><select id="match" ${base ? "" : "disabled"}><option value="">Select a completed match</option>${matchOptions}</select>${error ? `<p role="alert">${escape(error)}</p>` : ""}${base && !error && !matches.length ? `<p role="status">No completed standard Reversi matches are available.</p>` : ""}</section></main>`;
  document.querySelector<HTMLSelectElement>("#api-base")!.onchange = (event) => {
    const id = (event.currentTarget as HTMLSelectElement).value;
    const next = baseProfiles.find((item) => item.id === id);
    if (next) updateQuery(next.baseUrl);
  };
  document.querySelector<HTMLSelectElement>("#match")!.onchange = (event) => {
    const id = (event.currentTarget as HTMLSelectElement).value;
    if (base && id) updateQuery(base, id);
  };
}

function updateQuery(base: string, match?: string) { const url = new URL(location.href); url.searchParams.set("api", base); if (match) url.searchParams.set("match", match); else url.searchParams.delete("match"); history.replaceState({}, "", url); void start(); }
function render(model: ReplayModel, base?: string, selectedMatch?: string) {
  let index = 0, playing: number | undefined;
  const controls = base ? `<section class="discovery" aria-label="Replay discovery controls"><p>Viewing completed match ${escape(selectedMatch ?? "")}</p><button id="change-match" type="button">Choose another match</button></section>` : `<section class="discovery" aria-label="Replay discovery controls"><label for="api-base">Public API base</label><select id="api-base"><option value="">Select an environment</option>${baseProfiles.map((item) => `<option value="${item.id}">${item.label}</option>`).join("")}</select><label for="match">Completed Reversi match</label><select id="match" disabled><option>Select an environment first</option></select></section>`;
  app!.innerHTML = `<main class="shell"><header><p class="eyebrow">Public Reversi replay</p><h1>Reversi replay visualizer</h1></header>${controls}<section aria-label="Reversi board"><div id="board"></div></section><section class="summary" aria-live="polite"><p id="turn"></p><p id="score"></p><p id="result">Completed terminal replay</p></section><nav aria-label="Playback controls"><button id="previous" type="button">Previous turn</button><button id="play" type="button">Play</button><button id="next" type="button">Next turn</button></nav></main>`;
  document.querySelector<HTMLButtonElement>("#change-match")?.addEventListener("click", () => updateQuery(base!));
  document.querySelector<HTMLSelectElement>("#api-base")?.addEventListener("change", (event) => { const selected = baseProfiles.find((item) => item.id === (event.currentTarget as HTMLSelectElement).value); if (selected) updateQuery(selected.baseUrl); });
  const updateBoard = mountBoard(document.querySelector<HTMLElement>("#board")!, model.frames[0]);
  const update = () => { const current = model.frames[index]; updateBoard(current); document.querySelector("#turn")!.textContent = `Turn ${current.turn}: ${current.currentPlayer ?? "terminal"}`; document.querySelector("#score")!.textContent = `Score — black ${current.scores.black}, white ${current.scores.white}`; };
  document.querySelector<HTMLButtonElement>("#previous")!.onclick = () => { index = Math.max(0, index - 1); update(); };
  document.querySelector<HTMLButtonElement>("#next")!.onclick = () => { index = Math.min(model.frames.length - 1, index + 1); update(); };
  document.querySelector<HTMLButtonElement>("#play")!.onclick = (event) => { const button = event.currentTarget as HTMLButtonElement; if (playing) { clearInterval(playing); playing = undefined; button.textContent = "Play"; } else { playing = window.setInterval(() => { index = Math.min(model.frames.length - 1, index + 1); update(); if (index === model.frames.length - 1) { clearInterval(playing); playing = undefined; button.textContent = "Play"; } }, 700); button.textContent = "Pause"; } };
  update();
}
function errorMessage(error: unknown) { return `Replay unavailable: ${error instanceof Error ? error.message : "unknown error"}`; }
function escape(value: string) { const node = document.createElement("span"); node.textContent = value; return node.innerHTML; }

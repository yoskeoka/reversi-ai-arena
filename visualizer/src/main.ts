import "./style.css";
import { baseProfiles, listCompletedMatches, loadReplay, matchOptionLabel, normalizeBaseUrl, shortRevision, type PublicMatch, type PublicMatchDiscovery } from "./api/public-api";
import { mountBoard } from "./renderer/board";
import { buildReplay, type ReplayModel } from "./replay/model";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("missing #app root");
const fixture = { board_size: 8, ruleset: "standard", opening: [{ position: { row: 3, col: 3 }, disc: "white" }, { position: { row: 3, col: 4 }, disc: "black" }, { position: { row: 4, col: 3 }, disc: "black" }, { position: { row: 4, col: 4 }, disc: "white" }], turns: [] };
const finalState = { status: "completed", public_state: { completed: true, current_player: null, scores: { black: 2, white: 2 }, board: Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row === 3 && col === 3) || (row === 4 && col === 4) ? "white" : (row === 3 && col === 4) || (row === 4 && col === 3) ? "black" : "empty")) } };

let generation = 0;
void start();
async function start() {
  const current = ++generation;
  const params = new URLSearchParams(location.search);
  const base = params.get("api") && normalizeBaseUrl(params.get("api")!) || undefined;
  const matchId = params.get("match") || undefined;
  const ruleset = params.get("ruleset") || undefined;
  if (!base && !matchId && !ruleset) return render(buildReplay(fixture, finalState, "standard"));
  if (!base) return renderDiscovery(undefined, matchId, ruleset, undefined, "A match or ruleset link requires a valid public API base URL.");
  try {
    const discovery = await listCompletedMatches(base, { rulesetVersion: ruleset });
    if (current !== generation) return;
    const selected = discovery.matches.find((match) => match.match_id === matchId);
    renderDiscovery(base, matchId, ruleset, discovery);
    if (matchId) try {
      const expectedRuleset = ruleset ?? selected?.game.ruleset_version;
      const replay = await loadReplay(base, matchId, expectedRuleset);
      if (current === generation) render(replay.model, base, matchId, replay.match, ruleset);
    } catch (error) { if (current === generation) renderDiscovery(base, matchId, ruleset, discovery, errorMessage(error)); }
  } catch (error) { if (current === generation) renderDiscovery(base, matchId, ruleset, undefined, errorMessage(error)); }
}

function renderDiscovery(base?: string, selectedMatch?: string, selectedRuleset?: string, discovery?: PublicMatchDiscovery, error?: string) {
  const profile = baseProfiles.find((item) => item.baseUrl === base)?.id ?? (base ? "custom" : "");
  const options = baseProfiles.map((item) => `<option value="${item.id}"${profile === item.id ? " selected" : ""}>${item.label}</option>`).join("");
  const matches = discovery?.matches ?? [];
  const matchOptions = matches.map((match) => `<option value="${escapeAttribute(match.match_id)}"${match.match_id === selectedMatch ? " selected" : ""}>${escape(matchOptionLabel(match))}</option>`).join("");
  const knownRuleset = !selectedRuleset || discovery?.availableRulesetVersions.includes(selectedRuleset);
  const rulesetOptions = (discovery?.availableRulesetVersions ?? []).map((ruleset) => `<option value="${escapeAttribute(ruleset)}"${ruleset === selectedRuleset ? " selected" : ""}>${escape(ruleset)}</option>`).join("");
  const unavailable = selectedRuleset && !knownRuleset ? `<p role="status">Ruleset ${escape(selectedRuleset)} is unavailable for this Reversi scope.</p>` : "";
  app!.innerHTML = `<main class="shell"><header><p class="eyebrow">Public Reversi replay</p><h1>Reversi replay visualizer</h1></header><section class="discovery" aria-label="Replay discovery controls"><label for="api-base">Public API base</label><select id="api-base"><option value="">Select an environment</option>${options}${profile === "custom" ? `<option value="custom" selected>Custom: ${escape(base!)}</option>` : ""}</select><label for="ruleset">Reversi ruleset</label><select id="ruleset" ${base ? "" : "disabled"}><option value="">All available rulesets</option>${selectedRuleset && !knownRuleset ? `<option value="${escapeAttribute(selectedRuleset)}" selected>Unavailable: ${escape(selectedRuleset)}</option>` : ""}${rulesetOptions}</select><label for="match">Completed Reversi match</label><select id="match" ${base && knownRuleset ? "" : "disabled"}><option value="">Select a completed match</option>${matchOptions}</select>${error ? `<p role="alert">${escape(error)}</p>` : ""}${unavailable}${base && !error && knownRuleset && !matches.length ? `<p role="status">No completed Reversi matches are available for this scope.</p>` : ""}</section></main>`;
  document.querySelector<HTMLSelectElement>("#api-base")!.onchange = (event) => {
    const next = baseProfiles.find((item) => item.id === (event.currentTarget as HTMLSelectElement).value);
    if (next) updateQuery(next.baseUrl);
  };
  document.querySelector<HTMLSelectElement>("#ruleset")!.onchange = (event) => updateQuery(base!, undefined, (event.currentTarget as HTMLSelectElement).value || undefined);
  document.querySelector<HTMLSelectElement>("#match")!.onchange = (event) => {
    const id = (event.currentTarget as HTMLSelectElement).value;
    if (base && id) updateQuery(base, id, selectedRuleset);
  };
}

function updateQuery(base: string, match?: string, ruleset?: string) { const url = new URL(location.href); url.searchParams.set("api", base); if (match) url.searchParams.set("match", match); else url.searchParams.delete("match"); if (ruleset) url.searchParams.set("ruleset", ruleset); else url.searchParams.delete("ruleset"); history.replaceState({}, "", url); void start(); }
function render(model: ReplayModel, base?: string, selectedMatch?: string, match?: PublicMatch, selectedRuleset?: string) {
  let index = 0, playing: number | undefined;
  const controls = base ? `<section class="discovery" aria-label="Replay discovery controls"><p>Viewing completed match ${escape(selectedMatch ?? "")}</p><button id="change-match" type="button">Choose another match</button></section>` : `<section class="discovery" aria-label="Replay discovery controls"><label for="api-base">Public API base</label><select id="api-base"><option value="">Select an environment</option>${baseProfiles.map((item) => `<option value="${item.id}">${item.label}</option>`).join("")}</select></section>`;
  const players = match ? `<p id="black-player"></p><p id="white-player"></p>` : "";
  app!.innerHTML = `<main class="shell"><header><p class="eyebrow">Public Reversi replay</p><h1>Reversi replay visualizer</h1></header>${controls}<section aria-label="Reversi board"><div id="board"></div></section><section class="summary" aria-live="polite">${players}<p id="turn"></p><p id="score"></p><p id="result"></p></section><nav aria-label="Playback controls"><button id="initial" type="button">Initial state</button><button id="previous" type="button">Previous turn</button><button id="play" type="button">Play</button><button id="next" type="button">Next turn</button><button id="final" type="button">Final state</button></nav></main>`;
  document.querySelector<HTMLButtonElement>("#change-match")?.addEventListener("click", () => { if (playing) clearInterval(playing); updateQuery(base!, undefined, selectedRuleset); });
  document.querySelector<HTMLSelectElement>("#api-base")?.addEventListener("change", (event) => { const selected = baseProfiles.find((item) => item.id === (event.currentTarget as HTMLSelectElement).value); if (selected) updateQuery(selected.baseUrl); });
  const updateBoard = mountBoard(document.querySelector<HTMLElement>("#board")!, model.frames[0]);
  const update = () => { const current = model.frames[index]; updateBoard(current); const ended = current.currentPlayer === null; document.querySelector("#turn")!.textContent = ended ? "Final state: the match has ended." : `Turn ${current.turn}: ${current.currentPlayer}`; document.querySelector("#score")!.textContent = `Score — black ${current.scores.black}, white ${current.scores.white}`; document.querySelector("#result")!.textContent = ended ? "The match has ended." : ""; if (match) { document.querySelector("#black-player")!.textContent = `black (${match.participants![0].display_name}:${shortRevision(match.participants![0].ai_submission_id)}) ${current.scores.black}`; document.querySelector("#white-player")!.textContent = `white (${match.participants![1].display_name}:${shortRevision(match.participants![1].ai_submission_id)}) ${current.scores.white}`; } };
  document.querySelector<HTMLButtonElement>("#initial")!.onclick = () => { index = 0; update(); };
  document.querySelector<HTMLButtonElement>("#previous")!.onclick = () => { index = Math.max(0, index - 1); update(); };
  document.querySelector<HTMLButtonElement>("#next")!.onclick = () => { index = Math.min(model.frames.length - 1, index + 1); update(); };
  document.querySelector<HTMLButtonElement>("#final")!.onclick = () => { index = model.frames.length - 1; update(); };
  document.querySelector<HTMLButtonElement>("#play")!.onclick = (event) => { const button = event.currentTarget as HTMLButtonElement; if (playing) { clearInterval(playing); playing = undefined; button.textContent = "Play"; } else { playing = window.setInterval(() => { index = Math.min(model.frames.length - 1, index + 1); update(); if (index === model.frames.length - 1) { clearInterval(playing); playing = undefined; button.textContent = "Play"; } }, 700); button.textContent = "Pause"; } };
  update();
}
function errorMessage(error: unknown) { return `Replay unavailable: ${error instanceof Error ? error.message : "unknown error"}`; }
function escape(value: string) { const node = document.createElement("span"); node.textContent = value; return node.innerHTML; }
function escapeAttribute(value: string) { return value.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;"); }

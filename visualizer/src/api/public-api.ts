import { buildReplay, type ReplayModel } from "../replay/model";

export const baseProfiles = [
  { id: "local", label: "Local", baseUrl: "http://127.0.0.1:10000" },
  { id: "stg", label: "Staging", baseUrl: "https://ai-arena-staging-p4ml.onrender.com" },
  { id: "prod", label: "Production", baseUrl: "https://ai-arena-service.onrender.com" },
] as const;

export type PublicMatch = { match_id: string; selected_run_id: string; lifecycle_state: string; game: { game_id: string; game_version: string; ruleset_version: string } };
export type PublicMatchListResponse = { items: PublicMatch[] };
export type PublicStateResponse = Omit<PublicMatch, "game"> & { availability: string; state_version?: number; public_state?: unknown; retry_after_ms: number };
export type PublicReplayResponse = { availability: string; format?: string; version?: string; payload?: unknown };

export function normalizeBaseUrl(value: string): string | undefined {
  try {
    const url = new URL(value);
    if ((url.protocol !== "http:" && url.protocol !== "https:") || url.username || url.password) return undefined;
    url.pathname = url.pathname.replace(/\/$/, ""); url.search = ""; url.hash = "";
    return url.toString().replace(/\/$/, "");
  } catch { return undefined; }
}

export function isSupportedMatch(match: PublicMatch): boolean {
  return match.lifecycle_state === "completed" && match.game?.game_id === "reversi" && /^1(?:\.|$)/.test(match.game.game_version) && match.game.ruleset_version === "standard";
}

export async function listCompletedMatches(baseUrl: string, fetcher = fetch): Promise<PublicMatch[]> {
  const base = requireBaseUrl(baseUrl);
  const response = await json<PublicMatchListResponse>(`${base}/api/v1-alpha/public/matches`, fetcher);
  return response.items.filter(isSupportedMatch).sort((left, right) => left.match_id.localeCompare(right.match_id));
}

export async function loadReplay(baseUrl: string, matchId: string, fetcher = fetch): Promise<ReplayModel> {
  const base = requireBaseUrl(baseUrl);
  const path = `${base}/api/v1-alpha/public/matches/${encodeURIComponent(matchId)}`;
  const [match, state, replay] = await Promise.all([json<PublicMatch>(path, fetcher), json<PublicStateResponse>(`${path}/state`, fetcher), json<PublicReplayResponse>(`${path}/replay`, fetcher)]);
  if (!isSupportedMatch(match) || state.selected_run_id !== match.selected_run_id || state.lifecycle_state !== match.lifecycle_state || state.availability !== "available" || replay.availability !== "available" || replay.format !== "reversi/replay" || replay.version !== "1") throw new Error("public replay is unavailable for this match");
  return buildReplay(replay.payload, { status: match.lifecycle_state, public_state: state.public_state });
}

export class StatePoller {
  private last = new Map<string, number>();
  accept(state: PublicStateResponse): boolean { const key = state.selected_run_id; const version = state.state_version ?? -1; if ((this.last.get(key) ?? -1) >= version) return false; this.last.set(key, version); return true; }
  shouldStop(state: PublicStateResponse): boolean { return ["completed", "failed", "canceled"].includes(state.lifecycle_state) || state.retry_after_ms === 0; }
}
function requireBaseUrl(value: string): string { const normalized = normalizeBaseUrl(value); if (!normalized) throw new Error("a valid public API base URL is required"); return normalized; }
async function json<T>(url: string, fetcher: typeof fetch): Promise<T> { const response = await fetcher(url, { credentials: "omit" }); if (!response.ok) throw new Error(`public API request failed (${response.status})`); return response.json() as Promise<T>; }

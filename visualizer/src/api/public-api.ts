import { buildReplay, type ReplayModel } from "../replay/model";

export type PublicMatch = { selected_run_id: string; lifecycle_state: string };
export type PublicStateResponse = PublicMatch & { availability: string; state_version?: number; public_state?: unknown; retry_after_ms: number };
export type PublicReplayResponse = { availability: string; format?: string; version?: string; payload?: unknown };

export async function loadReplay(baseUrl: string, matchId: string, fetcher = fetch): Promise<ReplayModel> {
  const base = baseUrl.replace(/\/$/, "");
  const [match, state, replay] = await Promise.all([json<PublicMatch>(`${base}/api/v1-alpha/public/matches/${encodeURIComponent(matchId)}`, fetcher), json<PublicStateResponse>(`${base}/api/v1-alpha/public/matches/${encodeURIComponent(matchId)}/state`, fetcher), json<PublicReplayResponse>(`${base}/api/v1-alpha/public/matches/${encodeURIComponent(matchId)}/replay`, fetcher)]);
  if (match.lifecycle_state !== "completed" || state.availability !== "available" || replay.availability !== "available" || replay.format !== "reversi/replay" || replay.version !== "1") throw new Error("public replay is unavailable for this match");
  return buildReplay(replay.payload, { status: match.lifecycle_state, public_state: state.public_state });
}

export class StatePoller {
  private last = new Map<string, number>();
  accept(state: PublicStateResponse): boolean { const key = state.selected_run_id; const version = state.state_version ?? -1; if ((this.last.get(key) ?? -1) >= version) return false; this.last.set(key, version); return true; }
  shouldStop(state: PublicStateResponse): boolean { return ["completed", "failed", "canceled"].includes(state.lifecycle_state) || state.retry_after_ms === 0; }
}
async function json<T>(url: string, fetcher: typeof fetch): Promise<T> { const response = await fetcher(url, { credentials: "omit" }); if (!response.ok) throw new Error(`public API request failed (${response.status})`); return response.json() as Promise<T>; }

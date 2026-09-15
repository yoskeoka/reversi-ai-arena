import { buildReplay, type ReplayModel } from "../replay/model";

export const baseProfiles = [
  { id: "local", label: "Local", baseUrl: "http://127.0.0.1:10000" },
  { id: "stg", label: "Staging", baseUrl: "https://ai-arena-staging-p4ml.onrender.com" },
  { id: "prod", label: "Production", baseUrl: "https://ai-arena-service.onrender.com" },
] as const;

export type PublicParticipant = { player_id: string; display_name: string; ai_submission_id: string };
export type PublicMatch = { match_id: string; selected_run_id: string; lifecycle_state: string; game: { game_id: string; game_version: string; ruleset_version: string }; participants?: PublicParticipant[]; completed_at?: string };
export type PublicMatchListResponse = { items: PublicMatch[] };
export type PublicStateResponse = Omit<PublicMatch, "game"> & { availability: string; state_version?: number; public_state?: unknown; retry_after_ms: number };
export type PublicReplayResponse = { availability: string; format?: string; version?: string; payload?: unknown };
export type LoadedReplay = { model: ReplayModel; match: PublicMatch };

const canonicalUUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function normalizeBaseUrl(value: string): string | undefined {
  try {
    const url = new URL(value);
    if ((url.protocol !== "http:" && url.protocol !== "https:") || url.username || url.password) return undefined;
    url.pathname = url.pathname.replace(/\/$/, ""); url.search = ""; url.hash = "";
    return url.toString().replace(/\/$/, "");
  } catch { return undefined; }
}

export function isSupportedMatch(value: unknown): value is PublicMatch {
  if (!isRecord(value) || !isRecord(value.game)) return false;
  return typeof value.match_id === "string" && typeof value.selected_run_id === "string" && value.lifecycle_state === "completed" && value.game.game_id === "reversi" && typeof value.game.game_version === "string" && /^1(?:\.|$)/.test(value.game.game_version) && value.game.ruleset_version === "standard" && hasCompletedReversiMetadata(value);
}
export function shortMatchId(matchID: string): string { const uuid = matchID.slice("match-".length); return matchID.startsWith("match-") && canonicalUUID.test(uuid) ? `match-${uuid.slice(0, 8)}` : matchID; }
export function shortRevision(revision: string): string { return canonicalUUID.test(revision) ? revision.slice(0, 8) : revision; }
export function matchOptionLabel(match: PublicMatch): string { return `${shortMatchId(match.match_id)} — ${match.completed_at!}`; }

export async function listCompletedMatches(baseUrl: string, fetcher = fetch): Promise<PublicMatch[]> {
  const base = requireBaseUrl(baseUrl);
  const response = await json<PublicMatchListResponse>(`${base}/api/v1-alpha/public/matches`, fetcher);
  if (!Array.isArray(response.items)) throw new Error("public match list is malformed");
  return response.items.filter(isSupportedMatch).sort(compareCompletedMatches);
}

export async function loadReplay(baseUrl: string, matchId: string, fetcher = fetch): Promise<LoadedReplay> {
  const base = requireBaseUrl(baseUrl);
  const path = `${base}/api/v1-alpha/public/matches/${encodeURIComponent(matchId)}`;
  const [match, state, replay] = await Promise.all([json<PublicMatch>(path, fetcher), json<PublicStateResponse>(`${path}/state`, fetcher), json<PublicReplayResponse>(`${path}/replay`, fetcher)]);
  if (!isSupportedMatch(match) || !hasCompletedReversiMetadata(state) || state.selected_run_id !== match.selected_run_id || state.lifecycle_state !== match.lifecycle_state || !sameMetadata(match, state) || state.availability !== "available" || replay.availability !== "available" || replay.format !== "reversi/replay" || replay.version !== "1") throw new Error("public replay is unavailable for this match");
  return { match, model: buildReplay(replay.payload, { status: match.lifecycle_state, public_state: state.public_state }) };
}

export class StatePoller {
  private last = new Map<string, number>();
  accept(state: PublicStateResponse): boolean { const key = state.selected_run_id; const version = state.state_version ?? -1; if ((this.last.get(key) ?? -1) >= version) return false; this.last.set(key, version); return true; }
  shouldStop(state: PublicStateResponse): boolean { return ["completed", "failed", "canceled"].includes(state.lifecycle_state) || state.retry_after_ms === 0; }
}
function requireBaseUrl(value: string): string { const normalized = normalizeBaseUrl(value); if (!normalized) throw new Error("a valid public API base URL is required"); return normalized; }
async function json<T>(url: string, fetcher: typeof fetch): Promise<T> { const response = await fetcher(url, { credentials: "omit" }); if (!response.ok) throw new Error(`public API request failed (${response.status})`); return response.json() as Promise<T>; }
function hasCompletedReversiMetadata(match: unknown): match is Omit<PublicMatch, "game"> { return isRecord(match) && isUTCDate(match.completed_at) && Array.isArray(match.participants) && match.participants.length === 2 && match.participants.every(isPublicParticipant); }
function isUTCDate(value: unknown): value is string {
  if (typeof value !== "string") return false;
  const parts = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?Z$/.exec(value);
  if (!parts) return false;
  const [, year, month, day, hour, minute, second] = parts.map(Number);
  return month >= 1 && month <= 12 && day >= 1 && day <= daysInMonth(year, month) && hour <= 23 && minute <= 59 && second <= 59;
}
function isPublicParticipant(value: unknown): value is PublicParticipant { return typeof value === "object" && value !== null && ["player_id", "display_name", "ai_submission_id"].every((key) => typeof (value as Record<string, unknown>)[key] === "string" && Boolean((value as Record<string, string>)[key].trim())); }
function sameMetadata(match: PublicMatch, state: PublicStateResponse): boolean { return match.completed_at === state.completed_at && match.participants!.every((participant, index) => participant.player_id === state.participants?.[index]?.player_id && participant.display_name === state.participants?.[index]?.display_name && participant.ai_submission_id === state.participants?.[index]?.ai_submission_id); }
function isRecord(value: unknown): value is Record<string, unknown> { return typeof value === "object" && value !== null && !Array.isArray(value); }
function daysInMonth(year: number, month: number): number { return month === 2 ? (year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0) ? 29 : 28) : [4, 6, 9, 11].includes(month) ? 30 : 31; }
function compareCompletedMatches(left: PublicMatch, right: PublicMatch): number {
  const [leftSecond, leftFraction = ""] = left.completed_at!.slice(0, -1).split(".");
  const [rightSecond, rightFraction = ""] = right.completed_at!.slice(0, -1).split(".");
  const secondOrder = rightSecond.localeCompare(leftSecond);
  if (secondOrder) return secondOrder;
  const width = Math.max(leftFraction.length, rightFraction.length);
  const fractionOrder = rightFraction.padEnd(width, "0").localeCompare(leftFraction.padEnd(width, "0"));
  return fractionOrder || left.match_id.localeCompare(right.match_id);
}

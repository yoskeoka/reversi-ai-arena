import { describe, expect, it, vi } from "vitest";
import { isSupportedMatch, listCompletedMatches, loadReplay, matchOptionLabel, normalizeBaseUrl, shortMatchId, shortRevision, type PublicMatch } from "./public-api";

const supported: PublicMatch = { match_id: "match-a", selected_run_id: "run-a", lifecycle_state: "completed", completed_at: "2026-09-15T01:02:03Z", participants: [{ player_id: "black", display_name: "Black bot", ai_submission_id: "a2471327-1111-4111-8111-111111111111" }, { player_id: "white", display_name: "White bot", ai_submission_id: "revision-white" }], game: { game_id: "reversi", game_version: "1.0.0", ruleset_version: "standard" } };

describe("public replay discovery", () => {
  it("requests only the anonymous public list and returns supported completed matches in order", async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => ({ items: [{ ...supported, match_id: "z", completed_at: "2026-09-14T01:02:03Z" }, { ...supported, match_id: "a" }, { ...supported, lifecycle_state: "queued" }] }) });
    await expect(listCompletedMatches("https://example.test/", fetcher)).resolves.toEqual([{ ...supported, match_id: "a" }, { ...supported, match_id: "z", completed_at: "2026-09-14T01:02:03Z" }]);
    expect(fetcher).toHaveBeenCalledWith("https://example.test/api/v1-alpha/public/matches", { credentials: "omit" });
  });

  it("ignores malformed list items and preserves fractional-second completion order", async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => ({ items: [null, "not a match", { ...supported, completed_at: "2026-02-29T01:02:03Z" }, { ...supported, match_id: "no-fraction" }, { ...supported, match_id: "nanosecond-one", completed_at: "2026-09-15T01:02:03.000000001Z" }, { ...supported, match_id: "nanosecond-two", completed_at: "2026-09-15T01:02:03.000000002Z" }] }) });
    await expect(listCompletedMatches("https://example.test", fetcher)).resolves.toEqual([
      { ...supported, match_id: "nanosecond-two", completed_at: "2026-09-15T01:02:03.000000002Z" },
      { ...supported, match_id: "nanosecond-one", completed_at: "2026-09-15T01:02:03.000000001Z" },
      { ...supported, match_id: "no-fraction" },
    ]);
  });

  it("filters queued, other-game, incompatible-major, and non-standard records", () => {
    expect(isSupportedMatch(supported)).toBe(true);
    expect(isSupportedMatch({ ...supported, lifecycle_state: "queued" })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, game_id: "chess" } })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, game_version: "2.0.0" } })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, ruleset_version: "experimental" } })).toBe(false);
    expect(isSupportedMatch({ ...supported, participants: [{ ...supported.participants![0] }] })).toBe(false);
    expect(isSupportedMatch({ ...supported, completed_at: "2026-02-29T01:02:03Z" })).toBe(false);
    expect(isSupportedMatch({ ...supported, completed_at: "2026-09-15T24:02:03Z" })).toBe(false);
    expect(isSupportedMatch({ ...supported, completed_at: "2016-12-31T23:59:60Z" })).toBe(false);
  });

  it("uses compact canonical UUID labels without truncating other identifiers", () => {
    expect(shortMatchId("match-a2471327-1111-4111-8111-111111111111")).toBe("match-a2471327");
    expect(shortMatchId("match-not-a-uuid")).toBe("match-not-a-uuid");
    expect(shortRevision(supported.participants![0].ai_submission_id)).toBe("a2471327");
    expect(shortRevision("revision-white")).toBe("revision-white");
    expect(matchOptionLabel(supported)).toBe("match-a — 2026-09-15T01:02:03Z");
  });

  it("normalizes valid HTTP bases and rejects credential or non-HTTP URLs", () => {
    expect(normalizeBaseUrl("https://example.test/path/")).toBe("https://example.test/path");
    expect(normalizeBaseUrl("ftp://example.test")).toBeUndefined();
  });
});

const replayPayload = { board_size: 8, ruleset: "standard", opening: [{ position: { row: 3, col: 3 }, disc: "white" }, { position: { row: 3, col: 4 }, disc: "black" }, { position: { row: 4, col: 3 }, disc: "black" }, { position: { row: 4, col: 4 }, disc: "white" }], turns: [] };
const publicState = { selected_run_id: supported.selected_run_id, lifecycle_state: supported.lifecycle_state, completed_at: supported.completed_at, participants: supported.participants, availability: "available", retry_after_ms: 0, public_state: { completed: true, current_player: null, scores: { black: 2, white: 2 }, board: Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row === 3 && col === 3) || (row === 4 && col === 4) ? "white" : (row === 3 && col === 4) || (row === 4 && col === 3) ? "black" : "empty")) } };

function replayFetcher(state = publicState) {
  return vi.fn(async (url: RequestInfo | URL) => new Response(JSON.stringify(String(url).endsWith("/state") ? state : String(url).endsWith("/replay") ? { availability: "available", format: "reversi/replay", version: "1", payload: replayPayload } : supported)));
}

describe("public replay metadata", () => {
  it("accepts equivalent detail and state metadata regardless of property order", async () => {
    const participants = supported.participants!.map(({ player_id, display_name, ai_submission_id }) => ({ ai_submission_id, display_name, player_id }));
    await expect(loadReplay("https://example.test", supported.match_id, replayFetcher({ ...publicState, participants }))).resolves.toMatchObject({ match: supported });
  });

  it.each([
    ["a different completion instant", { ...publicState, completed_at: "2026-09-15T01:02:04Z" }],
    ["a changed participant field", { ...publicState, participants: [{ ...supported.participants![0], display_name: "Other" }, supported.participants![1]] }],
    ["a changed participant sequence", { ...publicState, participants: [...supported.participants!].reverse() }],
    ["malformed state metadata", { ...publicState, completed_at: "2026-02-29T01:02:03Z" }],
  ])("rejects %s", async (_reason, state) => {
    await expect(loadReplay("https://example.test", supported.match_id, replayFetcher(state))).rejects.toThrow("public replay is unavailable");
  });
});

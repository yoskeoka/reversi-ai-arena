import { describe, expect, it, vi } from "vitest";
import { isSupportedMatch, listCompletedMatches, loadReplay, matchOptionLabel, normalizeBaseUrl, shortMatchId, shortRevision, type PublicMatch } from "./public-api";

const supported: PublicMatch = { match_id: "match-a", selected_run_id: "run-a", lifecycle_state: "completed", completed_at: "2026-09-15T01:02:03Z", participants: [{ player_id: "black", display_name: "Black bot", ai_submission_id: "a2471327-1111-4111-8111-111111111111" }, { player_id: "white", display_name: "White bot", ai_submission_id: "revision-white" }], game: { game_id: "reversi", game_version: "1.0.0", ruleset_version: "standard" } };
const list = (items: unknown[], available_ruleset_versions = ["standard", "alternate"]) => ({ pagination: { page: 1, limit: 20, total: items.length, total_pages: items.length ? 1 : 0 }, available_ruleset_versions, items });

describe("public replay discovery", () => {
  it("requests the anonymous, server-filtered first page without a ruleset by default", async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => list([supported]) });
    await expect(listCompletedMatches("https://example.test/", {}, fetcher)).resolves.toMatchObject({ matches: [supported], availableRulesetVersions: ["standard", "alternate"] });
    expect(fetcher).toHaveBeenCalledWith("https://example.test/api/v1-alpha/public/matches?game_id=reversi&game_version_major=1&page=1&limit=20&sort=completed_at&sort_order=desc", { credentials: "omit" });
  });

  it("encodes a selected ruleset and rejects items outside that scope", async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => list([{ ...supported, game: { ...supported.game, ruleset_version: "alternate" } }], ["alternate value"]) });
    await expect(listCompletedMatches("https://example.test", { rulesetVersion: "alternate value" }, fetcher)).rejects.toThrow("outside the requested scope");
    expect(fetcher.mock.calls[0][0]).toContain("ruleset_version=alternate+value");
  });

  it("retains an unavailable deep-linked ruleset as an empty scope", async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => list([], ["standard"]) });
    await expect(listCompletedMatches("https://example.test", { rulesetVersion: "retired" }, fetcher)).resolves.toMatchObject({ matches: [], availableRulesetVersions: ["standard"] });
  });

  it("rejects malformed metadata and mixed global records rather than filtering them locally", async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => list([{ ...supported, game: { ...supported.game, game_id: "chess" } }]) });
    await expect(listCompletedMatches("https://example.test", {}, fetcher)).rejects.toThrow("outside the requested scope");
    await expect(listCompletedMatches("https://example.test", {}, vi.fn().mockResolvedValue({ ok: true, json: async () => ({ items: [] }) }))).rejects.toThrow("malformed");
  });

  it("accepts all Reversi major-1 rulesets unless one is selected", () => {
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, ruleset_version: "alternate" } })).toBe(true);
    expect(isSupportedMatch(supported, "alternate")).toBe(false);
    expect(isSupportedMatch({ ...supported, lifecycle_state: "queued" })).toBe(false);
    expect(isSupportedMatch({ ...supported, participants: [{ ...supported.participants![0] }] })).toBe(false);
    expect(isSupportedMatch({ ...supported, completed_at: "2016-12-31T23:59:60Z" })).toBe(false);
  });

  it("uses compact canonical UUID labels without truncating other identifiers", () => {
    expect(shortMatchId("match-a2471327-1111-4111-8111-111111111111")).toBe("match-a2471327");
    expect(shortMatchId("match-not-a-uuid")).toBe("match-not-a-uuid");
    expect(shortRevision(supported.participants![0].ai_submission_id)).toBe("a2471327");
    expect(shortRevision("revision-white")).toBe("revision-white");
    expect(matchOptionLabel(supported)).toBe("match-a — 2026-09-15T01:02:03Z");
    expect(normalizeBaseUrl("ftp://example.test")).toBeUndefined();
  });
});

const replayPayload = { board_size: 8, ruleset: "standard", opening: [{ position: { row: 3, col: 3 }, disc: "white" }, { position: { row: 3, col: 4 }, disc: "black" }, { position: { row: 4, col: 3 }, disc: "black" }, { position: { row: 4, col: 4 }, disc: "white" }], turns: [] };
const publicState = { selected_run_id: supported.selected_run_id, lifecycle_state: supported.lifecycle_state, completed_at: supported.completed_at, participants: supported.participants, availability: "available", retry_after_ms: 0, public_state: { completed: true, current_player: null, scores: { black: 2, white: 2 }, board: Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row === 3 && col === 3) || (row === 4 && col === 4) ? "white" : (row === 3 && col === 4) || (row === 4 && col === 3) ? "black" : "empty")) } };
function replayFetcher(state = publicState, payload = replayPayload) { return vi.fn(async (url: RequestInfo | URL) => new Response(JSON.stringify(String(url).endsWith("/state") ? state : String(url).endsWith("/replay") ? { availability: "available", format: "reversi/replay", version: "1", payload } : supported))); }

describe("public replay metadata", () => {
  it("requires list/detail/payload ruleset identity", async () => {
    await expect(loadReplay("https://example.test", supported.match_id, "standard", replayFetcher())).resolves.toMatchObject({ match: supported });
    await expect(loadReplay("https://example.test", supported.match_id, "alternate", replayFetcher())).rejects.toThrow("unavailable");
    await expect(loadReplay("https://example.test", supported.match_id, "standard", replayFetcher(publicState, { ...replayPayload, ruleset: "alternate" }))).rejects.toThrow("initial-position");
  });

  it.each([["a different completion instant", { ...publicState, completed_at: "2026-09-15T01:02:04Z" }], ["a changed participant sequence", { ...publicState, participants: [...supported.participants!].reverse() }]])("rejects %s", async (_reason, state) => {
    await expect(loadReplay("https://example.test", supported.match_id, "standard", replayFetcher(state))).rejects.toThrow("public replay is unavailable");
  });
});

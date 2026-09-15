import { describe, expect, it, vi } from "vitest";
import { isSupportedMatch, listCompletedMatches, matchOptionLabel, normalizeBaseUrl, shortMatchId, shortRevision, type PublicMatch } from "./public-api";

const supported: PublicMatch = { match_id: "match-a", selected_run_id: "run-a", lifecycle_state: "completed", completed_at: "2026-09-15T01:02:03Z", participants: [{ player_id: "black", display_name: "Black bot", ai_submission_id: "a2471327-1111-4111-8111-111111111111" }, { player_id: "white", display_name: "White bot", ai_submission_id: "revision-white" }], game: { game_id: "reversi", game_version: "1.0.0", ruleset_version: "standard" } };

describe("public replay discovery", () => {
  it("requests only the anonymous public list and returns supported completed matches in order", async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => ({ items: [{ ...supported, match_id: "z", completed_at: "2026-09-14T01:02:03Z" }, { ...supported, match_id: "a" }, { ...supported, lifecycle_state: "queued" }] }) });
    await expect(listCompletedMatches("https://example.test/", fetcher)).resolves.toEqual([{ ...supported, match_id: "a" }, { ...supported, match_id: "z", completed_at: "2026-09-14T01:02:03Z" }]);
    expect(fetcher).toHaveBeenCalledWith("https://example.test/api/v1-alpha/public/matches", { credentials: "omit" });
  });

  it("filters queued, other-game, incompatible-major, and non-standard records", () => {
    expect(isSupportedMatch(supported)).toBe(true);
    expect(isSupportedMatch({ ...supported, lifecycle_state: "queued" })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, game_id: "chess" } })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, game_version: "2.0.0" } })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, ruleset_version: "experimental" } })).toBe(false);
    expect(isSupportedMatch({ ...supported, participants: [{ ...supported.participants![0] }] })).toBe(false);
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

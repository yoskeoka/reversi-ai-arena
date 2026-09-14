import { describe, expect, it, vi } from "vitest";
import { isSupportedMatch, listCompletedMatches, normalizeBaseUrl, type PublicMatch } from "./public-api";

const supported: PublicMatch = { match_id: "match-a", selected_run_id: "run-a", lifecycle_state: "completed", game: { game_id: "reversi", game_version: "1.0.0", ruleset_version: "standard" } };

describe("public replay discovery", () => {
  it("requests only the anonymous public list and returns supported completed matches in order", async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => ({ items: [{ ...supported, match_id: "z" }, { ...supported, match_id: "a" }, { ...supported, lifecycle_state: "queued" }] }) });
    await expect(listCompletedMatches("https://example.test/", fetcher)).resolves.toEqual([{ ...supported, match_id: "a" }, { ...supported, match_id: "z" }]);
    expect(fetcher).toHaveBeenCalledWith("https://example.test/api/v1-alpha/public/matches", { credentials: "omit" });
  });

  it("filters queued, other-game, incompatible-major, and non-standard records", () => {
    expect(isSupportedMatch(supported)).toBe(true);
    expect(isSupportedMatch({ ...supported, lifecycle_state: "queued" })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, game_id: "chess" } })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, game_version: "2.0.0" } })).toBe(false);
    expect(isSupportedMatch({ ...supported, game: { ...supported.game, ruleset_version: "experimental" } })).toBe(false);
  });

  it("normalizes valid HTTP bases and rejects credential or non-HTTP URLs", () => {
    expect(normalizeBaseUrl("https://example.test/path/")).toBe("https://example.test/path");
    expect(normalizeBaseUrl("ftp://example.test")).toBeUndefined();
  });
});

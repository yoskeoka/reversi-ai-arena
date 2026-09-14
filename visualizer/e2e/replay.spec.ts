import { expect, test } from "@playwright/test";

test("renders a keyboard-accessible terminal replay summary", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Reversi replay visualizer" })).toBeVisible();
  await expect(page.getByText("Completed terminal replay")).toBeVisible();
  await expect(page.getByRole("button", { name: "Next turn" })).toBeVisible();
  await expect(page.getByText(/Score/)).toBeVisible();
});

test("discovers and replays a public completed match without private requests or run controls", async ({ page }) => {
  const base = "https://ai-arena-staging-p4ml.onrender.com";
  const match = { match_id: "public-match", selected_run_id: "selected-run", lifecycle_state: "completed", game: { game_id: "reversi", game_version: "1.0.0", ruleset_version: "standard" } };
  const board = Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row === 3 && col === 3) || (row === 4 && col === 4) ? "white" : (row === 3 && col === 4) || (row === 4 && col === 3) ? "black" : "empty"));
  const seen: string[] = [];
  page.on("request", (request) => { if (request.url().startsWith(base)) seen.push(request.url()); });
  await page.route(`${base}/**`, async (route) => {
    const path = new URL(route.request().url()).pathname;
    const body = path.endsWith("/matches") ? { items: [match] }
      : path.endsWith("/state") ? { selected_run_id: "selected-run", lifecycle_state: "completed", availability: "available", retry_after_ms: 0, public_state: { completed: true, current_player: null, scores: { black: 2, white: 2 }, board } }
        : path.endsWith("/replay") ? { availability: "available", format: "reversi/replay", version: "1", payload: { board_size: 8, ruleset: "standard", opening: [{ position: { row: 3, col: 3 }, disc: "white" }, { position: { row: 3, col: 4 }, disc: "black" }, { position: { row: 4, col: 3 }, disc: "black" }, { position: { row: 4, col: 4 }, disc: "white" }], turns: [] } }
          : match;
    await route.fulfill({ contentType: "application/json", body: JSON.stringify(body) });
  });
  await page.goto("/");
  await page.getByLabel("Public API base").selectOption("stg");
  await page.getByLabel("Completed Reversi match").selectOption("public-match");
  await expect(page).toHaveURL(/api=https%3A%2F%2Fai-arena-staging-p4ml\.onrender\.com.*match=public-match/);
  await expect(page.getByText("Completed terminal replay")).toBeVisible();
  expect(seen).toHaveLength(4);
  expect(seen.every((url) => url.includes("/api/v1-alpha/public/matches") && !url.includes("operator") && !url.includes("artifact"))).toBe(true);
  await expect(page.getByText(/run/i)).not.toBeVisible();
});

import { expect, test } from "@playwright/test";

test("renders a keyboard-accessible terminal replay summary", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Reversi replay visualizer" })).toBeVisible();
  await expect(page.getByText("Completed terminal replay")).toBeVisible();
  await expect(page.getByRole("button", { name: "Next turn" })).toBeVisible();
  await expect(page.getByText(/Score/)).toBeVisible();
});

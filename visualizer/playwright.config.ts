import { defineConfig } from "@playwright/test";
export default defineConfig({ use: { baseURL: "http://127.0.0.1:4174" }, testDir: "./e2e", webServer: { command: "npm run dev -- --port 4174", port: 4174, reuseExistingServer: false } });

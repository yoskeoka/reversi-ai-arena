import { defineConfig } from "@playwright/test";
export default defineConfig({ use: { baseURL: "http://127.0.0.1:4173" }, testDir: "./e2e", webServer: { command: "npm run dev -- --port 4173", port: 4173, reuseExistingServer: !process.env.CI } });

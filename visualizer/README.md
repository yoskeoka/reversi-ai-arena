# Visualizer Surface

`visualizer/` owns replay and future spectator clients.

The initial shell uses a lightweight Vite plus TypeScript setup with Phaser for
board rendering and no React dependency.

## Local Run

From this directory:

```sh
npm ci
npm run dev
```

Open `http://127.0.0.1:4173`. With no query parameters, the application renders
its built-in terminal fixture and does not need `ai-arena`.

To connect this shell to a local, staging, or production `ai-arena` public API,
see the repository [development guide](../DEVELOPMENT.md). The viewer only
sends credential-free public API requests and requires a completed,
discoverable Reversi match.

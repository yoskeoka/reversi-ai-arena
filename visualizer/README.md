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

To load a completed anonymous public match, use:

```text
http://127.0.0.1:4173/?api=http://127.0.0.1:10000&match=<match-id>
```

The API service needs normal `ai-arena` local setup first. Follow its
[development guide](https://github.com/yoskeoka/ai-arena/blob/main/DEVELOPMENT.md),
then start the backend with `make start-backend-local` from the `ai-arena`
repository. The viewer only sends credential-free public API requests and
requires a completed, discoverable Reversi match.

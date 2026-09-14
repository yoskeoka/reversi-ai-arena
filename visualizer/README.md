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
its built-in terminal fixture and does not contact `ai-arena`.

Choose Local, Staging, or Production, then choose a completed standard Reversi
match. The profiles are `http://127.0.0.1:10000`,
`https://ai-arena-staging-p4ml.onrender.com`, and
`https://ai-arena-service.onrender.com` respectively. The viewer only sends
credential-free public API requests.

A replay can be shared as a deep link:

```text
http://127.0.0.1:4173/?api=https://ai-arena-staging-p4ml.onrender.com&match=<match-id>
```

Any valid `api` URL in a deep link remains selected as a Custom base. The
viewer supports the current standard Reversi ruleset only; ruleset selection is
deferred to [#41](../docs/issues/0004-visualizer-ruleset-selection.md).

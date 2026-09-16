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

Choose Local, Staging, or Production, optionally choose a Reversi ruleset,
then choose a completed match. Discovery always requests the first 20 completed
Reversi major-1 matches in descending completion order; the public response
supplies the scope-wide ruleset options. The profiles are `http://127.0.0.1:10000`,
`https://ai-arena-staging-p4ml.onrender.com`, and
`https://ai-arena-service.onrender.com` respectively. The viewer only sends
credential-free public API requests.

Each completed-match option shows its compact identifier and UTC completion
time. Replay summaries show Black and White public display names, immutable
submission revisions, and scores. This requires the compatible public
participant-order contract tracked by [ai-arena#365](https://github.com/yoskeoka/ai-arena/issues/365).

A replay can be shared as a deep link:

```text
http://127.0.0.1:4173/?api=https://ai-arena-staging-p4ml.onrender.com&ruleset=standard&match=<match-id>
```

Any valid `api` URL in a deep link remains selected as a Custom base. A ruleset
that is unavailable in the returned scope stays visible as unavailable. This
feature requires ai-arena's filtered public-match-list API release to be
deployed before using it against staging or production.

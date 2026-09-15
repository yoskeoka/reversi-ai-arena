# Development Guide

## Connect The Visualizer To `ai-arena`

Start the visualizer from `visualizer/`:

```sh
npm ci
npm run dev
```

Without query parameters, `http://127.0.0.1:4173` renders the built-in terminal
fixture and does not contact `ai-arena`.

To load a completed, discoverable Reversi match from an `ai-arena` public API,
open the visualizer with both `api` and `match` query parameters:

```text
http://127.0.0.1:4173/?api=<api-base-url>&match=<match-id>
```

Select an environment by replacing `<api-base-url>` with one of the following
base URLs. Reload after changing the URL to switch environments.

| Environment | API base URL | Prerequisite |
| --- | --- | --- |
| Local | `http://127.0.0.1:10000` | Complete the [ai-arena development guide](https://github.com/yoskeoka/ai-arena/blob/main/DEVELOPMENT.md), then run `make start-backend-local` in that repository. |
| Staging | `https://ai-arena-staging-p4ml.onrender.com` | Use a completed, publicly discoverable Reversi match in staging. |
| Production | `https://ai-arena-service.onrender.com` | Use a completed, publicly discoverable Reversi match in production. |

For example, a staging replay URL is:

```text
http://127.0.0.1:4173/?api=https://ai-arena-staging-p4ml.onrender.com&match=<match-id>
```

The visualizer uses only the anonymous public spectator API. It never reads
private runner artifacts or sends credentials.

Completed Reversi discovery requires the public completion time plus ordered
participant metadata: the first entry is Black and the second is White. Live
API compatibility is tracked by [ai-arena#365](https://github.com/yoskeoka/ai-arena/issues/365).

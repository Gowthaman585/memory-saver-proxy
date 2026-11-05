# Architecture

[← Back to README](../README.md)

## Request flow

```
 1. Client sends chat request ──▶ proxy (:3000)
 2. Proxy logs every message in the request to SQLite
 3. Proxy searches past messages (FTS5) using the latest user message
 4. Top 3 matches are added as a system message at the start
 5. History is trimmed to the message and token budgets
 6. Request is forwarded to Ollama (:11434)
 7. Reply and latency are logged, reply is returned to the client
```

Requests that are not valid chat payloads (no `messages` array) skip steps 2 to 5 and are forwarded untouched.

## Modules

| File | Responsibility |
|---|---|
| `src/main.rs` | Loads `.env`, initializes the database, builds the Axum router (health check at `/`, everything else goes to the proxy), starts the server on port 3000. |
| `src/db.rs` | Opens `chat_history.db`, creates the schema, FTS5 index and triggers, migrates old databases, and provides `insert_message` and `search_messages`. |
| `src/proxy.rs` | The catch-all handler: parses the request, logs, retrieves memory, trims, forwards to Ollama, parses and logs the reply. |

## Key design points

- **Other request fields are preserved.** `ChatRequest` keeps unknown fields (`model`, `stream`, `options`, and so on) with `#[serde(flatten)]`, so rewriting the prompt does not drop settings.
- **System messages survive trimming.** A leading system message (including injected memory) is removed before trimming and put back afterwards.
- **Streamed replies are logged.** Ollama streams one JSON object per line. The proxy reads line by line and joins the content pieces before saving.
- **Search is safe by construction.** Query text is stripped to letters, digits and spaces, then each word is quoted before it reaches FTS5. If nothing matches, the most recent messages are returned as a fallback.

## Technology

| Purpose | Crate |
|---|---|
| Web server and routing | `axum` |
| Async runtime | `tokio` |
| HTTP client to Ollama | `reqwest` |
| JSON | `serde`, `serde_json` |
| Database | `rusqlite` (bundled SQLite with FTS5) |
| Configuration | `dotenvy` |
| Logging | `tracing`, `tracing-subscriber` (currently plain `println!` is used) |

Next: [Database](database.md)

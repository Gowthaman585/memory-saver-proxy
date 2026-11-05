# Limitations

[← Back to README](../README.md)

This is an MVP. These are the known gaps, so you can decide whether it fits your use.

| Area | Limitation | Effect |
|---|---|---|
| **Streaming** | The proxy waits for Ollama's full reply before returning it. | Replies arrive all at once instead of token by token. |
| **Search quality** | FTS5 matches keywords, not meaning. | Differently worded references to old content may not be found. |
| **Self-retrieval** | The new message is logged before searching, so it can match itself. | One of the 3 memory slots may be wasted on the current question. |
| **Duplicates** | Clients resend the whole history each turn and every message is stored again. | The database grows quickly and retrieval can return repeats. |
| **No sessions** | All conversations share one history. | Memory from one project can appear in another. |
| **Token estimate** | Tokens are estimated as characters ÷ 4. | Budgets are approximate, especially for code or non-English text. |
| **Endpoint coverage** | Only requests with a `messages` array are logged and trimmed. Replies are parsed in Ollama's chat format. | `/api/generate` is forwarded untouched; OpenAI-style `/v1/chat/completions` requests are trimmed but replies are not logged. |
| **Performance** | A new database connection and HTTP client are created per request. | Fine for one user, wasteful under load. |
| **Fixed settings** | Ports, Ollama URL and database path are hard-coded. | Cannot run on other ports or point at a remote Ollama without editing code. |
| **Security** | Binds to `127.0.0.1` only, no authentication, conversations stored unencrypted. | Intended for single-user local use only. |
| **Testing** | No automated tests yet. | Changes must be checked by hand. |

Planned fixes are listed in the [Roadmap](roadmap.md).

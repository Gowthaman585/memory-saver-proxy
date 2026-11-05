# Configuration

[← Back to README](../README.md)

Settings are read from environment variables. A `.env` file in the working directory is loaded automatically at startup.

| Variable | Default | Description |
|---|---|---|
| `MAX_CONTEXT_MESSAGES` | `6` | Maximum messages sent to Ollama, including the injected memory or system message. |
| `MAX_CONTEXT_TOKENS` | `2048` | Estimated token ceiling for the prompt. Oldest messages are dropped first. |
| `FTS_ENABLED` | `true` | Set to `false` or `0` to disable long-term memory retrieval. |

Example `.env`:

```env
MAX_CONTEXT_MESSAGES=8
MAX_CONTEXT_TOKENS=3072
FTS_ENABLED=true
```

## How the limits interact

1. If the message count is above `MAX_CONTEXT_MESSAGES`, the oldest messages are removed. A leading system message is always kept.
2. If the estimated tokens are still above `MAX_CONTEXT_TOKENS`, more old messages are removed, again keeping the system message.
3. Token counts are an estimate: total characters divided by 4.

## Tuning tips

- Start with the defaults, then raise limits until replies slow down or VRAM runs out.
- Memory retrieval adds up to 3 snippets as one system message. If your prompts feel too large, lower `MAX_CONTEXT_TOKENS` or turn retrieval off to compare.
- Trimming only saves VRAM if your Ollama context size (`num_ctx`) is large enough for the prompt to matter. For extra savings, also look at Ollama's `OLLAMA_FLASH_ATTENTION=1` and `OLLAMA_KV_CACHE_TYPE=q8_0` settings.

## Fixed values (not yet configurable)

| Setting | Value |
|---|---|
| Proxy address | `127.0.0.1:3000` |
| Ollama address | `127.0.0.1:11434` |
| Database file | `chat_history.db` in the working directory |
| Memory snippets per request | 3 |

Making these configurable is on the [Roadmap](roadmap.md).

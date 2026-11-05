<div align="center">

# Memory Saver Proxy

**Long-term memory for local AI models, on laptops with very little VRAM.**

A small Rust proxy that sits between your tool and [Ollama](https://ollama.com). It keeps prompts short and remembers everything you have ever said.

![Status](https://img.shields.io/badge/status-MVP-orange)
![Language](https://img.shields.io/badge/language-Rust-b7410e)
![License](https://img.shields.io/badge/license-GPLv3-blue)
![Backend](https://img.shields.io/badge/backend-Ollama-black)

</div>

> **🤖 Built with AI.** This project was designed and coded with the help of **Claude** (Anthropic) and **Gemini** (Google). The idea, direction and maintenance are by [Gowthaman M](#author).

---

## Overview

Small GPUs force a trade-off: a small model is fast but unreliable, and a larger model is smarter but leaves almost no VRAM for conversation history. Cutting old messages saves memory but makes the model forget.

Memory Saver Proxy fixes both sides:

| | What it does |
|---|---|
| **Trim** | Limits each request to a fixed message and token budget, so memory use stays predictable. |
| **Remember** | Stores every message in a local SQLite database and, before each request, injects the most relevant past snippets back into the prompt. |

Your tool only needs its base URL changed from `:11434` to `:3000`.

```
Your tool ──▶ Memory Saver Proxy (:3000) ──▶ Ollama (:11434)
                 │  log · search · trim
                 ▼
            chat_history.db (SQLite + FTS5)
```

## Features

- Transparent proxy: all paths are forwarded; non-chat requests pass through unchanged
- Persistent chat log with per-reply latency
- Long-term memory retrieval using SQLite FTS5 (BM25 ranking)
- Message-count and token budgeting that preserves system prompts
- Single small binary with bundled SQLite, no extra services

## Quick start

```bash
git clone https://github.com/<your-username>/memory-saver-proxy
cd memory-saver-proxy
cp .env.example .env
cargo run --release
```

Then point your tool at `http://127.0.0.1:3000`. Full steps are in [Getting Started](docs/getting-started.md).

## Documentation

| Page | What is inside |
|---|---|
| [Getting Started](docs/getting-started.md) | Requirements, install, first request, using it with your tools |
| [Configuration](docs/configuration.md) | Environment variables and tuning tips |
| [Architecture](docs/architecture.md) | How a request flows through the proxy, module by module |
| [Database](docs/database.md) | Tables, full-text index, queries for inspecting history |
| [Background](docs/background.md) | The problem, design decisions and what this is not |
| [Limitations](docs/limitations.md) | What the MVP does not do yet |
| [Roadmap](docs/roadmap.md) | Planned improvements in priority order |
| [Contributing](docs/contributing.md) | How to build, test and submit changes |

## Project status

This is an **MVP (v0.1.0)**. It works end to end for Ollama chat requests, but it is not production software. Read [Limitations](docs/limitations.md) before relying on it.

## Author

**Gowthaman M**, Computer Science and Engineering (Cybersecurity) student.

## License

Copyright (C) 2026 Gowthaman M. Licensed under the **GNU General Public License v3.0 or later**. See [LICENSE](LICENSE).

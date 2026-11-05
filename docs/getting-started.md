# Getting Started

[← Back to README](../README.md)

## Requirements

- [Rust](https://rustup.rs) 1.85 or newer (the project uses Rust edition 2024)
- A C compiler for the bundled SQLite build (`gcc` on Linux, Xcode command line tools on macOS)
- [Ollama](https://ollama.com) running locally on the default port `11434`

Fedora example:

```bash
sudo dnf install gcc
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Install and run

```bash
git clone https://github.com/<your-username>/memory-saver-proxy
cd memory-saver-proxy
cp .env.example .env      # optional, see Configuration
cargo run --release
```

Expected startup output:

```
⚙️ MAX_CONTEXT_MESSAGES loaded: 6 (default)
⚙️ MAX_CONTEXT_TOKENS loaded: 2048 (default)
📦 Initializing local database...
✅ Database initialized successfully!
🌐 Proxy server running at http://127.0.0.1:3000
```

Health check:

```bash
curl http://127.0.0.1:3000/
```

## Send your first request

```bash
curl http://127.0.0.1:3000/api/chat -d '{
  "model": "qwen2.5-coder:7b",
  "stream": false,
  "messages": [{"role": "user", "content": "Write a Rust function that reverses a string."}]
}'
```

Use `"stream": false` for now. Streaming responses are logged correctly but are delivered only after the full reply is ready (see [Limitations](limitations.md)).

## Use it with your tools

Change the Ollama base URL in your tool from `http://127.0.0.1:11434` to `http://127.0.0.1:3000`. Any client that uses Ollama's `/api/chat` endpoint should work.

## Check that it is remembering

```bash
sqlite3 chat_history.db "SELECT role, substr(content,1,80), latency_ms FROM messages ORDER BY id DESC LIMIT 10;"
```

Next: [Configuration](configuration.md)

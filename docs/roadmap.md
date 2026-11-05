# Roadmap

[← Back to README](../README.md)

Items are listed in priority order within each stage.

## Stage 1: Make it usable every day

- [ ] Stream Ollama responses through to the client while logging the full reply
- [ ] Add a conversation or session ID, and search within it by default
- [ ] De-duplicate messages so history is stored once
- [ ] Search before logging, so the current message cannot retrieve itself
- [ ] Reuse one database connection and one HTTP client

## Stage 2: Make memory smarter

- [ ] Semantic search with local embeddings (for example `nomic-embed-text` through Ollama), combined with FTS5
- [ ] Store embeddings in SQLite and rank by similarity
- [ ] Limit the size of injected memory and prefer shorter, more relevant snippets
- [ ] Use real tokenizer counts instead of characters ÷ 4

## Stage 3: Make it flexible

- [ ] Configurable proxy port, Ollama URL and database path
- [ ] OpenAI-compatible endpoint support (`/v1/chat/completions`), including reply logging
- [ ] Replace `println!` with `tracing` logs and log levels
- [ ] Commands to export, prune and clear history

## Stage 4: Make it trustworthy

- [ ] Unit and integration tests, including a mock Ollama server
- [ ] Continuous integration (format, lint, test)
- [ ] Benchmark: answer quality and latency with and without memory over a long session
- [ ] Prebuilt release binaries

Have an idea? Open an issue. See [Contributing](contributing.md).

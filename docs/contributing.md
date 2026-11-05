# Contributing

[← Back to README](../README.md)

Issues and pull requests are welcome, from bug reports to roadmap items.

## Set up

```bash
git clone https://github.com/<your-username>/memory-saver-proxy
cd memory-saver-proxy
cargo build
```

You need Rust 1.85 or newer and a running Ollama to test end to end.

## Before you submit

```bash
cargo fmt
cargo clippy
cargo build --release
```

Then test manually:

1. Start Ollama and the proxy.
2. Send a chat request through port 3000 with `"stream": false`, then with streaming on.
3. Confirm new rows appear in `chat_history.db`.

## Good first contributions

- Make the ports, Ollama URL and database path configurable
- Add unit tests for `estimate_tokens` and the trimming logic
- Replace `println!` calls with `tracing`
- Pick any unchecked item in the [Roadmap](roadmap.md)

## Guidelines

- Keep changes small and focused, one idea per pull request.
- Keep the project lightweight: avoid adding heavy dependencies without discussion.
- Never commit `chat_history.db` or `.env`.
- Mention in your pull request if AI tools helped write the code, as this project did.

## License

By contributing, you agree that your work is licensed under the GNU General Public License v3.0 or later.

# Background

[← Back to README](../README.md)

## The problem

On a laptop with a small GPU (for example 4 GB of VRAM), running local coding models involves a trade-off:

- A **small model** fits entirely in VRAM and is fast, but is less reliable for code analysis and debugging.
- A **larger model** is smarter, but part of it runs on the CPU, leaving little VRAM for the conversation.

Long sessions make this worse. Every message adds to the prompt, and a bigger prompt costs more memory and time.

## Why trimming alone is not enough

The obvious fix is to send only the last few messages. That saves memory, but the model then forgets earlier decisions, file names and instructions. The session feels like talking to someone who keeps losing their train of thought.

## The idea

Keep the prompt small, but never actually lose the history:

1. Save every message locally.
2. Before each request, look up older messages related to the new question.
3. Add only those few snippets to the prompt.

This is a lightweight form of retrieval-augmented generation (RAG).

## Design decisions

| Decision | Reason |
|---|---|
| **A proxy, not a chat app** | Works with any tool that speaks to Ollama; no new interface to learn. |
| **Rust** | Small, fast, single binary, low overhead on a memory-constrained machine. |
| **SQLite** | Zero setup, one file, easy to inspect and back up. |
| **FTS5 keyword search first** | No extra model needed, so no additional VRAM. Good enough to prove the idea. |
| **Local only** | Conversations (often source code) never leave the machine. |

## Known weakness of this approach

Keyword search matches words, not meaning. A request like "put the code back how it was" will not find an older message that says "initial version". Adding embedding-based search is the first planned improvement. See the [Roadmap](roadmap.md).

## What this project is not

- Not a chat interface
- Not a replacement for a full agent or memory framework
- Not a way to make a model smarter, only to help it remember

## Similar projects

Heavier alternatives include memory layers such as Mem0 and Letta, and chat front-ends with built-in memory such as Open WebUI. Memory Saver Proxy aims to stay small and tool-agnostic.

Next: [Limitations](limitations.md)

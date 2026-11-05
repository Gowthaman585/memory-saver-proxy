# Database

[← Back to README](../README.md)

All data is stored in a single SQLite file, `chat_history.db`, created in the working directory on first run.

> **Privacy note:** this file contains your full conversations, including any code or secrets you paste. It stays on your machine. Do not commit it to Git (add it to `.gitignore`).

## Tables

### `messages`

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER, primary key, autoincrement | |
| `role` | TEXT, not null | `system`, `user` or `assistant` |
| `content` | TEXT, not null | The message text |
| `latency_ms` | INTEGER | Set for assistant replies: time Ollama took to respond |
| `timestamp` | DATETIME | Defaults to the current time (UTC) |

### `messages_fts`

An FTS5 virtual table over `messages.content`, using the `porter unicode61` tokenizer, so "running" and "run" match each other. It is an external-content index tied to `messages` by row id.

## Triggers

| Trigger | Fires | Effect |
|---|---|---|
| `messages_ai` | After insert | Adds the new row to the FTS index |
| `messages_ad` | After delete | Removes the row from the FTS index |

There is no update trigger yet, because messages are never edited.

## Migration

Older databases created before latency tracking lack the `latency_ms` column. At startup, `init_db` checks the table and adds the column if it is missing.

## Retrieval query

```sql
SELECT m.content
FROM messages_fts fts
JOIN messages m ON fts.rowid = m.id
WHERE messages_fts MATCH ?1
ORDER BY rank
LIMIT ?2;
```

Words from the latest user message are joined with `OR`, and results are ranked by BM25.

## Handy queries

```bash
# Latest 10 messages
sqlite3 chat_history.db "SELECT id, role, substr(content,1,80) FROM messages ORDER BY id DESC LIMIT 10;"

# Average reply latency
sqlite3 chat_history.db "SELECT ROUND(AVG(latency_ms)) FROM messages WHERE latency_ms IS NOT NULL;"

# Search by hand
sqlite3 chat_history.db "SELECT content FROM messages_fts WHERE messages_fts MATCH 'borrow OR lifetime' LIMIT 5;"

# Start fresh
rm chat_history.db
```

Next: [Background](background.md)

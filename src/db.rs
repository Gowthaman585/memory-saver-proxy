use rusqlite::{Connection, Result, params};

pub fn init_db() -> Result<Connection> {
    // 1. Open (or create) a local SQLite database file named 'chat_history.db'
    let conn = Connection::open("chat_history.db")?;

    // 2. Create a table to store our conversation history
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            latency_ms INTEGER,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;    //?: allows the below code to execute only when table is created or else exit

    // 3. Create the FTS5 virtual table for lightning-fast full-text search indexing
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            content,
            content='messages',
            content_rowid='id',
            tokenize='porter unicode61'
        )",
        [],
    )?;

    // 4. Create triggers to automatically keep the FTS index in sync with the messages table
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
            INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
        END;",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
            INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', old.id, old.content);
        END;",
        [],
    )?;

    // 5. 🛠 MIGRATION: older databases were created WITHOUT the `latency_ms` column.
    //    `CREATE TABLE IF NOT EXISTS` skips existing tables, so we must add the column manually.
    let has_latency_col = conn
        .prepare("PRAGMA table_info(messages)")?
        .query_map([], |row| row.get::<_, String>(1))?   // column index 1 = column name
        .filter_map(Result::ok)
        .any(|name| name == "latency_ms");

    if !has_latency_col {
        conn.execute("ALTER TABLE messages ADD COLUMN latency_ms INTEGER", [])?;
        println!("🛠 Migrated database: added missing 'latency_ms' column.");
    }

    Ok(conn)        //returns the success state
}

/// Helper function to insert a chat message into the database
pub fn insert_message(conn: &Connection, role: &str, content: &str, latency_ms: Option<u64>) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO messages (role, content, latency_ms) VALUES (?1, ?2, ?3)",
        params![role, content, latency_ms.map(|l| l as i64)],
    )?;
    Ok(())
}

/// Search chat history using SQLite FTS5 with BM25 ranking.
/// If no FTS5 matches are found, it falls back to grabbing the most recent messages.
pub fn search_messages(conn: &Connection, query_str: &str, limit: usize) -> Result<Vec<String>> {
    // 1. Sanitize the query string to keep only alphanumeric characters and whitespace for FTS5 safety
    let cleaned_query = query_str
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>();
    let trimmed = cleaned_query.trim();

    let mut results = Vec::new();

    // 2. Run FTS5 search if query isn't empty
    if !trimmed.is_empty() {
        let fts_query = trimmed
            .split_whitespace()
            .map(|word| format!("\"{}\"", word))
            .collect::<Vec<String>>()
            .join(" OR ");

        let mut stmt = conn.prepare(
            "SELECT m.content 
             FROM messages_fts fts
             JOIN messages m ON fts.rowid = m.id
             WHERE messages_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![fts_query, limit as i64], |row| row.get(0))?;
        for row in rows {
            if let Ok(content) = row {
                results.push(content);
            }
        }
    }

    // 3. Fallback: If FTS5 found nothing, fetch the most recent messages from the main table
    if results.is_empty() {
        let mut stmt = conn.prepare(
            "SELECT content FROM messages ORDER BY id DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| row.get(0))?;
        for row in rows {
            if let Ok(content) = row {
                results.push(content);
            }
        }
        results.reverse();
    }

    Ok(results)
}

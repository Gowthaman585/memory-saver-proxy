use rusqlite::{Connection, Result,params};

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
    )?;     //?: allows the below code to execute only when table is created or else exit

    // 3. 🛠 MIGRATION: older databases were created WITHOUT the `latency_ms` column.
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

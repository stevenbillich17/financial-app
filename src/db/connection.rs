use rusqlite::{Connection, Result};

pub fn establish_connection() -> Result<Connection> {
    let conn = Connection::open("financial_app.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            description TEXT NOT NULL,
            amount TEXT NOT NULL,
            transaction_type TEXT NOT NULL CHECK (transaction_type IN ('income', 'expense')),
            category TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS category_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pattern TEXT NOT NULL,
            category TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

#[cfg(test)]
pub fn establish_test_connection() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute(
        "CREATE TABLE transactions (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            description TEXT NOT NULL,
            amount TEXT NOT NULL,
            transaction_type TEXT NOT NULL CHECK (transaction_type IN ('income', 'expense')),
            category TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE category_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pattern TEXT NOT NULL,
            category TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}
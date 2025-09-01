use rusqlite::{Connection, OpenFlags};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = Connection::open_with_flags(
        db_path(), 
        OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_URI,
    ).expect("failed to open db");

    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;
        "#,
    ).expect("Failed to apply PRAGMAs");

    Mutex::new(conn)
});

fn db_path() -> std::path::PathBuf {
    use directories::BaseDirs;
    let mut database_path = BaseDirs::new().unwrap().data_dir().to_path_buf();
    database_path.push("zealotd");
    std::fs::create_dir_all(&database_path).unwrap();
    database_path.push("scope.db");
    database_path
}
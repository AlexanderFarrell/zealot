use rusqlite::{Connection, OpenFlags};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::result::Result;

pub static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = Connection::open_with_flags(
        data_path(), 
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

    eprintln!("Connected to database at {:?}", data_path());

    Mutex::new(conn)
});

fn data_path() -> std::path::PathBuf {
    use directories::BaseDirs;
    let mut database_path = BaseDirs::new().unwrap().data_dir().to_path_buf();
    database_path.push("zealotd");
    std::fs::create_dir_all(&database_path).unwrap();
    database_path.push("scope.db");
    database_path
}

pub fn database_seed(sql: &String) -> Result<(), String> {
    let exists = std::fs::exists(data_path());
    if let Err(error) = exists {
        return Err(format!("Failed to seed database: {error}"));
    }

    if exists.unwrap() {
        println!("Database already initialized, skipping");
        return Ok(());
    }

    let db = DB.lock().unwrap();
    
    match db.execute_batch(sql.as_str()) {
        Err(err) => {
            Err(format!("Failed to seed database: {err}"))
        },
        Ok(_) => {Ok(())},
    }
}
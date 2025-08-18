import sqlite3
from .config import BASE_DIR, APP_NAME

DB_PATH = BASE_DIR / f"{APP_NAME}.db"
_CONN: sqlite3.Connection | None = None

def init_db(sql):
    global _CONN
    if _CONN is not None:
        raise Exception("Connection already initialized")
    BASE_DIR.mkdir(mode=0o700, parents=True, exist_ok=True)
    _CONN = sqlite3.connect(DB_PATH, timeout=5, isolation_level=None)
    sql_statement = "PRAGMA journal_mode=WAL; " + "PRAGMA foreign_keys=ON; " + sql;
    
    statements = sql_statement.split(';')

    for s in statements:
        _CONN.execute(s)


def close_db():
    raise NotImplementedError()
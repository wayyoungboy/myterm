pub mod models;
pub mod schema;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DbConn(pub Mutex<Connection>);

impl DbConn {
    pub fn new(path: PathBuf) -> Self {
        let conn = Connection::open(&path).expect("Failed to open database");
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .expect("Failed to set pragmas");
        schema::init_db(&conn).expect("Failed to initialize database");
        DbConn(Mutex::new(conn))
    }
}

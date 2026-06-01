use rusqlite::Connection;

pub fn init_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS groups (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            parent_id   TEXT REFERENCES groups(id) ON DELETE CASCADE,
            icon        TEXT,
            sort_order  INTEGER DEFAULT 0,
            created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS connections (
            id              TEXT PRIMARY KEY,
            group_id        TEXT REFERENCES groups(id) ON DELETE SET NULL,
            name            TEXT NOT NULL,
            host            TEXT NOT NULL,
            port            INTEGER DEFAULT 22,
            auth_type       TEXT CHECK(auth_type IN ('password','key','credential','interactive','ask')),
            username        TEXT,
            password_enc    TEXT,
            key_path        TEXT,
            credential_id   TEXT,
            proxy_type      TEXT,
            proxy_host      TEXT,
            proxy_port      INTEGER,
            proxy_jump_id   TEXT REFERENCES connections(id) ON DELETE SET NULL,
            init_command    TEXT,
            init_path       TEXT,
            timeout_ms      INTEGER,
            heartbeat_ms    INTEGER DEFAULT 5000,
            remark          TEXT,
            created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS quick_commands (
            id          TEXT PRIMARY KEY,
            group_id    TEXT,
            name        TEXT NOT NULL,
            command     TEXT NOT NULL,
            shortcut    TEXT,
            sort_order  INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS notes (
            id              TEXT PRIMARY KEY,
            connection_id   TEXT REFERENCES connections(id) ON DELETE CASCADE,
            group_id        TEXT REFERENCES groups(id) ON DELETE CASCADE,
            title           TEXT,
            content         TEXT,
            created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS ai_conversations (
            id          TEXT PRIMARY KEY,
            title       TEXT,
            created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS ai_messages (
            id              TEXT PRIMARY KEY,
            conversation_id TEXT REFERENCES ai_conversations(id) ON DELETE CASCADE,
            role            TEXT CHECK(role IN ('user','assistant','system')),
            content         TEXT NOT NULL,
            created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS settings (
            key     TEXT PRIMARY KEY,
            value   TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}

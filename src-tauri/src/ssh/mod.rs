pub mod connection;
pub mod sftp;

use ssh2::Session;
use std::net::TcpStream;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;

pub struct SshSession {
    pub session: Session,
    pub _stream: TcpStream,
}

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SshSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, id: &str) -> Option<ssh2::Session> {
        let sessions = self.sessions.lock();
        sessions.get(id).map(|s| s.session.clone())
    }

    pub fn insert(&self, id: String, session: SshSession) {
        let mut sessions = self.sessions.lock();
        sessions.insert(id, session);
    }

    pub fn remove(&self, id: &str) {
        let mut sessions = self.sessions.lock();
        sessions.remove(id);
    }
}

pub mod connection;
pub mod sftp;

use ssh2::Session;
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::sync::atomic::AtomicBool;
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use std::thread::JoinHandle;

pub struct SshSession {
    pub session: Session,
    pub _transport: SshTransport,
}

pub enum SshTransport {
    Tcp {
        _stream: TcpStream,
    },
    #[cfg(unix)]
    Jump {
        _stream: UnixStream,
        _jump: Box<SshSession>,
        running: Arc<AtomicBool>,
        _threads: Vec<JoinHandle<()>>,
    },
}

#[cfg(unix)]
impl Drop for SshTransport {
    fn drop(&mut self) {
        if let SshTransport::Jump {
            _stream, running, ..
        } = self
        {
            running.store(false, std::sync::atomic::Ordering::SeqCst);
            let _ = _stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

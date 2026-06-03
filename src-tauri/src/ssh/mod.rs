pub mod connection;
pub mod sftp;

use ssh2::Session;
use std::net::TcpStream;

pub struct SshSession {
    pub session: Session,
    pub _stream: TcpStream,
}

use ssh2::Session;
use std::net::TcpStream;
use std::time::Duration;
use std::io::Read;
use super::SshSession;

pub struct SshConnectParams {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub timeout_ms: Option<u32>,
    pub proxy_jump_id: Option<String>,
    pub init_command: Option<String>,
    pub init_path: Option<String>,
    pub heartbeat_ms: Option<i32>,
}

fn connect_direct(host: &str, port: u16, timeout_ms: u32) -> Result<TcpStream, String> {
    let addr = format!("{}:{}", host, port);
    let timeout = Duration::from_millis(timeout_ms as u64);

    let tcp = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("Invalid address: {}", e))?,
        timeout,
    )
    .map_err(|e| format!("TCP connect failed: {}", e))?;

    tcp.set_read_timeout(Some(timeout)).ok();
    tcp.set_write_timeout(Some(timeout)).ok();

    Ok(tcp)
}

fn auth_session(session: &Session, params: &SshConnectParams) -> Result<(), String> {
    match params.auth_type.as_str() {
        "password" => {
            let password = params.password.as_deref().unwrap_or("");
            session
                .userauth_password(&params.username, password)
                .map_err(|e| format!("Password auth failed: {}", e))?;
        }
        "key" => {
            let key_path = params.key_path.as_deref().ok_or("Key path required")?;
            session
                .userauth_pubkey_file(&params.username, None, std::path::Path::new(key_path), None)
                .map_err(|e| format!("Key auth failed: {}", e))?;
        }
        "interactive" => {
            let password = params.password.as_deref().unwrap_or("");
            session
                .userauth_password(&params.username, password)
                .map_err(|e| format!("Interactive auth failed: {}", e))?;
        }
        _ => {
            if !session.authenticated() {
                return Err("Unsupported auth type and no authentication succeeded".to_string());
            }
        }
    }

    if !session.authenticated() {
        return Err("Authentication failed".to_string());
    }

    Ok(())
}

pub fn connect(params: &SshConnectParams) -> Result<SshSession, String> {
    let timeout_ms = params.timeout_ms.unwrap_or(10000);

    // If proxy_jump_id is set, we need to connect through a jump host
    // For now, connect directly (proxy_jump_id lookup requires DB access)
    // TODO: Implement ProxyJump by accepting a pre-connected session

    let tcp = connect_direct(&params.host, params.port, timeout_ms)?;

    let mut session = Session::new()
        .map_err(|e| format!("Session creation failed: {}", e))?;

    session.set_tcp_stream(tcp.try_clone().map_err(|e| format!("Clone failed: {}", e))?);
    session
        .handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;

    // Set keepalive to prevent idle disconnects
    let heartbeat_secs = params.heartbeat_ms.unwrap_or(5000) / 1000;
    session.set_keepalive(true, heartbeat_secs as u32);

    auth_session(&session, params)?;

    Ok(SshSession {
        session,
        _stream: tcp,
    })
}

pub fn connect_through_jump(
    jump_session: &Session,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, String> {
    // Open a direct-tcpip channel through the jump host to the target
    let channel = jump_session
        .channel_direct_tcpip(target_host, target_port, None)
        .map_err(|e| format!("Jump tunnel failed: {}", e))?;

    // Create a TcpStream-like wrapper from the channel
    // For simplicity, we'll use the jump session's TCP stream
    // In production, this should properly tunnel through the channel
    Err("ProxyJump tunneling requires custom stream wrapper - not yet implemented".to_string())
}

pub fn exec_command(session: &Session, cmd: &str) -> Result<String, String> {
    let mut channel = session.channel_session()
        .map_err(|e| format!("Channel open failed: {}", e))?;
    channel.exec(cmd)
        .map_err(|e| format!("Exec failed: {}", e))?;

    let mut output = String::new();
    channel.read_to_string(&mut output)
        .map_err(|e| format!("Read failed: {}", e))?;

    // Also read stderr
    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr).ok();

    channel.wait_close().ok();

    if !stderr.is_empty() && output.is_empty() {
        output = stderr;
    }

    Ok(output)
}

use ssh2::Session;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::io::Read;
use super::SshSession;

#[allow(dead_code)]
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
    use std::net::ToSocketAddrs;

    let timeout = Duration::from_millis(timeout_ms as u64);

    // Resolve hostname (supports both IP addresses and domain names)
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("Invalid address: {}", e))?
        .next()
        .ok_or_else(|| format!("No addresses found for {}:{}", host, port))?;

    let tcp = TcpStream::connect_timeout(&addr, timeout)
        .map_err(|e| format!("TCP connect failed: {}", e))?;

    tcp.set_read_timeout(Some(timeout)).ok();
    tcp.set_write_timeout(Some(timeout)).ok();

    Ok(tcp)
}

fn auth_session(session: &Session, params: &SshConnectParams) -> Result<(), String> {
    match params.auth_type.as_str() {
        "password" => {
            if let Some(password) = params.password.as_deref().filter(|pwd| !pwd.is_empty()) {
                session
                    .userauth_password(&params.username, password)
                    .map_err(|e| format!("Password auth failed: {}", e))?;
            } else {
                authenticate_without_password(session, &params.username)?;
            }
        }
        "key" => {
            if let Some(key_path) = params.key_path.as_deref().filter(|path| !path.is_empty()) {
                let key_path = expand_home(key_path);
                session
                    .userauth_pubkey_file(&params.username, None, key_path.as_path(), None)
                    .map_err(|e| format!("Key auth failed: {}", e))?;
            } else {
                authenticate_without_password(session, &params.username)?;
            }
        }
        "interactive" => {
            if let Some(password) = params.password.as_deref().filter(|pwd| !pwd.is_empty()) {
                session
                    .userauth_password(&params.username, password)
                    .map_err(|e| format!("Interactive auth failed: {}", e))?;
            } else {
                authenticate_without_password(session, &params.username)?;
            }
        }
        _ => {
            authenticate_without_password(session, &params.username)?;
        }
    }

    if !session.authenticated() {
        return Err("Authentication failed".to_string());
    }

    Ok(())
}

fn authenticate_without_password(session: &Session, username: &str) -> Result<(), String> {
    let mut errors = Vec::new();

    match authenticate_with_agent(session, username) {
        Ok(()) => return Ok(()),
        Err(err) => errors.push(err),
    }

    match authenticate_with_default_keys(session, username) {
        Ok(()) => return Ok(()),
        Err(err) => errors.push(err),
    }

    Err(format!("Key/agent auth failed: {}", errors.join("; ")))
}

fn authenticate_with_agent(session: &Session, username: &str) -> Result<(), String> {
    let mut agent = session
        .agent()
        .map_err(|e| format!("Agent unavailable: {}", e))?;
    agent
        .connect()
        .map_err(|e| format!("Agent connect failed: {}", e))?;
    agent
        .list_identities()
        .map_err(|e| format!("Agent list identities failed: {}", e))?;

    let identities = agent
        .identities()
        .map_err(|e| format!("Agent identities failed: {}", e))?;
    if identities.is_empty() {
        return Err("Agent has no identities".to_string());
    }

    let mut last_error = None;
    for identity in identities {
        match agent.userauth(username, &identity) {
            Ok(()) if session.authenticated() => {
                log::info!(
                    target: "myterm::ssh",
                    "ssh agent auth success username={}",
                    username
                );
                return Ok(());
            }
            Ok(()) => {
                last_error = Some("agent did not authenticate session".to_string());
            }
            Err(e) => {
                last_error = Some(e.to_string());
            }
        }
    }

    Err(format!(
        "Agent auth failed{}",
        last_error.map(|e| format!(": {}", e)).unwrap_or_default()
    ))
}

fn authenticate_with_default_keys(session: &Session, username: &str) -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let candidates = [
        ".ssh/id_ed25519",
        ".ssh/id_rsa",
        ".ssh/id_ecdsa",
        ".ssh/id_dsa",
    ];

    let mut tried = 0;
    let mut last_error = None;
    for candidate in candidates {
        let key_path = Path::new(&home).join(candidate);
        if !key_path.exists() {
            continue;
        }

        tried += 1;
        match session.userauth_pubkey_file(username, None, key_path.as_path(), None) {
            Ok(()) if session.authenticated() => {
                log::info!(
                    target: "myterm::ssh",
                    "default key auth success username={} key_path={}",
                    username,
                    key_path.display()
                );
                return Ok(());
            }
            Ok(()) => {
                last_error = Some(format!("{} did not authenticate session", key_path.display()));
            }
            Err(e) => {
                last_error = Some(format!("{}: {}", key_path.display(), e));
            }
        }
    }

    if tried == 0 {
        Err("No default private keys found".to_string())
    } else {
        Err(format!(
            "Default key auth failed{}",
            last_error.map(|e| format!(": {}", e)).unwrap_or_default()
        ))
    }
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~" {
        return std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(path));
    }

    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(rest);
        }
    }

    PathBuf::from(path)
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

#[allow(dead_code)]
pub fn connect_through_jump(
    jump_session: &Session,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, String> {
    // Open a direct-tcpip channel through the jump host to the target
    let _channel = jump_session
        .channel_direct_tcpip(target_host, target_port, None)
        .map_err(|e| format!("Jump tunnel failed: {}", e))?;

    // Create a TcpStream-like wrapper from the channel
    // For simplicity, we'll use the jump session's TCP stream
    // In production, this should properly tunnel through the channel
    Err("ProxyJump tunneling requires custom stream wrapper - not yet implemented".to_string())
}

#[allow(dead_code)]
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

use super::{SshSession, SshTransport};
#[cfg(unix)]
use ssh2::Channel;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SshConnectParams {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub timeout_ms: Option<u32>,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
    pub proxy_jump_id: Option<String>,
    pub proxy_jump: Option<Box<SshConnectParams>>,
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

fn connect_transport(params: &SshConnectParams, timeout_ms: u32) -> Result<TcpStream, String> {
    match proxy_kind(params) {
        None => connect_direct(&params.host, params.port, timeout_ms),
        Some(kind) if kind == "http" || kind == "https" => connect_http_proxy(params, timeout_ms),
        Some(kind) if kind == "socks5" || kind == "socks" => {
            connect_socks5_proxy(params, timeout_ms)
        }
        Some(kind) => Err(format!("Unsupported SSH proxy type: {}", kind)),
    }
}

fn proxy_kind(params: &SshConnectParams) -> Option<String> {
    params
        .proxy_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("none"))
        .map(|value| value.to_ascii_lowercase())
}

fn proxy_endpoint(params: &SshConnectParams) -> Result<(&str, u16), String> {
    let host = params
        .proxy_host
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Proxy host is required".to_string())?;
    let port = params
        .proxy_port
        .filter(|port| *port > 0)
        .ok_or_else(|| "Proxy port is required".to_string())?;
    Ok((host, port))
}

fn connect_http_proxy(params: &SshConnectParams, timeout_ms: u32) -> Result<TcpStream, String> {
    let (proxy_host, proxy_port) = proxy_endpoint(params)?;
    log::info!(
        target: "myterm::ssh",
        "ssh transport http proxy start proxy_host={} proxy_port={} target={}:{}",
        proxy_host,
        proxy_port,
        params.host,
        params.port
    );

    let mut tcp = connect_direct(proxy_host, proxy_port, timeout_ms)?;
    let request = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\nProxy-Connection: Keep-Alive\r\n\r\n",
        params.host, params.port, params.host, params.port
    );
    tcp.write_all(request.as_bytes())
        .map_err(|e| format!("HTTP proxy CONNECT write failed: {}", e))?;
    tcp.flush()
        .map_err(|e| format!("HTTP proxy CONNECT flush failed: {}", e))?;

    let response = read_http_connect_response(&mut tcp)?;
    validate_http_connect_response(&response)?;

    log::info!(
        target: "myterm::ssh",
        "ssh transport http proxy connected proxy_host={} proxy_port={} target={}:{}",
        proxy_host,
        proxy_port,
        params.host,
        params.port
    );
    Ok(tcp)
}

fn read_http_connect_response(tcp: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut response = Vec::new();
    let mut byte = [0u8; 1];

    while response.len() < 8192 {
        let n = tcp
            .read(&mut byte)
            .map_err(|e| format!("HTTP proxy CONNECT read failed: {}", e))?;
        if n == 0 {
            break;
        }
        response.push(byte[0]);
        if response.windows(4).any(|window| window == b"\r\n\r\n") {
            return Ok(response);
        }
    }

    Err("HTTP proxy CONNECT response did not include a complete header".to_string())
}

fn parse_http_connect_status(response: &[u8]) -> Result<u16, String> {
    let header_end = response
        .windows(2)
        .position(|window| window == b"\r\n")
        .unwrap_or(response.len());
    let status_line = std::str::from_utf8(&response[..header_end])
        .map_err(|e| format!("HTTP proxy CONNECT response is not UTF-8: {}", e))?;
    let mut parts = status_line.split_whitespace();
    let version = parts
        .next()
        .ok_or_else(|| "HTTP proxy CONNECT response is empty".to_string())?;
    if !version.starts_with("HTTP/") {
        return Err(format!(
            "HTTP proxy CONNECT response has invalid status line: {}",
            status_line
        ));
    }
    let status = parts
        .next()
        .ok_or_else(|| "HTTP proxy CONNECT response missing status code".to_string())?
        .parse::<u16>()
        .map_err(|e| format!("HTTP proxy CONNECT status parse failed: {}", e))?;
    Ok(status)
}

fn validate_http_connect_response(response: &[u8]) -> Result<(), String> {
    let status = parse_http_connect_status(response)?;
    if (200..300).contains(&status) {
        Ok(())
    } else {
        Err(format!("HTTP proxy CONNECT failed with status {}", status))
    }
}

fn connect_socks5_proxy(params: &SshConnectParams, timeout_ms: u32) -> Result<TcpStream, String> {
    let (proxy_host, proxy_port) = proxy_endpoint(params)?;
    log::info!(
        target: "myterm::ssh",
        "ssh transport socks5 proxy start proxy_host={} proxy_port={} target={}:{}",
        proxy_host,
        proxy_port,
        params.host,
        params.port
    );

    let mut tcp = connect_direct(proxy_host, proxy_port, timeout_ms)?;
    tcp.write_all(&[0x05, 0x01, 0x00])
        .map_err(|e| format!("SOCKS5 greeting write failed: {}", e))?;
    tcp.flush()
        .map_err(|e| format!("SOCKS5 greeting flush failed: {}", e))?;

    let mut method = [0u8; 2];
    tcp.read_exact(&mut method)
        .map_err(|e| format!("SOCKS5 greeting read failed: {}", e))?;
    if method != [0x05, 0x00] {
        return Err(format!(
            "SOCKS5 proxy rejected no-auth method: version={} method={}",
            method[0], method[1]
        ));
    }

    let request = build_socks5_connect_request(&params.host, params.port)?;
    tcp.write_all(&request)
        .map_err(|e| format!("SOCKS5 connect write failed: {}", e))?;
    tcp.flush()
        .map_err(|e| format!("SOCKS5 connect flush failed: {}", e))?;

    read_socks5_connect_response(&mut tcp)?;
    log::info!(
        target: "myterm::ssh",
        "ssh transport socks5 proxy connected proxy_host={} proxy_port={} target={}:{}",
        proxy_host,
        proxy_port,
        params.host,
        params.port
    );
    Ok(tcp)
}

fn build_socks5_connect_request(host: &str, port: u16) -> Result<Vec<u8>, String> {
    let mut request = vec![0x05, 0x01, 0x00];
    if let Ok(ip) = host.parse::<IpAddr>() {
        request.extend(encode_socks5_address(ip));
    } else {
        let host_bytes = host.as_bytes();
        if host_bytes.is_empty() || host_bytes.len() > u8::MAX as usize {
            return Err("SOCKS5 target host must be 1-255 bytes".to_string());
        }
        request.push(0x03);
        request.push(host_bytes.len() as u8);
        request.extend_from_slice(host_bytes);
    }
    request.extend_from_slice(&port.to_be_bytes());
    Ok(request)
}

fn encode_socks5_address(ip: IpAddr) -> Vec<u8> {
    match ip {
        IpAddr::V4(addr) => {
            let mut encoded = vec![0x01];
            encoded.extend_from_slice(&addr.octets());
            encoded
        }
        IpAddr::V6(addr) => {
            let mut encoded = vec![0x04];
            encoded.extend_from_slice(&addr.octets());
            encoded
        }
    }
}

fn read_socks5_connect_response(tcp: &mut TcpStream) -> Result<(), String> {
    let mut header = [0u8; 4];
    tcp.read_exact(&mut header)
        .map_err(|e| format!("SOCKS5 connect response read failed: {}", e))?;
    if header[0] != 0x05 {
        return Err(format!("SOCKS5 response version mismatch: {}", header[0]));
    }
    if header[1] != 0x00 {
        return Err(format!(
            "SOCKS5 connect failed: {}",
            socks5_reply_message(header[1])
        ));
    }

    let address_len = match header[3] {
        0x01 => 4,
        0x03 => {
            let mut len = [0u8; 1];
            tcp.read_exact(&mut len)
                .map_err(|e| format!("SOCKS5 domain length read failed: {}", e))?;
            len[0] as usize
        }
        0x04 => 16,
        other => {
            return Err(format!(
                "SOCKS5 response address type unsupported: {}",
                other
            ))
        }
    };

    let mut discard = vec![0u8; address_len + 2];
    tcp.read_exact(&mut discard)
        .map_err(|e| format!("SOCKS5 bind address read failed: {}", e))?;
    Ok(())
}

fn socks5_reply_message(code: u8) -> &'static str {
    match code {
        0x01 => "general SOCKS server failure",
        0x02 => "connection not allowed by ruleset",
        0x03 => "network unreachable",
        0x04 => "host unreachable",
        0x05 => "connection refused",
        0x06 => "TTL expired",
        0x07 => "command not supported",
        0x08 => "address type not supported",
        _ => "unknown SOCKS5 error",
    }
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
                last_error = Some(format!(
                    "{} did not authenticate session",
                    key_path.display()
                ));
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

    if proxy_kind(params).is_none() && params.proxy_jump.is_some() {
        return connect_via_jump(params, timeout_ms);
    }

    let tcp = connect_transport(params, timeout_ms)?;

    let mut session = Session::new().map_err(|e| format!("Session creation failed: {}", e))?;

    session.set_tcp_stream(
        tcp.try_clone()
            .map_err(|e| format!("Clone failed: {}", e))?,
    );
    session
        .handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;

    configure_authenticated_session(&session, params)?;

    Ok(SshSession {
        session,
        _transport: SshTransport::Tcp { _stream: tcp },
    })
}

fn configure_authenticated_session(
    session: &Session,
    params: &SshConnectParams,
) -> Result<(), String> {
    let heartbeat_secs = params.heartbeat_ms.unwrap_or(5000) / 1000;
    session.set_keepalive(true, heartbeat_secs as u32);
    auth_session(session, params)
}

#[cfg(not(unix))]
fn connect_via_jump(_params: &SshConnectParams, _timeout_ms: u32) -> Result<SshSession, String> {
    Err("ProxyJump is only implemented on Unix-like platforms".to_string())
}

#[cfg(unix)]
fn connect_via_jump(params: &SshConnectParams, timeout_ms: u32) -> Result<SshSession, String> {
    let jump_params = params
        .proxy_jump
        .as_ref()
        .ok_or_else(|| "ProxyJump params are missing".to_string())?;
    log::info!(
        target: "myterm::ssh",
        "ssh proxyjump start jump_host={} jump_port={} target={}:{}",
        jump_params.host,
        jump_params.port,
        params.host,
        params.port
    );

    let jump = connect(jump_params)?;
    let channel = jump
        .session
        .channel_direct_tcpip(&params.host, params.port, None)
        .map_err(|e| format!("ProxyJump direct-tcpip failed: {}", e))?;

    let timeout = Duration::from_millis(timeout_ms as u64);
    let (session_stream, bridge_stream) =
        UnixStream::pair().map_err(|e| format!("ProxyJump local bridge failed: {}", e))?;
    session_stream.set_read_timeout(Some(timeout)).ok();
    session_stream.set_write_timeout(Some(timeout)).ok();

    let running = Arc::new(AtomicBool::new(true));
    let threads = start_jump_bridge(channel, bridge_stream, running.clone())?;

    let mut session = Session::new().map_err(|e| format!("Session creation failed: {}", e))?;
    session.set_tcp_stream(
        session_stream
            .try_clone()
            .map_err(|e| format!("ProxyJump stream clone failed: {}", e))?,
    );
    session
        .handshake()
        .map_err(|e| format!("SSH handshake failed through ProxyJump: {}", e))?;
    configure_authenticated_session(&session, params)?;

    log::info!(
        target: "myterm::ssh",
        "ssh proxyjump connected jump_host={} jump_port={} target={}:{}",
        jump_params.host,
        jump_params.port,
        params.host,
        params.port
    );

    Ok(SshSession {
        session,
        _transport: SshTransport::Jump {
            _stream: session_stream,
            _jump: Box::new(jump),
            running,
            _threads: threads,
        },
    })
}

#[cfg(unix)]
fn start_jump_bridge(
    channel: Channel,
    stream: UnixStream,
    running: Arc<AtomicBool>,
) -> Result<Vec<thread::JoinHandle<()>>, String> {
    let mut local_reader = stream
        .try_clone()
        .map_err(|e| format!("ProxyJump local stream clone failed: {}", e))?;
    let mut local_writer = stream;
    let mut channel_writer = channel.clone();
    let mut channel_reader = channel;

    let running_to_remote = running.clone();
    let to_remote = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        while running_to_remote.load(Ordering::SeqCst) {
            match local_reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if channel_writer.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = channel_writer.flush();
                }
                Err(_) => break,
            }
        }
        running_to_remote.store(false, Ordering::SeqCst);
        let _ = channel_writer.close();
    });

    let running_to_local = running.clone();
    let to_local = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        while running_to_local.load(Ordering::SeqCst) {
            match channel_reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if local_writer.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = local_writer.flush();
                }
                Err(_) => break,
            }
        }
        running_to_local.store(false, Ordering::SeqCst);
        let _ = local_writer.shutdown(std::net::Shutdown::Both);
    });

    Ok(vec![to_remote, to_local])
}

#[allow(dead_code)]
pub fn exec_command(session: &Session, cmd: &str) -> Result<String, String> {
    let mut channel = session
        .channel_session()
        .map_err(|e| format!("Channel open failed: {}", e))?;
    channel
        .exec(cmd)
        .map_err(|e| format!("Exec failed: {}", e))?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn parses_successful_http_connect_response() {
        let response = b"HTTP/1.1 200 Connection established\r\nProxy-Agent: test\r\n\r\n";

        let status = parse_http_connect_status(response).expect("status should parse");

        assert_eq!(status, 200);
    }

    #[test]
    fn rejects_unsuccessful_http_connect_response() {
        let response = b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n";

        let err = validate_http_connect_response(response).expect_err("407 must fail");

        assert!(err.contains("407"));
    }

    #[test]
    fn builds_socks5_domain_connect_request() {
        let request = build_socks5_connect_request("example.com", 22).expect("request");

        assert_eq!(
            request,
            vec![
                0x05, 0x01, 0x00, 0x03, 11, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c',
                b'o', b'm', 0x00, 0x16
            ]
        );
    }

    #[test]
    fn builds_socks5_ipv4_connect_request() {
        let request = build_socks5_connect_request("127.0.0.1", 2222).expect("request");

        assert_eq!(
            request,
            vec![0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1, 0x08, 0xae]
        );
    }

    #[test]
    fn encodes_socks5_address_variants() {
        assert_eq!(
            encode_socks5_address(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            vec![0x01, 10, 0, 0, 1]
        );
        assert_eq!(
            encode_socks5_address(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            vec![0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
    }
}

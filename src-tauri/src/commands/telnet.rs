use parking_lot::Mutex;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Emitter, State};

pub struct TelnetSession {
    pub stream: TcpStream,
}

pub struct TelnetManager {
    sessions: Arc<Mutex<HashMap<String, TelnetSession>>>,
}

impl TelnetManager {
    pub fn new() -> Self {
        TelnetManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tauri::command]
pub fn connect_telnet(
    tm: State<'_, TelnetManager>,
    app_handle: AppHandle,
    host: String,
    port: Option<u16>,
) -> Result<String, String> {
    let port = port.unwrap_or(23);

    // Resolve hostname (supports both IP addresses and domain names)
    use std::net::ToSocketAddrs;
    let addr = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| format!("Invalid address: {}", e))?
        .next()
        .ok_or_else(|| format!("No addresses found for {}:{}", host, port))?;

    let stream = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(10))
        .map_err(|e| format!("Telnet connect failed: {}", e))?;

    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(100)))
        .ok();

    let session_id = uuid::Uuid::new_v4().to_string();

    // Clone stream for reading
    let read_stream = stream
        .try_clone()
        .map_err(|e| format!("Clone failed: {}", e))?;
    let write_stream = stream;

    // Start reader thread
    let sid = session_id.clone();
    thread::spawn(move || {
        let mut stream = read_stream;
        let mut buf = [0u8; 8192];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    let _ = app_handle.emit(&format!("telnet-exit-{}", sid), ());
                    break;
                }
                Ok(n) => {
                    // Handle telnet protocol bytes
                    let mut data = Vec::new();
                    let mut i = 0;
                    while i < n {
                        if buf[i] == 0xFF && i + 1 < n {
                            // IAC command
                            match buf[i + 1] {
                                0xFE => {
                                    // DONT
                                    if i + 2 < n {
                                        // Respond with WONT
                                        data.extend_from_slice(&[0xFF, 0xFC, buf[i + 2]]);
                                        i += 3;
                                        continue;
                                    }
                                }
                                0xFD => {
                                    // DO
                                    if i + 2 < n {
                                        // Respond with WONT
                                        data.extend_from_slice(&[0xFF, 0xFC, buf[i + 2]]);
                                        i += 3;
                                        continue;
                                    }
                                }
                                0xFB => {
                                    // WILL
                                    if i + 2 < n {
                                        // Respond with DONT
                                        data.extend_from_slice(&[0xFF, 0xFE, buf[i + 2]]);
                                        i += 3;
                                        continue;
                                    }
                                }
                                0xFC => {
                                    // WONT
                                    if i + 2 < n {
                                        // Respond with DONT
                                        data.extend_from_slice(&[0xFF, 0xFE, buf[i + 2]]);
                                        i += 3;
                                        continue;
                                    }
                                }
                                _ => {
                                    i += 2;
                                    continue;
                                }
                            }
                        }
                        data.push(buf[i]);
                        i += 1;
                    }

                    if !data.is_empty() {
                        let _ = app_handle.emit(&format!("telnet-output-{}", sid), data);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(_) => {
                    let _ = app_handle.emit(&format!("telnet-exit-{}", sid), ());
                    break;
                }
            }
        }
    });

    tm.sessions.lock().insert(
        session_id.clone(),
        TelnetSession {
            stream: write_stream,
        },
    );

    Ok(session_id)
}

#[tauri::command]
pub fn telnet_write(
    tm: State<'_, TelnetManager>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let mut sessions = tm.sessions.lock();
    if let Some(session) = sessions.get_mut(&session_id) {
        session
            .stream
            .write_all(data.as_bytes())
            .map_err(|e| format!("Write failed: {}", e))?;
        session
            .stream
            .flush()
            .map_err(|e| format!("Flush failed: {}", e))?;
        Ok(())
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
pub fn disconnect_telnet(tm: State<'_, TelnetManager>, session_id: String) -> Result<(), String> {
    tm.sessions.lock().remove(&session_id);
    Ok(())
}

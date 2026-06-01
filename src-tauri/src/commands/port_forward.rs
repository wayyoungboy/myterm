use crate::terminal::TerminalManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use parking_lot::Mutex;
use std::thread;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortForward {
    pub id: String,
    pub session_id: String,
    pub forward_type: String,
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub active: bool,
}

pub struct PortForwardManager {
    forwards: Arc<Mutex<HashMap<String, PortForward>>>,
}

impl PortForwardManager {
    pub fn new() -> Self {
        PortForwardManager {
            forwards: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tauri::command]
pub fn create_port_forward(
    tm: State<'_, TerminalManager>,
    pfm: State<'_, PortForwardManager>,
    session_id: String,
    forward_type: String,
    local_host: String,
    local_port: u16,
    remote_host: String,
    remote_port: u16,
) -> Result<String, String> {
    let session = tm.get_session(&session_id).ok_or("Session not found")?;
    let id = uuid::Uuid::new_v4().to_string();

    match forward_type.as_str() {
        "local" => {
            let listener = TcpListener::bind(format!("{}:{}", local_host, local_port))
                .map_err(|e| format!("Bind failed: {}", e))?;

            let rh = remote_host.clone();
            thread::spawn(move || {
                for stream in listener.incoming() {
                    if let Ok(client) = stream {
                        let session = session.clone();
                        let rh = rh.clone();
                        let rp = remote_port;
                        thread::spawn(move || {
                            if let Ok(mut channel) = session.channel_direct_tcpip(&rh, rp, None) {
                                let mut client = client;
                                let _ = std::io::copy(&mut client, &mut channel);
                            }
                        });
                    }
                }
            });
        }
        "remote" => {
            // Remote forwarding requires channel_forward_listen which may not be available
            return Err("Remote forwarding not yet supported".to_string());
        }
        "dynamic" => {
            let listener = TcpListener::bind(format!("{}:{}", local_host, local_port))
                .map_err(|e| format!("Bind failed: {}", e))?;

            thread::spawn(move || {
                for stream in listener.incoming() {
                    if let Ok(client) = stream {
                        let session = session.clone();
                        thread::spawn(move || {
                            handle_socks5(client, &session);
                        });
                    }
                }
            });
        }
        _ => return Err("Invalid forward type".to_string()),
    }

    let forward = PortForward {
        id: id.clone(),
        session_id,
        forward_type,
        local_host,
        local_port,
        remote_host,
        remote_port,
        active: true,
    };

    pfm.forwards.lock().insert(id.clone(), forward);

    Ok(id)
}

#[tauri::command]
pub fn get_port_forwards(
    pfm: State<'_, PortForwardManager>,
) -> Result<Vec<PortForward>, String> {
    let forwards = pfm.forwards.lock();
    Ok(forwards.values().cloned().collect())
}

#[tauri::command]
pub fn close_port_forward(
    pfm: State<'_, PortForwardManager>,
    id: String,
) -> Result<(), String> {
    let mut forwards = pfm.forwards.lock();
    if let Some(forward) = forwards.get_mut(&id) {
        forward.active = false;
    }
    Ok(())
}

fn handle_socks5(mut client: TcpStream, session: &ssh2::Session) {
    use std::io::{Read, Write};

    let mut buf = [0u8; 256];
    if client.read(&mut buf).is_err() { return; }

    let _ = client.write_all(&[0x05, 0x00]);

    if client.read(&mut buf).is_err() { return; }

    if buf[0] != 0x05 || buf[1] != 0x01 { return; }

    let (host, port) = match buf[3] {
        0x01 => {
            let port = u16::from_be_bytes([buf[8], buf[9]]);
            (format!("{}.{}.{}.{}", buf[4], buf[5], buf[6], buf[7]), port)
        }
        0x03 => {
            let len = buf[4] as usize;
            let host = String::from_utf8_lossy(&buf[5..5 + len]).to_string();
            let port = u16::from_be_bytes([buf[5 + len], buf[6 + len]]);
            (host, port)
        }
        _ => return,
    };

    match session.channel_direct_tcpip(&host, port, None) {
        Ok(mut channel) => {
            let _ = client.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]);
            let _ = std::io::copy(&mut client, &mut channel);
        }
        Err(_) => {
            let _ = client.write_all(&[0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0]);
        }
    }
}

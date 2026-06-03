use serde::Serialize;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
pub struct PingResult {
    pub host: String,
    pub port: u16,
    pub latency_ms: f64,
    pub success: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub fn ping_host(host: String, port: Option<u16>) -> PingResult {
    let port = port.unwrap_or(22);

    // Resolve hostname (supports both IP addresses and domain names)
    let addr = match (host.as_str(), port).to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a,
            None => {
                let msg = format!("No addresses found for {}:{}", host, port);
                return PingResult {
                    host,
                    port,
                    latency_ms: 0.0,
                    success: false,
                    error: Some(msg),
                };
            }
        },
        Err(e) => {
            return PingResult {
                host,
                port,
                latency_ms: 0.0,
                success: false,
                error: Some(format!("Invalid address: {}", e)),
            };
        }
    };

    let start = Instant::now();
    match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
        Ok(_) => {
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            PingResult {
                host,
                port,
                latency_ms: (latency * 100.0).round() / 100.0,
                success: true,
                error: None,
            }
        }
        Err(e) => PingResult {
            host,
            port,
            latency_ms: 0.0,
            success: false,
            error: Some(e.to_string()),
        },
    }
}

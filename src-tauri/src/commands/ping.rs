use serde::Serialize;
use std::net::TcpStream;
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
    let addr = format!("{}:{}", host, port);

    let start = Instant::now();
    match TcpStream::connect_timeout(
        &addr.parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()),
        Duration::from_secs(5),
    ) {
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

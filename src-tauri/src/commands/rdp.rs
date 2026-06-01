use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdpConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub domain: Option<String>,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub fn connect_rdp(
    host: String,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    domain: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<String, String> {
    let port = port.unwrap_or(3389);
    let width = width.unwrap_or(1280);
    let height = height.unwrap_or(800);

    // Try to launch external RDP client
    #[cfg(target_os = "linux")]
    {
        let mut args = vec![
            format!("/v:{}", host),
            format!("/w:{}", width),
            format!("/h:{}", height),
        ];
        if let Some(ref u) = username {
            args.push(format!("/u:{}", u));
        }
        if let Some(ref p) = password {
            args.push(format!("/p:{}", p));
        }
        if let Some(ref d) = domain {
            args.push(format!("/d:{}", d));
        }

        // Try xfreerdp first, then rdesktop
        let result = std::process::Command::new("xfreerdp")
            .args(&args)
            .spawn();

        match result {
            Ok(_) => return Ok("RDP session launched with xfreerdp".to_string()),
            Err(_) => {
                let result = std::process::Command::new("rdesktop")
                    .arg(format!("{}:{}", host, port))
                    .spawn();
                match result {
                    Ok(_) => return Ok("RDP session launched with rdesktop".to_string()),
                    Err(e) => return Err(format!("No RDP client found. Install xfreerdp or rdesktop: {}", e)),
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("open");
        cmd.arg(format!("rdp://{}", host));
        match cmd.spawn() {
            Ok(_) => Ok("RDP session launched".to_string()),
            Err(e) => Err(format!("Failed to launch RDP: {}", e)),
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("mstsc");
        cmd.arg(format!("/v:{}", host));
        cmd.arg(format!("/w:{}", width));
        cmd.arg(format!("/h:{}", height));
        match cmd.spawn() {
            Ok(_) => Ok("RDP session launched with mstsc".to_string()),
            Err(e) => Err(format!("Failed to launch RDP: {}", e)),
        }
    }
}

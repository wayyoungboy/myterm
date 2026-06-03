use std::process::Command;
use tauri::{AppHandle, Manager};

/// Take a screenshot of the app window and save to a fixed path.
/// Returns the file path of the saved screenshot.
#[tauri::command]
pub fn take_screenshot(app_handle: AppHandle) -> Result<String, String> {
    let screenshot_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("screenshots");

    std::fs::create_dir_all(&screenshot_dir).map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("screenshot_{}.png", timestamp);
    let filepath = screenshot_dir.join(&filename);

    // Get the window
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // Get window bounds
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        // Use screencapture to capture the window region
        // -l captures a specific window by PID, but we'll use region capture for reliability
        let output = Command::new("screencapture")
            .args([
                "-R",
                &format!("{},{},{},{}", pos.x, pos.y, size.width, size.height),
                "-x", // no sound
                filepath.to_str().ok_or("Invalid path")?,
            ])
            .output()
            .map_err(|e| format!("screencapture failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("screencapture error: {}", stderr));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use PowerShell to capture the window
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             $screen = [System.Windows.Forms.Screen]::FromPoint([System.Drawing.Point]::new({}, {})); \
             $bitmap = New-Object System.Drawing.Bitmap($screen.Bounds.Width, $screen.Bounds.Height); \
             $graphics = [System.Drawing.Graphics]::FromImage($bitmap); \
             $graphics.CopyFromScreen($screen.Bounds.Location, [System.Drawing.Point]::Empty, $screen.Bounds.Size); \
             $bitmap.Save('{}'); \
             $graphics.Dispose(); $bitmap.Dispose()",
            pos.x, pos.y,
            filepath.to_str().ok_or("Invalid path")?.replace('\\', "\\\\")
        );
        Command::new("powershell")
            .args(["-Command", &script])
            .output()
            .map_err(|e| format!("PowerShell screenshot failed: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Use gnome-screenshot or scrot on Linux
        let result = Command::new("gnome-screenshot")
            .args(["-w", "-f", filepath.to_str().ok_or("Invalid path")?])
            .output()
            .or_else(|_| {
                Command::new("scrot")
                    .args(["-u", filepath.to_str().ok_or("Invalid path")?])
                    .output()
            })
            .map_err(|e| {
                format!(
                    "Screenshot tool not found: {}. Install gnome-screenshot or scrot.",
                    e
                )
            })?;

        if !result.status.success() {
            return Err("Screenshot capture failed".to_string());
        }
    }

    Ok(filepath.to_str().unwrap_or("").to_string())
}

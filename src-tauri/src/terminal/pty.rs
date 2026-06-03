use ssh2::Channel;
use ssh2::Session;

pub fn open_shell(session: &Session) -> Result<Channel, String> {
    // Blocking mode needed for synchronous setup operations
    session.set_blocking(true);

    let mut channel = session
        .channel_session()
        .map_err(|e| format!("Channel open failed: {}", e))?;

    channel
        .request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| format!("PTY request failed: {}", e))?;

    channel
        .shell()
        .map_err(|e| format!("Shell request failed: {}", e))?;

    // Switch to non-blocking for the reader thread
    session.set_blocking(false);

    Ok(channel)
}

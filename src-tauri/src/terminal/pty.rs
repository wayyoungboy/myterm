use ssh2::Session;
use ssh2::Channel;

pub fn open_shell(session: &Session) -> Result<Channel, String> {
    // Keep session in blocking mode for compatibility with SFTP operations
    // The reader thread will use blocking reads which is fine for a dedicated thread
    session.set_blocking(true);

    let mut channel = session.channel_session()
        .map_err(|e| format!("Channel open failed: {}", e))?;

    channel.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| format!("PTY request failed: {}", e))?;

    channel.shell()
        .map_err(|e| format!("Shell request failed: {}", e))?;

    Ok(channel)
}

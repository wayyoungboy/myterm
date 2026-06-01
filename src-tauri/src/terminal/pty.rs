use ssh2::Session;
use ssh2::Channel;

pub fn open_shell(session: &Session) -> Result<Channel, String> {
    // Set session to non-blocking mode for the terminal reader thread
    // This is necessary for async I/O but affects all channels on this session
    // The SFTP and monitor operations use separate sessions via get_session()
    session.set_blocking(false);

    let mut channel = session.channel_session()
        .map_err(|e| format!("Channel open failed: {}", e))?;

    channel.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| format!("PTY request failed: {}", e))?;

    channel.shell()
        .map_err(|e| format!("Shell request failed: {}", e))?;

    Ok(channel)
}

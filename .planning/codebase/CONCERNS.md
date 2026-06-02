# Codebase Concerns

**Analysis Date:** 2026-06-02

## Tech Debt

**Weak Encryption Implementation:**
- Issue: `src-tauri/src/crypto.rs` uses a hardcoded static salt (`b"myterm-app-v1-salt"`), derives IV deterministically from the key, and uses the machine hostname as the master password. This is cryptographically weak.
- Files: `src-tauri/src/crypto.rs`
- Impact: If an attacker gains access to the encrypted database and knows the hostname (trivial on shared systems), all stored passwords can be decrypted. The static salt and deterministic IV defeat the purpose of CBC mode.
- Fix approach: Use PBKDF2 or Argon2 for key derivation with a random salt per encryption. Generate a random IV for each encryption operation. Use a user-provided master password or OS keychain (macOS Keychain, Windows DPAPI, Linux Secret Service) instead of hostname-derived key.

**connect_terminal_for_sftp Is Dead Code:**
- Issue: `connect_terminal_for_sftp` in `src-tauri/src/commands/terminal.rs` (lines 107-163) creates an SSH session but never stores it anywhere. The function returns a session_id that cannot be used.
- Files: `src-tauri/src/commands/terminal.rs`
- Impact: SFTP operations must share the terminal session's SSH connection, which forces the session to stay in blocking mode (see SFTP blocking mode regression below). This is the root cause of the blocking mode conflict.
- Fix approach: Create a separate `SftpSessionManager` (similar to `TerminalManager`) that stores dedicated SFTP sessions. Register it in `src-tauri/src/lib.rs` and use it in `src-tauri/src/commands/sftp.rs`.

**ProxyJump Not Implemented:**
- Issue: `connect` in `src-tauri/src/ssh/connection.rs` (line 81) has a TODO comment acknowledging that ProxyJump is not implemented. The `connect_through_jump` function (lines 105-119) returns an error.
- Files: `src-tauri/src/ssh/connection.rs`
- Impact: Users who configure a proxy jump host will get an error. The UI allows setting `proxy_jump_id` but it is silently ignored.
- Fix approach: Implement a custom `Read+Write` wrapper over `ssh2::Channel` from `channel_direct_tcpip`, then pass it to `Session::set_tcp_stream`.

**Unused tokio Runtime:**
- Issue: `Cargo.toml` includes `tokio = { version = "1", features = ["full"] }` but all async work is done via `std::thread::spawn`. No async runtime is actually used.
- Files: `src-tauri/Cargo.toml`, all `src-tauri/src/commands/*.rs`
- Impact: Unnecessary binary size bloat and dependency surface.
- Fix approach: Either migrate to async/await with tokio, or remove the tokio dependency entirely.

**Hardcoded Chinese UI Strings:**
- Issue: Monitor sidebar and other components use hardcoded Chinese strings (e.g., "系统", "CPU", "内存", "网络", "硬盘", "显卡", "天", "小时", "分钟").
- Files: `src/components/monitor/MonitorSidebar.tsx` (lines 39, 42, 134, 166, 180, 207, etc.)
- Impact: Cannot support English or other languages despite having a language setting in SettingsView.
- Fix approach: Use i18n library (e.g., react-i18next) with translation files.

## Known Bugs

**SFTP Blocking Mode Regression:**
- Symptoms: SFTP operations (file listing, upload, download) cause terminal I/O to freeze or behave erratically.
- Files: `src-tauri/src/ssh/sftp.rs`, `src-tauri/src/terminal/mod.rs`
- Trigger: Open a terminal session, then perform any SFTP operation. The SFTP operation calls `session.sftp()` which needs blocking mode, but the terminal reader thread expects non-blocking mode.
- Workaround: None. The recent commit "fix: Keep SSH session in blocking mode for SFTP compatibility" is a band-aid that may cause terminal read issues.

**Monitor Data Fetch Race Condition:**
- Symptoms: Monitor data fetch may fail intermittently or cause terminal I/O stalls.
- Files: `src-tauri/src/monitor/mod.rs` (lines 46-62), `src-tauri/src/terminal/mod.rs`
- Trigger: When the monitor sidebar is open, `fetch_monitor_data` temporarily switches the SSH session to blocking mode (line 46) while the terminal reader thread is concurrently reading in non-blocking mode (line 90-101 of `terminal/mod.rs`).
- Workaround: None. This is a fundamental concurrency issue with shared session state.

**Telnet Reader Thread Never Stops:**
- Symptoms: After disconnecting a telnet session, the reader thread continues running until the next read error or connection close.
- Files: `src-tauri/src/commands/telnet.rs` (lines 56-129)
- Trigger: Call `disconnect_telnet` which removes the session from the HashMap, but the reader thread has no shutdown signal.
- Workaround: The thread will eventually exit when the TCP connection times out or the remote end closes.

**TerminalView Event Listener Leak on Reconnect:**
- Symptoms: Multiple event listeners accumulate if `setupTerminalIO` is called multiple times for different sessions.
- Files: `src/components/terminal/TerminalView.tsx` (lines 147-185)
- Trigger: If a component unmounts and remounts, or if `setupTerminalIO` is called before previous `listen()` promises resolve.
- Workaround: The `onDataDisposable` ref tracks the onData handler, but the `listen()` calls (lines 155-170) are fire-and-forget promises that may not be cleaned up on rapid unmount.

**TelnetView onData Handler Leak:**
- Symptoms: Each call to `handleConnect` adds a new `onData` handler without disposing the previous one.
- Files: `src/components/terminal/TelnetView.tsx` (line 76)
- Trigger: If `handleConnect` is called multiple times (e.g., due to React strict mode or rapid reconnects).
- Workaround: None. Unlike TerminalView which tracks `onDataDisposable`, TelnetView does not.

**Settings Debounce Race:**
- Symptoms: If the user changes multiple settings rapidly, only the last change is saved. Previous changes may be lost.
- Files: `src/components/settings/SettingsView.tsx` (lines 100-116)
- Trigger: Change two settings within 500ms of each other.
- Workaround: None. The debounce timer only saves the last key-value pair.

## Security Considerations

**Password Encryption Is Effectively Plaintext:**
- Risk: The encryption key is derived from the hostname using a single SHA-256 hash with a static salt. Any user on the same machine with the same hostname can decrypt all stored passwords.
- Files: `src-tauri/src/crypto.rs`
- Current mitigation: AES-256-CBC encryption is used, but with a predictable key.
- Recommendations: Use OS keychain for key storage. Implement proper KDF (PBKDF2 with 100k+ iterations or Argon2). Generate random IV per encryption.

**No Input Validation on SSH Commands:**
- Risk: `init_command` and `init_path` from the database are written directly to the SSH channel without sanitization.
- Files: `src-tauri/src/commands/terminal.rs` (lines 73-89)
- Current mitigation: These values are user-configured, so the risk is self-inflicted.
- Recommendations: Add a warning in the UI that init commands run automatically on connect.

**SQL Injection Not Possible but No Parameterized Queries for LIKE:**
- Risk: The `search_connections` function uses `format!("%{}%", query)` which is safe because rusqlite parameterizes it, but the pattern is fragile.
- Files: `src-tauri/src/commands/connections.rs` (line 377)
- Current mitigation: rusqlite parameterizes the query.
- Recommendations: No immediate action needed, but consider escaping `%` and `_` in the search query for correct LIKE behavior.

**RDP Password Passed on Command Line:**
- Risk: On Linux, the RDP password is passed as a command-line argument (`/p:password`), which is visible in process listings.
- Files: `src-tauri/src/commands/rdp.rs` (line 41)
- Current mitigation: None.
- Recommendations: Use xfreerdp's `/p` from stdin or environment variable, or use `.rdp` file with credentials.

## Performance Bottlenecks

**SFTP Per-Operation Session Creation:**
- Problem: Every SFTP operation (list dir, read file, write file) creates a new SFTP sub-session via `session.sftp()`.
- Files: `src-tauri/src/ssh/sftp.rs` (lines 5, 52, 64, 75, 82, 89, 96)
- Cause: No SFTP session pooling. Each call to `list_dir`, `read_file`, etc. creates and drops an SFTP session.
- Improvement path: Create an SFTP session once per connection and reuse it. Store it in a manager struct.

**Terminal Reader Thread Busy-Wait:**
- Problem: The terminal reader thread sleeps 10ms on WouldBlock, causing up to 10ms latency and unnecessary CPU usage.
- Files: `src-tauri/src/terminal/mod.rs` (lines 100-101, 113-115)
- Cause: `ssh2` in non-blocking mode returns WouldBlock when no data is available, requiring polling.
- Improvement path: Use `ssh2::Session::set_blocking(true)` with a timeout, or use a separate blocking thread per session with proper timeout handling.

**Database Lock Contention:**
- Problem: All database operations share a single `Mutex<rusqlite::Connection>`. Long-running operations (export, import) block all other DB access.
- Files: `src-tauri/src/db/mod.rs`, all `src-tauri/src/commands/*.rs`
- Cause: Single mutex-protected connection.
- Improvement path: Use a connection pool (r2d2) or WAL mode with read-write separation.

## Fragile Areas

**Blocking/Non-Blocking Mode Toggling:**
- Files: `src-tauri/src/terminal/pty.rs`, `src-tauri/src/monitor/mod.rs`, `src-tauri/src/ssh/sftp.rs`
- Why fragile: Multiple components (terminal, monitor, SFTP) share the same `ssh2::Session` and toggle blocking mode independently. There is no coordination mechanism, so concurrent access causes race conditions.
- Safe modification: Never share a session between terminal I/O and SFTP/monitor operations. Use separate SSH sessions for each purpose.
- Test coverage: None. No tests exist for concurrent session access.

**MainLayout Tab Rendering:**
- Files: `src/components/layout/MainLayout.tsx` (lines 18-25)
- Why fragile: All tab types (terminal, sftp, monitor) render `<TerminalView>`. There is no actual SFTP or monitor tab implementation. Adding new tab types requires modifying this switch.
- Safe modification: Add component routing per tab type. Use a registry pattern or lazy-loaded components.
- Test coverage: None.

**Port Forward Thread Lifecycle:**
- Files: `src-tauri/src/commands/port_forward.rs` (lines 54-88)
- Why fragile: Port forward listener threads run indefinitely with no shutdown mechanism. `close_port_forward` only sets `active = false` but does not close the `TcpListener` or join threads.
- Safe modification: Use a cancellation token (e.g., `AtomicBool`) shared with the listener thread. Store the `TcpListener` handle to close it on shutdown.
- Test coverage: None.

## Scaling Limits

**Single SQLite Database:**
- Current capacity: Handles hundreds of connections and notes comfortably.
- Limit: Concurrent writes from multiple Tauri commands may queue behind the single mutex.
- Scaling path: Enable WAL mode for concurrent reads, or migrate to a connection pool.

**In-Memory Session Storage:**
- Current capacity: Limited by available RAM for SSH sessions and terminal buffers.
- Limit: Each SSH session holds a TCP connection, channel, and 8KB read buffer. Hundreds of sessions will exhaust file descriptors.
- Scaling path: Implement session limits and idle session cleanup.

## Dependencies at Risk

**react-split-pane:**
- Risk: Last published 4 years ago (v3.2.0). No TypeScript types bundled. React 19 compatibility untested.
- Impact: May break with future React updates.
- Migration plan: Replace with CSS `resize` or `react-resizable-panels`.

**ssh2 crate:**
- Risk: Version 0.9 is outdated (current is 0.11+). The blocking/non-blocking API is complex and error-prone.
- Impact: Missing security fixes and performance improvements from newer versions.
- Migration plan: Update to latest ssh2 and audit blocking mode usage.

## Missing Critical Features

**No Tests:**
- Problem: Zero test files exist in the entire codebase (no `*.test.*`, `*.spec.*`, or `tests/` directory).
- Blocks: Confident refactoring, regression detection, CI/CD pipeline.

**No Error Boundary Recovery:**
- Problem: `ErrorBoundary` component exists but there is no recovery mechanism. A crash in one tab may leave the app in a broken state.
- Blocks: Production reliability.

## Test Coverage Gaps

**All Rust Backend Code:**
- What's not tested: Every command handler, SSH connection logic, encryption/decryption, SOCKS5 parser, monitor script parsing.
- Files: All `src-tauri/src/**/*.rs`
- Risk: Any change to backend logic can silently break functionality.
- Priority: High

**All Frontend Components:**
- What's not tested: Every React component, store logic, event handling.
- Files: All `src/**/*.tsx`
- Risk: UI regressions, event listener leaks, state management bugs.
- Priority: High

---

*Concerns audit: 2026-06-02*

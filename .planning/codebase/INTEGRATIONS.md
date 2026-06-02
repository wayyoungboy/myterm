# External Integrations

**Analysis Date:** 2026-06-02

## APIs & External Services

**SSH/SFTP:**
- Protocol: SSH2 via `ssh2` crate (0.9)
- Auth methods: Password, Public Key, Interactive
- Features: PTY allocation, command execution, SFTP file operations
- Connection pooling: `SessionManager` in `src-tauri/src/ssh/mod.rs`
- Heartbeat: Configurable keepalive (default 5000ms)

**Telnet:**
- Protocol: Raw TCP with Telnet protocol handling (IAC commands)
- Implementation: `src-tauri/src/commands/telnet.rs`
- Features: Basic DO/DONT/WILL/WONT negotiation

**RDP:**
- Integration: Delegates to external system clients
- macOS: `open rdp://{host}`
- Linux: `xfreerdp` or `rdesktop`
- Windows: `mstsc`
- Implementation: `src-tauri/src/commands/rdp.rs`

**Port Forwarding:**
- Local forwarding: TCP listener -> SSH channel_direct_tcpip
- Dynamic forwarding: SOCKS5 proxy implementation
- Remote forwarding: Not implemented (returns error)
- Implementation: `src-tauri/src/commands/port_forward.rs`

**Server Monitoring:**
- Method: SSH exec of POSIX shell script
- Data collected: CPU, Memory, Swap, Network, Disk, GPU (nvidia-smi), Load average
- Implementation: `src-tauri/src/monitor/mod.rs`
- Script reads from `/proc/stat`, `/proc/meminfo`, `/proc/net/dev`, `/proc/diskstats`

## Data Storage

**Databases:**
- SQLite via `rusqlite` 0.31 (bundled)
  - Location: `{app_data_dir}/myterm.db`
  - Connection: `DbConn` wrapper with `Mutex<Connection>`
  - Pragmas: WAL journal mode, foreign keys enabled
  - Schema: `src-tauri/src/db/schema.rs`

*Tables:*
| Table | Purpose |
|-------|---------|
| `groups` | Connection groups (hierarchical via parent_id) |
| `connections` | SSH/Telnet/RDP server configurations |
| `quick_commands` | Saved command snippets |
| `notes` | Connection-associated notes |
| `ai_conversations` | AI chat conversation metadata |
| `ai_messages` | Individual AI chat messages |
| `settings` | Key-value application settings |

**File Storage:**
- Local filesystem only
- SFTP: Remote file operations via SSH session
- Screenshots: Saved to `{app_data_dir}/screenshots/`

**Caching:**
- In-memory session managers (HashMap with Mutex):
  - `TerminalManager` - SSH terminal sessions
  - `TelnetManager` - Telnet sessions
  - `LocalTerminalManager` - Local shell sessions
  - `PortForwardManager` - Port forwarding tunnels

## Authentication & Identity

**Auth Provider:**
- Custom implementation per connection
- Auth types: `password`, `key`, `credential`, `interactive`, `ask`
- Password storage: AES-256-CBC encrypted in SQLite
- Master key: SHA-256 derived from hostname

**Key Management:**
- Static salt: `myterm-app-v1-salt` (in `src-tauri/src/crypto.rs`)
- Key derivation: SHA-256(SALT + password)
- IV derivation: SHA-256("iv" + key_hash)[:16]
- Storage: Base64-encoded ciphertext in `connections.password_enc`

## Monitoring & Observability

**Error Tracking:**
- None (errors returned as Rust `Result::Err` strings)

**Logs:**
- Rust: `log` crate + `env_logger`
- Frontend: `console.log` / `console.error`

**Metrics:**
- Server monitoring via SSH exec script
- Data: CPU, memory, network, disk, GPU, uptime, load average
- Visualization: Recharts library

## CI/CD & Deployment

**Hosting:**
- GitHub Actions (`.github/workflows/build.yml`)
- Release: Draft releases on tag push (`v*`)

**CI Pipeline:**
- Matrix: Windows + macOS (ARM64)
- Node.js 22.x, Rust stable
- Caching: npm cache + Rust cargo cache
- Artifacts: Windows (MSI/NSIS), macOS (DMG)

**Signing:**
- Tauri signing key via secrets: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

## Environment Configuration

**Required env vars:**
- `TAURI_DEV_HOST` - Optional: Dev server host for remote HMR
- `SHELL` - Optional: Default shell for local terminal (fallback: `/bin/sh`)

**Secrets location:**
- GitHub Actions secrets for CI signing
- Application data: `{app_data_dir}/myterm.db` (SQLite)

## Webhooks & Callbacks

**Incoming:**
- None (desktop application, no server endpoints)

**Outgoing:**
- None (all connections initiated by user action)

## Tauri IPC Commands

All backend functionality exposed via `invoke()` bridge:

**Connection Management:**
- `get_groups`, `create_group`, `update_group`, `delete_group`
- `get_connections`, `create_connection`, `update_connection`, `delete_connection`
- `test_connection`, `search_connections`, `collect_server_info`

**Terminal:**
- `connect_terminal`, `disconnect_terminal`, `terminal_write`, `terminal_resize`

**SFTP:**
- `sftp_list_dir`, `sftp_read_file`, `sftp_write_file`, `sftp_remove_file`, `sftp_rename`, `sftp_mkdir`

**Telnet:**
- `connect_telnet`, `telnet_write`, `disconnect_telnet`

**Local Terminal:**
- `open_local_terminal`, `local_terminal_write`, `close_local_terminal`

**Port Forwarding:**
- `create_port_forward`, `get_port_forwards`, `close_port_forward`

**Monitoring:**
- `get_monitor_data`

**Notes:**
- `get_notes`, `create_note`, `update_note`, `delete_note`

**AI Chat:**
- `get_ai_conversations`, `create_ai_conversation`, `delete_ai_conversation`
- `get_ai_messages`, `save_ai_message`

**Settings:**
- `get_settings`, `set_setting`, `get_setting`

**Import/Export:**
- `export_connections`, `import_connections`

**Utilities:**
- `ping_host`, `connect_rdp`, `take_screenshot`

---

*Integration audit: 2026-06-02*

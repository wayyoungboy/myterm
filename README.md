# MyTerm

MyTerm is a Tauri + React desktop SSH manager with XTerminal-inspired workflows. The current stable product path is SSH-first:

- SSH connection management.
- SSH terminal tabs.
- SFTP file management launched from an active terminal tab, using independent SSH sessions for blocking file operations.
- Server monitoring launched from an active terminal tab, using independent SSH sessions for monitor collection.
- SSH authentication with password, explicit private key path, ssh-agent, or default private keys such as `~/.ssh/id_ed25519` and `~/.ssh/id_rsa`.
- SSH outbound proxy support for HTTP CONNECT and SOCKS5.

Additional tool views are available in the app shell:

- Notes, settings, quick commands, import/export, and local terminal utilities.
- AI conversation storage with a placeholder response until a real model provider is wired.
- Telnet, RDP launcher, and port forwarding commands exist but still need deeper product validation before they should be treated as complete.

Cloud sync is not implemented.

## Stack

- Tauri 2 + Rust
- React 19 + TypeScript + Vite
- Zustand for client state
- xterm.js for terminal rendering
- ssh2 for SSH/SFTP
- SQLite via rusqlite

## Development

Install dependencies:

```bash
npm install
```

Run the frontend only:

```bash
npm run dev
```

Run the desktop app in development mode:

```bash
npm run tauri dev
```

Development mode starts a local Vite server, so it occupies a localhost port. Packaged desktop builds do not need that development port.

The app does not need to listen on a public port for normal SSH terminal, SFTP, monitoring, or outbound HTTP/SOCKS5 proxy workflows. Local listening is only needed for development server mode and features that intentionally expose a listener, such as local or dynamic port forwarding.

## Logs

The desktop app writes runtime logs to the Tauri app data directory as `myterm.log`. The log includes startup, connection CRUD, SSH connect/disconnect, SSH outbound proxy setup, SFTP operations, monitor fetches, and port-forward lifecycle events with operation IDs and elapsed times.

Default level is `info`. Use `MYTERM_LOG=debug` or `MYTERM_LOG=trace` before launching the app when deeper troubleshooting is needed.

Logs intentionally do not record passwords, private key contents, or terminal input bytes.

## Verification

Frontend build:

```bash
npm run build
```

Rust check:

```bash
cd src-tauri
cargo check
```

Rust tests:

```bash
cd src-tauri
cargo test
```

Real SSH connectivity smoke test:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

The test server is for runtime validation only. Do not commit passwords, private keys, or generated local connection databases.

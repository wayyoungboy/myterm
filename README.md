# MyTerm

MyTerm is a Tauri + React desktop SSH manager. The current development scope is intentionally focused on SSH workflows:

- SSH connection management.
- SSH terminal tabs.
- SFTP file management over an active SSH session.
- Server monitoring over an active SSH session.

The following features are currently out of scope and should not drive primary navigation or completion status: notes, AI assistant, port forwarding, Telnet, quick commands, cloud sync, and RDP.

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

Real SSH connectivity smoke test:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

The test server is for runtime validation only. Do not commit passwords, private keys, or generated local connection databases.


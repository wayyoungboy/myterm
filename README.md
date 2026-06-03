# MyTerm

MyTerm is a Tauri + React desktop SSH manager with XTerminal-inspired workflows. The current stable product path is SSH-first:

- SSH connection management.
- SSH terminal tabs.
- SFTP file management over an active SSH session.
- Server monitoring over an active SSH session.

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

The app does not need to listen on a public port for normal SSH terminal, SFTP, or monitoring workflows. Local listening is only needed for development server mode and features that intentionally expose a listener, such as local or dynamic port forwarding.

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

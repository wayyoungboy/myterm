<div align="center">

# MyTerm

**A Tauri desktop SSH manager for terminal tabs, SFTP, host monitoring, and proxy workflows.**

[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white)](#stack)
[![React](https://img.shields.io/badge/React-19-20232A?style=flat-square&logo=react&logoColor=61DAFB)](#stack)
[![Rust](https://img.shields.io/badge/Rust-desktop-000000?style=flat-square&logo=rust&logoColor=white)](#stack)
[![TypeScript](https://img.shields.io/badge/TypeScript-Vite-3178C6?style=flat-square&logo=typescript&logoColor=white)](#stack)

</div>

![MyTerm showcase](docs/assets/showcase.svg)

MyTerm is an SSH-first desktop operations workspace inspired by XTerminal. It focuses on the daily loop of connecting to servers, opening terminal tabs, moving files, checking host state, and routing through proxies without turning the app into a cloud sync product.

## Core Workflows

| Workflow | Status | Notes |
|---|---|---|
| SSH connection manager | Stable path | Password, explicit private key path, ssh-agent, and default private keys. |
| Terminal tabs | Stable path | xterm.js terminal sessions inside a Tauri desktop shell. |
| SFTP from terminal | Stable path | File operations launch independent SSH sessions for blocking work. |
| Host monitoring | Stable path | Monitoring launched from an active terminal tab with separate SSH collection. |
| Outbound proxy | Stable path | HTTP CONNECT, SOCKS5, and ProxyJump through SSH `direct-tcpip`. |
| Extra tool views | Experimental | Notes, quick commands, import/export, local terminal, Telnet, RDP launcher, port forwarding. |

Cloud sync is intentionally not implemented.

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

The desktop app writes runtime logs to the Tauri app data directory as `myterm.log`. The log includes startup, connection CRUD, SSH connect/disconnect, SSH outbound proxy and ProxyJump setup, SFTP operations, monitor fetches, and port-forward lifecycle events with operation IDs and elapsed times.

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

macOS package build:

```bash
npm run tauri -- build
```

In this automation environment the standard DMG bundler can fail when Finder AppleScript times out. The release binary and `.app` are still produced under `src-tauri/target/release`. A verified fallback is to clean stale `rw.*.dmg` files and run the generated `bundle_dmg.sh` with `--skip-jenkins`; the resulting DMG is written to `src-tauri/target/release/bundle/dmg/`.

Real SSH connectivity smoke test:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

The test server is for runtime validation only. Do not commit passwords, private keys, or generated local connection databases.

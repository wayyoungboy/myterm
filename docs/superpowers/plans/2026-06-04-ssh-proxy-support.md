# SSH Proxy Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make stored SSH outbound proxy fields work for connection tests, terminal sessions, SFTP, and monitor auxiliary SSH sessions.

**Architecture:** Keep proxy behavior in the shared Rust SSH connection layer so every SSH consumer gets the same transport path. Store UI choices in existing `connections.proxy_type`, `proxy_host`, and `proxy_port` fields; avoid adding schema churn.

**Tech Stack:** Tauri 2, Rust, ssh2, rusqlite, React 19, TypeScript, Vite.

---

## Scope

- Support direct SSH, HTTP CONNECT proxy, SOCKS5 proxy, and Unix-like ProxyJump.
- Load proxy fields through the shared `load_ssh_params` path.
- Allow connection tests and hardware info collection to use proxy fields from the connection form.
- Replace the disabled proxy UI with editable proxy type, host, and port controls.
- Add Rust unit tests for proxy protocol helpers.
- Update user docs and status docs.

## Out Of Scope

- Proxy authentication.
- Cloud sync, AI, RDP, Telnet, and non-SSH tool expansion.

## Tasks

- [x] Add failing Rust tests for HTTP CONNECT response parsing and SOCKS5 connect request encoding.
- [x] Implement proxy helper functions and route SSH TCP setup through direct/proxy transports.
- [x] Load and pass proxy fields from stored connections and connection form test paths.
- [x] Replace disabled proxy controls in `ConnectionForm` with real editable fields.
- [x] Add recursive ProxyJump parameter loading, cycle detection, and Unix-like direct-tcpip bridging.
- [x] Update README, DEV_PLAN, IMPLEMENTATION_STATUS, and PRD_COMPLIANCE.
- [x] Run `npm run build`, `cargo check`, `cargo test`, and the real SSH smoke test.
- [x] Commit and push the completed increment.

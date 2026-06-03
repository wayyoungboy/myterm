# SSH Core Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the SSH-centered product flow: connection management, terminal sessions, SFTP, and server monitoring.

**Architecture:** Keep the Tauri command boundary as the frontend/backend contract. Terminal tabs own long-lived SSH sessions; SFTP and monitor commands resolve the tab's connection and use independent SSH sessions for blocking subsystem work.

**Tech Stack:** Tauri 2, Rust, ssh2, SQLite/rusqlite, React 19, TypeScript, Vite, Zustand, xterm.js.

**Status update:** This plan captured the initial SSH-core completion pass. Later work restored the full app tool navigation and completed additional local-tool wiring for notes, quick commands, settings, and XTerminal-style monitor details. Treat the out-of-scope list below as historical scope for the first pass, not the current product boundary.

---

## Scope

In scope:
- SSH connection CRUD and connection center.
- SSH terminal tabs.
- SFTP remote file management launched from an active terminal tab.
- Local filesystem commands needed by the SFTP two-panel UI.
- Server monitor view launched from an active terminal tab.
- Documentation and staged git commits.

Out of scope:
- Notes.
- AI.
- Port forwarding.
- Telnet.
- Quick commands.
- Cloud sync.
- RDP.

## Verification Server

Use this server for real SSH validation:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

Password authentication can be tested manually with the password supplied by the project owner, but automated commits must not store passwords in source files or docs.

## Task 1: Baseline and Scope Documentation

**Files:**
- Create: `docs/superpowers/plans/2026-06-04-ssh-core-completion.md`
- Modify: `README.md`
- Modify: `DEV_PLAN.md`

- [x] Run baseline frontend build:

```bash
npm run build
```

Expected: TypeScript and Vite build exit 0.

- [x] Run baseline Rust check:

```bash
cargo check
```

Expected: Rust check exits 0. Existing warnings are allowed during baseline and should be reduced in later tasks when touching those modules.

- [x] Verify the real SSH server is reachable:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

Expected: output includes `MYTERM_SSH_OK`.

- [x] Commit the plan and baseline docs:

```bash
git add docs/superpowers/plans/2026-06-04-ssh-core-completion.md README.md DEV_PLAN.md
git commit -m "docs: define ssh core completion scope"
```

## Task 2: SSH-Only Navigation and Tab Routing

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/stores/appStore.ts`
- Modify: `src/components/layout/MainLayout.tsx`
- Modify: `src/components/layout/Sidebar.tsx`
- Modify: `src/components/layout/TabBar.tsx`
- Modify: `src/components/layout/StatusBar.tsx`
- Modify: `src/components/connections/ConnectionCenter.tsx`

- [x] Make the primary navigation expose SSH connection center, terminal, SFTP, monitor, and settings.
- [x] Route active tabs by `tab.type`: `terminal` renders `TerminalView`, `sftp` renders `SftpView`, `monitor` renders `MonitorView`.
- [x] Add clear actions to open terminal/SFTP/monitor tabs for the selected connection.
- [x] Keep tab close cleanup tied to `disconnect_terminal` when a session exists.
- [x] Run `npm run build`.
- [x] Commit:

```bash
git add src/types/index.ts src/stores/appStore.ts src/components/layout/MainLayout.tsx src/components/layout/Sidebar.tsx src/components/layout/TabBar.tsx src/components/layout/StatusBar.tsx src/components/connections/ConnectionCenter.tsx
git commit -m "feat: focus navigation on ssh workflows"
```

## Task 3: Terminal Session Reliability

**Files:**
- Modify: `src/components/terminal/TerminalView.tsx`
- Modify: `src-tauri/src/commands/terminal.rs`
- Modify: `src-tauri/src/terminal/mod.rs`
- Modify: `src-tauri/src/terminal/pty.rs`

- [x] Ensure one React terminal instance owns one SSH session.
- [x] Prevent stale listeners and duplicate `onData` handlers during tab switches.
- [x] Ensure initial resize happens after xterm fit.
- [x] Verify SSH command-line connectivity against the provided server.
- [x] Run `npm run build` and `cargo check`.
- [x] Commit:

```bash
git add src/components/terminal/TerminalView.tsx src-tauri/src/commands/terminal.rs src-tauri/src/terminal/mod.rs src-tauri/src/terminal/pty.rs
git commit -m "fix: harden ssh terminal sessions"
```

## Task 4: SFTP and Local Filesystem Commands

**Files:**
- Create: `src-tauri/src/commands/local_fs.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/sftp.rs`
- Modify: `src/components/files/SftpView.tsx`
- Modify: `src/utils/tauri.ts`

- [x] Add Tauri commands:
  - `list_local_dir(path)`
  - `write_local_file(path, data)`
  - `remove_local_file(path)`
  - `rename_local_file(src, dst)`
  - `create_local_dir(path)`
- [x] Return local entries using the existing `SftpEntry` shape.
- [x] Keep remote SFTP operations tied to active terminal tabs while using independent SSH sessions for file operations.
- [x] Improve empty-session and permission errors in the SFTP UI.
- [x] Run `npm run build` and `cargo check`.
- [x] Commit:

```bash
git add src-tauri/src/commands/local_fs.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/commands/sftp.rs src/components/files/SftpView.tsx src/utils/tauri.ts
git commit -m "feat: complete sftp file management commands"
```

## Task 5: Monitor Integration

**Files:**
- Modify: `src/components/monitor/MonitorView.tsx`
- Modify: `src/components/monitor/MonitorSidebar.tsx`
- Modify: `src-tauri/src/commands/monitor.rs`
- Modify: `src-tauri/src/monitor/mod.rs`
- Modify: `src/types/index.ts`

- [x] Ensure monitor tabs use the active terminal tab context while using independent SSH sessions for monitor collection.
- [x] Make no-session, loading, and command failure states actionable.
- [x] Verify remote monitor scripts work on the provided Ubuntu server where possible.
- [x] Run `npm run build` and `cargo check`.
- [x] Commit:

```bash
git add src/components/monitor/MonitorView.tsx src/components/monitor/MonitorSidebar.tsx src-tauri/src/commands/monitor.rs src-tauri/src/monitor/mod.rs src/types/index.ts
git commit -m "feat: wire monitor into ssh sessions"
```

## Task 6: Documentation and Final Verification

**Files:**
- Modify: `README.md`
- Modify: `DEV_PLAN.md`
- Modify: `IMPLEMENTATION_STATUS.md`
- Modify: `PRD_COMPLIANCE.md`

- [x] Update docs to reflect SSH-first current scope and restored tool views.
- [x] Document startup commands, dev-port behavior, and test-server validation.
- [x] Run final verification:

```bash
npm run build
cargo check
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

- [x] Commit:

```bash
git add README.md DEV_PLAN.md IMPLEMENTATION_STATUS.md PRD_COMPLIANCE.md
git commit -m "docs: update ssh core completion status"
```

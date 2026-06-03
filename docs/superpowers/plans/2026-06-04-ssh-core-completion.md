# SSH Core Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the SSH-centered product flow: connection management, terminal sessions, SFTP, and server monitoring.

**Architecture:** Keep the Tauri command boundary as the frontend/backend contract. Reuse the existing SSH `TerminalManager` sessions for terminal, SFTP, and monitor operations, and keep non-SSH features out of primary navigation.

**Tech Stack:** Tauri 2, Rust, ssh2, SQLite/rusqlite, React 19, TypeScript, Vite, Zustand, xterm.js.

---

## Scope

In scope:
- SSH connection CRUD and connection center.
- SSH terminal tabs.
- SFTP remote file management backed by an active SSH session.
- Local filesystem commands needed by the SFTP two-panel UI.
- Server monitor view backed by an active SSH session.
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

- [ ] Run baseline frontend build:

```bash
npm run build
```

Expected: TypeScript and Vite build exit 0.

- [ ] Run baseline Rust check:

```bash
cargo check
```

Expected: Rust check exits 0. Existing warnings are allowed during baseline and should be reduced in later tasks when touching those modules.

- [ ] Verify the real SSH server is reachable:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

Expected: output includes `MYTERM_SSH_OK`.

- [ ] Commit the plan and baseline docs:

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

- [ ] Make the primary navigation expose only SSH connection center, terminal, SFTP, monitor, and settings.
- [ ] Route active tabs by `tab.type`: `terminal` renders `TerminalView`, `sftp` renders `SftpView`, `monitor` renders `MonitorView`.
- [ ] Add clear actions to open terminal/SFTP/monitor tabs for the selected connection.
- [ ] Keep tab close cleanup tied to `disconnect_terminal` when a session exists.
- [ ] Run `npm run build`.
- [ ] Commit:

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

- [ ] Ensure one React terminal instance owns one SSH session.
- [ ] Prevent stale listeners and duplicate `onData` handlers during tab switches.
- [ ] Ensure initial resize happens after xterm fit.
- [ ] Verify SSH command-line connectivity against the provided server.
- [ ] Run `npm run build` and `cargo check`.
- [ ] Commit:

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

- [ ] Add Tauri commands:
  - `list_local_dir(path)`
  - `write_local_file(path, data)`
  - `remove_local_file(path)`
  - `rename_local_file(src, dst)`
  - `create_local_dir(path)`
- [ ] Return local entries using the existing `SftpEntry` shape.
- [ ] Keep remote SFTP operations backed by active SSH sessions.
- [ ] Improve empty-session and permission errors in the SFTP UI.
- [ ] Run `npm run build` and `cargo check`.
- [ ] Commit:

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

- [ ] Ensure monitor tabs use the active SSH session.
- [ ] Make no-session, loading, and command failure states actionable.
- [ ] Verify remote monitor scripts work on the provided Ubuntu server where possible.
- [ ] Run `npm run build` and `cargo check`.
- [ ] Commit:

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

- [ ] Update docs to reflect SSH-only current scope.
- [ ] Document startup commands, dev-port behavior, and test-server validation.
- [ ] Run final verification:

```bash
npm run build
cargo check
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

- [ ] Commit:

```bash
git add README.md DEV_PLAN.md IMPLEMENTATION_STATUS.md PRD_COMPLIANCE.md
git commit -m "docs: update ssh core completion status"
```


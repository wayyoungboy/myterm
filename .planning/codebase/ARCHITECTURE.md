<!-- refreshed: 2026-06-02 -->
# Architecture

**Analysis Date:** 2026-06-02

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│                     Frontend (React + TypeScript)                    │
│  ┌───────────────┐ ┌──────────────┐ ┌─────────────────────────────┐ │
│  │   Sidebar      │ │   TabBar     │ │  Content Area               │ │
│  │  `src/components│ │  `src/components│ │  `src/components/terminal/`│ │
│  │   /layout/      │ │   /layout/    │ │  `src/components/files/`    │ │
│  │   Sidebar.tsx`  │ │   TabBar.tsx` │ │  `src/components/monitor/`  │ │
│  └───────────────┘ └──────────────┘ └─────────────────────────────┘ │
│           │               │                      │                    │
│           ▼               ▼                      ▼                    │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │              Zustand Store (`src/stores/appStore.ts`)           │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│           │                                                          │
│           ▼                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │       Tauri API Bridge (`src/utils/tauri.ts`)                   │ │
│  │       invoke<T>('command_name', { args })                       │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ IPC (invoke / listen / emit)
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Backend (Rust + Tauri 2)                         │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │         Tauri Command Handlers (`src-tauri/src/commands/`)     │ │
│  │  connections│terminal│sftp│monitor│notes│ai│settings│...       │ │
│  └────────────────────────────────────────────────────────────────┘ │
│           │               │               │                          │
│           ▼               ▼               ▼                          │
│  ┌──────────────┐ ┌─────────────┐ ┌──────────────────┐             │
│  │ SSH Module    │ │ Terminal    │ │ Database (SQLite)│             │
│  │ `src-tauri/src│ │ Manager     │ │ `src-tauri/src/  │             │
│  │  /ssh/`       │ │ `src-tauri/ │ │  db/`            │             │
│  │              │ │  src/terminal│ │                  │             │
│  │              │ │  /`          │ │                  │             │
│  └──────────────┘ └─────────────┘ └──────────────────┘             │
│           │                                                          │
│           ▼                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │         Remote Servers (SSH / Telnet / RDP)                     │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| App | Root component, global keyboard shortcuts | `src/App.tsx` |
| MainLayout | Layout shell: sidebar + tabbar + content + statusbar | `src/components/layout/MainLayout.tsx` |
| Sidebar | Connection list, search, quick actions | `src/components/layout/Sidebar.tsx` |
| TabBar | Multi-tab management for terminal sessions | `src/components/layout/TabBar.tsx` |
| StatusBar | Connection status, monitor toggle | `src/components/layout/StatusBar.tsx` |
| ConnectionCenter | Connection management view (list/create/edit/delete) | `src/components/connections/ConnectionCenter.tsx` |
| ConnectionForm | Modal form for creating/editing connections | `src/components/connections/ConnectionForm.tsx` |
| TerminalView | xterm.js terminal with SSH session I/O | `src/components/terminal/TerminalView.tsx` |
| TelnetView | xterm.js terminal for Telnet sessions | `src/components/terminal/TelnetView.tsx` |
| SftpView | File browser using SFTP over existing SSH session | `src/components/files/SftpView.tsx` |
| MonitorSidebar | Real-time server monitoring (CPU/mem/net/disk/GPU) | `src/components/monitor/MonitorSidebar.tsx` |
| NotesView | Note-taking per connection | `src/components/notes/NotesView.tsx` |
| AiView | AI conversation interface | `src/components/ai/AiView.tsx` |
| SettingsView | Application settings | `src/components/settings/SettingsView.tsx` |
| PortForwardView | SSH port forwarding management | `src/components/portforward/PortForwardView.tsx` |
| QuickCommandsView | Saved command snippets | `src/components/quickcommands/QuickCommandsView.tsx` |
| ErrorBoundary | React error boundary fallback UI | `src/components/common/ErrorBoundary.tsx` |
| appStore | Zustand global state (tabs, connections, UI state) | `src/stores/appStore.ts` |
| tauri.ts | Typed wrappers for all Tauri IPC commands | `src/utils/tauri.ts` |
| types/index.ts | TypeScript interfaces matching Rust models | `src/types/index.ts` |

## Pattern Overview

**Overall:** Tauri 2 desktop application with a React SPA frontend and Rust backend, communicating via Tauri's IPC (invoke/event system).

**Key Characteristics:**
- **Two-process architecture:** Frontend (WebView) and backend (Rust) run in separate processes, connected by Tauri IPC
- **State managed by Zustand** on the frontend, with **Tauri managed state** (`State<T>`) on the backend for session managers and DB connection
- **Event-driven terminal I/O:** Backend spawns reader threads that emit events (`terminal-output-{id}`, `terminal-exit-{id}`) to the frontend; frontend listens and writes to xterm.js
- **Single SQLite database** for all persistent data (connections, groups, notes, AI conversations, settings)
- **SSH session reuse:** A single SSH session supports both terminal shell and SFTP operations simultaneously
- **No frontend router:** Navigation is state-driven via `currentView` and `tabs` in the Zustand store; `react-router-dom` is listed as a dependency but not used

## Layers

**Frontend Presentation Layer:**
- Purpose: Renders UI and handles user interaction
- Location: `src/components/`
- Contains: React functional components, xterm.js integration, recharts visualizations
- Depends on: Zustand store, Tauri API bridge
- Used by: `src/App.tsx` -> `src/main.tsx`

**Frontend State Layer:**
- Purpose: Manages application state (tabs, connections, UI toggles)
- Location: `src/stores/appStore.ts`
- Contains: Single Zustand store with all app state
- Depends on: TypeScript types from `src/types/index.ts`
- Used by: All components via `useAppStore()` hook

**Frontend API Bridge:**
- Purpose: Typed wrappers around `invoke()` for Tauri IPC commands
- Location: `src/utils/tauri.ts`
- Contains: One exported function per Tauri command
- Depends on: `@tauri-apps/api/core`
- Used by: Components that need backend data

**Backend Command Layer:**
- Purpose: Tauri command handlers that receive IPC calls from the frontend
- Location: `src-tauri/src/commands/`
- Contains: `#[tauri::command]` functions, one module per domain
- Depends on: SSH module, DB module, session managers
- Used by: Tauri's `invoke_handler` registration in `src-tauri/src/lib.rs`

**Backend SSH Layer:**
- Purpose: SSH connection management, SFTP operations, shell sessions
- Location: `src-tauri/src/ssh/`
- Contains: `connection.rs` (connect/auth), `sftp.rs` (file ops), `mod.rs` (SessionManager)
- Depends on: `ssh2` crate
- Used by: Terminal commands, SFTP commands, monitor commands

**Backend Terminal Layer:**
- Purpose: Manages SSH shell channels and background reader threads
- Location: `src-tauri/src/terminal/`
- Contains: `mod.rs` (TerminalManager, reader thread), `pty.rs` (PTY shell setup)
- Depends on: SSH module, Tauri event emitter
- Used by: Terminal commands, SFTP commands (session lookup)

**Backend Database Layer:**
- Purpose: SQLite persistence for connections, notes, settings, AI data
- Location: `src-tauri/src/db/`
- Contains: `mod.rs` (DbConn wrapper), `schema.rs` (table definitions), `models.rs` (data structs)
- Depends on: `rusqlite` crate
- Used by: All command handlers that read/write persistent data

**Backend Monitor Layer:**
- Purpose: Collects server metrics via SSH script execution
- Location: `src-tauri/src/monitor/mod.rs`
- Contains: Shell script execution and output parsing for CPU/memory/disk/network/GPU
- Depends on: SSH session, DB models
- Used by: Monitor command handler

**Backend Crypto Layer:**
- Purpose: AES-256-CBC encryption/decryption of stored passwords
- Location: `src-tauri/src/crypto.rs`
- Contains: Key derivation (SHA-256), encrypt/decrypt functions, master password generation
- Depends on: `aes`, `cbc`, `sha2`, `base64` crates
- Used by: Connection create/update (encrypt), terminal connect (decrypt)

## Data Flow

### SSH Terminal Connection

1. User double-clicks a connection in Sidebar (`src/components/layout/Sidebar.tsx:167`)
2. `handleConnect()` creates a Tab in Zustand store and sets `activeTabId` (`Sidebar.tsx:40-51`)
3. `MainLayout` renders `TerminalView` with `connectionId` prop (`src/components/layout/MainLayout.tsx:25`)
4. `TerminalView` calls `invoke('connect_terminal', { connectionId })` (`src/components/terminal/TerminalView.tsx:217`)
5. Backend `connect_terminal` command (`src-tauri/src/commands/terminal.rs:12`):
   - Reads connection from SQLite, decrypts password
   - Calls `ssh::connection::connect()` to establish SSH session
   - Calls `terminal::pty::open_shell()` to open interactive shell channel
   - Stores `TerminalSession` in `TerminalManager`
   - Spawns background reader thread via `tm.start_reader()`
   - Returns `session_id` (UUID)
6. Frontend receives `sessionId`, calls `setupTerminalIO()` (`TerminalView.tsx:147`):
   - Listens on `terminal-output-{sessionId}` events -> writes bytes to xterm.js
   - Listens on `terminal-exit-{sessionId}` events -> shows "Session ended"
   - Hooks `term.onData()` -> calls `invoke('terminal_write', { sessionId, data })` for user keystrokes
7. Backend reader thread (`src-tauri/src/terminal/mod.rs:88-124`):
   - Reads from SSH channel in a loop (non-blocking with 10ms sleep)
   - Emits `terminal-output-{sessionId}` event with raw bytes
   - On EOF/error, emits `terminal-exit-{sessionId}`

### SFTP File Operations

1. SFTP reuses the existing SSH session from the terminal tab
2. Frontend calls `invoke('sftp_list_dir', { sessionId, path })` (`src/components/files/SftpView.tsx`)
3. Backend `sftp_list_dir` (`src-tauri/src/commands/sftp.rs:6`):
   - Looks up `ssh2::Session` from `TerminalManager` by session ID
   - Calls `ssh::sftp::list_dir()` which opens an SFTP channel on the session
4. All SFTP operations (read, write, delete, rename, mkdir) follow the same pattern

### Server Monitoring

1. User opens monitor sidebar via toggle button
2. `MonitorSidebar` component (`src/components/monitor/MonitorSidebar.tsx:385`) starts a 3-second polling interval
3. Each tick calls `invoke('get_monitor_data', { sessionId })`
4. Backend `get_monitor_data` (`src-tauri/src/commands/monitor.rs:6`):
   - Gets `ssh2::Session` from `TerminalManager`
   - Calls `monitor::fetch_monitor_data()` (`src-tauri/src/monitor/mod.rs:44`)
   - Executes a POSIX shell script on the remote server via SSH exec
   - Parses structured output (===SECTION=== markers)
   - Returns `MonitorData` struct
5. Frontend updates recharts visualizations (CPU line chart, memory pie chart, network line chart)

### Telnet Connection

1. User opens a telnet connection via `invoke('connect_telnet', { host, port })`
2. Backend `connect_telnet` (`src-tauri/src/commands/telnet.rs:27`):
   - Opens a raw TCP connection
   - Spawns a reader thread that handles Telnet protocol negotiation (IAC commands)
   - Stores `TelnetSession` in `TelnetManager`
   - Returns `session_id`
3. Frontend listens on `telnet-output-{sessionId}` and `telnet-exit-{sessionId}` events
4. User input sent via `invoke('telnet_write', { sessionId, data })`

### Local Terminal

1. User opens a local terminal via `invoke('open_local_terminal', { shell })`
2. Backend `open_local_terminal` (`src-tauri/src/commands/local_terminal.rs:28`):
   - Spawns a child process (default shell or specified)
   - Spawns stdout and stderr reader threads
   - Emits `local-output-{sessionId}` and `local-exit-{sessionId}` events
   - Stores `LocalTerminalSession` in `LocalTerminalManager`

**State Management:**
- **Frontend:** Single Zustand store (`src/stores/appStore.ts`) holds all UI state: tabs, active tab, connections list, groups, sidebar collapsed state, current view mode, search query, monitor sidebar visibility
- **Backend:** Tauri managed state (`app_handle.manage()`) holds singleton instances of: `DbConn` (SQLite), `TerminalManager` (SSH sessions), `PortForwardManager`, `TelnetManager`, `LocalTerminalManager`
- **No shared mutable state between frontend and backend** -- all communication is via IPC

## Key Abstractions

**TerminalManager:**
- Purpose: Manages all active SSH terminal sessions, their channels, and background reader threads
- Examples: `src-tauri/src/terminal/mod.rs`
- Pattern: HashMap of `TerminalSession` behind `Arc<Mutex<...>>`, with `AtomicBool` for graceful thread shutdown

**SessionManager (SSH):**
- Purpose: Stores raw SSH sessions for reuse (e.g., SFTP on existing terminal connection)
- Examples: `src-tauri/src/ssh/mod.rs`
- Pattern: HashMap of `SshSession` behind `Arc<Mutex<...>>`

**DbConn:**
- Purpose: Thread-safe SQLite connection wrapper
- Examples: `src-tauri/src/db/mod.rs`
- Pattern: Single `rusqlite::Connection` behind `Mutex`, with WAL mode and foreign keys enabled

**Tab:**
- Purpose: Represents an open terminal/SFTP/monitor tab in the UI
- Examples: `src/types/index.ts:137-143`
- Pattern: `{ id, title, connectionId, sessionId, type }` -- sessionId is null until connected

**ViewMode:**
- Purpose: Controls which content view is shown when no tab is active
- Examples: `src/types/index.ts:145`
- Pattern: Union type `'terminal' | 'sftp' | 'monitor' | 'notes' | 'ai' | 'settings' | 'portforward' | 'telnet' | 'quickcommands'`

## Entry Points

**Frontend Entry:**
- Location: `src/main.tsx`
- Triggers: Tauri WebView loads `index.html` which loads this
- Responsibilities: Renders `<App />` into `#root` with React StrictMode

**Backend Entry:**
- Location: `src-tauri/src/main.rs` -> `src-tauri/src/lib.rs`
- Triggers: Tauri runtime starts
- Responsibilities: Initializes Tauri app, registers all managed state (`DbConn`, `TerminalManager`, `PortForwardManager`, `TelnetManager`, `LocalTerminalManager`), registers all IPC command handlers

**Tauri Config:**
- Location: `src-tauri/tauri.conf.json`
- Triggers: Tauri build/dev system
- Responsibilities: Window configuration (1280x800, min 900x600), build commands, bundle settings

## Architectural Constraints

- **Threading:** Backend uses OS threads (`std::thread::spawn`) for terminal/Telnet/local shell reader loops. Not using Tokio async for I/O -- Tokio is listed as a dependency but not actively used for SSH/terminal operations. The SSH session switches between blocking mode (for setup, SFTP, monitor script) and non-blocking mode (for terminal reader thread).
- **Global state:** Tauri managed state (`State<T>`) acts as singletons. `DbConn` uses `std::sync::Mutex`, while `TerminalManager`, `PortForwardManager`, `TelnetManager`, `LocalTerminalManager` use `parking_lot::Mutex`.
- **Blocking mode toggle:** SSH sessions toggle between `set_blocking(true)` for synchronous operations (SFTP, monitor script execution) and `set_blocking(false)` for the terminal reader thread. This is a critical constraint -- SFTP and terminal cannot truly run concurrently on the same session without careful coordination.
- **No circular imports:** Module dependencies are strictly hierarchical: commands -> ssh/terminal/monitor/crypto/db. No circular dependencies detected.
- **Single window:** The app uses a single Tauri window. No multi-window support.
- **CSP disabled:** Content Security Policy is set to `null` in `tauri.conf.json` -- no restrictions on resource loading.

## Anti-Patterns

### Shared SSH Session Between Terminal and SFTP

**What happens:** SFTP operations (`commands/sftp.rs`) look up the `ssh2::Session` from the `TerminalManager` by the terminal's session ID. This means SFTP reuses the same SSH session (and thus the same blocking mode state) as the terminal.

**Why it's wrong:** The terminal reader thread runs in non-blocking mode, while SFTP operations require blocking mode. Calling SFTP operations while the terminal is active forces a blocking mode switch, which can cause the reader thread to misbehave or miss data. The code in `monitor/mod.rs:46-59` explicitly toggles blocking mode, confirming this is a known issue.

**Do this instead:** Create separate SSH sessions for SFTP and monitoring, distinct from the terminal session. The `connect_terminal_for_sftp` function in `commands/terminal.rs:107` exists but is incomplete (returns session_id without actually storing it). Complete this pattern.

### Hardcoded Master Password Derivation

**What happens:** `crypto.rs:54-58` derives the master encryption password from `"myterm-" + hostname`. This is deterministic and not secret.

**Why it's wrong:** Anyone with access to the machine can derive the same key and decrypt all stored passwords. The hostname is not a secret.

**Do this instead:** Use OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service) to store a randomly generated master password. Or prompt the user for a master password on first use.

### Inline SQL in Command Handlers

**What happens:** SQL queries are written as string literals directly inside `#[tauri::command]` functions (e.g., `commands/connections.rs:59-60`).

**Why it's wrong:** Duplicated query patterns, hard to maintain, easy to introduce inconsistencies between INSERT and SELECT column lists.

**Do this instead:** Create a repository/query layer in `src-tauri/src/db/` that encapsulates all SQL queries. Commands should call `db.get_connection(id)` instead of writing raw SQL.

## Error Handling

**Strategy:** Backend commands return `Result<T, String>` -- all errors are converted to strings. Frontend catches errors from `invoke()` and displays them in the UI or logs to console.

**Patterns:**
- Backend: `.map_err(|e| format!("...: {}", e))` is the universal error conversion pattern
- Frontend: `try/catch` around `invoke()` calls, with `console.error()` for logging and `setError()` state for user-facing messages
- Terminal errors: Connection failures display red text in xterm.js (`TerminalView.tsx:238`)
- No structured error types on the backend -- all errors are `String`

## Cross-Cutting Concerns

**Logging:** Backend uses `eprintln!()` for debug output (e.g., `commands/connections.rs:265-269`). `log` and `env_logger` crates are in `Cargo.toml` but not actively used. Frontend uses `console.log/error`.

**Validation:** Minimal. Connection form validates that host is not empty (`TerminalView.tsx:197`). Backend does no input validation -- relies on SQLite constraints and SSH library error handling.

**Authentication:** Password encryption uses AES-256-CBC with a hostname-derived key (`src-tauri/src/crypto.rs`). Passwords are never sent to the frontend -- `ConnectionResponse` has `has_password: bool` instead of the actual password.

---

*Architecture analysis: 2026-06-02*

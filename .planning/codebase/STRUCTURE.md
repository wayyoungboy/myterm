# Codebase Structure

**Analysis Date:** 2026-06-02

## Directory Layout

```
myterm/
├── index.html                  # Vite entry HTML, loads src/main.tsx
├── package.json                # NPM dependencies and scripts
├── vite.config.ts              # Vite config (React + Tailwind plugins, port 1420)
├── tsconfig.json               # TypeScript config
├── tsconfig.node.json          # TypeScript config for Vite/Node
├── DEV_PLAN.md                 # Development plan document
├── IMPLEMENTATION_STATUS.md    # Feature implementation tracking
├── PRD_COMPLIANCE.md           # PRD compliance checklist
├── public/                     # Static assets served directly
├── src/                        # Frontend source (React + TypeScript)
│   ├── main.tsx                # React entry point
│   ├── App.tsx                 # Root component
│   ├── vite-env.d.ts           # Vite type declarations
│   ├── components/             # React components (feature-organized)
│   │   ├── layout/             # Shell layout components
│   │   │   ├── MainLayout.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   ├── TabBar.tsx
│   │   │   └── StatusBar.tsx
│   │   ├── connections/        # Connection management
│   │   │   ├── ConnectionCenter.tsx
│   │   │   └── ConnectionForm.tsx
│   │   ├── terminal/           # Terminal views
│   │   │   ├── TerminalView.tsx
│   │   │   └── TelnetView.tsx
│   │   ├── files/              # SFTP file browser
│   │   │   └── SftpView.tsx
│   │   ├── monitor/            # Server monitoring
│   │   │   ├── MonitorSidebar.tsx
│   │   │   └── MonitorView.tsx
│   │   ├── notes/              # Note-taking
│   │   │   └── NotesView.tsx
│   │   ├── ai/                 # AI conversations
│   │   │   └── AiView.tsx
│   │   ├── settings/           # App settings
│   │   │   └── SettingsView.tsx
│   │   ├── portforward/        # SSH port forwarding
│   │   │   └── PortForwardView.tsx
│   │   ├── quickcommands/      # Saved commands
│   │   │   └── QuickCommandsView.tsx
│   │   └── common/             # Shared components
│   │       └── ErrorBoundary.tsx
│   ├── stores/                 # State management
│   │   └── appStore.ts         # Single Zustand store
│   ├── types/                  # TypeScript type definitions
│   │   └── index.ts            # All interfaces matching Rust models
│   ├── utils/                  # Utility functions
│   │   └── tauri.ts            # Tauri IPC command wrappers
│   └── styles/                 # Global styles
│       └── globals.css         # CSS variables, base styles, component styles
├── src-tauri/                  # Backend source (Rust + Tauri 2)
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri configuration
│   ├── build.rs                # Tauri build script
│   ├── icons/                  # App icons for all platforms
│   └── src/
│       ├── main.rs             # Rust entry point (calls lib::run())
│       ├── lib.rs              # Tauri app setup, state registration, command handlers
│       ├── crypto.rs           # AES-256-CBC password encryption/decryption
│       ├── commands/           # Tauri IPC command handlers (one per domain)
│       │   ├── mod.rs          # Module declarations
│       │   ├── connections.rs  # CRUD for groups and connections
│       │   ├── terminal.rs     # SSH terminal connect/disconnect/write/resize
│       │   ├── sftp.rs         # SFTP file operations
│       │   ├── monitor.rs      # Server monitoring data fetch
│       │   ├── notes.rs        # Note CRUD
│       │   ├── ai.rs           # AI conversation/message CRUD
│       │   ├── settings.rs     # Settings key-value store
│       │   ├── port_forward.rs # SSH port forwarding (local/dynamic)
│       │   ├── ping.rs         # TCP ping utility
│       │   ├── rdp.rs          # RDP connection (stub)
│       │   ├── telnet.rs       # Telnet connection and I/O
│       │   ├── quick_commands.rs # Quick command CRUD
│       │   ├── import_export.rs  # Connection import/export as JSON
│       │   ├── local_terminal.rs # Local shell terminal
│       │   └── screenshot.rs   # Dev screenshot utility
│       ├── ssh/                # SSH protocol implementation
│       │   ├── mod.rs          # SshSession struct, SessionManager
│       │   ├── connection.rs   # TCP connect, SSH handshake, auth
│       │   └── sftp.rs         # SFTP operations (list, read, write, delete, rename, mkdir)
│       ├── terminal/           # Terminal session management
│       │   ├── mod.rs          # TerminalManager, background reader thread
│       │   └── pty.rs          # PTY/shell channel setup
│       ├── monitor/            # Remote server monitoring
│       │   └── mod.rs          # Shell script execution and output parsing
│       └── db/                 # Database layer
│           ├── mod.rs          # DbConn wrapper (SQLite with WAL mode)
│           ├── schema.rs       # Table creation (groups, connections, notes, ai_*, settings)
│           └── models.rs       # Rust data structs (Serialize/Deserialize)
└── .github/                    # GitHub Actions CI/CD
```

## Directory Purposes

**`src/components/`:**
- Purpose: All React UI components, organized by feature domain
- Contains: `.tsx` files (one component per file, sometimes with sub-components)
- Key files: Each subdirectory has a primary view component (e.g., `TerminalView.tsx`, `SftpView.tsx`)

**`src/stores/`:**
- Purpose: Zustand state management
- Contains: Single `appStore.ts` file with all application state
- Key files: `src/stores/appStore.ts`

**`src/types/`:**
- Purpose: TypeScript interface definitions that mirror Rust backend models
- Contains: `index.ts` with all shared types
- Key files: `src/types/index.ts`

**`src/utils/`:**
- Purpose: Utility functions, primarily Tauri IPC wrappers
- Contains: `tauri.ts` with one exported function per Tauri command
- Key files: `src/utils/tauri.ts`

**`src/styles/`:**
- Purpose: Global CSS with CSS custom properties (design tokens)
- Contains: `globals.css` with CSS variables, base resets, component styles
- Key files: `src/styles/globals.css`

**`src-tauri/src/commands/`:**
- Purpose: Tauri IPC command handlers -- the backend API surface
- Contains: One `.rs` file per feature domain, each with `#[tauri::command]` functions
- Key files: `connections.rs`, `terminal.rs`, `sftp.rs`, `monitor.rs`

**`src-tauri/src/ssh/`:**
- Purpose: SSH protocol implementation (connection, authentication, SFTP)
- Contains: Connection setup, session management, SFTP file operations
- Key files: `connection.rs` (connect + auth), `sftp.rs` (file ops)

**`src-tauri/src/terminal/`:**
- Purpose: SSH terminal session lifecycle management
- Contains: TerminalManager (session store), PTY setup, background reader thread
- Key files: `mod.rs` (manager + reader), `pty.rs` (shell channel setup)

**`src-tauri/src/db/`:**
- Purpose: SQLite database access layer
- Contains: Connection wrapper, schema initialization, data model structs
- Key files: `mod.rs` (DbConn), `schema.rs` (DDL), `models.rs` (structs)

**`src-tauri/src/monitor/`:**
- Purpose: Remote server monitoring via SSH script execution
- Contains: POSIX shell script for collecting CPU/memory/disk/network/GPU metrics, parser
- Key files: `mod.rs` (single file module)

## Key File Locations

**Entry Points:**
- `src/main.tsx`: Frontend entry -- renders `<App />` into DOM
- `src-tauri/src/main.rs`: Backend entry -- calls `myterm_app_lib::run()`
- `src-tauri/src/lib.rs`: Tauri app setup -- registers state managers and command handlers
- `index.html`: HTML shell loaded by Tauri WebView

**Configuration:**
- `package.json`: NPM dependencies and scripts (`dev`, `build`, `preview`, `tauri`)
- `vite.config.ts`: Vite bundler config (React plugin, Tailwind plugin, port 1420)
- `tsconfig.json`: TypeScript compiler options
- `src-tauri/Cargo.toml`: Rust dependencies
- `src-tauri/tauri.conf.json`: Tauri app config (window size, build commands, bundle settings)

**Core Logic:**
- `src-tauri/src/ssh/connection.rs`: SSH connection establishment and authentication
- `src-tauri/src/terminal/mod.rs`: Terminal session management and I/O reader thread
- `src-tauri/src/monitor/mod.rs`: Remote monitoring script and output parser
- `src-tauri/src/crypto.rs`: Password encryption/decryption
- `src/components/terminal/TerminalView.tsx`: xterm.js integration with SSH I/O
- `src/components/monitor/MonitorSidebar.tsx`: Real-time monitoring dashboard
- `src/stores/appStore.ts`: All frontend state management

**Testing:**
- No test files detected in either frontend or backend

## Naming Conventions

**Files:**
- Frontend components: PascalCase `.tsx` files (e.g., `TerminalView.tsx`, `ConnectionForm.tsx`)
- Frontend utilities: camelCase `.ts` files (e.g., `appStore.ts`, `tauri.ts`)
- Backend modules: snake_case `.rs` files (e.g., `connection.rs`, `port_forward.rs`, `local_terminal.rs`)
- One component per file, file name matches the exported component/function name

**Directories:**
- Frontend: lowercase, feature-based grouping (e.g., `terminal/`, `connections/`, `monitor/`)
- Backend: lowercase, module-based grouping (e.g., `commands/`, `ssh/`, `terminal/`, `db/`)

**Variables/Functions:**
- Frontend: camelCase for functions and variables (e.g., `handleConnect`, `sessionId`, `activeTabId`)
- Backend: snake_case for functions and variables (e.g., `connect_terminal`, `session_id`, `password_enc`)
- TypeScript interfaces: PascalCase (e.g., `Connection`, `MonitorData`, `TerminalViewProps`)

**Tauri Commands:**
- snake_case on the Rust side (e.g., `connect_terminal`, `sftp_list_dir`)
- camelCase on the TypeScript wrapper side (e.g., `connectTerminal`, `sftpListDir`)
- Tauri automatically converts between the two via `invoke()`

**CSS:**
- CSS custom properties with kebab-case (e.g., `--bg-primary`, `--text-muted`, `--accent`)
- Tailwind utility classes used alongside custom CSS classes (e.g., `.btn`, `.input`, `.modal`)

## Where to Add New Code

**New Feature (full stack):**
1. Backend command: Create `src-tauri/src/commands/{feature_name}.rs` with `#[tauri::command]` functions
2. Register module: Add `pub mod {feature_name};` to `src-tauri/src/commands/mod.rs`
3. Register commands: Add `commands::{feature_name}::*` to `invoke_handler` in `src-tauri/src/lib.rs`
4. Frontend types: Add interfaces to `src/types/index.ts`
5. Frontend API wrappers: Add functions to `src/utils/tauri.ts`
6. Frontend component: Create `src/components/{domain}/{FeatureView}.tsx`
7. Add to MainLayout: Wire the new view into `renderContent()` in `src/components/layout/MainLayout.tsx` and add a `ViewMode` to `src/types/index.ts`

**New Backend Command (single command):**
- Add `#[tauri::command]` function to the appropriate file in `src-tauri/src/commands/`
- Register in `invoke_handler` array in `src-tauri/src/lib.rs`
- Add typed wrapper to `src/utils/tauri.ts`

**New React Component:**
- Create `.tsx` file in the appropriate `src/components/{domain}/` subdirectory
- If no matching domain exists, create a new subdirectory
- Import and use from parent component or from `MainLayout.tsx`

**New Database Table:**
- Add `CREATE TABLE IF NOT EXISTS` statement to `src-tauri/src/db/schema.rs`
- Add Rust struct to `src-tauri/src/db/models.rs`
- Add TypeScript interface to `src/types/index.ts`

**New State in Zustand Store:**
- Add property and setter to `AppState` interface in `src/stores/appStore.ts`
- Add implementation in the `create()` call

**Shared Utilities:**
- Frontend: Add to `src/utils/` as a new `.ts` file or extend `tauri.ts`
- Backend: Add to the appropriate module or create a new module in `src-tauri/src/`

## Special Directories

**`node_modules/`:**
- Purpose: NPM dependencies
- Generated: Yes (by `npm install`)
- Committed: No (in `.gitignore`)

**`dist/`:**
- Purpose: Vite build output for frontend
- Generated: Yes (by `npm run build`)
- Committed: No (in `.gitignore`)

**`src-tauri/target/`:**
- Purpose: Rust compilation output
- Generated: Yes (by `cargo build`)
- Committed: No (in `.gitignore`)

**`src-tauri/icons/`:**
- Purpose: App icons for all platforms (macOS .icns, Windows .ico, Linux .png)
- Generated: No (manually created)
- Committed: Yes

**`.github/`:**
- Purpose: GitHub Actions CI/CD workflows
- Generated: No
- Committed: Yes

**`xterminal/`:**
- Purpose: Appears to be a reference/prototype directory (untracked in git)
- Generated: No
- Committed: No (untracked)

**`.planning/`:**
- Purpose: GSD planning documents (codebase analysis, phase plans)
- Generated: Yes (by GSD tooling)
- Committed: Not yet (untracked)

---

*Structure analysis: 2026-06-02*

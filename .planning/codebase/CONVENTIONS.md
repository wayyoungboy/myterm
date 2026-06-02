# Coding Conventions

**Analysis Date:** 2026-06-02

## Naming Patterns

**Files:**
- TypeScript components: PascalCase (e.g., `TerminalView.tsx`, `ConnectionForm.tsx`, `MainLayout.tsx`)
- TypeScript utilities/stores: camelCase (e.g., `appStore.ts`, `tauri.ts`)
- TypeScript types: camelCase with descriptive names (e.g., `index.ts` in `src/types/`)
- Rust modules: snake_case (e.g., `connection.rs`, `port_forward.rs`, `quick_commands.rs`)
- Rust command modules: snake_case matching feature domain (e.g., `connections.rs`, `terminal.rs`, `sftp.rs`)

**Functions:**
- TypeScript: camelCase (e.g., `getConnections`, `createConnection`, `handleConnect`, `formatBytes`)
- React event handlers: `handle` prefix (e.g., `handleConnect`, `handleSave`, `handleDeleteNote`)
- Rust: snake_case (e.g., `get_groups`, `create_connection`, `connect_terminal`)
- Rust Tauri commands: snake_case with `#[tauri::command]` attribute

**Variables:**
- TypeScript: camelCase (e.g., `sessionId`, `activeTabId`, `formHost`)
- State variables: descriptive names matching their purpose (e.g., `connecting`, `saving`, `error`)
- Form state: `form` prefix (e.g., `formName`, `formHost`, `formPort`, `formUsername`)
- Rust: snake_case (e.g., `session_id`, `connection_id`, `password_enc`)

**Types/Interfaces:**
- TypeScript: PascalCase (e.g., `Connection`, `ConnectionInput`, `MonitorData`, `Tab`)
- Response types: descriptive suffix (e.g., `ConnectionResponse`, `PingResult`)
- Rust structs: PascalCase with `#[derive(Debug, Serialize, Deserialize, Clone)]` (e.g., `Group`, `Connection`, `MonitorData`)

## Code Style

**Formatting:**
- No explicit Prettier or ESLint config detected
- TypeScript: 2-space indentation, single quotes for strings
- Rust: 4-space indentation (standard rustfmt)
- Tailwind CSS v4 with `@import "tailwindcss"` in `src/styles/globals.css`

**Linting:**
- TypeScript strict mode enabled in `tsconfig.json`: `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`, `noFallthroughCasesInSwitch: true`
- No ESLint config file detected (may rely on editor settings)

## Import Organization

**Order:**
1. React/framework imports (e.g., `import { useState, useEffect } from 'react'`)
2. Third-party library imports (e.g., `import { invoke } from '@tauri-apps/api/core'`)
3. Internal component imports (e.g., `import { useAppStore } from '../../stores/appStore'`)
4. Type imports using `import type` syntax (e.g., `import type { Connection } from '../../types'`)
5. CSS imports (e.g., `import '@xterm/xterm/css/xterm.css'`)

**Path Aliases:**
- No path aliases configured
- Use relative imports throughout (e.g., `../../stores/appStore`, `../types`)

**Rust Imports:**
- Standard library first
- External crates (e.g., `use ssh2::Session`, `use tauri::State`)
- Internal crate modules (e.g., `use crate::db::models::{Connection, ConnectionInput, Group}`)

## Error Handling

**TypeScript Patterns:**
- Try-catch with `console.error` for logging: `catch (e) { console.error('Failed:', e); }`
- Error state in components: `const [error, setError] = useState<string | null>(null)`
- Display errors in UI with styled error divs using `var(--error)` color
- Tauri invoke errors propagated as strings: `invoke('command').catch(() => {})`
- Error boundary component at `src/components/common/ErrorBoundary.tsx` for React rendering errors

**Rust Patterns:**
- All Tauri commands return `Result<T, String>` where errors are string messages
- Use `.map_err(|e| e.to_string())` to convert rusqlite/ssh2 errors to strings
- Use `.map_err(|e| format!("Descriptive message: {}", e))` for contextual errors
- Mutex lock errors: `.map_err(|e| e.to_string())?`
- Silent error handling with `.ok()` for non-critical operations (e.g., `channel.wait_close().ok()`)
- Early return with `?` operator for error propagation

**Example (Rust):**
```rust
#[tauri::command]
pub fn get_groups(db: State<'_, DbConn>) -> Result<Vec<Group>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT ...")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| { ... })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
```

## Logging

**Framework:** `eprintln!` macro for Rust debug output, `console.log`/`console.error` for TypeScript

**Patterns:**
- Rust: `eprintln!("[Delete] Backend: deleting connection id={}", id)` for debug logging
- TypeScript: `console.error('Failed to load connections:', e)` in catch blocks
- TypeScript: `console.log('[Screenshot] Saved to:', path)` for success feedback
- No structured logging framework configured (env_logger in Cargo.toml but unused)

## Comments

**When to Comment:**
- Section separators in larger files: `// ── Helpers ──`, `// ── Component ──`
- Inline comments for non-obvious logic
- Rust doc comments (`///`) for public functions (e.g., `/// Collect server hardware info`)
- TODO comments for incomplete features: `// TODO: Implement ProxyJump`

**JSDoc/TSDoc:**
- Not used. Comments are inline and brief.

## Function Design

**Size:** Components can be large (300+ lines) with inline sub-components and helpers

**Parameters:**
- TypeScript props: destructured in function signature `{ connectionId, initialData, onClose, onSaved }: Props`
- Rust commands: individual parameters, not structs (Tauri convention)
- Optional parameters: use `Option<T>` in Rust, `?` suffix in TypeScript interfaces

**Return Values:**
- TypeScript async functions: return Promises, no explicit return types
- Rust commands: `Result<T, String>` where `String` is the error message

## Module Design

**Exports:**
- Named exports for components: `export function TerminalView()`
- Default exports also used: `export default TerminalView`
- Store: single named export `export const useAppStore = create<AppState>(...)`
- Utility functions: individual named exports (no barrel files)
- Rust: `pub mod` declarations in `mod.rs`, `pub fn` for command functions

**Barrel Files:**
- `src/commands/mod.rs` re-exports all command modules
- `src/types/index.ts` contains all TypeScript type definitions
- `src/utils/tauri.ts` is a single file with all Tauri API wrappers

## State Management

**Zustand Store Pattern (`src/stores/appStore.ts`):**
- Single store with `create<AppState>((set) => ({ ... }))`
- Interface defines both state and actions
- Actions use `set` with partial state updates
- Derived state computed in components (not in store)

**Example:**
```typescript
interface AppState {
  tabs: Tab[];
  activeTabId: string | null;
  addTab: (tab: Tab) => void;
  removeTab: (id: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  tabs: [],
  activeTabId: null,
  addTab: (tab) => set((s) => ({
    tabs: [...s.tabs, tab],
    activeTabId: tab.id,
  })),
  removeTab: (id) => set((s) => {
    const newTabs = s.tabs.filter((t) => t.id !== id);
    const newActive = s.activeTabId === id
      ? (newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null)
      : s.activeTabId;
    return { tabs: newTabs, activeTabId: newActive };
  }),
}));
```

## Styling

**Approach:**
- Tailwind CSS v4 utility classes for layout and spacing
- CSS custom properties (variables) for theming: `var(--bg-primary)`, `var(--text-secondary)`
- Global CSS at `src/styles/globals.css` with component classes (`.btn`, `.input`, `.modal`)
- Dark theme only (hardcoded colors in `:root`)
- Inline styles for dynamic/conditional values: `style={{ color: 'var(--accent)' }}`

**CSS Custom Properties:**
- `--bg-primary`, `--bg-secondary`, `--bg-surface`, `--bg-hover`
- `--text-primary`, `--text-secondary`, `--text-muted`
- `--accent`, `--success`, `--warning`, `--error`
- `--border`, `--sidebar-width`, `--tab-height`

**Component Classes:**
- `.btn`, `.btn-primary`, `.btn-secondary`, `.btn-danger`, `.btn-ghost`
- `.input`, `.select`
- `.modal-overlay`, `.modal`
- `.tab`, `.tab-bar`, `.tab-close`
- `.context-menu`, `.context-menu-item`
- `.status-bar`

## Tauri Command Pattern

**Frontend (`src/utils/tauri.ts`):**
- Thin wrappers around `invoke` from `@tauri-apps/api/core`
- Typed generics for return values: `invoke<Group[]>('get_groups')`
- Optional parameters passed as-is (Tauri handles None)

**Backend (`src-tauri/src/commands/`):**
- Each feature in its own file under `src/commands/`
- `#[tauri::command]` attribute on every command function
- Database access via `State<'_, DbConn>` injection
- Manager access via `State<'_, TerminalManager>` injection
- Return `Result<T, String>` for all commands
- Register all commands in `src/lib.rs` via `tauri::generate_handler![...]`

---

*Convention analysis: 2026-06-02*

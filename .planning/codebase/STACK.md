# Technology Stack

**Analysis Date:** 2026-06-02

## Languages

**Primary:**
- TypeScript ~5.8.3 - Frontend application logic, React components, state management
- Rust 2021 Edition - Backend application logic, system commands, SSH/SFTP, database

**Secondary:**
- CSS - Custom properties for theming, Tailwind CSS utility classes
- SQL - SQLite queries for data persistence

## Runtime

**Environment:**
- Node.js 22.x - Frontend build tooling (npm ci, vite)
- Rust stable - Backend compilation via Tauri

**Package Manager:**
- npm - Frontend dependencies
- Cargo - Rust dependencies
- Lockfile: Both `package-lock.json` and `Cargo.lock` present

## Frameworks

**Core:**
- Tauri 2.x - Desktop application framework (Rust backend + web frontend)
- React 19.1.0 - UI component library
- Vite 7.0.4 - Frontend build tool and dev server

**Testing:**
- Not configured - No test framework detected

**Build/Dev:**
- Tauri CLI 2.x - Build and development commands (`npm run tauri`)
- TypeScript 5.8.3 - Type checking with strict mode enabled
- Tailwind CSS 4.3.0 - Utility-first CSS framework via Vite plugin

## Key Dependencies

**Critical:**

*Frontend (package.json):*
- `@tauri-apps/api` ^2 - Tauri IPC bridge for invoking Rust commands
- `@xterm/xterm` ^6.0.0 - Terminal emulator widget (xterm.js)
- `@xterm/addon-fit` ^0.11.0 - Terminal auto-resize
- `@xterm/addon-search` ^0.16.0 - Terminal search functionality
- `@xterm/addon-web-links` ^0.12.0 - Clickable URLs in terminal
- `zustand` ^5.0.14 - Lightweight state management
- `react-router-dom` ^7.16.0 - Client-side routing
- `react-split-pane` ^3.2.0 - Resizable split panels
- `recharts` ^3.8.1 - Charting library for monitor data
- `lucide-react` ^1.17.0 - Icon library
- `uuid` ^14.0.0 - UUID generation

*Rust (Cargo.toml):*
- `tauri` 2.x - Core framework
- `ssh2` 0.9 - SSH/SFTP protocol implementation
- `rusqlite` 0.31 (bundled) - SQLite database driver
- `tokio` 1.x (full) - Async runtime
- `serde` / `serde_json` 1.x - Serialization
- `aes` 0.8 + `cbc` 0.1 - AES-256-CBC encryption
- `sha2` 0.10 - SHA-256 hashing for key derivation
- `base64` 0.22 - Base64 encoding for encrypted data
- `chrono` 0.4 - Date/time handling
- `uuid` 1.x (v4) - UUID generation
- `parking_lot` 0.12 - Fast mutex implementation
- `thiserror` 1.x - Error type derivation
- `log` 0.4 + `env_logger` 0.11 - Logging

**Infrastructure:**
- `@tailwindcss/vite` ^4.3.0 - Tailwind CSS Vite integration
- `@vitejs/plugin-react` ^4.6.0 - React Fast Refresh for Vite
- `tauri-plugin-opener` 2.x - File/URL opening capability

## Configuration

**Environment:**
- `.env` files: None detected
- Tauri config: `src-tauri/tauri.conf.json` - App window, security, bundle settings
- CSP: Disabled (`"csp": null`)

**Build:**
- `vite.config.ts` - Dev server port 1420, HMR on port 1421
- `tsconfig.json` - ES2020 target, bundler module resolution, strict mode
- `src-tauri/tauri.conf.json` - App identifier `com.wayserver.myterm-app`, window 1280x800

**Encryption:**
- Master password: Derived from hostname (`myterm-{hostname}`)
- Key derivation: SHA-256 with static salt `myterm-app-v1-salt`
- Algorithm: AES-256-CBC with PKCS7 padding
- Implementation: `src-tauri/src/crypto.rs`

## Platform Requirements

**Development:**
- Node.js 22.x
- Rust stable toolchain
- Platform-specific: macOS (screencapture), Linux (xfreerdp/rdesktop), Windows (mstsc)

**Production:**
- Targets: Windows (MSI/NSIS), macOS (DMG), Linux
- CI/CD: GitHub Actions (`build.yml`)
- Bundling: Tauri bundler with all targets enabled

---

*Stack analysis: 2026-06-02*

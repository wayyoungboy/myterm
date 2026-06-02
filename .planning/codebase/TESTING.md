# Testing Patterns

**Analysis Date:** 2026-06-02

## Test Framework

**Runner:**
- No test framework configured
- No test runner (Jest, Vitest, cargo test) configured in `package.json` or `Cargo.toml`
- No test scripts in `package.json`

**Assertion Library:**
- None configured

**Run Commands:**
```bash
# No test commands available
# npm test - not configured
# cargo test - no tests written
```

## Test File Organization

**Location:**
- No test files exist in the codebase
- No `*.test.ts`, `*.spec.ts`, `*_test.rs`, or `tests/` directories found
- No test configuration files (`jest.config.*`, `vitest.config.*`)

**Naming:**
- Not applicable - no tests exist

**Structure:**
```
# No test structure exists
```

## Test Structure

**Suite Organization:**
- No test suites exist

**Patterns:**
- No testing patterns established
- No test utilities or helpers

## Mocking

**Framework:** None

**Patterns:**
- No mocking patterns established

**What to Mock:**
- Not applicable

**What NOT to Mock:**
- Not applicable

## Fixtures and Factories

**Test Data:**
- No test data patterns established

**Location:**
- No fixture directories

## Coverage

**Requirements:** None enforced

**View Coverage:**
```bash
# No coverage tooling configured
```

## Test Types

**Unit Tests:**
- Not implemented
- No unit tests for utility functions (`src/utils/tauri.ts`, `src-tauri/src/crypto.rs`)

**Integration Tests:**
- Not implemented
- No integration tests for Tauri commands (`src-tauri/src/commands/`)

**E2E Tests:**
- Not implemented
- No E2E test framework (Playwright, Cypress) configured

## Recommended Testing Approach

**Frontend (TypeScript/React):**

Add Vitest as the test runner (compatible with Vite):
```bash
npm install -D vitest @testing-library/react @testing-library/jest-dom
```

Test file naming convention: `*.test.tsx` co-located with source files

Example structure:
```
src/
├── stores/
│   ├── appStore.ts
│   └── appStore.test.ts
├── utils/
│   ├── tauri.ts
│   └── tauri.test.ts
├── components/
│   ├── terminal/
│   │   ├── TerminalView.tsx
│   │   └── TerminalView.test.tsx
```

Key areas to test:
- `src/stores/appStore.ts` - Store state transitions (addTab, removeTab, setActiveTab)
- `src/utils/tauri.ts` - API wrapper functions (mock invoke)
- `src/components/common/ErrorBoundary.tsx` - Error boundary behavior
- Component rendering and user interactions

**Backend (Rust):**

Use built-in `cargo test`:
```bash
cd src-tauri && cargo test
```

Test file naming: `#[cfg(test)] mod tests` blocks within source files, or separate `tests/` directory

Key areas to test:
- `src-tauri/src/crypto.rs` - encrypt_password/decrypt_password roundtrip
- `src-tauri/src/monitor/mod.rs` - parse_monitor_output with sample data
- `src-tauri/src/db/schema.rs` - Database initialization
- `src-tauri/src/ssh/connection.rs` - Connection parameter validation

**Integration Tests:**

Consider Tauri's testing utilities for command testing:
- Mock database state
- Test command error handling
- Verify Tauri event emission

## CI Pipeline

**Current CI (`/.github/workflows/build.yml`):**
- Build-only pipeline (no test steps)
- Windows and macOS builds
- No test execution in CI
- Uses `npm ci` and `tauri-apps/tauri-action@v0`

**Recommended CI Addition:**
```yaml
- name: Run frontend tests
  run: npm test

- name: Run Rust tests
  run: cd src-tauri && cargo test
```

---

*Testing analysis: 2026-06-02*

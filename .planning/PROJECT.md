# MyTerm

## What This Is

MyTerm 是一个跨平台 SSH 终端管理器，基于 Tauri 2 + React 19 构建。对标商业产品 XTerminal，提供 SSH 连接管理、SFTP 文件传输、远程监控、笔记、AI 助手等功能。当前版本 0.1.0，PRD 合规度 96%。

## Core Value

SSH 终端会话和 SFTP 文件传输必须稳定可靠 — 这是终端管理器的核心功能，其他一切皆依赖于此。

## Requirements

### Validated

<!-- 从现有代码推断 — 已实现并运行的功能 -->

- ✓ SSH 连接管理（密码/密钥/交互式认证）— existing
- ✓ xterm.js 终端渲染 — existing
- ✓ 数据库持久化（连接配置、分组、笔记）— existing
- ✓ 远程服务器监控（CPU/内存/网络/磁盘/GPU）— existing
- ✓ 笔记管理（Markdown 编辑器）— existing
- ✓ AI 助手对话 — existing
- ✓ 快捷命令管理 — existing
- ✓ Telnet 连接 — existing
- ✓ RDP 连接（外部客户端）— existing
- ✓ 连接导入导出 — existing
- ✓ 端口转发（本地/SOCKS5）— existing
- ✓ 设置管理 — existing

### Active

<!-- 当前修复范围 — 解决审查发现的关键问题 -->

- [ ] SFTP 功能修复 — blocking 模式回归导致 SFTP 完全不可用
- [ ] SSH session 竞态条件修复 — `set_blocking` 在终端/监控/SFTP 间的数据竞争
- [ ] TerminalView 事件监听器泄漏修复 — `listen()` unlistener 在重连时被覆盖
- [ ] TelnetView 事件监听器泄漏修复 — `onData` handler 从未追踪和清理

### Out of Scope

- 密码加密安全改进 — hostname 派生密钥问题暂不处理，后续专项修复
- MainLayout 路由修复 — 所有 tab 渲染 TerminalView 问题，后续功能迭代处理
- 本地终端 stdin 修复 — 后续功能迭代处理
- 端口转发线程生命周期 — 后续功能迭代处理
- 测试覆盖添加 — 后续质量提升专项处理

## Context

**技术环境：**
- Tauri 2.x 桌面应用，React 19 前端 + Rust 后端
- SSH 通过 `ssh2` crate 实现，SFTP 共享同一 session
- SQLite 数据库存储连接配置、笔记等
- xterm.js 渲染终端，Zustand 管理前端状态

**已知问题（来自代码审查）：**
- `src-tauri/src/commands/sftp.rs` — 近期删除了 `with_blocking_session` 辅助函数，SFTP 操作在 non-blocking session 上必然失败
- `src-tauri/src/monitor/mod.rs` — `fetch_monitor_data` 切换 `set_blocking` 与终端读取线程竞争
- `src/components/terminal/TerminalView.tsx:155-171` — `setupTerminalIO` 中 `listen()` unlistener 被覆盖不清理
- `src/components/terminal/TelnetView.tsx:76` — `term.onData()` disposable 从未存储

**关键架构决策：**
- SFTP/监控使用独立 SSH session，避免与终端 session 的 blocking 模式冲突

## Constraints

- **SSH session 策略**: SFTP 和监控各创建独立 SSH 连接，不共享终端 session — 避免 blocking 模式竞态
- **修改范围**: 仅修复三个核心问题，不扩展到其他 bug — 保持变更可控
- **验证标准**: 功能正常 + cargo clippy 通过 + tsc 通过 — 确保修复质量

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| SFTP/监控使用独立 session | 简单可靠，避免 blocking 模式竞态 | — Pending |
| 暂不修复加密安全 | 优先功能正确性，安全问题后续专项处理 | — Pending |
| 不添加测试覆盖 | 先修复功能，测试作为后续质量提升 | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-06-02 after initialization*

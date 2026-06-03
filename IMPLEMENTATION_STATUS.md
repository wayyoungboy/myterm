# MyTerm SSH 核心实现状态

更新时间：2026-06-04

## 当前范围

本轮只以 SSH 对接为完成目标：

- SSH 连接管理
- SSH 终端
- SFTP 文件管理
- SSH 会话监控

以下功能不纳入当前完成口径：笔记、AI、端口转发、Telnet、快捷命令、云同步、RDP。

## 已完成

### SSH 连接与终端

- 连接中心作为默认工作台。
- 侧边栏与连接中心均可打开 SSH 终端 tab。
- tab 类型按 `terminal` / `sftp` / `monitor` 路由到真实视图。
- 终端 tab 切换不再触发 SSH 断开；关闭 tab 才释放 session。
- SSH smoke test 已验证：

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

### SFTP

- SFTP tab 会基于连接自动建立 SSH session。
- 远程 SFTP 复用后端 `TerminalManager` 中的 SSH session。
- 本地文件面板补齐 Tauri 命令：
  - `list_local_dir`
  - `write_local_file`
  - `remove_local_file`
  - `rename_local_file`
  - `create_local_dir`
- 前端统一通过 `src/utils/tauri.ts` 调用本地文件 IPC。

### 服务器监控

- Monitor tab 会基于连接自动建立 SSH session。
- 修复监控脚本 section 解析，支持多行 section。
- Rust 模型补齐 `top_cpu_processes` 和 `top_mem_processes`。
- 新增 Rust 单元测试覆盖监控输出解析。

## 验证结果

最近一次完整验证：

- `npm run build`：通过。
- `cargo check`：通过，保留既有 23 个未用代码 warning。
- `cargo test`：通过，2 个测试。
- SSH smoke test：通过，测试服务器返回 `MYTERM_SSH_OK`。

## 已知遗留

- Rust warning 主要来自当前不纳入范围或尚未接入的旧模块，如 RDP、Telnet、quick commands、local terminal、ProxyJump TODO。
- SFTP 批量操作、远程书签、权限编辑不是本轮目标。
- ProxyJump/HTTP/SOCKS 出站代理字段已有雏形，但完整链路尚未实现。
- 未进行图形界面人工验收；已完成构建、Rust 测试和远端 SSH 命令级验证。


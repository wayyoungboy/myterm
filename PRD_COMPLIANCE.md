# PRD 对照：SSH 核心收敛版

更新时间：2026-06-04

## 当前产品口径

MyTerm 当前按 SSH 管理器推进，不按全量远程运维套件推进。本轮用户明确排除：

- 笔记
- AI
- 端口转发
- Telnet
- 快捷命令
- 云同步
- RDP

因此，本文件只记录 SSH 连接、终端、SFTP、监控的完成度。

## SSH 连接管理

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| 连接 CRUD | ✅ | SQLite + Tauri commands + 连接中心 UI |
| 密码认证 | ✅ | `ssh2::Session::userauth_password` |
| 密钥认证 | ✅ | `userauth_pubkey_file` |
| 连接测试 | ✅ | `test_connection` |
| 延迟检测 | ✅ | TCP ping |
| 分组字段 | ✅ | DB 和命令已支持 |
| ProxyJump | ⚠️ | 字段存在，完整链路未实现 |
| HTTP/SOCKS 出站代理 | ⚠️ | 字段/UI 雏形存在，完整链路未实现 |

## SSH 终端

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| xterm.js 渲染 | ✅ | `TerminalView.tsx` |
| SSH shell channel | ✅ | `commands/terminal.rs` |
| 多 tab | ✅ | `TabBar` + Zustand store |
| tab 切换保活 | ✅ | 组件 unmount 不再断开 SSH |
| tab 关闭释放 session | ✅ | `TabBar` 调用 `disconnect_terminal` |
| Resize | ✅ | xterm fit + `terminal_resize` |
| 初始化命令/路径 | ✅ | 后端连接后写入 shell |

## SFTP

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| 远程目录列表 | ✅ | `sftp_list_dir` |
| 远程读写文件 | ✅ | `sftp_read_file` / `sftp_write_file` |
| 远程删除/重命名/建目录 | ✅ | `sftp_remove_file` / `sftp_rename` / `sftp_mkdir` |
| SFTP tab 自动建 session | ✅ | `SftpView` 调用 `connectTerminal` |
| 本地目录列表 | ✅ | `list_local_dir` |
| 本地写入/删除/重命名/建目录 | ✅ | `local_fs.rs` |
| 双栏 UI | ✅ | 本地/远程面板 |
| 批量操作 | ⚠️ | 非本轮目标 |
| 权限编辑 | ⚠️ | 非本轮目标 |

## 服务器监控

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| Monitor tab 自动建 session | ✅ | `MonitorView` 调用 `connectTerminal` |
| 系统信息 | ✅ | hostname / OS / uptime / load avg |
| CPU | ✅ | `/proc/stat` |
| 内存 | ✅ | `/proc/meminfo` |
| 网络 | ✅ | `/proc/net/dev` |
| 磁盘 | ✅ | `df -B1` |
| GPU | ✅ | `nvidia-smi` 可用时采集 |
| Top 进程 | ✅ | `ps` 按 CPU/MEM 排序 |
| 解析测试 | ✅ | Rust unit tests 覆盖 section 和 process parsing |

## 验证

已执行：

```bash
npm run build
cd src-tauri && cargo check
cd src-tauri && cargo test
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

结果：

- TypeScript/Vite build 通过。
- Rust check 通过，仍有既有 warning。
- Rust tests 通过，2 个测试。
- 真实 SSH 测试服务器连接通过。


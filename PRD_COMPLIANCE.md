# PRD 对照：MyTerm 当前完成度

更新时间：2026-06-04

## 当前产品口径

MyTerm 当前的完成标准是 SSH-first：SSH 连接、终端、SFTP、服务器监控必须可用；其他工具作为桌面壳能力逐步恢复。XTerminal 主要作为监控和交互结构参考，不直接引入参考项目源码。

## SSH 连接管理

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| 连接 CRUD | ✅ | SQLite + Tauri commands + 连接中心 UI |
| 分组字段 | ✅ | DB、命令、表单选择已支持 |
| 密码认证 | ✅ | `ssh2::Session::userauth_password` |
| 密钥认证 | ✅ | `userauth_pubkey_file` |
| ssh-agent / 默认密钥 | ✅ | 密钥路径留空时尝试 agent 和默认私钥 |
| 交互式认证入口 | ✅ | 表单入口已恢复 |
| 连接测试 | ✅ | `test_connection` |
| 延迟检测 | ✅ | TCP ping |
| 硬件信息采集 | ✅ | 连接中心可采集 OS/CPU/内存/磁盘 |
| 导入导出 | ✅ | JSON 导入导出命令已接入 |
| ProxyJump | ⚠️ | 字段存在；当前明确报错，完整 SSH direct-tcpip 多跳链路未实现 |
| HTTP/SOCKS 出站代理 | ✅ | HTTP CONNECT 和 SOCKS5 已接入连接测试、终端、SFTP/Monitor 辅助 SSH 会话 |

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
| 快捷命令写入当前终端 | ✅ | 支持变量展开后写入 SSH session |

## SFTP

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| 远程目录列表 | ✅ | `sftp_list_dir` |
| 远程读写文件 | ✅ | `sftp_read_file` / `sftp_write_file` |
| 远程删除/重命名/建目录 | ✅ | `sftp_remove_file` / `sftp_rename` / `sftp_mkdir` |
| SFTP tab 自动建 session | ✅ | `SftpView` 调用 `connectTerminal` |
| SFTP 独立 SSH 会话 | ✅ | 文件操作使用独立 SSH 连接，避免影响终端 reader |
| 本地目录列表 | ✅ | `list_local_dir` |
| 本地写入/删除/重命名/建目录 | ✅ | `local_fs.rs` |
| 双栏 UI | ✅ | 本地/远程面板 |
| 上传/下载 | ✅ | 基础文件上传和下载已接入 |
| 批量操作 | ⚠️ | 增强项 |
| 权限编辑 | ⚠️ | 增强项 |
| 拖拽上传 | ⚠️ | 增强项 |

## 服务器监控

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| Monitor tab 自动建 session | ✅ | `MonitorView` 调用 `connectTerminal` |
| Monitor 独立 SSH 会话 | ✅ | 监控采集使用独立 SSH 连接，不切换终端 session blocking mode |
| 系统信息 | ✅ | hostname / OS / uptime / load avg |
| CPU 总使用率 | ✅ | `/proc/stat` 前后快照 |
| 每核心 CPU | ✅ | UI 显示 per-core 使用率 |
| 内存 | ✅ | `/proc/meminfo`，缓存口径参考 XTerminal |
| 网络累计流量 | ✅ | `/proc/net/dev` |
| 网络实时速率 | ✅ | 前后快照计算 |
| 网卡名称 | ✅ | `net_interfaces` |
| 磁盘分区 | ✅ | `df` + `/proc/mounts` |
| 文件系统类型 | ✅ | `fs_type` |
| 磁盘读写速率 | ✅ | `/proc/diskstats` 前后快照 |
| GPU | ✅ | `nvidia-smi` 可用时采集 |
| Top 进程 | ✅ | `ps` 按 CPU/MEM 排序 |
| 解析测试 | ✅ | Rust unit tests 覆盖关键解析路径 |

## 附加工具

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| 笔记 | ✅ | CRUD、连接过滤、自动保存 |
| 快捷命令 | ✅ | CRUD、变量展开、一键执行到当前 SSH 终端 |
| 设置 | ✅ | 本地设置读写 |
| AI 助手 | ⚠️ | 会话和消息可保存，真实模型 API 未接入 |
| 端口转发 | ⚠️ | local/dynamic 基础命令存在，关闭会停止监听循环；remote 和流量统计未完成 |
| Telnet | ⚠️ | 基础 TCP/Telnet I/O 存在，未完整验收 |
| RDP | ⚠️ | 外部客户端 launcher 存在，未完整验收 |
| 云同步 | ❌ | 未实现 |

## 日志和追溯

| 需求项 | 状态 | 说明 |
| --- | --- | --- |
| 文件日志 | ✅ | `myterm.log` 写入 Tauri app data 目录 |
| 操作关联 ID | ✅ | SSH/SFTP/Monitor 等关键操作带 `op_id` |
| 生命周期日志 | ✅ | 启动、连接 CRUD、SSH connect/disconnect、SFTP、Monitor、Port Forward |
| 敏感信息保护 | ✅ | 不记录密码、私钥内容、终端输入字节 |
| 可调日志等级 | ✅ | `MYTERM_LOG=debug/trace` |

## 验证

已执行或本轮应持续执行：

```bash
npm run build
cd src-tauri && cargo check
cd src-tauri && cargo test
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

最近结果：

- TypeScript/Vite build 通过。
- Rust check 通过且无 warning。
- 监控解析单元测试通过，当前覆盖 3 个测试。
- 真实 SSH 测试服务器连接通过。

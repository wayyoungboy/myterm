# MyTerm 实现状态

更新时间：2026-06-04

## 当前产品口径

MyTerm 当前按 SSH-first 桌面管理器推进，优先保证 SSH 连接、终端、SFTP 和监控体验完整可用。笔记、AI、端口转发、Telnet、快捷命令、云同步和 RDP 不纳入当前验收范围。参考 XTerminal 的部分能力已经吸收到监控采集和工具入口中，但 XTerminal 目录仅作为本地参考材料，不应作为产品源码提交。

## 已完成主线

### SSH 连接与终端

- 连接中心作为默认工作台，支持连接 CRUD、分组字段、搜索、延迟检测和连接测试。
- 侧边栏与连接中心均可打开 SSH 终端、SFTP 和监控 tab。
- tab 类型按 `terminal` / `sftp` / `monitor` 路由到真实视图。
- 终端 tab 切换不再触发 SSH 断开；关闭 tab 才释放 session。
- 终端 reader 收到 EOF 或读错误时会清理后端 session，并同步清空前端 tab 的 `sessionId`，避免重新附着到已断开的会话。
- 终端断线后状态栏显示显式 `Reconnect`，由用户触发新 SSH session，避免隐式重连循环。
- 终端输入、resize、连接和断开通过统一 Tauri 封装调用。
- SSH 认证支持密码、指定私钥路径、ssh-agent，以及默认私钥 `~/.ssh/id_ed25519` / `~/.ssh/id_rsa` 等。
- HTTP CONNECT 和 SOCKS5 出站代理已接入共享 SSH 连接层，连接测试、终端、SFTP 和监控辅助 SSH 会话走同一传输逻辑。
- ProxyJump 会递归解析跳板连接，在 Unix-like 平台通过 SSH `direct-tcpip` channel 和本地 UnixStream 桥接建立目标 SSH 会话，并检测跳板循环。
- SSH keepalive 已按连接的 `heartbeat_ms` 配置到 libssh2 session，后端将心跳间隔限制在 1 秒到 10 分钟并写入连接日志。
- SSH smoke test 已验证：

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

### SFTP

- SFTP tab 会基于连接自动建立 SSH session。
- 远程 SFTP 操作会根据 terminal session 找回连接配置，并为文件操作建立独立 SSH session，避免切换终端 session 的 blocking mode。
- 双栏文件管理已接入本地和远程文件操作。
- 本地文件命令已补齐：
  - `list_local_dir`
  - `read_local_file`
  - `write_local_file`
  - `remove_local_file`
  - `rename_local_file`
  - `create_local_dir`
- 远程命令覆盖列表、读取、写入、删除、重命名、建目录。
- 远程文件/目录权限编辑已接入 `chmod`，支持 3/4 位八进制模式并记录到 SFTP 操作日志。
- 远程文本文件在线编辑已接入，限制为 1 MB 内 UTF-8 文本文件，读取时拒绝明显二进制内容。
- SFTP 本地/远程面板支持拖拽文件上传和多文件选择上传，目录拖拽暂不展开。
- SFTP 面板支持当前目录多选/全选、批量删除、远程选中文件下载到本地面板、本地选中文件上传到远程面板。
- 远程删除已支持目录递归删除；选中文件夹时支持递归上传/下载到对侧当前目录。
- 本地/远程面板传输使用后端流式复制，传输遮罩显示当前文件字节级进度并支持取消；取消状态覆盖整个批量传输生命周期，会中止后续复制并保留已写入的部分文件。

### 服务器监控

- Monitor tab 会基于连接自动建立 SSH session。
- 监控采集使用独立 SSH session，不再复用终端 reader 的 SSH session。
- 监控采集参考 XTerminal 的快照思路，远端状态存放在 `~/.myterm`。
- CPU 使用率基于 `/proc/stat` 前后快照计算。
- 网络速率基于 `/proc/net/dev` 前后快照计算，并显示网卡名称。
- 磁盘分区显示文件系统类型、使用量和读写速率。
- 内存缓存口径包含 `Buffers`、`Cached`、`SReclaimable`。
- Top CPU/MEM 进程列表已解析并展示。
- Rust 单元测试覆盖 section 解析、进程解析和 XTerminal-style 监控速率/磁盘类型解析。

### 工具视图

- 侧边栏已恢复 Notes、AI、Forward、Telnet、Commands、Settings 入口，但除 Settings 和连接导入导出外不纳入当前 SSH-first 验收。
- Notes 支持本地 CRUD、按连接过滤和自动保存。
- Quick Commands 支持 CRUD、一键写入当前 SSH 终端，并展开 `${host}`、`${port}`、`${username}`、`${date}`、`${time}`。
- Settings 支持本地设置读写。
- AI 当前保留本地会话和消息存储，回复为占位提示；真实模型 API 不纳入当前验收。
- Port forwarding、Telnet、RDP launcher 有后端命令和 UI，但不纳入当前 SSH-first 验收。
- Port forwarding 的 local/dynamic 监听已接入停止标志，关闭转发时会停止监听循环，并使用独立 SSH session。

### 运行日志

- 应用启动时初始化 `myterm.log`，位于 Tauri app data 目录。
- 日志覆盖启动、连接配置变更、SSH 连接、断开、终端写入/resize 元数据、SFTP 操作、本地文件操作、监控采集和端口转发生命周期。
- 关键操作包含 `op_id`、`session_id` / `connection_id`、目标 host/port、耗时和错误原因，便于追溯。
- 终端写入日志只记录字节数，不记录用户输入内容；成功写入和 resize 默认为 `debug` 级别，错误使用 `error` / `warn`。
- 默认日志等级为 `info`，可通过 `MYTERM_LOG=debug` 或 `MYTERM_LOG=trace` 提升。
- 不记录密码、私钥内容或终端输入字节。

### 桌面安全配置

- Tauri capabilities 当前保持 `core:default` 和 `opener:default`，未开放额外插件权限。
- Tauri CSP 已显式启用，默认限制为 `self` / `asset`，`connect-src` 仅保留 Tauri IPC 所需的 `ipc:` 和 `http://ipc.localhost`。
- Rust 单元测试覆盖 `tauri.conf.json` 中 CSP 必须启用并包含 IPC connect-src，防止后续回退到 `csp: null`。

### 打包发布

- `npm run tauri -- build` 已验证 release binary 和 macOS `.app` 能成功生成。
- 当前自动化环境中标准 DMG bundler 会受 Finder AppleScript 超时和临时 `rw.*.dmg` 残留影响；已通过清理临时文件并以 `--skip-jenkins` 运行生成的 `bundle_dmg.sh` 成功产出 macOS arm64 DMG。
- 已验证 DMG 镜像信息，产物路径为 `src-tauri/target/release/bundle/dmg/myterm-app_0.1.0_aarch64.dmg`。
- Windows MSI / Linux DEB 尚未在对应平台验证。

## 验证结果

最近一次验证：

- `npm run build`：通过。
- `cd src-tauri && cargo check`：通过，当前无 Rust warning。
- `cd src-tauri && cargo test`：通过，21 个测试。
- SSH smoke test：通过，测试服务器返回 `MYTERM_SSH_OK`。

## 已知遗留

- HTTP CONNECT / SOCKS5 出站代理已接入；代理认证为后续增强。
- ProxyJump 已接入 Unix-like 平台；Windows 跳板桥接还需要单独实现和验证。
- Keepalive 已配置到 SSH session；仍建议进行数小时级空闲连接 soak test。
- SFTP/Monitor 当前为每次操作建立独立 SSH 连接，稳定性优先；后续可做连接池优化以减少握手开销。
- 不纳入当前 SSH-first 验收：笔记、AI、端口转发、Telnet、快捷命令、云同步、RDP。
- 未进行完整人工 UI 回归；已完成构建、Rust 检查、监控单测和远端 SSH 命令级验证。

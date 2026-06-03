# MyTerm 开发计划

> 目标：打造一款专业的 SSH-first 桌面终端管理器，部分交互和监控能力参考 XTerminal。
> 技术栈：Tauri 2 (Rust) + React 19 + TypeScript + Tailwind CSS

## 当前开发口径（2026-06-04）

MyTerm 当前优先完成 SSH 对接主线：

- SSH 连接管理
- SSH 终端
- SFTP 文件管理
- SSH 会话上的服务器监控

当前验收范围不包含笔记、AI、端口转发、Telnet、快捷命令、云同步和 RDP；这些入口即使保留在界面中，也不作为 SSH-first 主线完成度的阻塞项。

验证服务器：

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -p 17244 wayserver@103.112.184.13 'echo MYTERM_SSH_OK && uname -a && pwd'
```

---

## 第一阶段：SSH 核心功能稳定 ✅

**目标：连接稳定、终端可用**

| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 1.1 | SSH 连接建立 | ✅ | TCP + SSH 握手 + 认证 |
| 1.2 | 终端 I/O | ✅ | xterm.js + PTY |
| 1.3 | 会话保持 | ✅ | tab 切换不再断开 SSH |
| 1.4 | 非阻塞读取 | ✅ | reader 线程 EOF 检测 |
| 1.4.1 | 断线会话清理 | ✅ | EOF/read error 会清理后端 session 并清空前端 tab sessionId |
| 1.4.2 | 显式重连 | ✅ | 断线后终端状态栏显示 Reconnect，避免隐式重连循环 |
| 1.5 | 域名 DNS 解析 | ✅ | ToSocketAddrs 替换 parse |
| 1.6 | 多标签独立实例 | ✅ | 每个 tab 独立 session |
| 1.7 | 连接状态显示 | ✅ | StatusBar 事件驱动 |
| 1.8 | 终端窗口大小调整 | ✅ | ResizeObserver + PTY resize |
| 1.9 | Keepalive 心跳 | ✅ | libssh2 keepalive 已按连接心跳间隔配置，参数范围 1s-10min，仍建议做长时间 soak test |

---

## 第二阶段：连接中心与主导航 ✅

**目标：让 SSH 工作流成为默认入口，同时保留完整工具区**

| 任务 | 状态 | 说明 |
|------|------|------|
| 连接中心表格视图 | ✅ | 名称、地址、延迟、硬件信息、操作列 |
| 延迟检测 | ✅ | TCP ping |
| 硬件信息标签 | ✅ | OS/核数/内存/硬盘 |
| 收藏功能 | ✅ | UI 已接入 |
| 连接表单多标签 | ✅ | 基本信息/连接设置/代理/其他 |
| 分组选择 | ✅ | `getGroups` 接入表单 |
| 交互式认证入口 | ✅ | 表单入口已恢复 |
| ssh-agent / 默认密钥认证 | ✅ | 密钥路径留空时尝试 agent 和默认私钥 |
| SSH/SFTP/Monitor tab 路由 | ✅ | 按 tab type 渲染真实视图 |
| SFTP/Monitor session 隔离 | ✅ | 文件和监控操作使用独立 SSH 连接，避免影响终端 reader |
| 侧边栏工具入口 | ✅ | Notes/AI/Forward/Telnet/Commands/Settings |
| 批量操作 | 增强项 | 后续增强，不阻塞当前 SSH-first 主线 |
| HTTP/SOCKS5 出站代理 | ✅ | 连接测试、终端、SFTP/Monitor 辅助 SSH 会话共享同一代理链路 |
| ProxyJump 多跳链路 | ✅ | Unix-like 平台通过 SSH direct-tcpip 桥接，支持递归解析和循环检测 |

---

## 第三阶段：监控面板 ✅

**目标：参考 XTerminal 监控方式，提供专业级服务器监控**

| 模块 | 状态 | 说明 |
|------|------|------|
| 系统信息 | ✅ | 主机名、OS、内核、运行时间、负载 |
| CPU 总使用率 | ✅ | `/proc/stat` 快照差值 |
| 每核心 CPU | ✅ | 独立显示 |
| 负载曲线 | ✅ | UI 历史曲线 |
| Top CPU 进程 | ✅ | `ps` 解析 |
| 内存/缓存/Swap | ✅ | 缓存口径参考 XTerminal |
| Top 内存进程 | ✅ | `ps` 解析 |
| 网络速率 | ✅ | `/proc/net/dev` 快照差值 |
| 网卡名称 | ✅ | `net_interfaces` |
| 磁盘使用率 | ✅ | 分区进度条 |
| 文件系统类型 | ✅ | `/proc/mounts` |
| 磁盘读写速率 | ✅ | `/proc/diskstats` 快照差值 |
| GPU | ✅ | `nvidia-smi` 可用时采集 |
| 监控解析测试 | ✅ | Rust 单测覆盖 3 个关键解析路径 |
| BusyBox/procfs 多后端 | 增强项 | 可作为兼容性增强 |
| JSON 数据格式 | 增强项 | 当前仍是 section 文本解析 |

---

## 第四阶段：SFTP 文件管理 ✅

**目标：图形化双栏文件浏览器**

| 任务 | 状态 | 说明 |
|------|------|------|
| 双栏布局 | ✅ | 本地/远程 |
| 文件列表 | ✅ | 名称、大小、权限、修改时间 |
| 目录导航 | ✅ | 路径输入和目录进入 |
| 上传/下载 | ✅ | 基础文件传输 |
| 删除/重命名/建目录 | ✅ | 本地和远程均接入 |
| 本地文件 Tauri 命令 | ✅ | list/read/write/remove/rename/mkdir |
| 远程权限管理 chmod | ✅ | 远程文件/目录支持 3/4 位八进制权限编辑 |
| 远程文件在线编辑 | ✅ | 支持 1 MB 内 UTF-8 文本文件在线编辑和保存 |
| 拖拽上传 | ✅ | 本地/远程面板支持拖拽文件和多文件选择上传 |
| 批量操作 | ✅ | 当前目录多选/全选，支持批量删除、远程文件下载到本地面板、本地文件上传到远程面板 |
| 目录递归上传/下载 | ✅ | 选中文件夹时递归复制到对侧当前目录 |

---

## 第五阶段：辅助工具（当前不纳入 SSH-first 验收）

以下条目保留为历史状态和后续增强清单，不作为当前 SSH 对接主线的验收阻塞项。

### 5.1 快捷命令 ✅

- [x] 命令库 CRUD
- [x] 删除确认
- [x] 动态变量：`${host}`、`${port}`、`${username}`、`${date}`、`${time}`
- [x] 一键写入当前 SSH 终端
- [ ] 快捷键全局绑定
- [ ] 命令分组管理 UI

### 5.2 笔记 ✅

- [x] 本地笔记 CRUD
- [x] 按连接过滤
- [x] 自动保存
- [x] 删除确认
- [ ] Markdown 预览
- [ ] 按分组关联 UI

### 5.3 设置 ✅

- [x] 终端设置持久化
- [x] AI 设置字段持久化
- [x] 安全/通用设置字段持久化
- [ ] 设置项实际联动到所有运行中组件

### 5.4 端口转发

- [x] 本地转发基础命令
- [x] 动态 SOCKS5 基础命令
- [x] 转发列表 UI
- [x] 关闭转发时停止监听循环
- [x] 转发使用独立 SSH session
- [ ] 远程转发
- [ ] 流量统计
- [ ] 真实环境回归测试

### 5.5 Telnet / RDP

- [x] Telnet 基础连接、读写、断开
- [x] RDP 外部客户端 launcher
- [ ] Telnet 选项协商完善
- [ ] RDP 跨平台体验验收

### 5.6 AI / 云同步

- [x] AI 会话和消息本地存储
- [ ] 接入真实模型 API
- [ ] SSH 上下文注入和命令解释
- [ ] 云同步

---

## 第六阶段：稳定性和发布

- [x] 应用级文件日志（`myterm.log`）
- [x] 关键操作追溯 ID（SSH/SFTP/Monitor 等）
- [x] 本地文件操作日志（list/read/write/remove/rename/mkdir）
- [x] 连接断线后显式重连入口
- [ ] 会话录制与回放
- [x] HTTP CONNECT / SOCKS5 出站代理
- [x] 多跳代理连接（ProxyJump 链）
- [ ] 凭证管理（统一存储、复用）
- [ ] SFTP 大文件字节级进度和取消
- [ ] 前端组件拆分（TerminalView/SftpView 继续瘦身）
- [ ] 单元测试覆盖扩大到命令层和关键 UI 逻辑
- [ ] Browser/人工 UI 回归清单
- [x] macOS arm64 DMG 打包验证
- [ ] Windows MSI / Linux DEB 打包验证

---

## 技术债务

- [x] 清理 Rust warning
- [x] 统一终端写入/resize 的 Tauri 前端封装
- [x] SFTP/Monitor 从共享 SSH session 改为独立 SSH session
- [ ] CSS 变量和间距规范继续收敛
- [x] Tauri 权限配置最小化复核（capabilities 保持 core/opener；CSP 已启用并覆盖 IPC）
- [x] 端口转发线程生命周期重构
- [ ] 监控数据格式从 section 文本迁移到 JSON

# MyTerm 开发计划

> 目标：打造一款专业的 SSH-first 桌面终端管理器，部分交互和监控能力参考 XTerminal。
> 技术栈：Tauri 2 (Rust) + React 19 + TypeScript + Tailwind CSS

## 当前开发口径（2026-06-04）

MyTerm 当前优先完成 SSH 对接主线：

- SSH 连接管理
- SSH 终端
- SFTP 文件管理
- SSH 会话上的服务器监控

辅助工具入口已恢复，按成熟度分层：

- 已可作为本地工具使用：笔记、快捷命令、设置、连接导入导出。
- 可用但需继续验收：端口转发、Telnet、RDP launcher。
- 仅占位：AI 助手真实模型调用、云同步。

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
| 1.5 | 域名 DNS 解析 | ✅ | ToSocketAddrs 替换 parse |
| 1.6 | 多标签独立实例 | ✅ | 每个 tab 独立 session |
| 1.7 | 连接状态显示 | ✅ | StatusBar 事件驱动 |
| 1.8 | 终端窗口大小调整 | ✅ | ResizeObserver + PTY resize |
| 1.9 | Keepalive 心跳 | ⚠️ | 字段存在，仍需长时间连接验证 |

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
| 批量操作 | ⚠️ | 后续增强 |
| ProxyJump / 出站代理链路 | ⚠️ | 字段和 UI 存在，后端链路未完成 |

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
| BusyBox/procfs 多后端 | ⚠️ | 可作为兼容性增强 |
| JSON 数据格式 | ⚠️ | 当前仍是 section 文本解析 |

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
| 本地文件 Tauri 命令 | ✅ | list/write/remove/rename/mkdir |
| 拖拽上传 | ⚠️ | 后续增强 |
| 批量操作 | ⚠️ | 后续增强 |
| 权限管理 chmod | ⚠️ | 后续增强 |
| 远程文件在线编辑 | ⚠️ | 后续增强 |

---

## 第五阶段：辅助工具

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

### 5.4 端口转发 ⚠️

- [x] 本地转发基础命令
- [x] 动态 SOCKS5 基础命令
- [x] 转发列表 UI
- [x] 关闭转发时停止监听循环
- [x] 转发使用独立 SSH session
- [ ] 远程转发
- [ ] 流量统计
- [ ] 真实环境回归测试

### 5.5 Telnet / RDP ⚠️

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
- [ ] 连接断线自动重连
- [ ] 会话录制与回放
- [ ] 多跳代理连接（ProxyJump 链）
- [ ] 凭证管理（统一存储、复用）
- [ ] SFTP 大文件传输进度和取消
- [ ] 前端组件拆分（TerminalView/SftpView 继续瘦身）
- [ ] 单元测试覆盖扩大到命令层和关键 UI 逻辑
- [ ] Browser/人工 UI 回归清单
- [ ] 打包发布（DMG/MSI/DEB）

---

## 技术债务

- [x] 清理 Rust warning
- [x] 统一终端写入/resize 的 Tauri 前端封装
- [x] SFTP/Monitor 从共享 SSH session 改为独立 SSH session
- [ ] CSS 变量和间距规范继续收敛
- [ ] Tauri 权限配置最小化复核
- [x] 端口转发线程生命周期重构
- [ ] 监控数据格式从 section 文本迁移到 JSON

// Types matching the Rust backend models

export interface Group {
  id: string;
  name: string;
  parent_id: string | null;
  icon: string | null;
  sort_order: number;
  created_at: string | null;
}

export interface Connection {
  id: string;
  group_id: string | null;
  name: string;
  host: string;
  port: number;
  auth_type: string;
  username: string | null;
  password_enc: string | null;
  key_path: string | null;
  credential_id: string | null;
  proxy_type: string | null;
  proxy_host: string | null;
  proxy_port: number | null;
  proxy_jump_id: string | null;
  init_command: string | null;
  init_path: string | null;
  timeout_ms: number | null;
  heartbeat_ms: number | null;
  remark: string | null;
  created_at: string | null;
  updated_at: string | null;
}

export interface ConnectionInput {
  id?: string;
  group_id?: string;
  name: string;
  host: string;
  port?: number;
  auth_type?: string;
  username?: string;
  password?: string;
  key_path?: string;
  credential_id?: string;
  proxy_type?: string;
  proxy_host?: string;
  proxy_port?: number;
  proxy_jump_id?: string;
  init_command?: string;
  init_path?: string;
  timeout_ms?: number;
  heartbeat_ms?: number;
  remark?: string;
}

export interface SftpEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  permissions: string;
  modified: string;
}

export interface QuickCommand {
  id: string;
  group_id: string | null;
  name: string;
  command: string;
  shortcut: string | null;
  sort_order: number;
}

export interface Note {
  id: string;
  connection_id: string | null;
  group_id: string | null;
  title: string | null;
  content: string | null;
  created_at: string | null;
  updated_at: string | null;
}

export interface AiConversation {
  id: string;
  title: string | null;
  created_at: string | null;
}

export interface AiMessage {
  id: string;
  conversation_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  created_at: string | null;
}

export interface MonitorData {
  cpu_usage: number;
  cpu_cores: number[];
  mem_total: number;
  mem_used: number;
  mem_cached: number;
  swap_total: number;
  swap_used: number;
  net_rx: number;
  net_tx: number;
  net_rx_rate: number;
  net_tx_rate: number;
  net_interfaces: string[];
  disk_partitions: DiskPartition[];
  gpu: GpuData | null;
  uptime: number;
  hostname: string;
  os_info: string;
  load_avg: [number, number, number];
  top_cpu_processes: ProcessInfo[];
  top_mem_processes: ProcessInfo[];
}

export interface ProcessInfo {
  pid: number;
  cpu: number;
  memory: number;
  command: string;
  args: string;
}

export interface DiskPartition {
  mount: string;
  fs_type: string;
  total: number;
  used: number;
  read_rate: number;
  write_rate: number;
}

export interface GpuData {
  name: string;
  utilization: number;
  temperature: number;
  memory_total: number;
  memory_used: number;
  power: number;
}

export interface Tab {
  id: string;
  title: string;
  connectionId: string;
  sessionId: string | null;
  type: 'terminal' | 'sftp' | 'monitor';
}

export type ViewMode = 'terminal' | 'sftp' | 'monitor' | 'notes' | 'ai' | 'settings' | 'portforward' | 'telnet' | 'quickcommands';

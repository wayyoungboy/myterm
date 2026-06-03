use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    pub id: String,
    pub group_id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub auth_type: String,
    pub username: Option<String>,
    pub password_enc: Option<String>,
    pub key_path: Option<String>,
    pub credential_id: Option<String>,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i32>,
    pub proxy_jump_id: Option<String>,
    pub init_command: Option<String>,
    pub init_path: Option<String>,
    pub timeout_ms: Option<i32>,
    pub heartbeat_ms: Option<i32>,
    pub remark: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionInput {
    pub id: Option<String>,
    pub group_id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: Option<i32>,
    pub auth_type: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub credential_id: Option<String>,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i32>,
    pub proxy_jump_id: Option<String>,
    pub init_command: Option<String>,
    pub init_path: Option<String>,
    pub timeout_ms: Option<i32>,
    pub heartbeat_ms: Option<i32>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickCommand {
    pub id: String,
    pub group_id: Option<String>,
    pub name: String,
    pub command: String,
    pub shortcut: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    pub connection_id: Option<String>,
    pub group_id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConversation {
    pub id: String,
    pub title: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SftpEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
    pub modified: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorData {
    pub cpu_usage: f64,
    pub cpu_cores: Vec<f64>,
    pub mem_total: u64,
    pub mem_used: u64,
    pub mem_cached: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub net_rx: u64,
    pub net_tx: u64,
    pub net_rx_rate: u64,
    pub net_tx_rate: u64,
    pub net_interfaces: Vec<String>,
    pub disk_partitions: Vec<DiskPartition>,
    pub gpu: Option<GpuData>,
    pub uptime: u64,
    pub hostname: String,
    pub os_info: String,
    pub load_avg: (f64, f64, f64),
    pub top_cpu_processes: Vec<ProcessInfo>,
    pub top_mem_processes: Vec<ProcessInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu: f64,
    pub memory: f64,
    pub command: String,
    pub args: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiskPartition {
    pub mount: String,
    pub fs_type: String,
    pub total: u64,
    pub used: u64,
    pub read_rate: u64,
    pub write_rate: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuData {
    pub name: String,
    pub utilization: f64,
    pub temperature: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub power: f64,
}

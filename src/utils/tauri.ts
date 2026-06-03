import { invoke } from '@tauri-apps/api/core';
import type { Connection, ConnectionInput, Group, SftpEntry, MonitorData, Note, AiConversation, AiMessage } from '../types';

// Groups
export const getGroups = () => invoke<Group[]>('get_groups');
export const createGroup = (name: string, parentId?: string, icon?: string) =>
  invoke<Group>('create_group', { name, parentId, icon });
export const updateGroup = (id: string, name: string, icon?: string) =>
  invoke('update_group', { id, name, icon });
export const deleteGroup = (id: string) => invoke('delete_group', { id });

// Connections
export const getConnections = () => invoke<Connection[]>('get_connections');
export const createConnection = (input: ConnectionInput) =>
  invoke<Connection>('create_connection', { input });
export const updateConnection = (id: string, input: ConnectionInput) =>
  invoke('update_connection', { id, input });
export const deleteConnection = (id: string) => invoke('delete_connection', { id });
export const testConnection = (input: ConnectionInput) =>
  invoke<string>('test_connection', { input });

export interface ServerInfo {
  os: string;
  cpu_cores: number;
  memory_total: number;
  disk_total: number;
}
export const collectServerInfo = (input: ConnectionInput) =>
  invoke<ServerInfo>('collect_server_info', { input });
export const searchConnections = (query: string) =>
  invoke<Connection[]>('search_connections', { query });

// Terminal
export const connectTerminal = (connectionId: string) =>
  invoke<string>('connect_terminal', { connectionId });
export const terminalWrite = (sessionId: string, data: string) =>
  invoke('terminal_write', { sessionId, data });
export const terminalResize = (sessionId: string, cols: number, rows: number) =>
  invoke('terminal_resize', { sessionId, cols, rows });
export const disconnectTerminal = (sessionId: string) =>
  invoke('disconnect_terminal', { sessionId });

// SFTP
export const sftpListDir = (sessionId: string, path: string) =>
  invoke<SftpEntry[]>('sftp_list_dir', { sessionId, path });
export const sftpReadFile = (sessionId: string, path: string) =>
  invoke<number[]>('sftp_read_file', { sessionId, path });
export const sftpWriteFile = (sessionId: string, path: string, data: number[]) =>
  invoke('sftp_write_file', { sessionId, path, data });
export const sftpBeginTransfer = (transferId: string) =>
  invoke('sftp_begin_transfer', { transferId });
export const sftpDownloadPath = (transferId: string, sessionId: string, remotePath: string, localParent: string) =>
  invoke<number>('sftp_download_path', { transferId, sessionId, remotePath, localParent });
export const sftpUploadPath = (transferId: string, sessionId: string, localPath: string, remoteParent: string) =>
  invoke<number>('sftp_upload_path', { transferId, sessionId, localPath, remoteParent });
export const sftpFinishTransfer = (transferId: string) =>
  invoke('sftp_finish_transfer', { transferId });
export const sftpCancelTransfer = (transferId: string) =>
  invoke('sftp_cancel_transfer', { transferId });
export const sftpRemoveFile = (sessionId: string, path: string) =>
  invoke('sftp_remove_file', { sessionId, path });
export const sftpRename = (sessionId: string, src: string, dst: string) =>
  invoke('sftp_rename', { sessionId, src, dst });
export const sftpMkdir = (sessionId: string, path: string) =>
  invoke('sftp_mkdir', { sessionId, path });
export const sftpChmod = (sessionId: string, path: string, mode: string) =>
  invoke('sftp_chmod', { sessionId, path, mode });

// Local filesystem for SFTP two-pane view
export const listLocalDir = (path: string) =>
  invoke<SftpEntry[]>('list_local_dir', { path });
export const readLocalFile = (path: string) =>
  invoke<number[]>('read_local_file', { path });
export const writeLocalFile = (path: string, data: number[]) =>
  invoke('write_local_file', { path, data });
export const removeLocalFile = (path: string) =>
  invoke('remove_local_file', { path });
export const renameLocalFile = (src: string, dst: string) =>
  invoke('rename_local_file', { src, dst });
export const createLocalDir = (path: string) =>
  invoke('create_local_dir', { path });

// Monitor
export const getMonitorData = (sessionId: string) =>
  invoke<MonitorData>('get_monitor_data', { sessionId });

// Notes
export const getNotes = (connectionId?: string) =>
  invoke<Note[]>('get_notes', { connectionId });
export const createNote = (title: string, content: string, connectionId?: string, groupId?: string) =>
  invoke<Note>('create_note', { title, content, connectionId, groupId });
export const updateNote = (id: string, title: string, content: string) =>
  invoke('update_note', { id, title, content });
export const deleteNote = (id: string) => invoke('delete_note', { id });

// AI
export const getAiConversations = () => invoke<AiConversation[]>('get_ai_conversations');
export const createAiConversation = (title?: string) =>
  invoke<AiConversation>('create_ai_conversation', { title });
export const deleteAiConversation = (id: string) =>
  invoke('delete_ai_conversation', { id });
export const getAiMessages = (conversationId: string) =>
  invoke<AiMessage[]>('get_ai_messages', { conversationId });
export const saveAiMessage = (conversationId: string, role: string, content: string) =>
  invoke<AiMessage>('save_ai_message', { conversationId, role, content });

// Settings
export const getSettings = () => invoke<Record<string, string>>('get_settings');
export const setSetting = (key: string, value: string) =>
  invoke('set_setting', { key, value });
export const getSetting = (key: string) =>
  invoke<string | null>('get_setting', { key });

// Port Forwarding
export interface PortForward {
  id: string;
  session_id: string;
  forward_type: string;
  local_host: string;
  local_port: number;
  remote_host: string;
  remote_port: number;
  active: boolean;
}

export const createPortForward = (
  sessionId: string, forwardType: string,
  localHost: string, localPort: number,
  remoteHost: string, remotePort: number
) => invoke<string>('create_port_forward', {
  sessionId, forwardType, localHost, localPort, remoteHost, remotePort
});
export const getPortForwards = () => invoke<PortForward[]>('get_port_forwards');
export const closePortForward = (id: string) => invoke('close_port_forward', { id });

// Ping
export interface PingResult {
  host: string;
  port: number;
  latency_ms: number;
  success: boolean;
  error: string | null;
}
export const pingHost = (host: string, port?: number) =>
  invoke<PingResult>('ping_host', { host, port });

// RDP
export const connectRdp = (
  host: string, port?: number, username?: string, password?: string,
  domain?: string, width?: number, height?: number
) => invoke<string>('connect_rdp', { host, port, username, password, domain, width, height });

// Telnet
export const connectTelnet = (host: string, port?: number) =>
  invoke<string>('connect_telnet', { host, port });
export const telnetWrite = (sessionId: string, data: string) =>
  invoke('telnet_write', { sessionId, data });
export const disconnectTelnet = (sessionId: string) =>
  invoke('disconnect_telnet', { sessionId });

// Quick Commands
export interface QuickCommand {
  id: string;
  group_id: string | null;
  name: string;
  command: string;
  shortcut: string | null;
  sort_order: number;
}

export const getQuickCommands = (groupId?: string) =>
  invoke<QuickCommand[]>('get_quick_commands', { groupId });
export const createQuickCommand = (name: string, command: string, groupId?: string, shortcut?: string) =>
  invoke<QuickCommand>('create_quick_command', { name, command, groupId, shortcut });
export const updateQuickCommand = (id: string, name: string, command: string, shortcut?: string) =>
  invoke('update_quick_command', { id, name, command, shortcut });
export const deleteQuickCommand = (id: string) =>
  invoke('delete_quick_command', { id });

// Import/Export
export const exportConnections = () => invoke<string>('export_connections');
export const importConnections = (json: string) => invoke<number>('import_connections', { json });

// Local Terminal
export const openLocalTerminal = (shell?: string) =>
  invoke<string>('open_local_terminal', { shell });
export const localTerminalWrite = (sessionId: string, data: string) =>
  invoke('local_terminal_write', { sessionId, data });
export const closeLocalTerminal = (sessionId: string) =>
  invoke('close_local_terminal', { sessionId });

// Screenshot (dev tool)
export const takeScreenshot = () => invoke<string>('take_screenshot');

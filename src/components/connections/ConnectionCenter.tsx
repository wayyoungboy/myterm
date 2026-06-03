import { useState, useEffect } from 'react';
import { useAppStore } from '../../stores/appStore';
import { getConnections, deleteConnection, pingHost, collectServerInfo, type ServerInfo } from '../../utils/tauri';
import type { Connection } from '../../types';
import { Search, Plus, Trash2, Edit, Terminal, Star, Wifi, FolderOpen, Activity } from 'lucide-react';
import { ConnectionForm } from './ConnectionForm';

// ── Helpers ──────────────────────────────────────────────────────────

function formatLatency(ms: number | null): string {
  if (ms === null) return '—';
  if (ms < 1) return '<1ms';
  return `${Math.round(ms)}ms`;
}

function latencyColor(ms: number | null): string {
  if (ms === null) return 'var(--text-muted)';
  if (ms < 50) return 'var(--success)';
  if (ms < 150) return 'var(--warning)';
  return 'var(--error)';
}

function osIcon(os: string): string {
  const lower = os.toLowerCase();
  if (lower.includes('ubuntu') || lower.includes('debian')) return '🟠';
  if (lower.includes('centos') || lower.includes('rhel') || lower.includes('red hat')) return '🔴';
  if (lower.includes('alpine')) return '🏔️';
  if (lower.includes('darwin') || lower.includes('macos')) return '🍎';
  if (lower.includes('windows')) return '🪟';
  return '🐧';
}

// ── Connection Row ───────────────────────────────────────────────────

interface ConnectionRowProps {
  conn: Connection;
  latency: number | null;
  serverInfo: ServerInfo | null;
  onConnect: (conn: Connection) => void;
  onSftp: (conn: Connection) => void;
  onMonitor: (conn: Connection) => void;
  onEdit: (conn: Connection) => void;
  onDelete: (conn: Connection) => void;
}

function ConnectionRow({ conn, latency, serverInfo, onConnect, onSftp, onMonitor, onEdit, onDelete }: ConnectionRowProps) {
  const [starred, setStarred] = useState(false);
  const [deleteClicked, setDeleteClicked] = useState(false);

  // Format memory/disk for tags
  const formatMem = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${Math.round(gb)}G 内存`;
  };
  const formatDisk = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${Math.round(gb)}G 硬盘`;
  };

  return (
    <div
      className="flex items-center gap-3 px-4 py-2.5 border-b border-[var(--border)] hover:bg-[var(--bg-hover)] transition-colors group"
      style={{ background: deleteClicked ? 'rgba(239,68,68,0.1)' : undefined }}
    >
      {/* Star */}
      <button
        className="shrink-0 transition-colors"
        style={{ color: starred ? 'var(--warning)' : 'var(--text-muted)' }}
        onClick={() => setStarred(!starred)}
      >
        <Star size={14} fill={starred ? 'var(--warning)' : 'none'} />
      </button>

      {/* OS Icon */}
      <span className="text-lg shrink-0">{osIcon(conn.host)}</span>

      {/* Latency */}
      <div className="flex items-center gap-1 shrink-0 w-16">
        <Wifi size={10} style={{ color: latencyColor(latency) }} />
        <span className="text-[11px] font-mono" style={{ color: latencyColor(latency) }}>
          {formatLatency(latency)}
        </span>
      </div>

      {/* Name */}
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium truncate" style={{ color: 'var(--text-primary)' }}>
          {conn.name}
        </div>
      </div>

      {/* Address */}
      <div className="flex items-center gap-1.5 shrink-0">
        <span className="text-xs font-mono" style={{ color: 'var(--text-secondary)' }}>
          {conn.username || 'root'}@{conn.host}:{conn.port}
        </span>
      </div>

      {/* Info tags */}
      <div className="flex gap-1.5 shrink-0">
        {serverInfo ? (
          <>
            <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
              {serverInfo.os.split(' ')[0] || 'Linux'}
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
              {serverInfo.cpu_cores} 核
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
              {formatMem(serverInfo.memory_total)}
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
              {formatDisk(serverInfo.disk_total)}
            </span>
          </>
        ) : (
          <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
            SSH
          </span>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-1 shrink-0">
        <button
          className="px-3 py-1 rounded-md text-xs font-medium transition-colors flex items-center gap-1"
          style={{ background: 'var(--success)', color: '#fff' }}
          onClick={() => onConnect(conn)}
          title="打开 SSH 终端"
        >
          <Terminal size={12} />
          终端
        </button>
        <button
          className="p-1 rounded transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: 'var(--text-muted)' }}
          onClick={() => onSftp(conn)}
          title="打开 SFTP"
        >
          <FolderOpen size={13} />
        </button>
        <button
          className="p-1 rounded transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: 'var(--text-muted)' }}
          onClick={() => onMonitor(conn)}
          title="打开监控"
        >
          <Activity size={13} />
        </button>
        <button
          className="p-1 rounded transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: 'var(--text-muted)' }}
          onClick={() => onEdit(conn)}
          title="编辑"
        >
          <Edit size={13} />
        </button>
        <button
          className="p-1 rounded transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: 'var(--text-muted)' }}
          onClick={(e) => { e.stopPropagation(); setDeleteClicked(true); setTimeout(() => setDeleteClicked(false), 2000); onDelete(conn); }}
          title="删除"
        >
          <Trash2 size={13} />
        </button>
      </div>
    </div>
  );
}

// ── Main Component ───────────────────────────────────────────────────

export function ConnectionCenter() {
  const { connections, setConnections, addTab, setSelectedConnectionId, setView } = useAppStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [editConn, setEditConn] = useState<Connection | null>(null);
  const [latencies, setLatencies] = useState<Record<string, number | null>>({});
  const [serverInfos, setServerInfos] = useState<Record<string, ServerInfo | null>>({});

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const c = await getConnections();
      setConnections(c);
    } catch (e) {
      console.error('Failed to load connections:', e);
    }
  };

  // Ping all connections in background
  useEffect(() => {
    if (connections.length === 0) return;
    connections.forEach(async (conn) => {
      try {
        const result = await pingHost(conn.host, conn.port);
        setLatencies((prev) => ({ ...prev, [conn.id]: result.latency_ms }));

        // If latency is good, try to collect server info
        if (result.success) {
          try {
            const info = await collectServerInfo({
              name: conn.name,
              host: conn.host,
              port: conn.port,
              auth_type: conn.auth_type,
              username: conn.username || undefined,
              password: undefined, // Can't decrypt here, so skip info collection
              key_path: conn.key_path || undefined,
            });
            setServerInfos((prev) => ({ ...prev, [conn.id]: info }));
          } catch {
            // Info collection failed, that's ok
          }
        }
      } catch {
        setLatencies((prev) => ({ ...prev, [conn.id]: null }));
      }
    });
  }, [connections]);

  const filtered = searchQuery
    ? connections.filter(
        (c) =>
          c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          c.host.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : connections;

  const openConnectionTab = (conn: Connection, type: 'terminal' | 'sftp' | 'monitor') => {
    const tabId = `tab-${type}-${Date.now()}`;
    const suffix = type === 'terminal' ? '' : type === 'sftp' ? ' / SFTP' : ' / Monitor';
    addTab({
      id: tabId,
      title: `${conn.name}${suffix}`,
      connectionId: conn.id,
      sessionId: null,
      type,
    });
    setSelectedConnectionId(conn.id);
    setView(type);
  };

  const handleConnect = (conn: Connection) => openConnectionTab(conn, 'terminal');

  const handleEdit = (conn: Connection) => {
    setEditConn(conn);
    setShowForm(true);
  };

  const handleDelete = async (conn: Connection) => {
    console.log('[Delete] Attempting to delete:', conn.id, conn.name);
    try {
      await deleteConnection(conn.id);
      console.log('[Delete] Success:', conn.id);
      // Remove from local state immediately
      setConnections(connections.filter(c => c.id !== conn.id));
      console.log('[Delete] State updated');
    } catch (e) {
      console.error('[Delete] Failed:', e);
    }
  };

  return (
    <div className="flex flex-col h-full" style={{ background: 'var(--bg-primary)' }}>
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3 border-b border-[var(--border)]">
        <div className="flex items-center gap-3">
          <h2 className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
            连接中心
          </h2>
          <span className="text-xs px-2 py-0.5 rounded-full" style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
            {connections.length} 个连接
          </span>
        </div>
        <button
          className="btn btn-primary text-xs px-3 py-1.5"
          onClick={() => { setEditConn(null); setShowForm(true); }}
        >
          <Plus size={12} /> 新建连接
        </button>
      </div>

      {/* Search + Filters */}
      <div className="flex items-center gap-3 px-5 py-2 border-b border-[var(--border)]">
        <div className="relative flex-1 max-w-md">
          <Search size={13} className="absolute left-2.5 top-1/2 -translate-y-1/2" style={{ color: 'var(--text-muted)' }} />
          <input
            type="text"
            placeholder="搜索连接..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="input pl-8 py-1.5 text-xs"
          />
        </div>
      </div>

      {/* Table Header */}
      <div className="flex items-center gap-3 px-4 py-2 text-[10px] uppercase tracking-wider border-b border-[var(--border)]" style={{ color: 'var(--text-muted)' }}>
        <span className="w-6" />
        <span className="w-7" />
        <span className="w-16">延迟</span>
        <span className="flex-1">名称</span>
        <span className="shrink-0">地址</span>
        <span className="shrink-0 w-12">标签</span>
        <span className="shrink-0 w-32 text-right">操作</span>
      </div>

      {/* Connection List */}
      <div className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 gap-3">
            <div className="w-12 h-12 rounded-xl flex items-center justify-center" style={{ background: 'var(--bg-surface)' }}>
              <Terminal size={20} style={{ color: 'var(--text-muted)' }} />
            </div>
            <span className="text-xs" style={{ color: 'var(--text-muted)' }}>
              {searchQuery ? '没有匹配的连接' : '还没有连接'}
            </span>
            {!searchQuery && (
              <button className="btn btn-primary text-xs" onClick={() => { setEditConn(null); setShowForm(true); }}>
                <Plus size={12} /> 创建第一个连接
              </button>
            )}
          </div>
        ) : (
          filtered.map((conn) => (
            <ConnectionRow
              key={conn.id}
              conn={conn}
              latency={latencies[conn.id] ?? null}
              serverInfo={serverInfos[conn.id] ?? null}
              onConnect={handleConnect}
              onSftp={(conn) => openConnectionTab(conn, 'sftp')}
              onMonitor={(conn) => openConnectionTab(conn, 'monitor')}
              onEdit={handleEdit}
              onDelete={handleDelete}
            />
          ))
        )}
      </div>

      {/* Connection Form Modal */}
      {showForm && (
        <ConnectionForm
          connectionId={editConn?.id}
          initialData={editConn ? { name: editConn.name, host: editConn.host, port: editConn.port, username: editConn.username || undefined, auth_type: editConn.auth_type, key_path: editConn.key_path || undefined } : undefined}
          onClose={() => { setShowForm(false); setEditConn(null); }}
          onSaved={() => { setShowForm(false); setEditConn(null); loadData(); }}
        />
      )}
    </div>
  );
}

import { useState, useEffect } from 'react';
import { useAppStore } from '../../stores/appStore';
import { getConnections, deleteConnection } from '../../utils/tauri';
import type { Connection } from '../../types';
import { Search, Plus, Terminal, Trash2, Server, Activity, ChevronRight, FolderOpen, Settings } from 'lucide-react';
import { ConnectionForm } from '../connections/ConnectionForm';

export function Sidebar() {
  const {
    sidebarCollapsed, connections, setConnections,
    addTab, setSelectedConnectionId, setView,
    selectedConnectionId, setSelectedConnectionId: setSelConn,
  } = useAppStore();

  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; item: Connection | null } | null>(null);
  const [showNewConnection, setShowNewConnection] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

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

  const filtered = searchQuery
    ? connections.filter(c =>
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

  if (sidebarCollapsed) {
    return (
      <div className="w-12 flex flex-col items-center py-3 gap-3 border-r border-[var(--border)]" style={{ background: 'var(--bg-secondary)' }}>
        <button
          className="w-8 h-8 rounded-lg flex items-center justify-center transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: 'var(--accent)' }}
          onClick={() => setShowNewConnection(true)}
          title="New Connection"
        >
          <Plus size={16} />
        </button>
        <div className="w-6 h-px" style={{ background: 'var(--border)' }} />
        {connections.slice(0, 8).map(conn => (
          <button
            key={conn.id}
            className="w-8 h-8 rounded-lg flex items-center justify-center transition-colors hover:bg-[var(--bg-surface)]"
            style={{ color: selectedConnectionId === conn.id ? 'var(--accent)' : 'var(--text-muted)' }}
            onClick={() => handleConnect(conn)}
            title={conn.name}
          >
            <Server size={14} />
          </button>
        ))}
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full border-r border-[var(--border)]" style={{ width: '260px', background: 'var(--bg-secondary)' }}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border)]">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded-md flex items-center justify-center" style={{ background: 'var(--accent)', color: '#fff' }}>
            <Terminal size={12} />
          </div>
          <span className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>MyTerm</span>
        </div>
        <button
          className="w-7 h-7 rounded-md flex items-center justify-center transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: 'var(--text-muted)' }}
          onClick={() => setShowNewConnection(true)}
          title="New Connection"
        >
          <Plus size={14} />
        </button>
      </div>

      {/* Search */}
      <div className="px-3 py-2">
        <div className="relative">
          <Search size={13} className="absolute left-2.5 top-1/2 -translate-y-1/2" style={{ color: 'var(--text-muted)' }} />
          <input
            type="text"
            placeholder="Search..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="input pl-8 py-1.5 text-xs"
          />
        </div>
      </div>

      {/* Quick Actions */}
      <div className="flex gap-1 px-3 pb-2">
        <button
          className="flex-1 flex items-center justify-center gap-1 py-1.5 rounded-md text-[10px] transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: 'var(--text-muted)' }}
          onClick={() => setView('terminal')}
        >
          <Server size={11} /> Connections
        </button>
        <button
          className="flex-1 flex items-center justify-center gap-1 py-1.5 rounded-md text-[10px] transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: 'var(--text-muted)' }}
          onClick={() => setView('settings')}
        >
          <Settings size={11} /> Settings
        </button>
      </div>

      {/* Divider */}
      <div className="mx-3 mb-2 h-px" style={{ background: 'var(--border)' }} />

      {/* Connection List */}
      <div className="flex-1 overflow-y-auto px-2">
        {filtered.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 gap-3">
            <div className="w-10 h-10 rounded-xl flex items-center justify-center" style={{ background: 'var(--bg-surface)' }}>
              <Server size={18} style={{ color: 'var(--text-muted)' }} />
            </div>
            <span className="text-[11px]" style={{ color: 'var(--text-muted)' }}>
              {searchQuery ? 'No matches' : 'No connections'}
            </span>
            {!searchQuery && (
              <button
                className="btn btn-primary text-xs px-3 py-1.5"
                onClick={() => setShowNewConnection(true)}
              >
                <Plus size={12} /> Add Connection
              </button>
            )}
          </div>
        ) : (
          filtered.map(conn => (
            <div
              key={conn.id}
              className="group flex items-center gap-2.5 px-2.5 py-2 rounded-lg cursor-pointer transition-colors mb-0.5"
              style={{
                background: selectedConnectionId === conn.id ? 'var(--bg-surface)' : 'transparent',
              }}
              onClick={() => {
                setSelConn(conn.id);
              }}
              onDoubleClick={() => handleConnect(conn)}
              onContextMenu={(e) => {
                e.preventDefault();
                setContextMenu({ x: e.clientX, y: e.clientY, item: conn });
              }}
            >
              <div
                className="w-8 h-8 rounded-lg flex items-center justify-center shrink-0"
                style={{ background: 'var(--bg-surface)' }}
              >
                <Server size={14} style={{ color: 'var(--success)' }} />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-xs font-medium truncate" style={{ color: 'var(--text-primary)' }}>{conn.name}</div>
                <div className="text-[10px] truncate" style={{ color: 'var(--text-muted)' }}>{conn.username}@{conn.host}:{conn.port}</div>
              </div>
              <ChevronRight
                size={12}
                className="opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                style={{ color: 'var(--text-muted)' }}
              />
            </div>
          ))
        )}
      </div>

      {/* New Connection Form Modal */}
      {showNewConnection && (
        <ConnectionForm
          onClose={() => setShowNewConnection(false)}
          onSaved={() => {
            setShowNewConnection(false);
            loadData();
          }}
        />
      )}

      {/* Context menu */}
      {contextMenu && (
        <>
          <div className="fixed inset-0 z-[1999]" onClick={() => setContextMenu(null)} />
          <div
            className="context-menu"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            <div className="context-menu-item" onClick={() => {
              handleConnect(contextMenu.item!);
              setContextMenu(null);
            }}>
              <Terminal size={14} /> Open Terminal
            </div>
            <div className="context-menu-item" onClick={() => {
              openConnectionTab(contextMenu.item!, 'sftp');
              setContextMenu(null);
            }}>
              <FolderOpen size={14} /> Open SFTP
            </div>
            <div className="context-menu-item" onClick={() => {
              openConnectionTab(contextMenu.item!, 'monitor');
              setContextMenu(null);
            }}>
              <Activity size={14} /> Open Monitor
            </div>
            <div className="context-menu-divider" />
            <div className="context-menu-item text-[var(--error)]" onClick={async () => {
              const conn = contextMenu.item!;
              setContextMenu(null);
              try {
                await deleteConnection(conn.id);
                loadData();
              } catch (e) {
                console.error('Failed to delete:', e);
              }
            }}>
              <Trash2 size={14} /> Delete
            </div>
          </div>
        </>
      )}
    </div>
  );
}

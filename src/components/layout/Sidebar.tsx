import React, { useState, useEffect } from 'react';
import { useAppStore } from '../../stores/appStore';
import { getGroups, getConnections, deleteConnection, deleteGroup } from '../../utils/tauri';
import type { Connection, Group } from '../../types';
import {
  Search, Plus, Folder, FolderOpen, Terminal, Settings,
  FileText, MessageSquare, ChevronRight, ChevronDown,
  Plug, Network, Wifi, Trash2, Zap
} from 'lucide-react';
import { ConnectionForm } from '../connections/ConnectionForm';

interface TreeItem {
  type: 'group' | 'connection';
  data: Group | Connection;
  children: TreeItem[];
  expanded: boolean;
}

export function Sidebar() {
  const {
    sidebarCollapsed, groups, connections, searchQuery, setSearchQuery,
    setGroups, setConnections, addTab, setSelectedConnectionId, setView,
  } = useAppStore();

  const [tree, setTree] = useState<TreeItem[]>([]);
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; item: TreeItem | null } | null>(null);
  const [showNewConnection, setShowNewConnection] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  useEffect(() => {
    buildTree();
  }, [groups, connections, expandedGroups]);

  const loadData = async () => {
    try {
      const [g, c] = await Promise.all([getGroups(), getConnections()]);
      setGroups(g);
      setConnections(c);
    } catch (e) {
      console.error('Failed to load data:', e);
    }
  };

  const buildTree = () => {
    const groupMap = new Map<string, TreeItem>();
    const roots: TreeItem[] = [];

    // Create group nodes
    groups.forEach((g) => {
      groupMap.set(g.id, {
        type: 'group',
        data: g,
        children: [],
        expanded: expandedGroups.has(g.id),
      });
    });

    // Build hierarchy
    groups.forEach((g) => {
      const item = groupMap.get(g.id)!;
      if (g.parent_id && groupMap.has(g.parent_id)) {
        groupMap.get(g.parent_id)!.children.push(item);
      } else {
        roots.push(item);
      }
    });

    // Add connections
    connections.forEach((c) => {
      const item: TreeItem = { type: 'connection', data: c, children: [], expanded: false };
      if (c.group_id && groupMap.has(c.group_id)) {
        groupMap.get(c.group_id)!.children.push(item);
      } else {
        roots.push(item);
      }
    });

    setTree(roots);
  };

  const toggleGroup = (id: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleConnect = async (conn: Connection) => {
    const tabId = `tab-${Date.now()}`;
    addTab({
      id: tabId,
      title: conn.name,
      connectionId: conn.id,
      sessionId: null,
      type: 'terminal',
    });
    setSelectedConnectionId(conn.id);
    setView('terminal');
  };

  const handleContextMenu = (e: React.MouseEvent, item: TreeItem) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, item });
  };

  const filteredTree = searchQuery
    ? tree.filter((item) => filterItem(item, searchQuery.toLowerCase()))
    : tree;

  const filterItem = (item: TreeItem, query: string): boolean => {
    if (item.type === 'connection') {
      const conn = item.data as Connection;
      return conn.name.toLowerCase().includes(query) || conn.host.toLowerCase().includes(query);
    }
    const group = item.data as Group;
    if (group.name.toLowerCase().includes(query)) return true;
    return item.children.some((child) => filterItem(child, query));
  };

  const renderItem = (item: TreeItem, depth: number = 0) => {
    const isGroup = item.type === 'group';
    const data = item.data;

    return (
      <React.Fragment key={data.id}>
        <div
          className="flex items-center gap-1 px-2 py-1 cursor-pointer hover:bg-[var(--bg-surface)] rounded mx-1 group"
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
          onClick={() => {
            if (isGroup) toggleGroup(data.id);
            else handleConnect(data as Connection);
          }}
          onContextMenu={(e) => handleContextMenu(e, item)}
        >
          {isGroup ? (
            <>
              {item.expanded ? (
                <ChevronDown size={14} className="text-[var(--text-muted)] shrink-0" />
              ) : (
                <ChevronRight size={14} className="text-[var(--text-muted)] shrink-0" />
              )}
              {item.expanded ? (
                <FolderOpen size={14} className="text-[var(--accent)] shrink-0" />
              ) : (
                <Folder size={14} className="text-[var(--text-muted)] shrink-0" />
              )}
            </>
          ) : (
            <>
              <span className="w-3.5 shrink-0" />
              <Plug size={14} className="text-[var(--success)] shrink-0" />
            </>
          )}
          <span className="truncate text-xs">{data.name}</span>
          {!isGroup && (
            <span className="ml-auto text-[10px] text-[var(--text-muted)] opacity-0 group-hover:opacity-100">
              {(data as Connection).host}
            </span>
          )}
        </div>
        {isGroup && item.expanded && item.children.map((child) => renderItem(child, depth + 1))}
      </React.Fragment>
    );
  };

  if (sidebarCollapsed) {
    return (
      <div className="w-12 bg-[var(--bg-secondary)] border-r border-[var(--border)] flex flex-col items-center py-2 gap-2">
        <button className="btn-ghost p-2 rounded" onClick={() => setView('terminal')} title="Terminal">
          <Terminal size={18} />
        </button>
        <button className="btn-ghost p-2 rounded" onClick={() => setView('settings')} title="Settings">
          <Settings size={18} />
        </button>
      </div>
    );
  }

  return (
    <div className="w-[260px] bg-[var(--bg-secondary)] border-r border-[var(--border)] flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border)]">
        <span className="text-sm font-semibold text-[var(--text-primary)]">MyTerm</span>
        <button className="btn-ghost p-1 rounded" onClick={() => setShowNewConnection(true)} title="New Connection">
          <Plus size={16} />
        </button>
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

      {/* Search */}
      <div className="px-2 py-1.5">
        <div className="relative">
          <Search size={14} className="absolute left-2 top-1/2 -translate-y-1/2 text-[var(--text-muted)]" />
          <input
            type="text"
            placeholder="Search connections..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="input pl-7 py-1 text-xs"
          />
        </div>
      </div>

      {/* Quick actions */}
      <div className="flex flex-wrap gap-1 px-2 py-1 border-b border-[var(--border)]">
        <button className="btn-ghost p-1.5 rounded flex-1 flex items-center justify-center gap-1 text-[10px]" onClick={() => setView('terminal')}>
          <Terminal size={12} /> Terminal
        </button>
        <button className="btn-ghost p-1.5 rounded flex-1 flex items-center justify-center gap-1 text-[10px]" onClick={() => setView('notes')}>
          <FileText size={12} /> Notes
        </button>
        <button className="btn-ghost p-1.5 rounded flex-1 flex items-center justify-center gap-1 text-[10px]" onClick={() => setView('ai')}>
          <MessageSquare size={12} /> AI
        </button>
        <button className="btn-ghost p-1.5 rounded flex-1 flex items-center justify-center gap-1 text-[10px]" onClick={() => setView('quickcommands')}>
          <Zap size={12} /> Commands
        </button>
        <button className="btn-ghost p-1.5 rounded flex-1 flex items-center justify-center gap-1 text-[10px]" onClick={() => setView('portforward')}>
          <Network size={12} /> Forward
        </button>
        <button className="btn-ghost p-1.5 rounded flex-1 flex items-center justify-center gap-1 text-[10px]" onClick={() => setView('telnet')}>
          <Wifi size={12} /> Telnet
        </button>
      </div>

      {/* Tree */}
      <div className="flex-1 overflow-y-auto py-1">
        {filteredTree.map((item) => renderItem(item))}
        {filteredTree.length === 0 && (
          <div className="text-center text-xs text-[var(--text-muted)] py-8">
            {searchQuery ? 'No matches found' : 'No connections yet'}
          </div>
        )}
      </div>

      {/* Context menu */}
      {contextMenu && (
        <>
          <div className="fixed inset-0 z-[1999]" onClick={() => setContextMenu(null)} />
          <div
            className="context-menu"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            {contextMenu.item?.type === 'group' ? (
              <>
                <div className="context-menu-item" onClick={() => {
                  setContextMenu(null);
                  setShowNewConnection(true);
                }}>
                  <Plus size={14} /> New Connection
                </div>
                <div className="context-menu-divider" />
                <div className="context-menu-item text-[var(--error)]" onClick={async () => {
                  const group = contextMenu.item!.data as Group;
                  setContextMenu(null);
                  if (confirm(`Delete group "${group.name}"?`)) {
                    try {
                      await deleteGroup(group.id);
                      loadData();
                    } catch (e) {
                      console.error('Failed to delete group:', e);
                    }
                  }
                }}>
                  <Trash2 size={14} /> Delete Group
                </div>
              </>
            ) : (
              <>
                <div className="context-menu-item" onClick={() => { handleConnect(contextMenu.item!.data as Connection); setContextMenu(null); }}>
                  <Terminal size={14} /> Connect
                </div>
                <div className="context-menu-divider" />
                <div className="context-menu-item text-[var(--error)]" onClick={async () => {
                  const conn = contextMenu.item!.data as Connection;
                  setContextMenu(null);
                  if (confirm(`Delete connection "${conn.name}"?`)) {
                    try {
                      await deleteConnection(conn.id);
                      loadData();
                    } catch (e) {
                      console.error('Failed to delete connection:', e);
                    }
                  }
                }}>
                  <Trash2 size={14} /> Delete
                </div>
              </>
            )}
          </div>
        </>
      )}
    </div>
  );
}

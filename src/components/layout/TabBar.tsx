import { useAppStore } from '../../stores/appStore';
import { X, Terminal, FolderOpen, Activity } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

function TabIcon({ type, active }: { type: string; active: boolean }) {
  const color = active ? 'var(--accent)' : 'var(--text-muted)';
  const props = { size: 11, style: { color, flexShrink: 0 } };

  if (type === 'sftp') return <FolderOpen {...props} />;
  if (type === 'monitor') return <Activity {...props} />;
  return <Terminal {...props} />;
}

export function TabBar() {
  const { tabs, activeTabId, setActiveTab, removeTab } = useAppStore();

  if (tabs.length === 0) return null;

  const handleClose = async (tab: typeof tabs[0], e: React.MouseEvent) => {
    e.stopPropagation();
    if (tab.sessionId) {
      try {
        await invoke('disconnect_terminal', { sessionId: tab.sessionId });
      } catch { /* ignore */ }
    }
    removeTab(tab.id);
  };

  return (
    <div className="flex items-center h-[34px] border-b border-[var(--border)] overflow-x-auto" style={{ background: 'var(--bg-secondary)' }}>
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className="flex items-center gap-1.5 px-3 h-full cursor-pointer border-r border-[var(--border)] transition-colors min-w-0 group"
          style={{
            background: tab.id === activeTabId ? 'var(--bg-primary)' : 'transparent',
            borderBottom: tab.id === activeTabId ? '2px solid var(--accent)' : '2px solid transparent',
          }}
          onClick={() => setActiveTab(tab.id)}
        >
          <TabIcon type={tab.type} active={tab.id === activeTabId} />
          <span
            className="text-[11px] truncate max-w-[100px]"
            style={{ color: tab.id === activeTabId ? 'var(--text-primary)' : 'var(--text-secondary)' }}
          >
            {tab.title}
          </span>
          <button
            className="w-4 h-4 rounded flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all hover:bg-[var(--bg-surface)]"
            style={{ color: 'var(--text-muted)', flexShrink: 0 }}
            onClick={(e) => handleClose(tab, e)}
          >
            <X size={10} />
          </button>
        </div>
      ))}
    </div>
  );
}

import { useAppStore } from '../../stores/appStore';
import { X, Terminal, FolderTree, Monitor, FileText, MessageSquare, Settings } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

const viewIcons: Record<string, React.ReactNode> = {
  terminal: <Terminal size={14} />,
  sftp: <FolderTree size={14} />,
  monitor: <Monitor size={14} />,
  notes: <FileText size={14} />,
  ai: <MessageSquare size={14} />,
  settings: <Settings size={14} />,
};

export function TabBar() {
  const { tabs, activeTabId, setActiveTab, removeTab } = useAppStore();

  if (tabs.length === 0) return null;

  const handleClose = async (tab: typeof tabs[0]) => {
    // Disconnect the session if one exists
    if (tab.sessionId) {
      try {
        await invoke('disconnect_terminal', { sessionId: tab.sessionId });
      } catch {
        // Ignore errors on disconnect
      }
    }
    removeTab(tab.id);
  };

  return (
    <div className="tab-bar">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={`tab ${tab.id === activeTabId ? 'active' : ''}`}
          onClick={() => setActiveTab(tab.id)}
        >
          {viewIcons[tab.type] || <Terminal size={14} />}
          <span className="truncate max-w-[120px]">{tab.title}</span>
          <div
            className="tab-close"
            onClick={(e) => {
              e.stopPropagation();
              handleClose(tab);
            }}
          >
            <X size={12} />
          </div>
        </div>
      ))}
    </div>
  );
}

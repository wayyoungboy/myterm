import { useAppStore } from '../../stores/appStore';
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Activity } from 'lucide-react';

export function StatusBar() {
  const { tabs, activeTabId, toggleMonitorSidebar, monitorSidebarVisible } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);

  // Track which tabIds are connected
  const [connectedMap, setConnectedMap] = useState<Record<string, boolean>>({});

  // When active tab changes, also check store's sessionId as fallback
  const currentStatus = activeTabId
    ? (connectedMap[activeTabId] ?? activeTab?.sessionId != null)
    : false;

  // Listen for connect/disconnect events
  useEffect(() => {
    const unlisten1 = listen<string>('terminal-connected', (event) => {
      const tabId = event.payload;
      setConnectedMap((prev) => ({ ...prev, [tabId]: true }));
    });
    const unlisten2 = listen<string>('terminal-disconnected', (event) => {
      const tabId = event.payload;
      setConnectedMap((prev) => ({ ...prev, [tabId]: false }));
    });
    return () => {
      unlisten1.then((fn) => fn());
      unlisten2.then((fn) => fn());
    };
  }, []);

  return (
    <div
      className="flex items-center justify-between h-[22px] px-3 border-t border-[var(--border)]"
      style={{ background: 'var(--bg-secondary)' }}
    >
      <div className="flex items-center gap-3">
        {activeTab && (
          <>
            <span className="flex items-center gap-1">
              <span
                className="w-1.5 h-1.5 rounded-full"
                style={{ background: currentStatus ? 'var(--success)' : 'var(--error)' }}
              />
              <span
                className="text-[10px]"
                style={{ color: currentStatus ? 'var(--success)' : 'var(--error)' }}
              >
                {currentStatus ? 'Connected' : 'Disconnected'}
              </span>
            </span>
            <span className="text-[10px]" style={{ color: 'var(--text-muted)' }}>
              {activeTab.title}
            </span>
          </>
        )}
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={toggleMonitorSidebar}
          className="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors hover:bg-[var(--bg-surface)]"
          style={{ color: monitorSidebarVisible ? 'var(--accent)' : 'var(--text-muted)' }}
          title="Toggle monitor"
        >
          <Activity size={10} />
          <span className="text-[10px]">Monitor</span>
        </button>
        <span className="text-[10px]" style={{ color: 'var(--text-muted)' }}>v0.1.0</span>
      </div>
    </div>
  );
}

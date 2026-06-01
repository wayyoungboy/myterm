import { useAppStore } from '../../stores/appStore';

export function StatusBar() {
  const { tabs, activeTabId } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);
  const isConnected = activeTab?.sessionId != null;

  return (
    <div className="status-bar">
      <div className="flex items-center gap-3">
        {activeTab && (
          <>
            <span className="flex items-center gap-1">
              <span
                className="w-2 h-2 rounded-full"
                style={{ background: isConnected ? 'var(--success)' : 'var(--error)' }}
              />
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
            <span>{activeTab.title}</span>
          </>
        )}
      </div>
      <div className="flex items-center gap-3">
        <span>UTF-8</span>
        <span>LF</span>
        <span>v0.1.0</span>
      </div>
    </div>
  );
}

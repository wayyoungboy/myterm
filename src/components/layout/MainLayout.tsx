import { Sidebar } from './Sidebar';
import { TabBar } from './TabBar';
import { StatusBar } from './StatusBar';
import { MonitorSidebar } from '../monitor/MonitorSidebar';
import { ErrorBoundary } from '../common/ErrorBoundary';
import { useAppStore } from '../../stores/appStore';
import { TerminalView } from '../terminal/TerminalView';
import SftpView from '../files/SftpView';
import { MonitorView } from '../monitor/MonitorView';
import { SettingsView } from '../settings/SettingsView';
import { ConnectionCenter } from '../connections/ConnectionCenter';

export function MainLayout() {
  const { tabs, activeTabId, currentView } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);

  const renderContent = () => {
    // If there's an active tab, render based on tab type
    if (activeTab) {
      switch (activeTab.type) {
        case 'sftp':
          return <SftpView />;
        case 'monitor':
          return <MonitorView />;
        default:
          return <TerminalView connectionId={activeTab.connectionId || undefined} />;
      }
    }

    // No tabs - show connection center or settings
    switch (currentView) {
      case 'settings':
        return <SettingsView />;
      default:
        return <ConnectionCenter />;
    }
  };

  return (
    <div className="flex h-full">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0 relative">
        <TabBar />
        <div className="flex-1 overflow-hidden flex flex-col">
          <ErrorBoundary>
            {renderContent()}
          </ErrorBoundary>
        </div>
        <StatusBar />
      </div>
      <MonitorSidebar />
    </div>
  );
}

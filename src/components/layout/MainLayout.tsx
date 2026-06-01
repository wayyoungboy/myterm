import { Sidebar } from './Sidebar';
import { TabBar } from './TabBar';
import { StatusBar } from './StatusBar';
import { ErrorBoundary } from '../common/ErrorBoundary';
import { useAppStore } from '../../stores/appStore';
import { TerminalView } from '../terminal/TerminalView';
import SftpView from '../files/SftpView';
import { MonitorView } from '../monitor/MonitorView';
import { NotesView } from '../notes/NotesView';
import AiView from '../ai/AiView';
import { SettingsView } from '../settings/SettingsView';
import { PortForwardView } from '../portforward/PortForwardView';
import { TelnetView } from '../terminal/TelnetView';
import { QuickCommandsView } from '../quickcommands/QuickCommandsView';

export function MainLayout() {
  const { currentView, tabs, activeTabId } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);

  const renderContent = () => {
    if (activeTab) {
      switch (activeTab.type) {
        case 'sftp':
          return <SftpView sessionId={activeTab.sessionId || undefined} />;
        case 'monitor':
          return <MonitorView />;
        default:
          return <TerminalView connectionId={activeTab.connectionId || undefined} />;
      }
    }

    switch (currentView) {
      case 'terminal':
        return <TerminalView />;
      case 'sftp':
        return <SftpView />;
      case 'monitor':
        return <MonitorView />;
      case 'notes':
        return <NotesView />;
      case 'ai':
        return <AiView />;
      case 'settings':
        return <SettingsView />;
      case 'portforward':
        return <PortForwardView />;
      case 'telnet':
        return <TelnetView />;
      case 'quickcommands':
        return <QuickCommandsView />;
      default:
        return <TerminalView />;
    }
  };

  return (
    <div className="flex h-full">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <TabBar />
        <div className="flex-1 overflow-hidden">
          <ErrorBoundary>
            {renderContent()}
          </ErrorBoundary>
        </div>
        <StatusBar />
      </div>
    </div>
  );
}

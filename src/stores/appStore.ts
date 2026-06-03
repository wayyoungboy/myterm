import { create } from 'zustand';
import type { Connection, Group, Tab, ViewMode } from '../types';

interface AppState {
  // Sidebar
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;

  // Current view
  currentView: ViewMode;
  setView: (view: ViewMode) => void;

  // Groups
  groups: Group[];
  setGroups: (groups: Group[]) => void;

  // Connections
  connections: Connection[];
  setConnections: (connections: Connection[]) => void;

  // Tabs
  tabs: Tab[];
  activeTabId: string | null;
  addTab: (tab: Tab) => void;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTab: (id: string, updates: Partial<Tab>) => void;

  // Search
  searchQuery: string;
  setSearchQuery: (q: string) => void;

  // Selected connection for SFTP/monitor
  selectedConnectionId: string | null;
  setSelectedConnectionId: (id: string | null) => void;

  // Monitor sidebar
  monitorSidebarVisible: boolean;
  toggleMonitorSidebar: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  sidebarCollapsed: false,
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),

  currentView: 'terminal',
  setView: (view) => set({ currentView: view }),

  groups: [],
  setGroups: (groups) => set({ groups }),

  connections: [],
  setConnections: (connections) => set({ connections }),

  tabs: [],
  activeTabId: null,
  addTab: (tab) => set((s) => ({
    tabs: [...s.tabs, tab],
    activeTabId: tab.id,
  })),
  removeTab: (id) => set((s) => {
    const newTabs = s.tabs.filter((t) => t.id !== id);
    const newActive = s.activeTabId === id
      ? (newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null)
      : s.activeTabId;
    return { tabs: newTabs, activeTabId: newActive };
  }),
  setActiveTab: (id) => set({ activeTabId: id }),
  updateTab: (id, updates) => set((s) => ({
    tabs: s.tabs.map((t) => (t.id === id ? { ...t, ...updates } : t)),
  })),

  searchQuery: '',
  setSearchQuery: (q) => set({ searchQuery: q }),

  selectedConnectionId: null,
  setSelectedConnectionId: (id) => set({ selectedConnectionId: id }),

  monitorSidebarVisible: false,
  toggleMonitorSidebar: () => set((s) => ({ monitorSidebarVisible: !s.monitorSidebarVisible })),
}));

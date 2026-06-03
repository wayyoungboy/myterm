import { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { listen, emit } from '@tauri-apps/api/event';
import { useAppStore } from '../../stores/appStore';
import { connectTerminal, createConnection, disconnectTerminal, terminalResize, terminalWrite } from '../../utils/tauri';
import type { ConnectionInput } from '../../types';

import '@xterm/xterm/css/xterm.css';

const TERMINAL_THEME = {
  background: '#000000',
  foreground: '#e5e5e5',
  cursor: '#3b82f6',
  cursorAccent: '#000000',
  selectionBackground: '#3b82f640',
  selectionForeground: '#ffffff',
  black: '#171717',
  red: '#ef4444',
  green: '#22c55e',
  yellow: '#eab308',
  blue: '#3b82f6',
  magenta: '#a855f7',
  cyan: '#06b6d4',
  white: '#d4d4d4',
  brightBlack: '#404040',
  brightRed: '#f87171',
  brightGreen: '#4ade80',
  brightYellow: '#facc15',
  brightBlue: '#60a5fa',
  brightMagenta: '#c084fc',
  brightCyan: '#22d3ee',
  brightWhite: '#ffffff',
};

type AuthType = 'password' | 'key';

interface TerminalViewProps {
  connectionId?: string;
}

export function TerminalView({ connectionId }: TerminalViewProps) {
  const { updateTab, activeTabId, tabs } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);

  const terminalRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const unlistenOutput = useRef<(() => void) | null>(null);
  const unlistenExit = useRef<(() => void) | null>(null);
  const ioVersionRef = useRef(0);
  const autoStartedTabRef = useRef<string | null>(null);

  const [sessionId, setSessionId] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [terminalReady, setTerminalReady] = useState(false);

  const [formName, setFormName] = useState('');
  const [formHost, setFormHost] = useState('');
  const [formPort, setFormPort] = useState('22');
  const [formUsername, setFormUsername] = useState('');
  const [formPassword, setFormPassword] = useState('');
  const [formKeyPath, setFormKeyPath] = useState('');
  const [formAuthType, setFormAuthType] = useState<AuthType>('password');

  const onDataDisposable = useRef<(() => void) | null>(null);

  const cleanupTerminalIO = useCallback(() => {
    ioVersionRef.current += 1;
    unlistenOutput.current?.();
    unlistenOutput.current = null;
    unlistenExit.current?.();
    unlistenExit.current = null;
    onDataDisposable.current?.();
    onDataDisposable.current = null;
  }, []);

  const clearSessionFromTab = useCallback((sid: string) => {
    const state = useAppStore.getState();
    const tab = state.tabs.find((item) => item.sessionId === sid);
    if (!tab) return;
    state.updateTab(tab.id, { sessionId: null });
    emit('terminal-disconnected', tab.id);
  }, []);

  // Cleanup frontend listeners on unmount. The tab close handler owns SSH disconnects.
  useEffect(() => cleanupTerminalIO, [cleanupTerminalIO]);

  // Initialize xterm.js
  useEffect(() => {
    if (!terminalRef.current || termInstance.current) return;

    const terminal = new Terminal({
      theme: TERMINAL_THEME,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
      fontSize: 14,
      lineHeight: 1.2,
      cursorBlink: true,
      cursorStyle: 'bar',
      scrollback: 10000,
      allowTransparency: true,
    });

    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(terminalRef.current);
    fit.fit();

    termInstance.current = terminal;
    fitAddon.current = fit;
    setTerminalReady(true);

    return () => {
      terminal.dispose();
      termInstance.current = null;
      fitAddon.current = null;
      setTerminalReady(false);
    };
  }, []);

  // Handle resize
  useEffect(() => {
    const handleResize = () => {
      if (fitAddon.current) {
        fitAddon.current.fit();
        if (sessionId && termInstance.current) {
          const { cols, rows } = termInstance.current;
          terminalResize(sessionId, cols, rows).catch(() => {});
        }
      }
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [sessionId]);

  // ResizeObserver
  useEffect(() => {
    if (!terminalRef.current) return;
    const observer = new ResizeObserver(() => {
      if (fitAddon.current) {
        fitAddon.current.fit();
        if (sessionId && termInstance.current) {
          const { cols, rows } = termInstance.current;
          terminalResize(sessionId, cols, rows).catch(() => {});
        }
      }
    });
    observer.observe(terminalRef.current);
    return () => observer.disconnect();
  }, [sessionId]);

  const setupTerminalIO = useCallback((sid: string) => {
    const term = termInstance.current;
    if (!term) return;

    cleanupTerminalIO();
    const ioVersion = ioVersionRef.current;

    // Listen for output from SSH channel
    listen<number[]>(`terminal-output-${sid}`, (event) => {
      if (ioVersionRef.current !== ioVersion) return;
      const data = new Uint8Array(event.payload);
      term.write(data);
    }).then((unlisten) => {
      if (ioVersionRef.current !== ioVersion) {
        unlisten();
        return;
      }
      unlistenOutput.current = unlisten;
    }).catch(() => {});

    // Listen for session exit
    listen(`terminal-exit-${sid}`, () => {
      if (ioVersionRef.current !== ioVersion) return;
      term.writeln('');
      term.writeln('\x1b[38;2;249;226;175m  Session ended.\x1b[0m');
      setSessionId(null);
      clearSessionFromTab(sid);
    }).then((unlisten) => {
      if (ioVersionRef.current !== ioVersion) {
        unlisten();
        return;
      }
      unlistenExit.current = unlisten;
    }).catch(() => {});

    // Send user input to SSH channel - track disposable for cleanup
    const disposable = term.onData((data) => {
      terminalWrite(sid, data).catch(() => {});
    });
    onDataDisposable.current = () => disposable.dispose();

    // Send initial resize
    const { cols, rows } = term;
    terminalResize(sid, cols, rows).catch(() => {});

    // Focus the terminal
    term.focus();
  }, [cleanupTerminalIO, clearSessionFromTab]);

  const handleConnect = useCallback(
    async (targetConnectionId?: string) => {
      const connId = targetConnectionId || null;
      setError(null);
      setConnecting(true);

      try {
        let resolvedId = connId;

        if (!resolvedId) {
          if (!formHost.trim()) {
            setError('Host is required');
            setConnecting(false);
            return;
          }

          const input: ConnectionInput = {
            name: formName.trim() || `${formUsername}@${formHost}`,
            host: formHost.trim(),
            port: parseInt(formPort, 10) || 22,
            auth_type: formAuthType,
            username: formUsername.trim() || undefined,
            password: formAuthType === 'password' ? formPassword : undefined,
            key_path: formAuthType === 'key' ? formKeyPath.trim() || undefined : undefined,
          };

          const created = await createConnection(input);
          resolvedId = created.id;
        }

        if (!resolvedId) {
          throw new Error('Connection id is required');
        }

        const sid = await connectTerminal(resolvedId);

        setSessionId(sid);

        if (activeTabId) {
          updateTab(activeTabId, {
            sessionId: sid,
            connectionId: resolvedId,
          });
          emit('terminal-connected', activeTabId);
        }

        // Setup real terminal I/O
        setupTerminalIO(sid);

      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        if (termInstance.current) {
          termInstance.current.writeln(`\x1b[38;2;243;139;168m  Connection failed: ${message}\x1b[0m`);
        }
      } finally {
        setConnecting(false);
      }
    },
    [formHost, formName, formPort, formUsername, formPassword, formKeyPath, formAuthType, activeTabId, updateTab, setupTerminalIO]
  );

  // Reattach to an existing tab session after switching tabs.
  useEffect(() => {
    if (!terminalReady) return;
    if (!activeTab?.sessionId || sessionId === activeTab.sessionId) return;
    setSessionId(activeTab.sessionId);
    setupTerminalIO(activeTab.sessionId);
  }, [activeTab?.sessionId, sessionId, setupTerminalIO, terminalReady]);

  // Connect immediately if connectionId provided and the tab has no session yet.
  useEffect(() => {
    if (!terminalReady) return;
    if (
      connectionId &&
      activeTabId &&
      autoStartedTabRef.current !== activeTabId &&
      !activeTab?.sessionId &&
      !sessionId &&
      !connecting
    ) {
      autoStartedTabRef.current = activeTabId;
      handleConnect(connectionId);
    }
  }, [
    connectionId,
    activeTabId,
    activeTab?.sessionId,
    sessionId,
    connecting,
    handleConnect,
    terminalReady,
  ]);

  const handleDisconnect = useCallback(async () => {
    if (!sessionId) return;

    cleanupTerminalIO();

    try {
      await disconnectTerminal(sessionId);
    } catch {}

    setSessionId(null);
    clearSessionFromTab(sessionId);

    if (termInstance.current) {
      termInstance.current.writeln('\x1b[38;2;249;226;175m  Disconnected.\x1b[0m');
    }
  }, [cleanupTerminalIO, clearSessionFromTab, sessionId]);

  const showForm = !sessionId && !connectionId;

  return (
    <div className="flex flex-col h-full w-full" style={{ background: 'var(--bg-primary)' }}>
      {showForm && (
        <div className="flex items-center justify-center h-full w-full">
          <div
            className="w-full max-w-md p-6 rounded-xl border"
            style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}
          >
            <h2 className="text-lg font-semibold mb-5" style={{ color: 'var(--text-primary)' }}>
              New SSH Connection
            </h2>
            <div className="flex flex-col gap-3">
              <div>
                <label className="block text-xs font-medium mb-1" style={{ color: 'var(--text-secondary)' }}>Name</label>
                <input type="text" className="input" placeholder="My Server" value={formName} onChange={(e) => setFormName(e.target.value)} />
              </div>
              <div>
                <label className="block text-xs font-medium mb-1" style={{ color: 'var(--text-secondary)' }}>Host <span style={{ color: 'var(--error)' }}>*</span></label>
                <input type="text" className="input" placeholder="192.168.1.100" value={formHost} onChange={(e) => setFormHost(e.target.value)} autoFocus />
              </div>
              <div>
                <label className="block text-xs font-medium mb-1" style={{ color: 'var(--text-secondary)' }}>Port</label>
                <input type="number" className="input" placeholder="22" value={formPort} onChange={(e) => setFormPort(e.target.value)} min={1} max={65535} />
              </div>
              <div>
                <label className="block text-xs font-medium mb-1" style={{ color: 'var(--text-secondary)' }}>Username</label>
                <input type="text" className="input" placeholder="root" value={formUsername} onChange={(e) => setFormUsername(e.target.value)} />
              </div>
              <div>
                <label className="block text-xs font-medium mb-1" style={{ color: 'var(--text-secondary)' }}>Authentication</label>
                <select className="select" value={formAuthType} onChange={(e) => setFormAuthType(e.target.value as AuthType)}>
                  <option value="password">Password</option>
                  <option value="key">Private Key</option>
                </select>
              </div>
              {formAuthType === 'password' ? (
                <div>
                  <label className="block text-xs font-medium mb-1" style={{ color: 'var(--text-secondary)' }}>Password</label>
                  <input type="password" className="input" placeholder="Enter password" value={formPassword} onChange={(e) => setFormPassword(e.target.value)} />
                </div>
              ) : (
                <div>
                  <label className="block text-xs font-medium mb-1" style={{ color: 'var(--text-secondary)' }}>Private Key Path</label>
                  <input type="text" className="input" placeholder="~/.ssh/id_rsa" value={formKeyPath} onChange={(e) => setFormKeyPath(e.target.value)} />
                </div>
              )}
              {error && (
                <div className="text-xs px-3 py-2 rounded-md" style={{ color: 'var(--error)', background: 'rgba(239,68,68,0.1)' }}>
                  {error}
                </div>
              )}
              <button className="btn btn-primary w-full mt-2 justify-center" onClick={() => handleConnect()} disabled={connecting || !formHost.trim()}>
                {connecting ? <><span className="inline-block animate-spin">&#x21bb;</span> Connecting...</> : 'Connect'}
              </button>
            </div>
          </div>
        </div>
      )}

      {sessionId && !connectionId && (
        <div className="flex items-center justify-between px-3 py-1.5 text-xs border-b" style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)', color: 'var(--text-secondary)' }}>
          <span>Connected &mdash; <span style={{ color: 'var(--accent)' }}>{sessionId.slice(0, 8)}</span></span>
          <button className="btn btn-danger text-xs py-1 px-2" onClick={handleDisconnect}>Disconnect</button>
        </div>
      )}

      <div ref={terminalRef} className="flex-1 min-h-0" style={{ display: showForm ? 'none' : 'block' }} />
    </div>
  );
}

export default TerminalView;

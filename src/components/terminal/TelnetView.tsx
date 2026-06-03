import { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { listen } from '@tauri-apps/api/event';
import { connectTelnet, telnetWrite, disconnectTelnet } from '../../utils/tauri';

import '@xterm/xterm/css/xterm.css';

const TERMINAL_THEME = {
  background: '#000000',
  foreground: '#e5e5e5',
  cursor: '#3b82f6',
  cursorAccent: '#000000',
  selectionBackground: '#3b82f640',
  black: '#171717', red: '#ef4444', green: '#22c55e', yellow: '#eab308',
  blue: '#3b82f6', magenta: '#a855f7', cyan: '#06b6d4', white: '#d4d4d4',
  brightBlack: '#404040', brightRed: '#f87171', brightGreen: '#4ade80',
  brightYellow: '#facc15', brightBlue: '#60a5fa', brightMagenta: '#c084fc',
  brightCyan: '#22d3ee', brightWhite: '#ffffff',
};

export function TelnetView() {
  const terminalRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const unlistenOutput = useRef<(() => void) | null>(null);
  const unlistenExit = useRef<(() => void) | null>(null);
  const onDataDisposable = useRef<(() => void) | null>(null);
  const ioVersionRef = useRef(0);
  const sessionIdRef = useRef<string | null>(null);

  const [sessionId, setSessionId] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [formHost, setFormHost] = useState('');
  const [formPort, setFormPort] = useState('23');

  const cleanupTelnetIO = useCallback(() => {
    ioVersionRef.current += 1;
    unlistenOutput.current?.();
    unlistenOutput.current = null;
    unlistenExit.current?.();
    unlistenExit.current = null;
    onDataDisposable.current?.();
    onDataDisposable.current = null;
  }, []);

  useEffect(() => {
    if (!terminalRef.current || termInstance.current) return;
    const terminal = new Terminal({
      theme: TERMINAL_THEME,
      fontFamily: "'JetBrains Mono', 'Fira Code', Menlo, monospace",
      fontSize: 14, cursorBlink: true, scrollback: 10000,
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(terminalRef.current);
    fit.fit();
    termInstance.current = terminal;
    fitAddon.current = fit;
    return () => {
      cleanupTelnetIO();
      if (sessionIdRef.current) {
        disconnectTelnet(sessionIdRef.current).catch(() => {});
        sessionIdRef.current = null;
      }
      terminal.dispose();
      termInstance.current = null;
      fitAddon.current = null;
    };
  }, [cleanupTelnetIO]);

  useEffect(() => {
    const handleResize = () => fitAddon.current?.fit();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const handleConnect = useCallback(async () => {
    if (!formHost.trim()) { setError('Host is required'); return; }
    setError(null);
    setConnecting(true);
    try {
      const sid = await connectTelnet(formHost, parseInt(formPort) || 23);
      setSessionId(sid);
      sessionIdRef.current = sid;
      const term = termInstance.current;
      if (!term) return;
      cleanupTelnetIO();
      const ioVersion = ioVersionRef.current;

      listen<number[]>(`telnet-output-${sid}`, (event) => {
        if (ioVersionRef.current !== ioVersion) return;
        term.write(new Uint8Array(event.payload));
      }).then((unlisten) => {
        if (ioVersionRef.current !== ioVersion) {
          unlisten();
          return;
        }
        unlistenOutput.current = unlisten;
      }).catch(() => {});

      listen(`telnet-exit-${sid}`, () => {
        if (ioVersionRef.current !== ioVersion) return;
        term.writeln('\r\n\x1b[38;2;249;226;175m  Connection closed.\x1b[0m');
        setSessionId(null);
        sessionIdRef.current = null;
      }).then((unlisten) => {
        if (ioVersionRef.current !== ioVersion) {
          unlisten();
          return;
        }
        unlistenExit.current = unlisten;
      }).catch(() => {});

      const disposable = term.onData((data) => telnetWrite(sid, data).catch(() => {}));
      onDataDisposable.current = () => disposable.dispose();
      term.focus();
    } catch (e) {
      setError(String(e));
    } finally {
      setConnecting(false);
    }
  }, [cleanupTelnetIO, formHost, formPort]);

  const handleDisconnect = useCallback(async () => {
    if (!sessionId) return;
    cleanupTelnetIO();
    await disconnectTelnet(sessionId).catch(() => {});
    setSessionId(null);
    sessionIdRef.current = null;
    termInstance.current?.writeln('\r\n\x1b[38;2;249;226;175m  Disconnected.\x1b[0m');
  }, [cleanupTelnetIO, sessionId]);

  return (
    <div className="flex flex-col h-full" style={{ background: 'var(--bg-primary)' }}>
      {!sessionId && (
        <div className="flex items-center justify-center h-full">
          <div className="w-full max-w-sm p-6 rounded-xl border" style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}>
            <h2 className="text-lg font-semibold mb-4" style={{ color: 'var(--text-primary)' }}>Telnet Connection</h2>
            <div className="flex flex-col gap-3">
              <div>
                <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Host</label>
                <input className="input" value={formHost} onChange={(e) => setFormHost(e.target.value)} placeholder="192.168.1.1" autoFocus />
              </div>
              <div>
                <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Port</label>
                <input className="input" type="number" value={formPort} onChange={(e) => setFormPort(e.target.value)} />
              </div>
              {error && <div className="text-xs" style={{ color: 'var(--error)' }}>{error}</div>}
              <button className="btn btn-primary w-full justify-center" onClick={handleConnect} disabled={connecting}>
                {connecting ? 'Connecting...' : 'Connect'}
              </button>
            </div>
          </div>
        </div>
      )}
      {sessionId && (
        <div className="flex items-center justify-between px-3 py-1.5 text-xs border-b" style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}>
          <span style={{ color: 'var(--accent)' }}>Telnet: {formHost}</span>
          <button className="btn btn-danger text-xs py-1 px-2" onClick={handleDisconnect}>Disconnect</button>
        </div>
      )}
      <div ref={terminalRef} className="flex-1 min-h-0" style={{ display: sessionId ? 'block' : 'none' }} />
    </div>
  );
}

export default TelnetView;

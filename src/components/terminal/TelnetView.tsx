import { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { listen } from '@tauri-apps/api/event';
import { connectTelnet, telnetWrite, disconnectTelnet } from '../../utils/tauri';

import '@xterm/xterm/css/xterm.css';

const CATPPUCCIN_MOCHA = {
  background: '#1e1e2e',
  foreground: '#cdd6f4',
  cursor: '#f5e0dc',
  cursorAccent: '#1e1e2e',
  selectionBackground: '#585b7066',
  black: '#45475a', red: '#f38ba8', green: '#a6e3a1', yellow: '#f9e2af',
  blue: '#89b4fa', magenta: '#f5c2e7', cyan: '#94e2d5', white: '#bac2de',
  brightBlack: '#585b70', brightRed: '#f38ba8', brightGreen: '#a6e3a1',
  brightYellow: '#f9e2af', brightBlue: '#89b4fa', brightMagenta: '#f5c2e7',
  brightCyan: '#94e2d5', brightWhite: '#a6adc8',
};

export function TelnetView() {
  const terminalRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const unlistenOutput = useRef<(() => void) | null>(null);
  const unlistenExit = useRef<(() => void) | null>(null);

  const [sessionId, setSessionId] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [formHost, setFormHost] = useState('');
  const [formPort, setFormPort] = useState('23');

  useEffect(() => {
    if (!terminalRef.current || termInstance.current) return;
    const terminal = new Terminal({
      theme: CATPPUCCIN_MOCHA,
      fontFamily: "'JetBrains Mono', 'Fira Code', Menlo, monospace",
      fontSize: 14, cursorBlink: true, scrollback: 10000,
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(terminalRef.current);
    fit.fit();
    termInstance.current = terminal;
    fitAddon.current = fit;
    return () => { terminal.dispose(); termInstance.current = null; fitAddon.current = null; };
  }, []);

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
      const term = termInstance.current;
      if (!term) return;

      listen<number[]>(`telnet-output-${sid}`, (event) => {
        term.write(new Uint8Array(event.payload));
      }).then((unlisten) => { unlistenOutput.current = unlisten; });

      listen(`telnet-exit-${sid}`, () => {
        term.writeln('\r\n\x1b[38;2;249;226;175m  Connection closed.\x1b[0m');
        setSessionId(null);
      }).then((unlisten) => { unlistenExit.current = unlisten; });

      term.onData((data) => telnetWrite(sid, data).catch(() => {}));
      term.focus();
    } catch (e) {
      setError(String(e));
    } finally {
      setConnecting(false);
    }
  }, [formHost, formPort]);

  const handleDisconnect = useCallback(async () => {
    if (!sessionId) return;
    unlistenOutput.current?.();
    unlistenExit.current?.();
    await disconnectTelnet(sessionId).catch(() => {});
    setSessionId(null);
    termInstance.current?.writeln('\r\n\x1b[38;2;249;226;175m  Disconnected.\x1b[0m');
  }, [sessionId]);

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

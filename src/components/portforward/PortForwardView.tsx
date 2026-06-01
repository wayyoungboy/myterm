import { useState, useEffect } from 'react';
import { useAppStore } from '../../stores/appStore';
import { createPortForward, getPortForwards, closePortForward } from '../../utils/tauri';
import type { PortForward } from '../../utils/tauri';
import { Plus, Trash2, Network, RefreshCw } from 'lucide-react';

export function PortForwardView() {
  const { tabs, activeTabId } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);
  const sessionId = activeTab?.sessionId;

  const [forwards, setForwards] = useState<PortForward[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [formType, setFormType] = useState('local');
  const [formLocalHost, setFormLocalHost] = useState('127.0.0.1');
  const [formLocalPort, setFormLocalPort] = useState('8080');
  const [formRemoteHost, setFormRemoteHost] = useState('127.0.0.1');
  const [formRemotePort, setFormRemotePort] = useState('80');
  const [error, setError] = useState('');

  useEffect(() => {
    loadForwards();
  }, []);

  const loadForwards = async () => {
    try {
      const data = await getPortForwards();
      setForwards(data);
    } catch (e) {
      console.error('Failed to load forwards:', e);
    }
  };

  const handleCreate = async () => {
    if (!sessionId) {
      setError('No active terminal session');
      return;
    }
    setError('');
    try {
      await createPortForward(
        sessionId, formType,
        formLocalHost, parseInt(formLocalPort),
        formRemoteHost, parseInt(formRemotePort)
      );
      setShowForm(false);
      loadForwards();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleClose = async (id: string) => {
    try {
      await closePortForward(id);
      loadForwards();
    } catch (e) {
      console.error('Failed to close forward:', e);
    }
  };

  return (
    <div className="flex flex-col h-full" style={{ background: 'var(--bg-primary)' }}>
      <div className="flex items-center justify-between px-4 py-3 border-b" style={{ borderColor: 'var(--border)' }}>
        <h2 className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>Port Forwarding</h2>
        <div className="flex gap-2">
          <button className="btn btn-ghost" onClick={loadForwards}><RefreshCw size={14} /></button>
          <button className="btn btn-primary" onClick={() => setShowForm(true)}><Plus size={14} /> Add</button>
        </div>
      </div>

      {showForm && (
        <div className="p-4 border-b" style={{ borderColor: 'var(--border)', background: 'var(--bg-secondary)' }}>
          <div className="grid grid-cols-2 gap-3 mb-3">
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Type</label>
              <select className="select" value={formType} onChange={(e) => setFormType(e.target.value)}>
                <option value="local">Local (-L)</option>
                <option value="remote">Remote (-R)</option>
                <option value="dynamic">Dynamic SOCKS5 (-D)</option>
              </select>
            </div>
            <div />
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Local Host</label>
              <input className="input" value={formLocalHost} onChange={(e) => setFormLocalHost(e.target.value)} />
            </div>
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Local Port</label>
              <input className="input" type="number" value={formLocalPort} onChange={(e) => setFormLocalPort(e.target.value)} />
            </div>
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Remote Host</label>
              <input className="input" value={formRemoteHost} onChange={(e) => setFormRemoteHost(e.target.value)} />
            </div>
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Remote Port</label>
              <input className="input" type="number" value={formRemotePort} onChange={(e) => setFormRemotePort(e.target.value)} />
            </div>
          </div>
          {error && <div className="text-xs mb-2" style={{ color: 'var(--error)' }}>{error}</div>}
          <div className="flex gap-2">
            <button className="btn btn-primary" onClick={handleCreate}>Create</button>
            <button className="btn btn-ghost" onClick={() => setShowForm(false)}>Cancel</button>
          </div>
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-4">
        {forwards.length === 0 ? (
          <div className="text-center py-12">
            <Network size={48} className="mx-auto mb-3" style={{ color: 'var(--text-muted)' }} />
            <p className="text-sm" style={{ color: 'var(--text-muted)' }}>No port forwards configured</p>
          </div>
        ) : (
          <div className="space-y-2">
            {forwards.map((f) => (
              <div key={f.id} className="flex items-center justify-between p-3 rounded-lg" style={{ background: 'var(--bg-secondary)' }}>
                <div>
                  <div className="text-sm font-medium" style={{ color: 'var(--text-primary)' }}>
                    {f.forward_type.toUpperCase()}: {f.local_host}:{f.local_port} → {f.remote_host}:{f.remote_port}
                  </div>
                  <div className="text-xs" style={{ color: f.active ? 'var(--success)' : 'var(--error)' }}>
                    {f.active ? 'Active' : 'Inactive'}
                  </div>
                </div>
                <button className="btn btn-ghost" onClick={() => handleClose(f.id)}><Trash2 size={14} /></button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default PortForwardView;

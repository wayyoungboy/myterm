import { useState, useEffect } from 'react';
import { createConnection, updateConnection, testConnection, getGroups } from '../../utils/tauri';
import type { ConnectionInput, Group } from '../../types';
import { X, TestTube, Save, Key, Lock } from 'lucide-react';

interface Props {
  connectionId?: string;
  initialData?: Partial<ConnectionInput>;
  onClose: () => void;
  onSaved: () => void;
}

export function ConnectionForm({ connectionId, initialData, onClose, onSaved }: Props) {
  const [form, setForm] = useState<ConnectionInput>({
    name: '',
    host: '',
    port: 22,
    auth_type: 'password',
    username: 'root',
    ...initialData,
  });
  const [groups, setGroups] = useState<Group[]>([]);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    getGroups().then(setGroups).catch(console.error);
  }, []);

  const handleChange = (field: keyof ConnectionInput, value: any) => {
    setForm((prev) => ({ ...prev, [field]: value }));
  };

  const handleTest = async () => {
    setTesting(true);
    setError('');
    try {
      const result = await testConnection(form);
      alert(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    if (!form.name.trim()) { setError('Name is required'); return; }
    if (!form.host.trim()) { setError('Host is required'); return; }

    setSaving(true);
    setError('');
    try {
      if (connectionId) {
        await updateConnection(connectionId, form);
      } else {
        await createConnection(form);
      }
      onSaved();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal animate-slide-in" onClick={(e) => e.stopPropagation()} style={{ minWidth: 480 }}>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-base font-semibold">
            {connectionId ? 'Edit Connection' : 'New Connection'}
          </h2>
          <button className="btn-ghost p-1 rounded" onClick={onClose}>
            <X size={18} />
          </button>
        </div>

        {error && (
          <div className="mb-3 p-2 rounded text-xs" style={{ background: 'rgba(243,139,168,0.1)', color: 'var(--error)' }}>
            {error}
          </div>
        )}

        <div className="grid grid-cols-2 gap-3">
          <div className="col-span-2">
            <label className="block text-xs text-[var(--text-secondary)] mb-1">Name *</label>
            <input className="input" value={form.name} onChange={(e) => handleChange('name', e.target.value)} placeholder="My Server" />
          </div>

          <div>
            <label className="block text-xs text-[var(--text-secondary)] mb-1">Host *</label>
            <input className="input" value={form.host} onChange={(e) => handleChange('host', e.target.value)} placeholder="192.168.1.1" />
          </div>

          <div>
            <label className="block text-xs text-[var(--text-secondary)] mb-1">Port</label>
            <input className="input" type="number" value={form.port} onChange={(e) => handleChange('port', parseInt(e.target.value) || 22)} />
          </div>

          <div>
            <label className="block text-xs text-[var(--text-secondary)] mb-1">Username</label>
            <input className="input" value={form.username || ''} onChange={(e) => handleChange('username', e.target.value)} placeholder="root" />
          </div>

          <div>
            <label className="block text-xs text-[var(--text-secondary)] mb-1">Auth Type</label>
            <select className="select" value={form.auth_type} onChange={(e) => handleChange('auth_type', e.target.value)}>
              <option value="password">Password</option>
              <option value="key">Private Key</option>
              <option value="interactive">Interactive</option>
            </select>
          </div>

          {form.auth_type === 'password' && (
            <div className="col-span-2">
              <label className="block text-xs text-[var(--text-secondary)] mb-1">
                <Lock size={12} className="inline mr-1" />Password
              </label>
              <input className="input" type="password" value={form.password || ''} onChange={(e) => handleChange('password', e.target.value)} />
            </div>
          )}

          {form.auth_type === 'key' && (
            <div className="col-span-2">
              <label className="block text-xs text-[var(--text-secondary)] mb-1">
                <Key size={12} className="inline mr-1" />Private Key Path
              </label>
              <input className="input" value={form.key_path || ''} onChange={(e) => handleChange('key_path', e.target.value)} placeholder="~/.ssh/id_rsa" />
            </div>
          )}

          <div>
            <label className="block text-xs text-[var(--text-secondary)] mb-1">Group</label>
            <select className="select" value={form.group_id || ''} onChange={(e) => handleChange('group_id', e.target.value || undefined)}>
              <option value="">None</option>
              {groups.map((g) => (
                <option key={g.id} value={g.id}>{g.name}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-xs text-[var(--text-secondary)] mb-1">Timeout (ms)</label>
            <input className="input" type="number" value={form.timeout_ms || 10000} onChange={(e) => handleChange('timeout_ms', parseInt(e.target.value) || 10000)} />
          </div>

          <div className="col-span-2">
            <label className="block text-xs text-[var(--text-secondary)] mb-1">Remark</label>
            <input className="input" value={form.remark || ''} onChange={(e) => handleChange('remark', e.target.value)} placeholder="Optional notes" />
          </div>
        </div>

        <div className="flex justify-end gap-2 mt-4 pt-3 border-t border-[var(--border)]">
          <button className="btn btn-secondary" onClick={handleTest} disabled={testing}>
            <TestTube size={14} />
            {testing ? 'Testing...' : 'Test'}
          </button>
          <button className="btn btn-ghost" onClick={onClose}>Cancel</button>
          <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
            <Save size={14} />
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  );
}

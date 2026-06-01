import { useState, useEffect } from 'react';
import { useAppStore } from '../../stores/appStore';
import {
  getQuickCommands, createQuickCommand, updateQuickCommand, deleteQuickCommand,
  type QuickCommand
} from '../../utils/tauri';
import { Plus, Trash2, Edit, Play, Terminal, Search } from 'lucide-react';

export function QuickCommandsView() {
  const { tabs, activeTabId } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);

  const [commands, setCommands] = useState<QuickCommand[]>([]);
  const [, setLoading] = useState(false);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formName, setFormName] = useState('');
  const [formCommand, setFormCommand] = useState('');
  const [formShortcut, setFormShortcut] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    loadCommands();
  }, []);

  const loadCommands = async () => {
    setLoading(true);
    try {
      const data = await getQuickCommands();
      setCommands(data);
    } catch (e) {
      console.error('Failed to load commands:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!formName.trim() || !formCommand.trim()) {
      setError('Name and command are required');
      return;
    }
    setError('');
    try {
      if (editingId) {
        await updateQuickCommand(editingId, formName, formCommand, formShortcut || undefined);
      } else {
        await createQuickCommand(formName, formCommand, undefined, formShortcut || undefined);
      }
      setShowForm(false);
      setEditingId(null);
      setFormName('');
      setFormCommand('');
      setFormShortcut('');
      loadCommands();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleEdit = (cmd: QuickCommand) => {
    setEditingId(cmd.id);
    setFormName(cmd.name);
    setFormCommand(cmd.command);
    setFormShortcut(cmd.shortcut || '');
    setShowForm(true);
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this command?')) return;
    try {
      await deleteQuickCommand(id);
      loadCommands();
    } catch (e) {
      console.error('Failed to delete:', e);
    }
  };

  const handleExecute = async (cmd: QuickCommand) => {
    if (!activeTab?.sessionId) {
      setError('No active terminal session');
      return;
    }
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('terminal_write', {
        sessionId: activeTab.sessionId,
        data: cmd.command + '\n',
      });
    } catch (e) {
      setError(String(e));
    }
  };

  const filtered = searchQuery
    ? commands.filter((c) =>
        c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        c.command.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : commands;

  return (
    <div className="flex flex-col h-full" style={{ background: 'var(--bg-primary)' }}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b" style={{ borderColor: 'var(--border)' }}>
        <h2 className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>Quick Commands</h2>
        <button className="btn btn-primary" onClick={() => { setShowForm(true); setEditingId(null); setFormName(''); setFormCommand(''); setFormShortcut(''); }}>
          <Plus size={14} /> Add
        </button>
      </div>

      {/* Search */}
      <div className="px-4 py-2">
        <div className="relative">
          <Search size={14} className="absolute left-2 top-1/2 -translate-y-1/2 text-[var(--text-muted)]" />
          <input
            type="text"
            placeholder="Search commands..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="input pl-7 py-1 text-xs"
          />
        </div>
      </div>

      {/* Form */}
      {showForm && (
        <div className="px-4 py-3 border-b" style={{ borderColor: 'var(--border)', background: 'var(--bg-secondary)' }}>
          <div className="grid grid-cols-2 gap-3 mb-3">
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Name</label>
              <input className="input" value={formName} onChange={(e) => setFormName(e.target.value)} placeholder="My Command" />
            </div>
            <div>
              <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Shortcut</label>
              <input className="input" value={formShortcut} onChange={(e) => setFormShortcut(e.target.value)} placeholder="Ctrl+Shift+1" />
            </div>
            <div className="col-span-2">
              <label className="block text-xs mb-1" style={{ color: 'var(--text-secondary)' }}>Command</label>
              <input className="input font-mono" value={formCommand} onChange={(e) => setFormCommand(e.target.value)} placeholder="ls -la ${host}" />
              <div className="text-[10px] mt-1" style={{ color: 'var(--text-muted)' }}>
                Variables: ${'{host}'}, ${'{port}'}, ${'{username}'}, ${'{date}'}, ${'{time}'}
              </div>
            </div>
          </div>
          {error && <div className="text-xs mb-2" style={{ color: 'var(--error)' }}>{error}</div>}
          <div className="flex gap-2">
            <button className="btn btn-primary" onClick={handleSave}>Save</button>
            <button className="btn btn-ghost" onClick={() => { setShowForm(false); setEditingId(null); }}>Cancel</button>
          </div>
        </div>
      )}

      {/* Commands list */}
      <div className="flex-1 overflow-y-auto p-4">
        {filtered.length === 0 ? (
          <div className="text-center py-12">
            <Terminal size={48} className="mx-auto mb-3" style={{ color: 'var(--text-muted)' }} />
            <p className="text-sm" style={{ color: 'var(--text-muted)' }}>
              {searchQuery ? 'No matching commands' : 'No quick commands yet'}
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {filtered.map((cmd) => (
              <div
                key={cmd.id}
                className="flex items-center justify-between p-3 rounded-lg"
                style={{ background: 'var(--bg-secondary)' }}
              >
                <div className="flex-1 min-w-0">
                  <div className="text-sm font-medium" style={{ color: 'var(--text-primary)' }}>
                    {cmd.name}
                    {cmd.shortcut && (
                      <span className="ml-2 text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
                        {cmd.shortcut}
                      </span>
                    )}
                  </div>
                  <div className="text-xs font-mono truncate" style={{ color: 'var(--text-secondary)' }}>
                    {cmd.command}
                  </div>
                </div>
                <div className="flex gap-1 ml-2">
                  <button
                    className="btn-ghost p-1.5 rounded"
                    onClick={() => handleExecute(cmd)}
                    title="Execute in active terminal"
                  >
                    <Play size={14} style={{ color: 'var(--success)' }} />
                  </button>
                  <button className="btn-ghost p-1.5 rounded" onClick={() => handleEdit(cmd)}>
                    <Edit size={14} />
                  </button>
                  <button className="btn-ghost p-1.5 rounded" onClick={() => handleDelete(cmd.id)}>
                    <Trash2 size={14} style={{ color: 'var(--error)' }} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default QuickCommandsView;

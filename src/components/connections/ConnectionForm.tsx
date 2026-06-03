import { useEffect, useState } from 'react';
import { createConnection, updateConnection, testConnection, getGroups, getConnections } from '../../utils/tauri';
import type { Connection, ConnectionInput, Group } from '../../types';
import { X, TestTube, Save, Key, Lock, Server, Link, Globe, Settings } from 'lucide-react';

interface Props {
  connectionId?: string;
  initialData?: Partial<ConnectionInput>;
  onClose: () => void;
  onSaved: () => void;
}

type TabId = 'basic' | 'connection' | 'proxy' | 'other';

const TABS: { id: TabId; label: string; icon: React.ReactNode }[] = [
  { id: 'basic', label: '基本信息', icon: <Server size={13} /> },
  { id: 'connection', label: '连接设置', icon: <Link size={13} /> },
  { id: 'proxy', label: '代理设置', icon: <Globe size={13} /> },
  { id: 'other', label: '其他', icon: <Settings size={13} /> },
];

export function ConnectionForm({ connectionId, initialData, onClose, onSaved }: Props) {
  const [form, setForm] = useState<ConnectionInput>({
    name: '',
    host: '',
    port: 22,
    auth_type: 'password',
    username: 'root',
    timeout_ms: 10000,
    heartbeat_ms: 5000,
    ...initialData,
  });
  const [groups, setGroups] = useState<Group[]>([]);
  const [connections, setConnections] = useState<Connection[]>([]);
  const [activeTab, setActiveTab] = useState<TabId>('basic');
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [testResult, setTestResult] = useState<'success' | 'fail' | null>(null);

  useEffect(() => {
    getGroups().then(setGroups).catch((e) => console.error('Failed to load groups:', e));
    getConnections().then(setConnections).catch((e) => console.error('Failed to load connections:', e));
  }, []);

  const handleChange = (field: keyof ConnectionInput, value: any) => {
    setForm((prev) => ({ ...prev, [field]: value }));
    setTestResult(null);
  };

  const handleProxyTypeChange = (value: string) => {
    setForm((prev) => ({
      ...prev,
      proxy_type: value === 'none' ? undefined : value,
      proxy_host: value === 'none' ? undefined : prev.proxy_host,
      proxy_port: value === 'none' ? undefined : (prev.proxy_port || (value === 'http' ? 8080 : 1080)),
      proxy_jump_id: undefined,
    }));
    setTestResult(null);
  };

  const handleProxyJumpChange = (value: string) => {
    setForm((prev) => ({
      ...prev,
      proxy_jump_id: value || undefined,
      proxy_type: value ? undefined : prev.proxy_type,
      proxy_host: value ? undefined : prev.proxy_host,
      proxy_port: value ? undefined : prev.proxy_port,
    }));
    setTestResult(null);
  };

  const validateProxy = () => {
    if (!form.proxy_type || form.proxy_type === 'none') return true;
    if (!form.proxy_host?.trim()) {
      setError('代理主机不能为空');
      return false;
    }
    if (!form.proxy_port || form.proxy_port <= 0) {
      setError('代理端口无效');
      return false;
    }
    return true;
  };

  const handleTest = async () => {
    setError('');
    setTestResult(null);
    if (!validateProxy()) return;

    setTesting(true);
    try {
      await testConnection(form);
      setTestResult('success');
    } catch (e) {
      setError(String(e));
      setTestResult('fail');
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    if (!form.name.trim()) { setError('名称不能为空'); return; }
    if (!form.host.trim()) { setError('主机地址不能为空'); return; }
    if (!validateProxy()) return;

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

  const renderBasicTab = () => (
    <div className="flex flex-col gap-4">
      {/* Name */}
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>名称 *</label>
        <input className="input" value={form.name} onChange={(e) => handleChange('name', e.target.value)} placeholder="我的服务器" />
      </div>

      {/* Address */}
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
          地址 * <span className="text-[10px]" style={{ color: 'var(--text-muted)' }}>填写目标 SSH 地址和端口</span>
        </label>
        <div className="grid grid-cols-3 gap-2">
          <input className="input col-span-2" value={form.host} onChange={(e) => handleChange('host', e.target.value)} placeholder="192.168.1.1 或 example.com" />
          <input className="input" type="number" value={form.port} onChange={(e) => handleChange('port', parseInt(e.target.value) || 22)} />
        </div>
      </div>

      {/* Auth */}
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>验证方式</label>
        <select className="select" value={form.auth_type} onChange={(e) => handleChange('auth_type', e.target.value)}>
          <option value="password">密码</option>
          <option value="key">密钥</option>
          <option value="interactive">交互认证</option>
        </select>
      </div>

      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>连接分组</label>
        <select
          className="select"
          value={form.group_id || ''}
          onChange={(e) => handleChange('group_id', e.target.value || undefined)}
        >
          <option value="">未分组</option>
          {groups.map((group) => (
            <option key={group.id} value={group.id}>{group.name}</option>
          ))}
        </select>
      </div>

      {/* Username + Password */}
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>登录用户</label>
          <input className="input" value={form.username || ''} onChange={(e) => handleChange('username', e.target.value)} placeholder="root" />
        </div>
        {form.auth_type === 'key' ? (
          <div>
            <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
              <Key size={10} className="inline mr-1" />密钥路径
            </label>
            <input className="input" value={form.key_path || ''} onChange={(e) => handleChange('key_path', e.target.value)} placeholder="留空使用 ssh-agent 或 ~/.ssh/id_ed25519" />
          </div>
        ) : (
          <div>
            <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
              <Lock size={10} className="inline mr-1" />登录密码
            </label>
            <input className="input" type="password" value={form.password || ''} onChange={(e) => handleChange('password', e.target.value)} />
          </div>
        )}
      </div>

      {/* Remark */}
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>主机备注</label>
        <textarea
          className="input min-h-[60px] resize-none"
          value={form.remark || ''}
          onChange={(e) => handleChange('remark', e.target.value)}
          placeholder="可选的备注信息"
        />
      </div>
    </div>
  );

  const renderConnectionTab = () => (
    <div className="flex flex-col gap-4">
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
          超时时间（毫秒）
        </label>
        <input className="input" type="number" value={form.timeout_ms || 10000} onChange={(e) => handleChange('timeout_ms', parseInt(e.target.value) || 10000)} />
      </div>
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
          心跳时间（毫秒）
        </label>
        <input
          className="input"
          type="number"
          min={1000}
          max={600000}
          step={1000}
          value={form.heartbeat_ms || 5000}
          onChange={(e) => {
            const parsed = parseInt(e.target.value) || 5000;
            handleChange('heartbeat_ms', Math.min(600000, Math.max(1000, parsed)));
          }}
        />
      </div>
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
          终端显示编码
        </label>
        <select className="select" value="utf-8" disabled>
          <option value="utf-8">UTF-8（默认）</option>
        </select>
      </div>
    </div>
  );

  const renderProxyTab = () => (
    <div className="flex flex-col gap-4">
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
          跳板机
        </label>
        <select
          className="select"
          value={form.proxy_jump_id || ''}
          onChange={(e) => handleProxyJumpChange(e.target.value)}
        >
          <option value="">不使用跳板机</option>
          {connections
            .filter((conn) => conn.id !== connectionId)
            .map((conn) => (
              <option key={conn.id} value={conn.id}>
                {conn.name} ({conn.username || 'root'}@{conn.host}:{conn.port})
              </option>
            ))}
        </select>
      </div>
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
          代理类型
        </label>
        <select
          className="select"
          value={form.proxy_type || 'none'}
          onChange={(e) => handleProxyTypeChange(e.target.value)}
          disabled={Boolean(form.proxy_jump_id)}
        >
          <option value="none">不使用代理</option>
          <option value="http">HTTP CONNECT</option>
          <option value="socks5">SOCKS5</option>
        </select>
      </div>
      <div className="grid grid-cols-3 gap-2">
        <div className="col-span-2">
          <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
            代理主机
          </label>
          <input
            className="input"
            value={form.proxy_host || ''}
            onChange={(e) => handleChange('proxy_host', e.target.value)}
            placeholder="127.0.0.1"
            disabled={Boolean(form.proxy_jump_id) || !form.proxy_type || form.proxy_type === 'none'}
          />
        </div>
        <div>
          <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
            端口
          </label>
          <input
            className="input"
            type="number"
            value={form.proxy_port || ''}
            onChange={(e) => handleChange('proxy_port', parseInt(e.target.value) || undefined)}
            placeholder={form.proxy_type === 'http' ? '8080' : '1080'}
            disabled={Boolean(form.proxy_jump_id) || !form.proxy_type || form.proxy_type === 'none'}
          />
        </div>
      </div>
    </div>
  );

  const renderOtherTab = () => (
    <div className="flex flex-col gap-4">
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
          初始化命令
        </label>
        <input className="input" value={form.init_command || ''} onChange={(e) => handleChange('init_command', e.target.value)} placeholder="连接后自动执行的命令" />
      </div>
      <div>
        <label className="block text-xs mb-1.5" style={{ color: 'var(--text-secondary)' }}>
          初始路径
        </label>
        <input className="input" value={form.init_path || ''} onChange={(e) => handleChange('init_path', e.target.value)} placeholder="例如 /var/log" />
      </div>
    </div>
  );

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal animate-slide-in flex flex-col"
        onClick={(e) => e.stopPropagation()}
        style={{ minWidth: 560, maxWidth: 600, maxHeight: '85vh' }}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
            {connectionId ? '编辑 SSH 连接' : '新建 SSH 连接'}
          </h2>
          <button className="p-1 rounded transition-colors hover:bg-[var(--bg-surface)]" onClick={onClose} style={{ color: 'var(--text-muted)' }}>
            <X size={16} />
          </button>
        </div>

        {/* Tabs + Content */}
        <div className="flex gap-4 flex-1 min-h-0">
          {/* Left tabs */}
          <div className="flex flex-col gap-1 w-28 shrink-0">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                className="flex items-center gap-2 px-2.5 py-2 rounded-lg text-xs transition-colors text-left"
                style={{
                  background: activeTab === tab.id ? 'var(--bg-surface)' : 'transparent',
                  color: activeTab === tab.id ? 'var(--text-primary)' : 'var(--text-muted)',
                }}
                onClick={() => setActiveTab(tab.id)}
              >
                {tab.icon}
                {tab.label}
              </button>
            ))}
          </div>

          {/* Right content */}
          <div className="flex-1 overflow-y-auto min-h-0">
            {activeTab === 'basic' && renderBasicTab()}
            {activeTab === 'connection' && renderConnectionTab()}
            {activeTab === 'proxy' && renderProxyTab()}
            {activeTab === 'other' && renderOtherTab()}
          </div>
        </div>

        {/* Error */}
        {error && (
          <div className="mt-3 p-2.5 rounded-lg text-xs" style={{ background: 'rgba(239,68,68,0.08)', color: 'var(--error)', border: '1px solid rgba(239,68,68,0.15)' }}>
            {error}
          </div>
        )}

        {testResult === 'success' && (
          <div className="mt-3 p-2.5 rounded-lg text-xs" style={{ background: 'rgba(34,197,94,0.08)', color: 'var(--success)', border: '1px solid rgba(34,197,94,0.15)' }}>
            连接测试通过！
          </div>
        )}

        {/* Actions */}
        <div className="flex justify-between items-center mt-4 pt-3 border-t border-[var(--border)]">
          <button className="btn btn-secondary text-xs" onClick={handleTest} disabled={testing}>
            <TestTube size={12} />
            {testing ? '测试中...' : '测试连接'}
          </button>
          <div className="flex gap-2">
            <button className="btn btn-ghost text-xs" onClick={onClose}>取消</button>
            <button className="btn btn-primary text-xs" onClick={handleSave} disabled={saving}>
              <Save size={12} />
              {saving ? '保存中...' : (connectionId ? '更新' : '创建')}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

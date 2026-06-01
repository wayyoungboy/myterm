import { useState, useEffect, useCallback, useRef } from 'react';
import { Settings, Terminal, Brain, Shield, Save } from 'lucide-react';
import { getSettings, setSetting } from '../../utils/tauri';

interface SettingField {
  key: string;
  label: string;
  type: 'text' | 'number' | 'password' | 'select' | 'toggle' | 'range';
  defaultValue: string;
  options?: { value: string; label: string }[];
  min?: number;
  max?: number;
  step?: number;
}

interface SettingSection {
  id: string;
  title: string;
  icon: React.ReactNode;
  fields: SettingField[];
}

const sections: SettingSection[] = [
  {
    id: 'general',
    title: 'General',
    icon: <Settings size={18} />,
    fields: [
      { key: 'theme', label: 'Theme', type: 'select', defaultValue: 'dark', options: [{ value: 'dark', label: 'Dark' }, { value: 'light', label: 'Light' }] },
      { key: 'language', label: 'Language', type: 'select', defaultValue: 'en', options: [{ value: 'en', label: 'English' }, { value: 'zh', label: 'Chinese' }] },
      { key: 'font_family', label: 'Font Family', type: 'text', defaultValue: 'JetBrains Mono' },
      { key: 'font_size', label: 'Font Size', type: 'number', defaultValue: '14', min: 8, max: 32 },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminal',
    icon: <Terminal size={18} />,
    fields: [
      { key: 'cursor_style', label: 'Cursor Style', type: 'select', defaultValue: 'block', options: [{ value: 'block', label: 'Block' }, { value: 'underline', label: 'Underline' }, { value: 'bar', label: 'Bar' }] },
      { key: 'cursor_blink', label: 'Cursor Blink', type: 'toggle', defaultValue: 'false' },
      { key: 'scrollback_lines', label: 'Scrollback Lines', type: 'number', defaultValue: '1000', min: 100, max: 100000 },
      { key: 'copy_on_select', label: 'Copy on Select', type: 'toggle', defaultValue: 'false' },
    ],
  },
  {
    id: 'ai',
    title: 'AI',
    icon: <Brain size={18} />,
    fields: [
      { key: 'ai_api_key', label: 'API Key', type: 'password', defaultValue: '' },
      { key: 'ai_api_base', label: 'API Base URL', type: 'text', defaultValue: 'https://api.openai.com/v1' },
      { key: 'ai_model', label: 'Model', type: 'text', defaultValue: 'gpt-4' },
      { key: 'ai_temperature', label: 'Temperature', type: 'range', defaultValue: '0.5', min: 0, max: 1, step: 0.1 },
      { key: 'ai_max_tokens', label: 'Max Tokens', type: 'number', defaultValue: '8192', min: 1, max: 128000 },
      { key: 'ai_context_messages', label: 'Context Messages Count', type: 'number', defaultValue: '4', min: 0, max: 50 },
    ],
  },
  {
    id: 'ssh',
    title: 'SSH',
    icon: <Shield size={18} />,
    fields: [
      { key: 'ssh_default_port', label: 'Default Port', type: 'number', defaultValue: '22', min: 1, max: 65535 },
      { key: 'ssh_timeout_ms', label: 'Connection Timeout (ms)', type: 'number', defaultValue: '10000', min: 1000, max: 120000 },
      { key: 'ssh_keepalive_ms', label: 'Keepalive Interval (ms)', type: 'number', defaultValue: '5000', min: 1000, max: 60000 },
    ],
  },
];

export function SettingsView() {
  const [values, setValues] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const settings = await getSettings();
      const merged: Record<string, string> = {};
      for (const section of sections) {
        for (const field of section.fields) {
          merged[field.key] = settings[field.key] ?? field.defaultValue;
        }
      }
      setValues(merged);
    } catch (err) {
      console.error('Failed to load settings:', err);
    } finally {
      setLoading(false);
    }
  };

  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleChange = useCallback((key: string, value: string) => {
    setValues((prev) => ({ ...prev, [key]: value }));

    // Debounce the save operation
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current);
    }
    saveTimerRef.current = setTimeout(async () => {
      try {
        await setSetting(key, value);
        setSaved(true);
        setTimeout(() => setSaved(false), 2000);
      } catch (err) {
        console.error('Failed to save setting:', err);
      }
    }, 500);
  }, []);

  const renderField = (field: SettingField) => {
    const value = values[field.key] ?? field.defaultValue;

    switch (field.type) {
      case 'select':
        return (
          <select
            className="select"
            value={value}
            onChange={(e) => handleChange(field.key, e.target.value)}
          >
            {field.options?.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        );

      case 'toggle':
        return (
          <button
            type="button"
            onClick={() => handleChange(field.key, value === 'true' ? 'false' : 'true')}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              value === 'true' ? 'bg-[var(--accent)]' : 'bg-[var(--bg-surface)]'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                value === 'true' ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        );

      case 'range':
        return (
          <div className="flex items-center gap-3 w-full">
            <input
              type="range"
              className="flex-1 accent-[var(--accent)]"
              min={field.min}
              max={field.max}
              step={field.step}
              value={value}
              onChange={(e) => handleChange(field.key, e.target.value)}
            />
            <span className="text-xs text-[var(--text-secondary)] w-10 text-right">{value}</span>
          </div>
        );

      case 'password':
        return (
          <input
            type="password"
            className="input"
            value={value}
            placeholder={field.defaultValue || 'Enter value...'}
            onChange={(e) => handleChange(field.key, e.target.value)}
          />
        );

      case 'number':
        return (
          <input
            type="number"
            className="input"
            value={value}
            min={field.min}
            max={field.max}
            onChange={(e) => handleChange(field.key, e.target.value)}
          />
        );

      default:
        return (
          <input
            type="text"
            className="input"
            value={value}
            placeholder={field.defaultValue}
            onChange={(e) => handleChange(field.key, e.target.value)}
          />
        );
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--text-muted)]">
        Loading settings...
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-6">
      <div className="max-w-2xl mx-auto space-y-6">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-xl font-semibold text-[var(--text-primary)]">Settings</h1>
          {saved && (
            <span className="flex items-center gap-1.5 text-xs text-[var(--success)] animate-fade-in">
              <Save size={14} />
              Saved
            </span>
          )}
        </div>

        {sections.map((section) => (
          <div
            key={section.id}
            className="rounded-lg border border-[var(--border)] bg-[var(--bg-secondary)] overflow-hidden"
          >
            <div className="flex items-center gap-2 px-4 py-3 border-b border-[var(--border)] bg-[var(--bg-surface)]">
              <span className="text-[var(--accent)]">{section.icon}</span>
              <h2 className="text-sm font-medium text-[var(--text-primary)]">{section.title}</h2>
            </div>
            <div className="divide-y divide-[var(--border)]">
              {section.fields.map((field) => (
                <div
                  key={field.key}
                  className="flex items-center justify-between gap-4 px-4 py-3"
                >
                  <label className="text-sm text-[var(--text-secondary)] whitespace-nowrap min-w-[160px]">
                    {field.label}
                  </label>
                  <div className="flex-1 max-w-xs">
                    {renderField(field)}
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

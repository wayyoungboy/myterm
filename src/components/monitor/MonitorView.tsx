import { useState, useEffect, useCallback, useRef } from 'react';
import { useAppStore } from '../../stores/appStore';
import { connectTerminal, getMonitorData } from '../../utils/tauri';
import type { MonitorData, DiskPartition, GpuData, ProcessInfo } from '../../types';
import { Cpu, HardDrive, Wifi, Server, Activity, Thermometer, Zap } from 'lucide-react';

// ── Helpers ──────────────────────────────────────────────────────────

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i > 1 ? 1 : 0)} ${units[i]}`;
}

function formatUptime(seconds: number): string {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const parts: string[] = [];
  if (d > 0) parts.push(`${d}d`);
  if (h > 0) parts.push(`${h}h`);
  parts.push(`${m}m`);
  return parts.join(' ');
}

function formatRate(bytesPerSec: number): string {
  if (bytesPerSec < 1024) return `${bytesPerSec.toFixed(0)} B/s`;
  if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
}

function usageColor(pct: number): string {
  if (pct < 60) return 'var(--success)';
  if (pct < 85) return 'var(--warning)';
  return 'var(--error)';
}

// ── Sub-components ───────────────────────────────────────────────────

function Card({ title, icon, children }: { title: string; icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg p-4 flex flex-col gap-3">
      <div className="flex items-center gap-2 text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wide">
        {icon}
        {title}
      </div>
      {children}
    </div>
  );
}

function Bar({ label, used, total, color }: { label: string; used: number; total: number; color?: string }) {
  const pct = total > 0 ? (used / total) * 100 : 0;
  const c = color ?? usageColor(pct);
  return (
    <div className="flex flex-col gap-1">
      <div className="flex justify-between text-[11px]">
        <span className="text-[var(--text-secondary)] truncate">{label}</span>
        <span className="text-[var(--text-muted)] shrink-0 ml-2">{formatBytes(used)} / {formatBytes(total)}</span>
      </div>
      <div className="h-2 rounded-full bg-[var(--bg-surface)] overflow-hidden">
        <div className="h-full rounded-full transition-all duration-500" style={{ width: `${Math.min(pct, 100)}%`, background: c }} />
      </div>
    </div>
  );
}

function CircularGauge({ value, size = 80, strokeWidth = 6 }: { value: number; size?: number; strokeWidth?: number }) {
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (Math.min(value, 100) / 100) * circumference;
  const color = usageColor(value);

  return (
    <svg width={size} height={size} className="transform -rotate-90">
      <circle cx={size / 2} cy={size / 2} r={radius} fill="none" stroke="var(--bg-surface)" strokeWidth={strokeWidth} />
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke={color}
        strokeWidth={strokeWidth}
        strokeDasharray={circumference}
        strokeDashoffset={offset}
        strokeLinecap="round"
        className="transition-all duration-500"
      />
      <text
        x={size / 2}
        y={size / 2}
        textAnchor="middle"
        dominantBaseline="central"
        className="fill-[var(--text-primary)] text-sm font-semibold"
        transform={`rotate(90 ${size / 2} ${size / 2})`}
      >
        {Math.round(value)}%
      </text>
    </svg>
  );
}

// ── Cards ────────────────────────────────────────────────────────────

function SystemInfoCard({ data }: { data: MonitorData }) {
  const [l1, l2, l3] = data.load_avg;
  return (
    <Card title="System Info" icon={<Server size={14} />}>
      <div className="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
        <div>
          <span className="text-[var(--text-muted)]">Hostname</span>
          <div className="text-[var(--text-primary)] font-medium truncate">{data.hostname}</div>
        </div>
        <div>
          <span className="text-[var(--text-muted)]">OS</span>
          <div className="text-[var(--text-primary)] font-medium truncate">{data.os_info}</div>
        </div>
        <div>
          <span className="text-[var(--text-muted)]">Uptime</span>
          <div className="text-[var(--text-primary)] font-medium">{formatUptime(data.uptime)}</div>
        </div>
        <div>
          <span className="text-[var(--text-muted)]">Load Avg</span>
          <div className="text-[var(--text-primary)] font-medium">{l1.toFixed(2)} / {l2.toFixed(2)} / {l3.toFixed(2)}</div>
        </div>
      </div>
    </Card>
  );
}

function CpuCard({ data }: { data: MonitorData }) {
  return (
    <Card title="CPU" icon={<Cpu size={14} />}>
      <div className="flex items-center gap-4">
        <CircularGauge value={data.cpu_usage} />
        <div className="flex-1 flex flex-col gap-1 min-w-0">
          {data.cpu_cores.map((usage, i) => (
            <div key={i} className="flex items-center gap-2">
              <span className="text-[10px] text-[var(--text-muted)] w-6 shrink-0">C{i}</span>
              <div className="flex-1 h-1.5 rounded-full bg-[var(--bg-surface)] overflow-hidden">
                <div
                  className="h-full rounded-full transition-all duration-500"
                  style={{ width: `${Math.min(usage, 100)}%`, background: usageColor(usage) }}
                />
              </div>
              <span className="text-[10px] text-[var(--text-muted)] w-7 text-right">{Math.round(usage)}%</span>
            </div>
          ))}
        </div>
      </div>
    </Card>
  );
}

function MemoryCard({ data }: { data: MonitorData }) {
  const memPct = data.mem_total > 0 ? (data.mem_used / data.mem_total) * 100 : 0;
  const swapPct = data.swap_total > 0 ? (data.swap_used / data.swap_total) * 100 : 0;

  return (
    <Card title="Memory" icon={<Activity size={14} />}>
      <Bar label="RAM" used={data.mem_used} total={data.mem_total} />
      <div className="flex justify-between text-[11px] text-[var(--text-muted)]">
        <span>Cached: {formatBytes(data.mem_cached)}</span>
        <span>{memPct.toFixed(1)}% used</span>
      </div>
      {data.swap_total > 0 && (
        <>
          <Bar label="Swap" used={data.swap_used} total={data.swap_total} />
          <div className="flex justify-between text-[11px] text-[var(--text-muted)]">
            <span>{formatBytes(data.swap_used)} / {formatBytes(data.swap_total)}</span>
            <span>{swapPct.toFixed(1)}%</span>
          </div>
        </>
      )}
    </Card>
  );
}

function NetworkCard({ data }: { data: MonitorData }) {
  return (
    <Card title="Network" icon={<Wifi size={14} />}>
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col gap-1">
          <span className="text-[10px] text-[var(--text-muted)] uppercase">Download</span>
          <span className="text-lg font-semibold text-[var(--accent)]">{formatRate(data.net_rx_rate)}</span>
          <span className="text-[10px] text-[var(--text-muted)]">Total: {formatBytes(data.net_rx)}</span>
        </div>
        <div className="flex flex-col gap-1">
          <span className="text-[10px] text-[var(--text-muted)] uppercase">Upload</span>
          <span className="text-lg font-semibold text-[var(--accent)]">{formatRate(data.net_tx_rate)}</span>
          <span className="text-[10px] text-[var(--text-muted)]">Total: {formatBytes(data.net_tx)}</span>
        </div>
      </div>
      {data.net_interfaces.length > 0 && (
        <div className="text-[10px] text-[var(--text-muted)] truncate">
          Interfaces: {data.net_interfaces.join(', ')}
        </div>
      )}
    </Card>
  );
}

function DiskCard({ partitions }: { partitions: DiskPartition[] }) {
  return (
    <Card title="Disk" icon={<HardDrive size={14} />}>
      <div className="flex flex-col gap-2.5">
        {partitions.map((p) => (
          <div key={p.mount} className="flex flex-col gap-1">
            <Bar label={`${p.mount} (${p.fs_type})`} used={p.used} total={p.total} />
            {(p.read_rate > 0 || p.write_rate > 0) && (
              <div className="flex gap-3 text-[10px] text-[var(--text-muted)]">
                <span>R: {formatRate(p.read_rate)}</span>
                <span>W: {formatRate(p.write_rate)}</span>
              </div>
            )}
          </div>
        ))}
        {partitions.length === 0 && (
          <span className="text-xs text-[var(--text-muted)]">No disk data available</span>
        )}
      </div>
    </Card>
  );
}

function ProcessTable({ title, processes, sortBy }: { title: string; processes: ProcessInfo[]; sortBy: 'cpu' | 'memory' }) {
  return (
    <Card title={title} icon={<Activity size={14} />}>
      <div className="grid gap-1 text-[11px]">
        <div className="grid grid-cols-[52px_64px_1fr] gap-2 pb-1 border-b border-[var(--border)]" style={{ color: 'var(--text-muted)' }}>
          <span>PID</span>
          <span>{sortBy === 'cpu' ? 'CPU' : 'MEM'}</span>
          <span>Command</span>
        </div>
        {processes.slice(0, 6).map((proc) => (
          <div key={String(proc.pid) + '-' + proc.command + '-' + sortBy} className="grid grid-cols-[52px_64px_1fr] gap-2 items-center min-w-0">
            <span className="font-mono" style={{ color: 'var(--text-muted)' }}>{proc.pid}</span>
            <span className="font-mono" style={{ color: usageColor(sortBy === 'cpu' ? proc.cpu : proc.memory) }}>
              {(sortBy === 'cpu' ? proc.cpu : proc.memory).toFixed(1)}%
            </span>
            <span className="truncate" title={proc.args} style={{ color: 'var(--text-primary)' }}>{proc.command || proc.args}</span>
          </div>
        ))}
        {processes.length === 0 && (
          <span className="text-xs" style={{ color: 'var(--text-muted)' }}>No process data available</span>
        )}
      </div>
    </Card>
  );
}

function GpuCard({ gpu }: { gpu: GpuData }) {
  const memPct = gpu.memory_total > 0 ? (gpu.memory_used / gpu.memory_total) * 100 : 0;

  return (
    <Card title="GPU" icon={<Zap size={14} />}>
      <div className="text-xs font-medium text-[var(--text-primary)] mb-1 truncate">{gpu.name}</div>
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-2">
            <CircularGauge value={gpu.utilization} size={56} strokeWidth={5} />
            <div className="flex flex-col gap-0.5">
              <span className="text-[10px] text-[var(--text-muted)]">Utilization</span>
              <span className="text-xs text-[var(--text-primary)]">{gpu.utilization.toFixed(1)}%</span>
            </div>
          </div>
        </div>
        <div className="flex flex-col gap-1.5 text-xs">
          <div className="flex items-center gap-1.5">
            <Thermometer size={12} className="text-[var(--text-muted)]" />
            <span className="text-[var(--text-muted)]">Temp:</span>
            <span style={{ color: usageColor(gpu.temperature) }}>{gpu.temperature} C</span>
          </div>
          <div className="flex items-center gap-1.5">
            <Zap size={12} className="text-[var(--text-muted)]" />
            <span className="text-[var(--text-muted)]">Power:</span>
            <span className="text-[var(--text-primary)]">{gpu.power} W</span>
          </div>
        </div>
      </div>
      <Bar label="VRAM" used={gpu.memory_used} total={gpu.memory_total} />
      <div className="text-[10px] text-[var(--text-muted)] text-right">{memPct.toFixed(1)}%</div>
    </Card>
  );
}

// ── Main Component ───────────────────────────────────────────────────

export function MonitorView() {
  const { tabs, activeTabId, updateTab } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);
  const sessionId = activeTab?.sessionId ?? null;

  const [data, setData] = useState<MonitorData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [sessionLoading, setSessionLoading] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const connectStartedRef = useRef(false);
  const fetchingRef = useRef(false);

  useEffect(() => {
    if (sessionId || !activeTab?.connectionId || connectStartedRef.current) return;

    connectStartedRef.current = true;
    setSessionLoading(true);
    setError(null);

    connectTerminal(activeTab.connectionId)
      .then((sid) => {
        updateTab(activeTab.id, { sessionId: sid });
      })
      .catch((e) => {
        connectStartedRef.current = false;
        setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => setSessionLoading(false));
  }, [activeTab?.connectionId, activeTab?.id, sessionId, updateTab]);

  const fetchData = useCallback(async () => {
    if (!sessionId) return;
    if (fetchingRef.current) return;
    fetchingRef.current = true;
    try {
      const result = await getMonitorData(sessionId);
      setData(result);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch monitor data');
    } finally {
      fetchingRef.current = false;
    }
  }, [sessionId]);

  useEffect(() => {
    if (!sessionId) {
      setData(null);
      setError(null);
      fetchingRef.current = false;
      return;
    }

    fetchData();
    intervalRef.current = setInterval(fetchData, 3000);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      fetchingRef.current = false;
    };
  }, [sessionId, fetchData]);

  if (!sessionId) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--text-muted)] text-sm">
        {sessionLoading ? 'Connecting SSH session...' : (error || 'No connection selected')}
      </div>
    );
  }

  if (error && !data) {
    return (
      <div className="flex items-center justify-center h-full flex-col gap-2">
        <span className="text-[var(--error)] text-sm">{error}</span>
        <button className="btn btn-secondary text-xs" onClick={fetchData}>Retry</button>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--text-muted)] text-sm">
        Loading monitor data...
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-4">
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 max-w-[1400px] mx-auto">
        <SystemInfoCard data={data} />
        <CpuCard data={data} />
        <MemoryCard data={data} />
        <NetworkCard data={data} />
        <DiskCard partitions={data.disk_partitions} />
        {data.gpu && <GpuCard gpu={data.gpu} />}
        <ProcessTable title="Top CPU Processes" processes={data.top_cpu_processes || []} sortBy="cpu" />
        <ProcessTable title="Top Memory Processes" processes={data.top_mem_processes || []} sortBy="memory" />
      </div>
    </div>
  );
}

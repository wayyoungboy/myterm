import { useState, useEffect, useCallback, useRef } from 'react';
import { useAppStore } from '../../stores/appStore';
import { getMonitorData } from '../../utils/tauri';
import type { MonitorData, DiskPartition, GpuData } from '../../types';
import { Cpu, HardDrive, Wifi, Server, Activity, Zap, ChevronRight, ChevronLeft } from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';

// ── Constants ────────────────────────────────────────────────────────

const SIDEBAR_WIDTH = 340;

const COLORS = {
  success: '#22c55e',
  warning: '#eab308',
  error: '#ef4444',
  accent: '#3b82f6',
  purple: '#a855f7',
  cyan: '#06b6d4',
  green: '#22c55e',
  yellow: '#eab308',
  red: '#ef4444',
};

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
  if (d > 0) parts.push(`${d}天`);
  if (h > 0) parts.push(`${h}小时`);
  parts.push(`${m}分钟`);
  return parts.join('');
}

function formatRate(bytesPerSec: number): string {
  if (bytesPerSec < 1024) return `${bytesPerSec.toFixed(0)} B/s`;
  if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
}

function usageColor(pct: number): string {
  if (pct < 50) return '#22c55e';
  if (pct < 80) return '#eab308';
  return '#ef4444';
}

// ── XTerminal-style CPU bar (colored blocks) ─────────────────────────

function CpuBlockBar({ usage }: { usage: number }) {
  const blocks = 20;
  const filled = Math.round((usage / 100) * blocks);
  return (
    <div className="flex gap-0.5">
      {Array.from({ length: blocks }).map((_, i) => {
        let color = '#1e1e1e'; // empty
        if (i < filled) {
          if (i < blocks * 0.5) color = '#22c55e';
          else if (i < blocks * 0.8) color = '#eab308';
          else color = '#ef4444';
        }
        return (
          <div
            key={i}
            className="w-2 h-3 rounded-sm"
            style={{ background: color }}
          />
        );
      })}
    </div>
  );
}

// ── History buffer ───────────────────────────────────────────────────

const MAX_HISTORY = 60;

interface HistoryPoint {
  t: number;
  cpu: number;
  netRx: number;
  netTx: number;
}

// ── Section wrapper ──────────────────────────────────────────────────

function Section({ title, icon, children }: {
  title: string; icon: React.ReactNode; children: React.ReactNode;
}) {
  return (
    <div className="px-4 py-3 border-b border-[var(--border)]">
      <div className="flex items-center gap-2 text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wide mb-3">
        {icon}
        {title}
      </div>
      {children}
    </div>
  );
}

// ── Chart tooltip ────────────────────────────────────────────────────

function ChartTooltip({ active, payload }: any) {
  if (!active || !payload?.length) return null;
  return (
    <div className="bg-[var(--bg-surface)] border border-[var(--border)] rounded-lg px-2.5 py-1.5 text-[11px] shadow-lg">
      {payload.map((p: any, i: number) => (
        <div key={i} style={{ color: p.color }}>{p.name}: {typeof p.value === 'number' ? p.value.toFixed(1) : p.value}</div>
      ))}
    </div>
  );
}

// ── Sections ─────────────────────────────────────────────────────────

function SystemSection({ data }: { data: MonitorData }) {
  const osLower = data.os_info.toLowerCase();
  let osIcon = '🐧';
  if (osLower.includes('ubuntu') || osLower.includes('debian')) osIcon = '🟠';
  else if (osLower.includes('centos') || osLower.includes('rhel')) osIcon = '🔴';
  else if (osLower.includes('alpine')) osIcon = '🏔️';
  else if (osLower.includes('darwin')) osIcon = '🍎';

  return (
    <Section title="系统" icon={<Server size={13} />}>
      <div className="flex items-center gap-3 mb-2">
        <span className="text-xl">{osIcon}</span>
        <div className="flex-1 min-w-0">
          <div className="text-sm font-medium truncate" style={{ color: 'var(--text-primary)' }}>{data.hostname}</div>
          <div className="text-xs truncate" style={{ color: 'var(--text-muted)' }}>{data.os_info}</div>
        </div>
      </div>
      <div className="flex justify-between text-xs" style={{ color: 'var(--text-muted)' }}>
        <span>时区 {Intl.DateTimeFormat().resolvedOptions().timeZone}</span>
        <span>运行时间 {formatUptime(data.uptime)}</span>
      </div>
    </Section>
  );
}

function CpuSection({ data, history }: { data: MonitorData; history: HistoryPoint[] }) {
  const chartData = history.map((h, i) => ({ name: i.toString(), cpu: h.cpu }));

  return (
    <Section title="CPU" icon={<Cpu size={13} />}>
      {/* Per-core block bars */}
      <div className="flex flex-col gap-1.5 mb-3">
        {data.cpu_cores.slice(0, 8).map((usage, i) => (
          <div key={i} className="flex items-center gap-2">
            <span className="text-[11px] w-4 text-right" style={{ color: 'var(--text-muted)' }}>{i}</span>
            <CpuBlockBar usage={usage} />
            <span className="text-[11px] w-10 text-right font-mono" style={{ color: 'var(--text-muted)' }}>{usage.toFixed(1)}%</span>
          </div>
        ))}
      </div>
      {data.cpu_cores.length > 8 && (
        <div className="text-[10px] mb-2" style={{ color: 'var(--text-muted)' }}>+{data.cpu_cores.length - 8} 更多核心</div>
      )}

      {/* Total usage */}
      <div className="flex items-center gap-3 mb-3">
        <span className="text-2xl font-bold" style={{ color: usageColor(data.cpu_usage) }}>
          {data.cpu_usage.toFixed(1)}%
        </span>
        <div className="flex-1 h-2 rounded-full overflow-hidden" style={{ background: 'var(--bg-surface)' }}>
          <div className="h-full rounded-full transition-all duration-500" style={{ width: `${Math.min(data.cpu_usage, 100)}%`, background: usageColor(data.cpu_usage) }} />
        </div>
      </div>

      {/* Load curve */}
      <div className="text-[10px] mb-1" style={{ color: 'var(--text-muted)' }}>负载曲线</div>
      <div className="h-16">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData}>
            <XAxis dataKey="name" hide />
            <YAxis domain={[0, 100]} hide />
            <Tooltip content={<ChartTooltip />} />
            <Line type="monotone" dataKey="cpu" name="CPU" stroke={COLORS.accent} strokeWidth={1.5} dot={false} isAnimationActive={false} />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </Section>
  );
}

function MemorySection({ data }: { data: MonitorData }) {
  const memPct = data.mem_total > 0 ? (data.mem_used / data.mem_total) * 100 : 0;
  const free = data.mem_total - data.mem_used - data.mem_cached;
  const swapPct = data.swap_total > 0 ? (data.swap_used / data.swap_total) * 100 : 0;

  const pieData = [
    { name: '已使用', value: data.mem_used, color: COLORS.error },
    { name: '缓存', value: data.mem_cached, color: COLORS.warning },
    { name: '空闲', value: Math.max(free, 0), color: COLORS.success },
  ];

  return (
    <Section title="内存" icon={<Activity size={13} />}>
      <div className="flex items-center gap-4 mb-3">
        {/* Ring chart */}
        <div className="relative shrink-0">
          <PieChart width={72} height={72}>
            <Pie data={pieData} cx={36} cy={36} innerRadius={22} outerRadius={30} dataKey="value" startAngle={90} endAngle={-270} isAnimationActive={false}>
              {pieData.map((entry, i) => <Cell key={i} fill={entry.color} />)}
            </Pie>
          </PieChart>
          <div className="absolute inset-0 flex items-center justify-center">
            <span className="text-xs font-bold" style={{ color: 'var(--text-primary)' }}>{memPct.toFixed(0)}%</span>
          </div>
        </div>
        {/* Legend */}
        <div className="flex flex-col gap-1 text-xs flex-1">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full shrink-0" style={{ background: COLORS.error }} />
            <span style={{ color: 'var(--text-muted)' }}>已使用</span>
            <span className="ml-auto font-medium" style={{ color: 'var(--text-primary)' }}>{formatBytes(data.mem_used)}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full shrink-0" style={{ background: COLORS.warning }} />
            <span style={{ color: 'var(--text-muted)' }}>缓存</span>
            <span className="ml-auto font-medium" style={{ color: 'var(--text-primary)' }}>{formatBytes(data.mem_cached)}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full shrink-0" style={{ background: COLORS.success }} />
            <span style={{ color: 'var(--text-muted)' }}>空闲</span>
            <span className="ml-auto font-medium" style={{ color: 'var(--text-primary)' }}>{formatBytes(Math.max(free, 0))}</span>
          </div>
        </div>
      </div>

      {/* Swap */}
      {data.swap_total > 0 && (
        <div className="mt-2 pt-2 border-t border-[var(--border)]">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-xs" style={{ color: 'var(--text-muted)' }}>Swap</span>
            <div className="flex-1 h-1.5 rounded-full overflow-hidden" style={{ background: 'var(--bg-surface)' }}>
              <div className="h-full rounded-full transition-all duration-500" style={{ width: `${Math.min(swapPct, 100)}%`, background: COLORS.purple }} />
            </div>
            <span className="text-[11px] w-10 text-right" style={{ color: 'var(--text-muted)' }}>{swapPct.toFixed(0)}%</span>
          </div>
          <div className="text-[10px]" style={{ color: 'var(--text-muted)' }}>
            {formatBytes(data.swap_used)} / {formatBytes(data.swap_total)}
          </div>
        </div>
      )}
    </Section>
  );
}

function NetworkSection({ data, history }: { data: MonitorData; history: HistoryPoint[] }) {
  const chartData = history.map((h, i) => ({
    name: i.toString(),
    '下载': h.netRx / 1024,
    '上传': h.netTx / 1024,
  }));

  return (
    <Section title="网络" icon={<Wifi size={13} />}>
      <div className="mb-3">
        <div className="text-[10px] mb-1" style={{ color: 'var(--text-muted)' }}>速度</div>
        <div className="h-16">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={chartData}>
              <XAxis dataKey="name" hide />
              <YAxis hide />
              <Tooltip content={<ChartTooltip />} />
              <Line type="monotone" dataKey="下载" stroke={COLORS.success} strokeWidth={1.5} dot={false} isAnimationActive={false} />
              <Line type="monotone" dataKey="上传" stroke={COLORS.accent} strokeWidth={1.5} dot={false} isAnimationActive={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <div className="flex items-center gap-1.5 mb-1">
            <span className="w-2 h-2 rounded-full" style={{ background: COLORS.success }} />
            <span className="text-xs" style={{ color: 'var(--text-muted)' }}>上传</span>
          </div>
          <div className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>{formatRate(data.net_tx_rate)}</div>
          <div className="text-[10px]" style={{ color: 'var(--text-muted)' }}>已用流量</div>
          <div className="text-xs font-medium" style={{ color: 'var(--text-primary)' }}>{formatBytes(data.net_tx)}</div>
        </div>
        <div>
          <div className="flex items-center gap-1.5 mb-1">
            <span className="w-2 h-2 rounded-full" style={{ background: COLORS.accent }} />
            <span className="text-xs" style={{ color: 'var(--text-muted)' }}>下载</span>
          </div>
          <div className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>{formatRate(data.net_rx_rate)}</div>
          <div className="text-[10px]" style={{ color: 'var(--text-muted)' }}>已用流量</div>
          <div className="text-xs font-medium" style={{ color: 'var(--text-primary)' }}>{formatBytes(data.net_rx)}</div>
        </div>
      </div>
      {data.net_interfaces.length > 0 && (
        <div className="text-[10px] mt-2 truncate" style={{ color: 'var(--text-muted)' }}>
          网卡 {data.net_interfaces.join(', ')}
        </div>
      )}
    </Section>
  );
}

function DiskSection({ partitions }: { partitions: DiskPartition[] }) {
  if (partitions.length === 0) return null;

  // Calculate total disk usage
  const totalUsed = partitions.reduce((sum, p) => sum + p.used, 0);
  const totalSize = partitions.reduce((sum, p) => sum + p.total, 0);

  return (
    <Section title="硬盘" icon={<HardDrive size={13} />}>
      {/* Total overview */}
      <div className="flex items-center gap-2 mb-3">
        <span className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
          {formatBytes(totalUsed)} / {formatBytes(totalSize)}
        </span>
      </div>

      {/* Per-partition */}
      <div className="flex flex-col gap-2">
        {partitions.map((p) => {
          const pct = p.total > 0 ? (p.used / p.total) * 100 : 0;
          return (
            <div key={p.mount} className="flex flex-col gap-1">
              <div className="flex items-center gap-2">
                <span className="text-xs w-16 truncate" style={{ color: 'var(--text-muted)' }} title={`${p.mount} (${p.fs_type})`}>
                  {p.mount}
                </span>
                <span className="text-[10px] w-10 truncate" style={{ color: 'var(--text-muted)' }}>{p.fs_type}</span>
                <div className="flex-1 h-1.5 rounded-full overflow-hidden" style={{ background: 'var(--bg-surface)' }}>
                  <div className="h-full rounded-full transition-all duration-500" style={{ width: `${Math.min(pct, 100)}%`, background: usageColor(pct) }} />
                </div>
                <span className="text-[11px] shrink-0 w-20 text-right" style={{ color: 'var(--text-muted)' }}>
                  {formatBytes(p.used)} / {formatBytes(p.total)}
                </span>
              </div>
              {(p.read_rate > 0 || p.write_rate > 0) && (
                <div className="flex gap-4 text-[10px] pl-[6.5rem]" style={{ color: 'var(--text-muted)' }}>
                  <span>读 {formatRate(p.read_rate)}</span>
                  <span>写 {formatRate(p.write_rate)}</span>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </Section>
  );
}

function GpuSection({ gpu }: { gpu: GpuData }) {
  return (
    <Section title="显卡" icon={<Zap size={13} />}>
      <div className="text-sm font-medium mb-2 truncate" style={{ color: 'var(--text-primary)' }}>{gpu.name}</div>
      <div className="flex items-center gap-3 mb-2">
        <span className="text-lg font-bold" style={{ color: usageColor(gpu.utilization) }}>
          {gpu.utilization.toFixed(0)}%
        </span>
        <div className="flex-1 h-2 rounded-full overflow-hidden" style={{ background: 'var(--bg-surface)' }}>
          <div className="h-full rounded-full transition-all duration-500" style={{ width: `${Math.min(gpu.utilization, 100)}%`, background: usageColor(gpu.utilization) }} />
        </div>
      </div>
      <div className="flex justify-between text-xs mb-2" style={{ color: 'var(--text-muted)' }}>
        <span>温度 {gpu.temperature}°C</span>
        <span>功耗 {gpu.power}W</span>
      </div>
      {gpu.memory_total > 0 && (
        <div className="flex items-center gap-2">
          <span className="text-xs" style={{ color: 'var(--text-muted)' }}>显存</span>
          <div className="flex-1 h-1.5 rounded-full overflow-hidden" style={{ background: 'var(--bg-surface)' }}>
            <div className="h-full rounded-full transition-all duration-500" style={{ width: `${(gpu.memory_used / gpu.memory_total) * 100}%`, background: COLORS.purple }} />
          </div>
          <span className="text-[11px] shrink-0" style={{ color: 'var(--text-muted)' }}>
            {formatBytes(gpu.memory_used)} / {formatBytes(gpu.memory_total)}
          </span>
        </div>
      )}
    </Section>
  );
}

// ── Main Sidebar ─────────────────────────────────────────────────────

export function MonitorSidebar() {
  const { tabs, activeTabId, monitorSidebarVisible, toggleMonitorSidebar } = useAppStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);
  const sessionId = activeTab?.sessionId ?? null;

  const [data, setData] = useState<MonitorData | null>(null);
  const historyRef = useRef<HistoryPoint[]>([]);
  const [history, setHistory] = useState<HistoryPoint[]>([]);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchData = useCallback(async () => {
    if (!sessionId) return;
    try {
      const result = await getMonitorData(sessionId);
      setData(result);
      const point: HistoryPoint = { t: Date.now(), cpu: result.cpu_usage, netRx: result.net_rx_rate, netTx: result.net_tx_rate };
      historyRef.current = [...historyRef.current.slice(-MAX_HISTORY + 1), point];
      setHistory([...historyRef.current]);
    } catch { /* ignore */ }
  }, [sessionId]);

  useEffect(() => {
    if (!monitorSidebarVisible || !sessionId) {
      setData(null);
      historyRef.current = [];
      setHistory([]);
      if (intervalRef.current) { clearInterval(intervalRef.current); intervalRef.current = null; }
      return;
    }
    fetchData();
    intervalRef.current = setInterval(fetchData, 3000);
    return () => { if (intervalRef.current) { clearInterval(intervalRef.current); intervalRef.current = null; } };
  }, [monitorSidebarVisible, sessionId, fetchData]);

  const toggleBtn = (
    <button
      onClick={toggleMonitorSidebar}
      className="absolute top-1/2 -translate-y-1/2 z-30 w-5 h-14 flex items-center justify-center rounded-l-lg transition-colors"
      style={{
        right: monitorSidebarVisible ? SIDEBAR_WIDTH : '0',
        background: 'var(--bg-surface)',
        border: '1px solid var(--border)',
        borderRight: 'none',
        color: 'var(--text-secondary)',
      }}
      title={monitorSidebarVisible ? '收起监控' : '展开监控'}
    >
      {monitorSidebarVisible ? <ChevronRight size={14} /> : <ChevronLeft size={14} />}
    </button>
  );

  if (!monitorSidebarVisible) return toggleBtn;

  return (
    <>
      {toggleBtn}
      <div
        className="flex flex-col h-full border-l border-[var(--border)]"
        style={{ width: SIDEBAR_WIDTH, background: 'var(--bg-primary)', flexShrink: 0 }}
      >
        <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--border)]">
          <span className="text-sm font-semibold" style={{ color: 'var(--text-secondary)' }}>服务器监控</span>
          <button onClick={toggleMonitorSidebar} style={{ color: 'var(--text-muted)' }} className="hover:text-[var(--text-primary)] transition-colors">
            <ChevronRight size={14} />
          </button>
        </div>
        <div className="flex-1 overflow-y-auto">
          {!sessionId ? (
            <div className="flex items-center justify-center h-32 text-sm" style={{ color: 'var(--text-muted)' }}>请先连接服务器</div>
          ) : !data ? (
            <div className="flex items-center justify-center h-32 text-sm" style={{ color: 'var(--text-muted)' }}>加载中...</div>
          ) : (
            <>
              <SystemSection data={data} />
              <CpuSection data={data} history={history} />
              <MemorySection data={data} />
              <NetworkSection data={data} history={history} />
              <DiskSection partitions={data.disk_partitions} />
              {data.gpu && <GpuSection gpu={data.gpu} />}
            </>
          )}
        </div>
      </div>
    </>
  );
}

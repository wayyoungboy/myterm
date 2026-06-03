use ssh2::Session;
use std::io::Read;
use crate::db::models::{MonitorData, DiskPartition, GpuData, ProcessInfo};

const MONITOR_SCRIPT: &str = r#"#!/bin/sh
# System monitor script - pure POSIX sh
STATE_DIR="$HOME/.myterm"
mkdir -p "$STATE_DIR" 2>/dev/null || true

echo "===HOSTNAME==="
hostname 2>/dev/null || echo "unknown"
echo "===OS==="
uname -srm 2>/dev/null || echo "unknown"
echo "===UPTIME==="
cat /proc/uptime 2>/dev/null | awk '{print int($1)}' || echo "0"
echo "===LOAD==="
cat /proc/loadavg 2>/dev/null || echo "0 0 0"
echo "===CPU==="
nproc 2>/dev/null || echo "1"
echo "===CPU_USAGE==="
if [ -f /proc/stat ]; then
  CPU_PREV="$STATE_DIR/cpu.prev"
  CPU_NEXT="$STATE_DIR/cpu.next"
  : > "$CPU_NEXT"
  if command -v awk >/dev/null 2>&1; then
    awk -v prev="$CPU_PREV" -v next="$CPU_NEXT" '
      BEGIN {
        while ((getline line < prev) > 0) {
          split(line, p, " ")
          old_idle[p[1]] = p[2]
          old_total[p[1]] = p[3]
        }
      }
      /^cpu/ {
        name = $1
        user_total = $2 + $3
        system_total = $4 + $7 + $8 + $9
        idle_total = $5 + $6
        total = user_total + system_total + idle_total
        if (old_total[name] > 0) {
          delta_total = total - old_total[name]
          delta_idle = idle_total - old_idle[name]
          usage = delta_total > 0 ? (delta_total - delta_idle) * 100 / delta_total : 0
        } else {
          usage = total > 0 ? (total - idle_total) * 100 / total : 0
        }
        if (usage < 0) usage = 0
        if (usage > 100) usage = 100
        printf "%s %.2f\n", name, usage
        print name, idle_total, total >> next
      }
    ' /proc/stat
    mv "$CPU_NEXT" "$CPU_PREV" 2>/dev/null || true
  else
    head -1 /proc/stat
  fi
fi
echo "===MEMORY==="
if [ -f /proc/meminfo ]; then
  grep -E "^(MemTotal|MemFree|MemAvailable|Buffers|Cached|SReclaimable|SwapTotal|SwapFree):" /proc/meminfo 2>/dev/null
fi
echo "===NETWORK==="
if [ -f /proc/net/dev ]; then
  NET_PREV="$STATE_DIR/network.prev"
  now=$(date +%s 2>/dev/null || echo 0)
  rx_total=0
  tx_total=0
  interfaces=""
  while IFS= read -r line; do
    case "$line" in
      *:*)
        iface=$(printf '%s' "$line" | cut -d ':' -f 1 | tr -d '[:space:]')
        case "$iface" in
          ''|lo|docker0|veth*) continue ;;
        esac
        data=${line#*:}
        set -- $data
        rx=${1:-0}
        tx=${9:-0}
        rx_total=$((rx_total + rx))
        tx_total=$((tx_total + tx))
        if [ -n "$interfaces" ]; then
          interfaces="$interfaces,$iface"
        else
          interfaces="$iface"
        fi
        ;;
    esac
  done < /proc/net/dev
  rx_rate=0
  tx_rate=0
  if [ -f "$NET_PREV" ]; then
    read -r prev_time prev_rx prev_tx < "$NET_PREV"
    interval=$((now - ${prev_time:-0}))
    [ "$interval" -gt 0 ] 2>/dev/null || interval=1
    rx_delta=$((rx_total - ${prev_rx:-0}))
    tx_delta=$((tx_total - ${prev_tx:-0}))
    [ "$rx_delta" -ge 0 ] 2>/dev/null || rx_delta=0
    [ "$tx_delta" -ge 0 ] 2>/dev/null || tx_delta=0
    rx_rate=$((rx_delta / interval))
    tx_rate=$((tx_delta / interval))
  fi
  echo "$now $rx_total $tx_total" > "$NET_PREV"
  echo "$rx_total $tx_total $rx_rate $tx_rate $interfaces"
fi
echo "===DISK==="
lookup_fs_type() {
  target_mount="$1"
  while read -r fs mount fs_type _; do
    [ "$mount" = "$target_mount" ] || continue
    printf '%s\n' "$fs_type"
    return 0
  done < /proc/mounts 2>/dev/null
  printf 'unknown\n'
}
df -B1 -P 2>/dev/null | {
  IFS= read -r _
  while read -r fs size used available percent mount; do
    [ -n "$fs" ] || continue
    [ -n "$mount" ] || continue
    fs_type=$(lookup_fs_type "$mount")
    case "$fs_type" in
      tmpfs|devtmpfs) continue ;;
    esac
    case "$fs" in
      /dev/*) echo "$fs $fs_type $size $used $mount" ;;
    esac
  done
}
echo "===DISKSTATS==="
if [ -f /proc/diskstats ]; then
  DISK_PREV="$STATE_DIR/disk.prev"
  now=$(date +%s 2>/dev/null || echo 0)
  read_bytes=$(awk '/ (sd|vd|xvd|nvme)/ { total += $6 * 512 } END { printf "%d", total + 0 }' /proc/diskstats 2>/dev/null)
  write_bytes=$(awk '/ (sd|vd|xvd|nvme)/ { total += $10 * 512 } END { printf "%d", total + 0 }' /proc/diskstats 2>/dev/null)
  read_rate=0
  write_rate=0
  if [ -f "$DISK_PREV" ]; then
    read -r prev_time prev_read prev_write < "$DISK_PREV"
    interval=$((now - ${prev_time:-0}))
    [ "$interval" -gt 0 ] 2>/dev/null || interval=1
    read_delta=$((read_bytes - ${prev_read:-0}))
    write_delta=$((write_bytes - ${prev_write:-0}))
    [ "$read_delta" -ge 0 ] 2>/dev/null || read_delta=0
    [ "$write_delta" -ge 0 ] 2>/dev/null || write_delta=0
    read_rate=$((read_delta / interval))
    write_rate=$((write_delta / interval))
  fi
  echo "$now $read_bytes $write_bytes" > "$DISK_PREV"
  echo "$read_rate $write_rate"
fi
echo "===GPU==="
if command -v nvidia-smi >/dev/null 2>&1; then
  nvidia-smi --query-gpu=name,utilization.gpu,temperature.gpu,memory.total,memory.used,power.draw --format=csv,noheader,nounits 2>/dev/null || echo "none"
else
  echo "none"
fi
echo "===TOP_CPU==="
ps -eo pid=,pcpu=,pmem=,comm=,args= --sort=-pcpu 2>/dev/null | head -n 8 || true
echo "===TOP_MEM==="
ps -eo pid=,pcpu=,pmem=,comm=,args= --sort=-pmem 2>/dev/null | head -n 8 || true
echo "===END==="
"#;

pub fn fetch_monitor_data(session: &Session) -> Result<MonitorData, String> {
    let script = MONITOR_SCRIPT.replace("\r\n", "\n");
    let mut channel = session.channel_session()
        .map_err(|e| format!("Channel open failed: {}", e))?;
    channel.exec(&format!("sh -c '{}'", script.replace("'", "'\\''")))
        .map_err(|e| format!("Exec failed: {}", e))?;

    let mut output = String::new();
    channel.read_to_string(&mut output)
        .map_err(|e| format!("Read failed: {}", e))?;
    channel.wait_close().ok();

    parse_monitor_output(&output)
}

fn parse_monitor_output(output: &str) -> Result<MonitorData, String> {
    let sections: Vec<&str> = output.split("===END===").collect();
    let data = sections.first().ok_or("No data")?;

    let hostname = extract_section(data, "HOSTNAME").unwrap_or_default();
    let os_info = extract_section(data, "OS").unwrap_or_default();
    let uptime = extract_section(data, "UPTIME")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let load_str = extract_section(data, "LOAD").unwrap_or_default();
    let load_parts: Vec<f64> = load_str.split_whitespace()
        .take(3)
        .filter_map(|s| s.parse().ok())
        .collect();
    let load_avg = (
        load_parts.first().copied().unwrap_or(0.0),
        load_parts.get(1).copied().unwrap_or(0.0),
        load_parts.get(2).copied().unwrap_or(0.0),
    );

    let cpu_cores_count: usize = extract_section(data, "CPU")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let cpu_usage_section = extract_section(data, "CPU_USAGE").unwrap_or_default();
    let (cpu_usage, cpu_cores) = parse_cpu_usage(&cpu_usage_section, cpu_cores_count);

    let mem_section = extract_section(data, "MEMORY").unwrap_or_default();
    let mem_total = extract_mem_value(&mem_section, "MemTotal");
    let mem_free = extract_mem_value(&mem_section, "MemFree");
    let mem_available = extract_mem_value(&mem_section, "MemAvailable");
    let mem_buffers = extract_mem_value(&mem_section, "Buffers");
    let mem_cached = extract_mem_value(&mem_section, "Cached");
    let mem_reclaimable = extract_mem_value(&mem_section, "SReclaimable");
    let swap_total = extract_mem_value(&mem_section, "SwapTotal");
    let swap_free = extract_mem_value(&mem_section, "SwapFree");
    let mem_buff_cache = mem_buffers + mem_cached + mem_reclaimable;
    let mem_used = if mem_available > 0 {
        mem_total.saturating_sub(mem_available)
    } else {
        mem_total.saturating_sub(mem_free + mem_buff_cache)
    };

    let net_section = extract_section(data, "NETWORK").unwrap_or_default();
    let mut net_tokens = net_section.split_whitespace();
    let net_rx = net_tokens.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let net_tx = net_tokens.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let net_rx_rate = net_tokens.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let net_tx_rate = net_tokens.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let net_interfaces = net_tokens
        .next()
        .unwrap_or("")
        .split(',')
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
        .collect::<Vec<_>>();

    let disk_stats_section = extract_section(data, "DISKSTATS").unwrap_or_default();
    let disk_rates: Vec<u64> = disk_stats_section
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    let disk_read_rate = disk_rates.first().copied().unwrap_or(0);
    let disk_write_rate = disk_rates.get(1).copied().unwrap_or(0);

    let disk_section = extract_section(data, "DISK").unwrap_or_default();
    let disk_partitions: Vec<DiskPartition> = disk_section.lines()
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .filter_map(|(idx, line)| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                Some(DiskPartition {
                    mount: parts[4].to_string(),
                    fs_type: parts[1].to_string(),
                    total: parts[2].parse().unwrap_or(0),
                    used: parts[3].parse().unwrap_or(0),
                    read_rate: if idx == 0 { disk_read_rate } else { 0 },
                    write_rate: if idx == 0 { disk_write_rate } else { 0 },
                })
            } else {
                None
            }
        })
        .collect();

    let gpu_section = extract_section(data, "GPU").unwrap_or_default();
    let gpu = if gpu_section != "none" && !gpu_section.is_empty() {
        let parts: Vec<&str> = gpu_section.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 6 {
            Some(GpuData {
                name: parts[0].to_string(),
                utilization: parts[1].parse().unwrap_or(0.0),
                temperature: parts[2].parse().unwrap_or(0.0),
                memory_total: parts[3].parse::<u64>().unwrap_or(0) * 1024 * 1024,
                memory_used: parts[4].parse::<u64>().unwrap_or(0) * 1024 * 1024,
                power: parts[5].parse().unwrap_or(0.0),
            })
        } else {
            None
        }
    } else {
        None
    };

    let top_cpu_processes = parse_processes(&extract_section(data, "TOP_CPU").unwrap_or_default());
    let top_mem_processes = parse_processes(&extract_section(data, "TOP_MEM").unwrap_or_default());

    Ok(MonitorData {
        cpu_usage,
        cpu_cores,
        mem_total: mem_total * 1024,
        mem_used: mem_used * 1024,
        mem_cached: mem_buff_cache * 1024,
        swap_total: swap_total * 1024,
        swap_used: (swap_total - swap_free) * 1024,
        net_rx,
        net_tx,
        net_rx_rate,
        net_tx_rate,
        net_interfaces,
        disk_partitions,
        gpu,
        uptime,
        hostname,
        os_info,
        load_avg,
        top_cpu_processes,
        top_mem_processes,
    })
}

fn extract_section(data: &str, section: &str) -> Option<String> {
    let marker = format!("==={}===", section);
    let start_idx = data.find(&marker)?;
    let content_start = start_idx + marker.len();
    let remaining = data[content_start..].trim_start_matches('\n');
    let end_idx = remaining.find("\n===").unwrap_or(remaining.len());
    Some(remaining[..end_idx].trim().to_string())
}

fn extract_mem_value(mem_section: &str, key: &str) -> u64 {
    mem_section.lines()
        .find(|line| line.starts_with(key))
        .and_then(|line| {
            line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(0)
}

fn parse_cpu_usage(section: &str, expected_cores: usize) -> (f64, Vec<f64>) {
    let mut total = 0.0;
    let mut cores = Vec::new();

    for line in section.lines() {
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or("");
        let usage = parts
            .next()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0)
            .clamp(0.0, 100.0);

        if name == "cpu" {
            total = usage;
        } else if name.starts_with("cpu") {
            cores.push(usage);
        }
    }

    if cores.is_empty() && expected_cores > 0 {
        cores = vec![total; expected_cores];
    }

    (total, cores)
}

fn parse_processes(section: &str) -> Vec<ProcessInfo> {
    section
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let pid = parts.next()?.parse().ok()?;
            let cpu = parts.next()?.parse().unwrap_or(0.0);
            let memory = parts.next()?.parse().unwrap_or(0.0);
            let command = parts.next().unwrap_or("").to_string();
            let args = parts.collect::<Vec<_>>().join(" ");

            Some(ProcessInfo {
                pid,
                cpu,
                memory,
                command,
                args,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_multiline_section_between_markers() {
        let output = "===HOSTNAME===\nserver-1\n===MEMORY===\nMemTotal: 10 kB\nCached: 2 kB\n===END===";

        assert_eq!(extract_section(output, "HOSTNAME").as_deref(), Some("server-1"));
        assert_eq!(
            extract_section(output, "MEMORY").as_deref(),
            Some("MemTotal: 10 kB\nCached: 2 kB")
        );
    }

    #[test]
    fn parses_process_rows_with_args() {
        let rows = "123 4.5 1.2 sshd sshd: wayserver [priv]\n456 0.1 3.4 java java -jar app.jar";
        let processes = parse_processes(rows);

        assert_eq!(processes.len(), 2);
        assert_eq!(processes[0].pid, 123);
        assert_eq!(processes[0].cpu, 4.5);
        assert_eq!(processes[0].memory, 1.2);
        assert_eq!(processes[0].command, "sshd");
        assert_eq!(processes[0].args, "sshd: wayserver [priv]");
    }

    #[test]
    fn parses_xterminal_style_monitor_rates_and_disk_types() {
        let output = r#"===HOSTNAME===
server-1
===OS===
Linux 5.4 x86_64
===UPTIME===
120
===LOAD===
0.1 0.2 0.3 1/2 3
===CPU===
2
===CPU_USAGE===
cpu 12.50
cpu0 10.00
cpu1 15.00
===MEMORY===
MemTotal: 1000 kB
MemFree: 100 kB
Buffers: 50 kB
Cached: 200 kB
SReclaimable: 25 kB
SwapTotal: 500 kB
SwapFree: 400 kB
===NETWORK===
10000 20000 300 400 eth0,eth1
===DISKSTATS===
1024 2048
===DISK===
/dev/vda1 ext4 100000 45000 /
===GPU===
none
===TOP_CPU===
1 2.0 3.0 init /sbin/init
===TOP_MEM===
1 2.0 3.0 init /sbin/init
===END==="#;

        let data = parse_monitor_output(output).expect("monitor output should parse");

        assert_eq!(data.cpu_usage, 12.5);
        assert_eq!(data.cpu_cores, vec![10.0, 15.0]);
        assert_eq!(data.mem_used, 625 * 1024);
        assert_eq!(data.mem_cached, 275 * 1024);
        assert_eq!(data.net_rx_rate, 300);
        assert_eq!(data.net_tx_rate, 400);
        assert_eq!(data.net_interfaces, vec!["eth0".to_string(), "eth1".to_string()]);
        assert_eq!(data.disk_partitions[0].fs_type, "ext4");
        assert_eq!(data.disk_partitions[0].read_rate, 1024);
        assert_eq!(data.disk_partitions[0].write_rate, 2048);
    }
}

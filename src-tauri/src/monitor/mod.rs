use ssh2::Session;
use std::io::Read;
use crate::db::models::{MonitorData, DiskPartition, GpuData};

const MONITOR_SCRIPT: &str = r#"#!/bin/sh
# System monitor script - pure POSIX sh
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
  head -1 /proc/stat
fi
echo "===MEMORY==="
if [ -f /proc/meminfo ]; then
  grep -E "^(MemTotal|MemFree|MemAvailable|Cached|SwapTotal|SwapFree):" /proc/meminfo 2>/dev/null
fi
echo "===NETWORK==="
if [ -f /proc/net/dev ]; then
  grep -E "^\s*(eth|ens|wlan|enp)" /proc/net/dev 2>/dev/null | head -1
fi
echo "===DISK==="
df -B1 2>/dev/null | grep -E "^/dev/"
echo "===DISKSTATS==="
if [ -f /proc/diskstats ]; then
  grep -E "^\s+\d+\s+\d+\s+(sd|vd|nvme)" /proc/diskstats 2>/dev/null | head -5
fi
echo "===GPU==="
if command -v nvidia-smi >/dev/null 2>&1; then
  nvidia-smi --query-gpu=name,utilization.gpu,temperature.gpu,memory.total,memory.used,power.draw --format=csv,noheader,nounits 2>/dev/null || echo "none"
else
  echo "none"
fi
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

    let cpu_cores_count = extract_section(data, "CPU")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let cpu_line = extract_section(data, "CPU_USAGE").unwrap_or_default();
    let cpu_parts: Vec<u64> = cpu_line.split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let cpu_total: u64 = cpu_parts.iter().sum();
    let cpu_idle = cpu_parts.get(3).copied().unwrap_or(0);
    let cpu_usage = if cpu_total > 0 {
        ((cpu_total - cpu_idle) as f64 / cpu_total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    let mem_section = extract_section(data, "MEMORY").unwrap_or_default();
    let mem_total = extract_mem_value(&mem_section, "MemTotal");
    let mem_free = extract_mem_value(&mem_section, "MemFree");
    let mem_available = extract_mem_value(&mem_section, "MemAvailable");
    let mem_cached = extract_mem_value(&mem_section, "Cached");
    let swap_total = extract_mem_value(&mem_section, "SwapTotal");
    let swap_free = extract_mem_value(&mem_section, "SwapFree");
    let mem_used = if mem_available > 0 { mem_total - mem_available } else { mem_total - mem_free };

    let net_section = extract_section(data, "NETWORK").unwrap_or_default();
    let net_parts: Vec<u64> = net_section.split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    let net_rx = net_parts.first().copied().unwrap_or(0);
    let net_tx = net_parts.get(8).copied().unwrap_or(0);

    let disk_section = extract_section(data, "DISK").unwrap_or_default();
    let disk_partitions: Vec<DiskPartition> = disk_section.lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                Some(DiskPartition {
                    mount: parts[5].to_string(),
                    total: parts[1].parse().unwrap_or(0),
                    used: parts[2].parse().unwrap_or(0),
                    read_rate: 0,
                    write_rate: 0,
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

    Ok(MonitorData {
        cpu_usage,
        cpu_cores: vec![cpu_usage; cpu_cores_count as usize],
        mem_total: mem_total * 1024,
        mem_used: mem_used * 1024,
        mem_cached: mem_cached * 1024,
        swap_total: swap_total * 1024,
        swap_used: (swap_total - swap_free) * 1024,
        net_rx,
        net_tx,
        net_rx_rate: 0,
        net_tx_rate: 0,
        disk_partitions,
        gpu,
        uptime,
        hostname,
        os_info,
        load_avg,
    })
}

fn extract_section(data: &str, section: &str) -> Option<String> {
    let marker = format!("{}===", section);
    let start_idx = data.find(&marker)?;
    let content_start = start_idx + marker.len();
    let remaining = &data[content_start..];
    let end_idx = remaining.find('\n').unwrap_or(remaining.len());
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

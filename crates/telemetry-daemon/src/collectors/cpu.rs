use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuSnapshot {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
}

impl CpuSnapshot {
    pub fn total(&self) -> u64 {
        self.user + self.nice + self.system + self.idle + self.iowait + self.irq + self.softirq + self.steal
    }

    pub fn idle_total(&self) -> u64 {
        self.idle + self.iowait
    }
}

pub fn parse_proc_stat<R: Read>(reader: R) -> Option<CpuSnapshot> {
    let buf_reader = BufReader::new(reader);
    for line in buf_reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => return None,
        };
        if line.starts_with("cpu ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let user: u64 = parts[1].parse().ok()?;
                let nice: u64 = parts[2].parse().ok()?;
                let system: u64 = parts[3].parse().ok()?;
                let idle: u64 = parts[4].parse().ok()?;
                let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
                let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
                let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
                let steal: u64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);
                return Some(CpuSnapshot {
                    user,
                    nice,
                    system,
                    idle,
                    iowait,
                    irq,
                    softirq,
                    steal,
                });
            }
        }
    }
    None
}

pub fn calculate_cpu_usage(prev: &CpuSnapshot, curr: &CpuSnapshot) -> f32 {
    let prev_total = prev.total();
    let curr_total = curr.total();
    let prev_idle = prev.idle_total();
    let curr_idle = curr.idle_total();

    let total_delta = curr_total.saturating_sub(prev_total);
    let idle_delta = curr_idle.saturating_sub(prev_idle);

    if total_delta == 0 {
        0.0
    } else {
        let active_delta = total_delta.saturating_sub(idle_delta);
        (active_delta as f32 / total_delta as f32) * 100.0
    }
}

pub struct CpuCollector {
    proc_stat_path: String,
    last_snapshot: Option<CpuSnapshot>,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            proc_stat_path: "/proc/stat".to_string(),
            last_snapshot: None,
        }
    }

    pub fn with_path(path: &str) -> Self {
        Self {
            proc_stat_path: path.to_string(),
            last_snapshot: None,
        }
    }

    pub fn read_current_snapshot(&self) -> Option<CpuSnapshot> {
        match File::open(&self.proc_stat_path) {
            Ok(file) => parse_proc_stat(file),
            Err(e) => {
                warn!("Failed to read CPU stat file {}: {}", self.proc_stat_path, e);
                None
            }
        }
    }

    pub fn sample(&mut self) -> Option<f32> {
        let curr = self.read_current_snapshot()?;
        let usage = if let Some(prev) = &self.last_snapshot {
            calculate_cpu_usage(prev, &curr)
        } else {
            0.0
        };
        self.last_snapshot = Some(curr);
        Some(usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proc_stat() {
        let data = "cpu  101 20 30 400 5 6 7 8 0 0\ncpu0 50 10 15 200 2 3 3 4 0 0\n";
        let snap = parse_proc_stat(data.as_bytes()).unwrap();
        assert_eq!(snap.user, 101);
        assert_eq!(snap.idle, 400);
        assert_eq!(snap.total(), 101 + 20 + 30 + 400 + 5 + 6 + 7 + 8);
    }

    #[test]
    fn test_calculate_cpu_usage() {
        let snap1 = CpuSnapshot {
            user: 100,
            nice: 0,
            system: 0,
            idle: 100,
            iowait: 0,
            irq: 0,
            softirq: 0,
            steal: 0,
        };
        let snap2 = CpuSnapshot {
            user: 150,
            nice: 0,
            system: 0,
            idle: 150,
            iowait: 0,
            irq: 0,
            softirq: 0,
            steal: 0,
        };
        let usage = calculate_cpu_usage(&snap1, &snap2);
        assert!((usage - 50.0).abs() < f32::EPSILON);
    }
}

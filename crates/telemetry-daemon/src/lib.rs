pub mod collectors;
pub mod server;

use collectors::battery::BatteryCollector;
use collectors::cpu::CpuCollector;
use collectors::git::GitCollector;
use collectors::idle::IdleCollector;
use collectors::window::WindowCollector;
use companion_types::TelemetryEvent;

use server::{UdsServer, DEFAULT_SOCKET_PATH};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::info;

pub struct TelemetryDaemonConfig {
    pub socket_path: String,
    pub proc_stat_path: Option<String>,
    pub sys_power_path: Option<String>,
    pub git_config_path: Option<std::path::PathBuf>,
    pub cpu_interval: Duration,
    pub battery_interval: Duration,
    pub idle_interval: Duration,
    pub window_interval: Duration,
}

impl Default for TelemetryDaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: DEFAULT_SOCKET_PATH.to_string(),
            proc_stat_path: None,
            sys_power_path: None,
            git_config_path: None,
            cpu_interval: Duration::from_secs(10),
            battery_interval: Duration::from_secs(30),
            idle_interval: Duration::from_secs(5),
            window_interval: Duration::from_secs(2),
        }
    }
}

pub async fn run_daemon(config: TelemetryDaemonConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (bcast_tx, _) = broadcast::channel::<TelemetryEvent>(100);
    let (async_tx, mut async_rx) = mpsc::unbounded_channel::<TelemetryEvent>();

    // Forward internal mpsc events (e.g. from notify thread) to broadcast channel
    let bcast_tx_clone = bcast_tx.clone();
    tokio::spawn(async move {
        while let Some(event) = async_rx.recv().await {
            let _ = bcast_tx_clone.send(event);
        }
    });

    // 1. CPU Collector task
    let bcast_tx_cpu = bcast_tx.clone();
    let proc_path = config.proc_stat_path.clone();
    let cpu_interval = config.cpu_interval;
    tokio::spawn(async move {
        let mut collector = match proc_path {
            Some(p) => CpuCollector::with_path(&p),
            None => CpuCollector::new(),
        };
        // Initial sample
        collector.sample();
        let mut timer = tokio::time::interval(cpu_interval);
        loop {
            timer.tick().await;
            if let Some(usage) = collector.sample() {
                let _ = bcast_tx_cpu.send(TelemetryEvent::CpuUsage(usage));
            }
        }
    });

    // 2. Battery Collector task
    let bcast_tx_bat = bcast_tx.clone();
    let bat_path = config.sys_power_path.clone();
    let bat_interval = config.battery_interval;
    tokio::spawn(async move {
        let collector = match bat_path {
            Some(p) => BatteryCollector::with_path(&p),
            None => BatteryCollector::new(),
        };
        let mut timer = tokio::time::interval(bat_interval);
        loop {
            timer.tick().await;
            let level = collector.sample();
            let _ = bcast_tx_bat.send(TelemetryEvent::BatteryLevel(level));
        }
    });

    // 3. Idle Collector task
    let bcast_tx_idle = bcast_tx.clone();
    let idle_interval = config.idle_interval;
    tokio::spawn(async move {
        let collector = IdleCollector::new().await;
        let mut timer = tokio::time::interval(idle_interval);
        loop {
            timer.tick().await;
            let seconds = collector.sample().await;
            let _ = bcast_tx_idle.send(TelemetryEvent::IdleTime(seconds));
        }
    });

    // 4. Window Collector task
    let bcast_tx_win = bcast_tx.clone();
    let win_interval = config.window_interval;
    tokio::spawn(async move {
        let collector = WindowCollector::new().await;
        let mut timer = tokio::time::interval(win_interval);
        loop {
            timer.tick().await;
            let win_info = collector.sample().await;
            let _ = bcast_tx_win.send(TelemetryEvent::ActiveWindow {
                class: win_info.class,
                title: win_info.title,
            });
        }
    });

    // 5. Git Collector task
    let git_collector = GitCollector::new(async_tx, config.git_config_path.as_deref());
    let _watcher = git_collector.start_watching();

    info!("Starting UDS Server at {}", config.socket_path);
    let server = UdsServer::new(&config.socket_path, bcast_tx);
    server.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TelemetryDaemonConfig::default();
        assert_eq!(config.socket_path, DEFAULT_SOCKET_PATH);
    }
}

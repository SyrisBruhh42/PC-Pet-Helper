use companion_types::TelemetryEvent;
use std::fs;
use std::time::Duration;
use telemetry_daemon::{run_daemon, TelemetryDaemonConfig};
use tempfile::TempDir;
use tokio::io::AsyncBufReadExt;
use tokio::net::UnixStream;

#[tokio::test]
async fn test_full_daemon_pipeline_integration() {
    let temp_dir = TempDir::new().unwrap();
    let sock_path = temp_dir.path().join("telemetry.sock");
    let sock_str = sock_path.to_str().unwrap().to_string();

    // Mock proc stat file
    let proc_stat_path = temp_dir.path().join("proc_stat");
    fs::write(
        &proc_stat_path,
        "cpu  100 0 100 800 0 0 0 0 0 0\n",
    )
    .unwrap();

    // Mock battery file
    let sys_power_dir = temp_dir.path().join("power_supply");
    let bat0_dir = sys_power_dir.join("BAT0");
    fs::create_dir_all(&bat0_dir).unwrap();
    fs::write(bat0_dir.join("capacity"), "92\n").unwrap();

    let config = TelemetryDaemonConfig {
        socket_path: sock_str.clone(),
        proc_stat_path: Some(proc_stat_path.to_str().unwrap().to_string()),
        sys_power_path: Some(sys_power_dir.to_str().unwrap().to_string()),
        git_config_path: None,
        cpu_interval: Duration::from_millis(100),
        battery_interval: Duration::from_millis(100),
        idle_interval: Duration::from_millis(100),
        window_interval: Duration::from_millis(100),
    };

    let daemon_handle = tokio::spawn(async move {
        let _ = run_daemon(config).await;
    });

    // Wait for server socket creation
    tokio::time::sleep(Duration::from_millis(50)).await;

    let stream = UnixStream::connect(&sock_path).await.unwrap();
    let mut reader = tokio::io::BufReader::new(stream);

    let mut received_events = Vec::new();

    for _ in 0..3 {
        let mut line = String::new();
        let res = tokio::time::timeout(Duration::from_secs(2), reader.read_line(&mut line)).await;
        if let Ok(Ok(n)) = res {
            if n > 0 {
                if let Ok(event) = serde_json::from_str::<TelemetryEvent>(line.trim()) {
                    received_events.push(event);
                }
            }
        }
    }

    assert!(!received_events.is_empty(), "Should receive telemetry events over UDS");
    daemon_handle.abort();
}

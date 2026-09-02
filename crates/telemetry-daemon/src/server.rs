use companion_types::TelemetryEvent;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/desktop_pet_telemetry.sock";

pub struct UdsServer {
    socket_path: String,
    tx: broadcast::Sender<TelemetryEvent>,
}

impl UdsServer {
    pub fn new(socket_path: &str, tx: broadcast::Sender<TelemetryEvent>) -> Self {
        Self {
            socket_path: socket_path.to_string(),
            tx,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = std::path::Path::new(&self.socket_path);
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                warn!("Failed to remove existing socket file {}: {}", self.socket_path, e);
            }
        }

        let listener = UnixListener::bind(path)?;
        info!("UDS Telemetry Server bound to {}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let tx_incoming = self.tx.clone();
                    let mut rx = self.tx.subscribe();

                    tokio::spawn(async move {
                        let (read_half, mut write_half) = stream.into_split();

                        // Spawn task to read incoming telemetry events from client
                        tokio::spawn(async move {
                            let mut lines = BufReader::new(read_half).lines();
                            while let Ok(Some(line)) = lines.next_line().await {
                                let trimmed = line.trim();
                                if trimmed.is_empty() {
                                    continue;
                                }
                                if let Ok(event) = serde_json::from_str::<TelemetryEvent>(trimmed) {
                                    let _ = tx_incoming.send(event);
                                }
                            }
                        });

                        while let Ok(event) = rx.recv().await {
                            match serde_json::to_string(&event) {
                                Ok(json) => {
                                    let mut line = json;
                                    line.push('\n');
                                    if write_half.write_all(line.as_bytes()).await.is_err() {
                                        // Client disconnected or write error
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to serialize TelemetryEvent: {}", e);
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("UDS Listener accept error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn test_uds_server_broadcast() {
        let temp_dir = TempDir::new().unwrap();
        let sock_path = temp_dir.path().join("test.sock");
        let sock_str = sock_path.to_str().unwrap();

        let (tx, _rx) = broadcast::channel::<TelemetryEvent>(16);
        let server = UdsServer::new(sock_str, tx.clone());

        let server_handle = tokio::spawn(async move {
            let _ = server.run().await;
        });

        // Give server time to bind
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let stream = UnixStream::connect(&sock_path).await.unwrap();
        let mut reader = BufReader::new(stream);

        // Wait brief delay to ensure client handler task has subscribed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let test_event = TelemetryEvent::CpuUsage(50.0);
        tx.send(test_event.clone()).unwrap();

        let mut line = String::new();
        tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            reader.read_line(&mut line),
        )
        .await
        .expect("read_line timed out")
        .unwrap();

        assert_eq!(line.trim(), r#"{"type":"CpuUsage","payload":50.0}"#);

        server_handle.abort();
    }
}

use companion_types::{RenderFrameState, TelemetryEvent};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

pub const TELEMETRY_SOCKET_PATH: &str = "/tmp/desktop_pet_telemetry.sock";
pub const STATE_SOCKET_PATH: &str = "/tmp/desktop_pet_state.sock";

pub fn spawn_telemetry_client(
    socket_path: String,
    tx: mpsc::Sender<TelemetryEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if Path::new(&socket_path).exists() {
                match UnixStream::connect(&socket_path).await {
                    Ok(stream) => {
                        info!("Connected to telemetry socket at {}", socket_path);
                        let reader = BufReader::new(stream);
                        let mut lines = reader.lines();

                        while let Ok(Some(line)) = lines.next_line().await {
                            if line.trim().is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<TelemetryEvent>(&line) {
                                Ok(event) => {
                                    if tx.send(event).await.is_err() {
                                        error!("Telemetry receiver channel closed");
                                        return;
                                    }
                                }
                                Err(err) => {
                                    warn!("Failed to parse TelemetryEvent line: {err}");
                                }
                            }
                        }
                        warn!("Telemetry socket stream ended");
                    }
                    Err(err) => {
                        warn!("Failed to connect to telemetry socket: {err}");
                    }
                }
            }
            sleep(Duration::from_secs(2)).await;
        }
    })
}

pub async fn start_state_broadcast_server(
    socket_path: String,
    mut state_rx: broadcast::Receiver<RenderFrameState>,
) -> Result<tokio::task::JoinHandle<()>, std::io::Error> {
    if Path::new(&socket_path).exists() {
        let _ = fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(&socket_path)?;
    info!("State broadcast server listening on {}", socket_path);

    let (broadcaster_tx, _) = broadcast::channel::<Arc<String>>(100);

    let broadcaster_tx_clone = broadcaster_tx.clone();
    tokio::spawn(async move {
        while let Ok(frame) = state_rx.recv().await {
            if let Ok(json_line) = serde_json::to_string(&frame) {
                let line_with_newline = format!("{}\n", json_line);
                let _ = broadcaster_tx_clone.send(Arc::new(line_with_newline));
            }
        }
    });

    let handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut stream, _addr)) => {
                    let mut rx = broadcaster_tx.subscribe();
                    tokio::spawn(async move {
                        while let Ok(msg) = rx.recv().await {
                            if stream.write_all(msg.as_bytes()).await.is_err() {
                                break;
                            }
                        }
                    });
                }
                Err(err) => {
                    error!("Error accepting state client connection: {err}");
                    break;
                }
            }
        }
    });

    Ok(handle)
}

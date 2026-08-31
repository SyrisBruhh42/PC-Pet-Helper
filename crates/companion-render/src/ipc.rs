use companion_types::{RenderFrameState, TelemetryEvent};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::Mutex;

pub const STATE_SOCKET_PATH: &str = "/tmp/desktop_pet_state.sock";
pub const TELEMETRY_SOCKET_PATH: &str = "/tmp/desktop_pet_telemetry.sock";

pub struct IpcClient {
    state: Arc<Mutex<Option<RenderFrameState>>>,
    telemetry_tx: tokio::sync::mpsc::Sender<TelemetryEvent>,
}

impl IpcClient {
    pub fn new() -> (Self, tokio::sync::mpsc::Receiver<TelemetryEvent>) {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        (
            Self {
                state: Arc::new(Mutex::new(None)),
                telemetry_tx: tx,
            },
            rx,
        )
    }

    pub fn state_handle(&self) -> Arc<Mutex<Option<RenderFrameState>>> {
        Arc::clone(&self.state)
    }

    pub async fn start_state_listener(path: &str, state_handle: Arc<Mutex<Option<RenderFrameState>>>) {
        loop {
            if let Ok(stream) = UnixStream::connect(path).await {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break;
                    }
                    if let Ok(frame) = serde_json::from_str::<RenderFrameState>(&line) {
                        let mut lock = state_handle.lock().await;
                        *lock = Some(frame);
                    }
                    line.clear();
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    pub async fn start_telemetry_dispatcher(
        path: &str,
        mut rx: tokio::sync::mpsc::Receiver<TelemetryEvent>,
    ) {
        while let Some(event) = rx.recv().await {
            if let Ok(mut stream) = UnixStream::connect(path).await {
                if let Ok(json_str) = serde_json::to_string(&event) {
                    let mut payload = json_str.into_bytes();
                    payload.push(b'\n');
                    let _ = stream.write_all(&payload).await;
                }
            }
        }
    }

    pub fn send_click(&self, x: i32, y: i32) {
        let _ = self.telemetry_tx.try_send(TelemetryEvent::UserClick { x, y });
    }
}

mod dialogue;

use clap::{Parser, Subcommand};
use companion_types::{RenderFrameState, TelemetryEvent};
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::process::{Child, Command};
use tokio::signal;
use tracing::{error, info, warn};

const STATE_SOCKET_PATH: &str = "/tmp/desktop_pet_state.sock";
const TELEMETRY_SOCKET_PATH: &str = "/tmp/desktop_pet_telemetry.sock";

#[derive(Parser, Debug)]
#[command(name = "desktop-pet", author, version, about = "Desktop Pet CLI & Orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Spawns and supervises all daemons (telemetry, core, render) in development mode
    Start,
    /// Reads current vitals and state from state socket and prints a terminal dashboard
    Status,
    /// Injects a pet/feed interaction event into telemetry socket
    Feed,
    /// Injects a pet interaction event into telemetry socket
    Pet,
    /// Injects custom telemetry events directly into telemetry socket
    Emit {
        /// Raw JSON representation of a TelemetryEvent or preset shortcut
        #[arg(short, long)]
        json: Option<String>,

        /// Event type shortcut if json is not provided: cpu, idle, battery, window, commit, click
        #[arg(short, long)]
        event_type: Option<String>,

        /// Value for event type shortcut
        #[arg(short, long)]
        value: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => handle_start().await?,
        Commands::Status => handle_status().await?,
        Commands::Feed => handle_feed().await?,
        Commands::Pet => handle_pet().await?,
        Commands::Emit { json, event_type, value } => handle_emit(json, event_type, value).await?,
    }

    Ok(())
}

async fn handle_start() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Desktop Pet daemons in development mode...");

    let target_dir = find_target_dir();

    let daemons = [
        ("telemetry", "telemetry-daemon"),
        ("core", "companion-core"),
        ("render", "companion-render"),
    ];

    let mut children: Vec<(&str, Child)> = Vec::new();

    for &(prefix, bin_name) in &daemons {
        let mut cmd = if let Some(ref dir) = target_dir {
            let bin_path = dir.join("debug").join(bin_name);
            if bin_path.exists() {
                Command::new(bin_path)
            } else {
                let mut c = Command::new("cargo");
                c.args(["run", "-p", bin_name]);
                c
            }
        } else {
            let mut c = Command::new("cargo");
            c.args(["run", "-p", bin_name]);
            c
        };

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        info!("Spawning daemon [{}]...", prefix);
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to spawn daemon [{}]: {}", prefix, e);
                shutdown_children(&mut children).await;
                return Err(e.into());
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let tag = prefix.to_string();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    println!("[{}] {}", tag, line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let tag = prefix.to_string();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    eprintln!("[{}] {}", tag, line);
                }
            });
        }

        children.push((prefix, child));
    }

    info!("All 3 daemons spawned successfully. Press Ctrl+C to terminate.");

    let shutdown_signal = async {
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received SIGINT signal.");
            }
            _ = wait_for_sigterm() => {
                info!("Received SIGTERM signal.");
            }
        }
    };

    shutdown_signal.await;
    info!("Shutting down supervised daemons...");
    shutdown_children(&mut children).await;
    info!("All daemons terminated gracefully.");

    Ok(())
}

async fn wait_for_sigterm() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut stream) = signal(SignalKind::terminate()) {
            stream.recv().await;
        } else {
            let () = std::future::pending().await;
        }
    }
    #[cfg(not(unix))]
    {
        let () = std::future::pending().await;
    }
}

async fn shutdown_children(children: &mut Vec<(&str, Child)>) {
    for (prefix, child) in children.iter_mut().rev() {
        info!("Terminating daemon [{}]...", prefix);
        if let Err(e) = child.kill().await {
            warn!("Failed to kill daemon [{}]: {}", prefix, e);
        }
    }
}

fn find_target_dir() -> Option<PathBuf> {
    if let Ok(cargo_target) = env::var("CARGO_TARGET_DIR") {
        let path = PathBuf::from(cargo_target);
        if path.exists() {
            return Some(path);
        }
    }

    let mut current = env::current_dir().ok()?;
    loop {
        let target = current.join("target");
        if target.is_dir() {
            return Some(target);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

async fn handle_status() -> Result<(), Box<dyn std::error::Error>> {
    info!("Connecting to state socket: {}", STATE_SOCKET_PATH);
    let mut stream = match UnixStream::connect(STATE_SOCKET_PATH).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Error connecting to {}: {}\nIs companion-core running?",
                STATE_SOCKET_PATH, e
            );
            return Err(e.into());
        }
    };

    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line).await?;

    if bytes_read == 0 {
        eprintln!("Received empty response from state socket.");
        return Ok(());
    }

    let frame_state: RenderFrameState = serde_json::from_str(line.trim())?;
    print_dashboard(&frame_state);

    Ok(())
}

fn print_dashboard(state: &RenderFrameState) {
    println!("=== DESKTOP PET STATUS DASHBOARD ===");
    println!("State    : {:?}", state.state);
    println!("Dialogue : {}", state.dialogue.as_deref().unwrap_or("None"));
    println!("Timestamp: {} ms", state.timestamp_ms);
    println!("------------------------------------");
    println!("Vitals:");
    println!("  Energy   : {}", make_progress_bar(state.vitals.energy));
    println!("  Hunger   : {}", make_progress_bar(state.vitals.hunger));
    println!("  Focus    : {}", make_progress_bar(state.vitals.focus));
    println!("  Affection: {}", make_progress_bar(state.vitals.affection));
    println!("  Stress   : {}", make_progress_bar(state.vitals.stress));
    println!("====================================");
}

fn make_progress_bar(val: f32) -> String {
    let clamped = val.clamp(0.0, 100.0);
    let total_bars: usize = 20;
    let filled = ((clamped / 100.0) * total_bars as f32).round() as usize;
    let empty = total_bars.saturating_sub(filled);
    format!(
        "[{}{}] {:>5.1}%",
        "=".repeat(filled),
        " ".repeat(empty),
        clamped
    )
}

async fn send_telemetry_event(event: TelemetryEvent) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = match UnixStream::connect(TELEMETRY_SOCKET_PATH).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Error connecting to {}: {}\nIs telemetry daemon/core running?",
                TELEMETRY_SOCKET_PATH, e
            );
            return Err(e.into());
        }
    };

    let payload = serde_json::to_string(&event)?;
    stream.write_all(payload.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    println!("Successfully sent telemetry event to {}", TELEMETRY_SOCKET_PATH);
    Ok(())
}

async fn handle_feed() -> Result<(), Box<dyn std::error::Error>> {
    println!("Feeding Desktop Pet...");
    let event = TelemetryEvent::UserClick { x: 32, y: 32 };
    send_telemetry_event(event).await
}

async fn handle_pet() -> Result<(), Box<dyn std::error::Error>> {
    println!("Petting Desktop Pet...");
    let event = TelemetryEvent::UserClick { x: 32, y: 32 };
    send_telemetry_event(event).await
}

async fn handle_emit(
    json: Option<String>,
    event_type: Option<String>,
    value: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event: TelemetryEvent = if let Some(json_str) = json {
        serde_json::from_str(&json_str)?
    } else if let Some(kind) = event_type {
        match kind.to_lowercase().as_str() {
            "cpu" => {
                let v = value.unwrap_or_else(|| "50.0".to_string()).parse::<f32>()?;
                TelemetryEvent::CpuUsage(v)
            }
            "idle" => {
                let v = value.unwrap_or_else(|| "10".to_string()).parse::<u32>()?;
                TelemetryEvent::IdleTime(v)
            }
            "battery" => {
                let v = value.and_then(|s| s.parse::<u8>().ok());
                TelemetryEvent::BatteryLevel(v)
            }
            "click" => TelemetryEvent::UserClick { x: 32, y: 32 },
            "commit" => TelemetryEvent::DevCommit {
                repo: "desktop-pet".to_string(),
                hash: value.unwrap_or_else(|| "abc1234".to_string()),
            },
            "window" => TelemetryEvent::ActiveWindow {
                class: "Neovim".to_string(),
                title: value.unwrap_or_else(|| "main.rs".to_string()),
            },
            _ => {
                return Err(format!("Unknown event type: {}", kind).into());
            }
        }
    } else {
        TelemetryEvent::UserClick { x: 32, y: 32 }
    };

    send_telemetry_event(event).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_progress_bar() {
        let bar = make_progress_bar(50.0);
        assert!(bar.contains("=========="));
        assert!(bar.contains("50.0%"));
    }
}

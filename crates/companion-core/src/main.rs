use companion_core::db::Database;
use companion_core::fsm::FsmState;
use companion_core::ipc::{
    spawn_telemetry_client, start_state_broadcast_server, STATE_SOCKET_PATH,
    TELEMETRY_SOCKET_PATH,
};
use companion_core::vitals::VitalsEngine;
use companion_types::{RenderFrameState, TelemetryEvent};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, mpsc};
use tokio::time::interval;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting companion-core engine...");

    let db = match Database::open_default() {
        Ok(db) => db,
        Err(err) => {
            warn!("Could not open default SQLite DB: {err}");
            Database::open(":memory:")?
        }
    };

    let mut vitals_engine = VitalsEngine::new();
    let mut fsm = FsmState::new();

    // Load persisted state if exists
    if let Ok(Some((loaded_state, loaded_vitals, _))) = db.load_latest_state() {
        vitals_engine = VitalsEngine::from_vector(loaded_vitals);
        fsm.current_state = loaded_state;
        info!("Loaded state {:?} from SQLite", loaded_state);
    }

    let (telemetry_tx, mut telemetry_rx) = mpsc::channel::<TelemetryEvent>(100);
    let (state_tx, _) = broadcast::channel::<RenderFrameState>(100);

    spawn_telemetry_client(TELEMETRY_SOCKET_PATH.to_string(), telemetry_tx);
    start_state_broadcast_server(STATE_SOCKET_PATH.to_string(), state_tx.subscribe()).await?;

    let mut tick_interval = interval(Duration::from_millis(1000));
    let mut last_tick_time = Instant::now();
    let mut time_since_last_autosave = Duration::ZERO;
    let mut time_since_last_history = Duration::ZERO;

    loop {
        tick_interval.tick().await;

        let now_instant = Instant::now();
        let dt_duration = now_instant.duration_since(last_tick_time);
        last_tick_time = now_instant;
        let dt_secs = dt_duration.as_secs_f32();

        time_since_last_autosave += dt_duration;
        time_since_last_history += dt_duration;

        // Drain pending telemetry events
        let mut user_clicked = false;
        while let Ok(event) = telemetry_rx.try_recv() {
            if matches!(event, TelemetryEvent::UserClick { .. }) {
                user_clicked = true;
            }
            fsm.handle_event(&event);
        }

        if user_clicked {
            vitals_engine.apply_user_click();
        }

        let previous_fsm_state = fsm.current_state;

        // Update continuous vitals
        vitals_engine.apply_tick(dt_secs, fsm.current_state, fsm.cpu_usage);

        // Evaluate state transitions
        fsm.update(dt_duration, vitals_engine.vector());

        let state_changed = fsm.current_state != previous_fsm_state;

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // DB autosave every 30s or immediately on state transition
        if state_changed || time_since_last_autosave >= Duration::from_secs(30) {
            time_since_last_autosave = Duration::ZERO;
            if let Err(err) =
                db.save_pet_state(fsm.current_state, vitals_engine.vector(), None, timestamp_ms)
            {
                warn!("Failed to save pet state: {err}");
            }
        }

        // DB vitals_history snapshot every 60s
        if time_since_last_history >= Duration::from_secs(60) {
            time_since_last_history = Duration::ZERO;
            if let Err(err) = db.append_vitals_history(vitals_engine.vector(), timestamp_ms) {
                warn!("Failed to append vitals history: {err}");
            }
        }

        // Broadcast RenderFrameState
        let frame = RenderFrameState {
            state: fsm.current_state,
            vitals: vitals_engine.vector().clone(),
            dialogue: None,
            timestamp_ms,
        };

        let _ = state_tx.send(frame);
    }
}

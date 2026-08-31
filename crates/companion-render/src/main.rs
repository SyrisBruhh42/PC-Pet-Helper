use companion_render::graphics::physics::PetPhysics;
use companion_render::graphics::sprite_animator::SpriteAnimator;
use companion_render::ipc::{IpcClient, STATE_SOCKET_PATH, TELEMETRY_SOCKET_PATH};
use companion_render::wayland::layer_surface::WaylandLayerSurfaceManager;
use companion_types::PetState;
use std::time::Duration;
use tiny_skia::Pixmap;
use tokio::time::interval;

#[tokio::main]
async fn main() {
    println!("Starting Desktop Pet Companion Render Runtime...");

    let (ipc, telemetry_rx) = IpcClient::new();
    let state_handle = ipc.state_handle();

    // Spawn IPC background tasks
    tokio::spawn(IpcClient::start_state_listener(
        STATE_SOCKET_PATH,
        state_handle.clone(),
    ));
    tokio::spawn(IpcClient::start_telemetry_dispatcher(
        TELEMETRY_SOCKET_PATH,
        telemetry_rx,
    ));

    let screen_w = 1920.0;
    let screen_h = 96.0;

    let mut physics = PetPhysics::new(screen_w, screen_h);
    let mut layer_manager = WaylandLayerSurfaceManager::new(screen_w as u32, screen_h as u32);
    let animator = SpriteAnimator::new();

    let mut pixmap = Pixmap::new(screen_w as u32, screen_h as u32).unwrap();
    let mut frame_tick: u64 = 0;
    let mut ticker = interval(Duration::from_millis(16)); // ~60 FPS

    println!("Render loop initialized. Running...");

    for _ in 0..100 { // Loop or main event loop
        ticker.tick().await;

        let current_state = {
            let lock = state_handle.lock().await;
            lock.as_ref().map(|s| s.state).unwrap_or(PetState::Walking)
        };

        let current_dialogue = {
            let lock = state_handle.lock().await;
            lock.as_ref().and_then(|s| s.dialogue.clone())
        };

        // Update physics state
        physics.tick(current_state);

        // Update Wayland input region to pet bounding box
        layer_manager.update_input_region(&physics);

        // Render frame
        pixmap.fill(tiny_skia::Color::TRANSPARENT);
        animator.render_frame(
            &mut pixmap.as_mut(),
            current_state,
            frame_tick,
            physics.x,
            physics.y,
            current_dialogue.as_deref(),
        );

        frame_tick = frame_tick.wrapping_add(1);
    }

    println!("Render execution finished cleanly.");
}

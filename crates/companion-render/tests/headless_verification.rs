use companion_render::{
    IpcClient, PetPhysics, SpriteAnimator, WaylandLayerSurfaceManager,
};
use companion_types::{PetState, RenderFrameState, TelemetryEvent, VitalsVector};
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::time::{sleep, Duration};

#[test]
fn test_bounding_box_and_input_region_calculation() {
    let mut physics = PetPhysics::new(1920.0, 96.0);
    physics.x = 100.0;
    physics.y = 32.0;

    let (x1, y1, x2, y2) = physics.bounding_box();
    assert_eq!((x1, y1, x2, y2), (100, 32, 164, 96));

    let mut layer_manager = WaylandLayerSurfaceManager::new(1920, 96);
    let region = layer_manager.update_input_region(&physics);
    assert_eq!(region, (100, 32, 164, 96));

    assert!(physics.contains_point(120, 50));
    assert!(!physics.contains_point(200, 50));
    assert!(!physics.contains_point(50, 50));
}

#[test]
fn test_physics_patrol_wandering_and_border_bounce() {
    let mut physics = PetPhysics::new(100.0, 96.0); // 100px width surface, sprite is 64px, max_x = 36px
    physics.x = 35.0;
    physics.vx = 2.0;

    physics.tick(PetState::Walking);
    assert_eq!(physics.x, 36.0);
    assert_eq!(physics.vx, -2.0);

    physics.tick(PetState::Walking);
    assert_eq!(physics.x, 34.0);

    physics.x = 1.0;
    physics.vx = -2.0;
    physics.tick(PetState::Walking);
    assert_eq!(physics.x, 0.0);
    assert_eq!(physics.vx, 2.0);
}

#[test]
fn test_celebrating_jumping_physics() {
    let mut physics = PetPhysics::new(1920.0, 96.0);
    physics.y = physics.ground_level;
    physics.vy = -7.0; // Initial hop velocity

    physics.tick(PetState::Celebrating);
    assert!(physics.y < physics.ground_level); // Up in the air
    assert_eq!(physics.vy, -6.5); // Gravity applied (+0.5)
}

#[test]
fn test_sprite_animator_render() {
    let animator = SpriteAnimator::new();
    let mut pixmap = tiny_skia::Pixmap::new(200, 96).unwrap();

    // Render Idle frame
    animator.render_frame(&mut pixmap.as_mut(), PetState::Idle, 0, 10.0, 32.0, None);

    // Render Celebrating with dialogue
    animator.render_frame(
        &mut pixmap.as_mut(),
        PetState::Celebrating,
        15,
        10.0,
        32.0,
        Some("Hello World!"),
    );

    // Verify pixmap is not completely transparent
    let non_transparent_pixels = pixmap.pixels().iter().filter(|p| p.alpha() > 0).count();
    assert!(non_transparent_pixels > 0);
}

#[tokio::test]
async fn test_ipc_state_ingestion_and_telemetry_emission() {
    let dir = tempdir().unwrap();
    let state_sock = dir.path().join("state.sock");
    let telemetry_sock = dir.path().join("telemetry.sock");

    let state_sock_str = state_sock.to_str().unwrap();
    let telemetry_sock_str = telemetry_sock.to_str().unwrap();

    let state_listener = UnixListener::bind(&state_sock).unwrap();
    let telemetry_listener = UnixListener::bind(&telemetry_sock).unwrap();

    let (ipc, telemetry_rx) = IpcClient::new();
    let state_handle = ipc.state_handle();

    tokio::spawn(IpcClient::start_state_listener(
        state_sock_str.to_string().leak(),
        state_handle.clone(),
    ));

    tokio::spawn(IpcClient::start_telemetry_dispatcher(
        telemetry_sock_str.to_string().leak(),
        telemetry_rx,
    ));

    // Server accepts state listener connection and sends a frame
    let server_handle = tokio::spawn(async move {
        if let Ok((mut stream, _)) = state_listener.accept().await {
            let frame = RenderFrameState {
                state: PetState::Celebrating,
                vitals: VitalsVector {
                    energy: 1.0,
                    hunger: 0.0,
                    focus: 1.0,
                    affection: 1.0,
                    stress: 0.0,
                },
                dialogue: Some("Yay!".to_string()),
                timestamp_ms: 123456789,
            };
            let mut payload = serde_json::to_string(&frame).unwrap();
            payload.push('\n');
            stream.write_all(payload.as_bytes()).await.unwrap();
        }
    });

    sleep(Duration::from_millis(150)).await;

    // Verify state ingested
    {
        let lock = state_handle.lock().await;
        assert!(lock.is_some());
        let frame = lock.as_ref().unwrap();
        assert_eq!(frame.state, PetState::Celebrating);
        assert_eq!(frame.dialogue.as_deref(), Some("Yay!"));
    }

    // Trigger telemetry event (user click)
    ipc.send_click(42, 84);

    if let Ok((stream, _)) = telemetry_listener.accept().await {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let event: TelemetryEvent = serde_json::from_str(&line).unwrap();
        assert_eq!(event, TelemetryEvent::UserClick { x: 42, y: 84 });
    }

    server_handle.await.unwrap();
}

use companion_core::db::Database;
use companion_core::fsm::FsmState;
use companion_core::vitals::VitalsEngine;
use companion_types::{PetState, TelemetryEvent};
use std::time::Duration;

#[test]
fn test_mock_tick_stream_fsm_and_vitals() {
    let mut vitals_engine = VitalsEngine::new();
    let mut fsm = FsmState::new();

    // Initial state check
    assert_eq!(fsm.current_state, PetState::Idle);
    assert_eq!(vitals_engine.vector().energy, 100.0);
    assert_eq!(vitals_engine.vector().hunger, 0.0);

    // Stream event: Active window matching IDE -> WORKING state
    let event = TelemetryEvent::ActiveWindow {
        class: "Alacritty".to_string(),
        title: "nvim main.rs".to_string(),
    };
    fsm.handle_event(&event);
    fsm.update(Duration::from_secs(1), vitals_engine.vector());
    assert_eq!(fsm.current_state, PetState::Working);

    // Run 10 ticks while working
    for _ in 0..10 {
        vitals_engine.apply_tick(1.0, fsm.current_state, fsm.cpu_usage);
        fsm.update(Duration::from_secs(1), vitals_engine.vector());
    }

    // Energy drops: 100 - (0.05 * 10) = 99.5
    // Focus rises: 50 + (0.10 * 10) = 51.0
    // Hunger rises: 0 + (0.05 * 10) = 0.5
    assert!((vitals_engine.vector().energy - 99.5).abs() < 1e-3);
    assert!((vitals_engine.vector().focus - 51.0).abs() < 1e-3);
    assert!((vitals_engine.vector().hunger - 0.5).abs() < 1e-3);

    // Stream event: DevCommit -> CELEBRATING state
    fsm.handle_event(&TelemetryEvent::DevCommit {
        repo: "desktop-pet".to_string(),
        hash: "abcdef".to_string(),
    });
    assert_eq!(fsm.current_state, PetState::Celebrating);

    // Tick for 4 seconds -> still celebrating
    for _ in 0..4 {
        vitals_engine.apply_tick(1.0, fsm.current_state, fsm.cpu_usage);
        fsm.update(Duration::from_secs(1), vitals_engine.vector());
    }
    assert_eq!(fsm.current_state, PetState::Celebrating);

    // Tick 1 more second (total 5s) -> revert to WORKING
    vitals_engine.apply_tick(1.0, fsm.current_state, fsm.cpu_usage);
    fsm.update(Duration::from_secs(1), vitals_engine.vector());
    assert_eq!(fsm.current_state, PetState::Working);

    // UserClick event boost
    vitals_engine.apply_user_click();
    assert!((vitals_engine.vector().affection - 84.8).abs() < 1e-1); // 80 - 15*0.02 + 5.0 = 84.7
}

#[test]
fn test_mock_tick_stream_distressed_and_sleeping() {
    let mut vitals_engine = VitalsEngine::new();
    let mut fsm = FsmState::new();

    // High CPU for > 60 seconds
    fsm.handle_event(&TelemetryEvent::CpuUsage(90.0));
    for _ in 0..61 {
        vitals_engine.apply_tick(1.0, fsm.current_state, fsm.cpu_usage);
        fsm.update(Duration::from_secs(1), vitals_engine.vector());
    }
    assert_eq!(fsm.current_state, PetState::Distressed);

    // Lower CPU usage -> reverts to Idle
    fsm.handle_event(&TelemetryEvent::CpuUsage(20.0));
    vitals_engine.apply_tick(1.0, fsm.current_state, fsm.cpu_usage);
    fsm.update(Duration::from_secs(1), vitals_engine.vector());
    assert_eq!(fsm.current_state, PetState::Idle);

    // Idle time > 300 -> SLEEPING
    fsm.handle_event(&TelemetryEvent::IdleTime(305));
    vitals_engine.apply_tick(1.0, fsm.current_state, fsm.cpu_usage);
    fsm.update(Duration::from_secs(1), vitals_engine.vector());
    assert_eq!(fsm.current_state, PetState::Sleeping);
}

#[test]
fn test_persistence_integration() {
    let db = Database::open(":memory:").unwrap();
    let vitals_engine = VitalsEngine::new();
    let state = PetState::Working;

    db.save_pet_state(state, vitals_engine.vector(), None, 1000)
        .unwrap();
    db.append_vitals_history(vitals_engine.vector(), 1000)
        .unwrap();

    let loaded = db.load_latest_state().unwrap().unwrap();
    assert_eq!(loaded.0, PetState::Working);
    assert_eq!(loaded.1.energy, 100.0);
}

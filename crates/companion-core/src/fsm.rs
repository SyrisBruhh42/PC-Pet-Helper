use companion_types::{PetState, TelemetryEvent, VitalsVector};
use std::time::Duration;

pub const IDE_KEYWORDS: &[&str] = &[
    "code",
    "zed",
    "nvim",
    "neovim",
    "alacritty",
    "kitty",
    "wezterm",
];

#[derive(Debug, Clone)]
pub struct FsmState {
    pub current_state: PetState,
    pub previous_state: Option<PetState>,
    pub celebration_timer: Option<Duration>,
    pub distressed_high_cpu_duration: Duration,
    pub idle_time: u32,
    pub cpu_usage: f32,
    pub is_ide_active: bool,
}

impl Default for FsmState {
    fn default() -> Self {
        Self::new()
    }
}

impl FsmState {
    pub fn new() -> Self {
        Self {
            current_state: PetState::Idle,
            previous_state: None,
            celebration_timer: None,
            distressed_high_cpu_duration: Duration::ZERO,
            idle_time: 0,
            cpu_usage: 0.0,
            is_ide_active: false,
        }
    }

    pub fn handle_event(&mut self, event: &TelemetryEvent) -> bool {
        let state_before = self.current_state;

        match event {
            TelemetryEvent::CpuUsage(cpu) => {
                self.cpu_usage = *cpu;
            }
            TelemetryEvent::IdleTime(idle) => {
                self.idle_time = *idle;
            }
            TelemetryEvent::ActiveWindow { class, title } => {
                let class_lower = class.to_lowercase();
                let title_lower = title.to_lowercase();
                self.is_ide_active = IDE_KEYWORDS.iter().any(|kw| {
                    class_lower.contains(kw) || title_lower.contains(kw)
                });
            }
            TelemetryEvent::DevCommit { .. } => {
                if self.current_state != PetState::Celebrating {
                    self.previous_state = Some(self.current_state);
                    self.current_state = PetState::Celebrating;
                    self.celebration_timer = Some(Duration::from_secs(5));
                } else {
                    // Reset celebration timer back to 5s if another commit arrives
                    self.celebration_timer = Some(Duration::from_secs(5));
                }
            }
            _ => {}
        }

        self.current_state != state_before
    }

    pub fn update(&mut self, dt: Duration, vitals: &VitalsVector) -> bool {
        let initial_state = self.current_state;

        // Update high CPU duration tracker
        if self.cpu_usage > 85.0 {
            self.distressed_high_cpu_duration += dt;
        } else {
            self.distressed_high_cpu_duration = Duration::ZERO;
        }

        // Handle Celebrating timer logic
        if self.current_state == PetState::Celebrating {
            if let Some(timer) = self.celebration_timer {
                if timer <= dt {
                    self.celebration_timer = None;
                    let target = self.previous_state.take().unwrap_or(PetState::Idle);
                    self.current_state = target;
                } else {
                    self.celebration_timer = Some(timer - dt);
                }
            }
        }

        // FSM transition evaluation:
        // Priority 1: Sleep condition (* -> SLEEPING)
        if self.idle_time > 300 || vitals.energy < 10.0 {
            if self.current_state != PetState::Sleeping {
                if self.current_state == PetState::Celebrating {
                    // Sleep overrides celebration
                    self.celebration_timer = None;
                    self.previous_state = None;
                }
                self.current_state = PetState::Sleeping;
            }
        } else if self.current_state == PetState::Sleeping {
            // SLEEPING -> IDLE
            if self.idle_time < 10 && vitals.energy > 30.0 {
                self.current_state = PetState::Idle;
            }
        } else if self.current_state != PetState::Celebrating {
            // Non-sleeping, non-celebrating transitions
            if self.distressed_high_cpu_duration > Duration::from_secs(60) {
                self.current_state = PetState::Distressed;
            } else if self.current_state == PetState::Distressed {
                if self.cpu_usage <= 85.0 {
                    self.current_state = if self.is_ide_active {
                        PetState::Working
                    } else {
                        PetState::Idle
                    };
                }
            } else if self.is_ide_active {
                self.current_state = PetState::Working;
            } else {
                // If not sleeping, not celebrating, not distressed, not working (IDE active)
                self.current_state = PetState::Idle;
            }
        }

        self.current_state != initial_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_active_transition() {
        let mut fsm = FsmState::new();
        let vitals = VitalsVector::default();

        let event = TelemetryEvent::ActiveWindow {
            class: "Code".to_string(),
            title: "main.rs".to_string(),
        };
        fsm.handle_event(&event);
        fsm.update(Duration::from_secs(1), &vitals);
        assert_eq!(fsm.current_state, PetState::Working);

        let event2 = TelemetryEvent::ActiveWindow {
            class: "Firefox".to_string(),
            title: "Mozilla Firefox".to_string(),
        };
        fsm.handle_event(&event2);
        fsm.update(Duration::from_secs(1), &vitals);
        assert_eq!(fsm.current_state, PetState::Idle);
    }

    #[test]
    fn test_sleeping_transition() {
        let mut fsm = FsmState::new();
        let mut vitals = VitalsVector::default();

        fsm.handle_event(&TelemetryEvent::IdleTime(301));
        fsm.update(Duration::from_secs(1), &vitals);
        assert_eq!(fsm.current_state, PetState::Sleeping);

        // Cannot wake up if energy is low
        vitals.energy = 20.0;
        fsm.handle_event(&TelemetryEvent::IdleTime(5));
        fsm.update(Duration::from_secs(1), &vitals);
        assert_eq!(fsm.current_state, PetState::Sleeping);

        // Can wake up when energy > 30.0 and idle < 10
        vitals.energy = 35.0;
        fsm.update(Duration::from_secs(1), &vitals);
        assert_eq!(fsm.current_state, PetState::Idle);
    }

    #[test]
    fn test_celebrating_revert() {
        let mut fsm = FsmState::new();
        let vitals = VitalsVector::default();

        fsm.handle_event(&TelemetryEvent::ActiveWindow {
            class: "zed".to_string(),
            title: "project".to_string(),
        });
        fsm.update(Duration::from_secs(1), &vitals);
        assert_eq!(fsm.current_state, PetState::Working);

        fsm.handle_event(&TelemetryEvent::DevCommit {
            repo: "test".to_string(),
            hash: "abc".to_string(),
        });
        assert_eq!(fsm.current_state, PetState::Celebrating);

        fsm.update(Duration::from_secs(3), &vitals);
        assert_eq!(fsm.current_state, PetState::Celebrating);

        fsm.update(Duration::from_secs(3), &vitals);
        assert_eq!(fsm.current_state, PetState::Working);
    }

    #[test]
    fn test_distressed_transition() {
        let mut fsm = FsmState::new();
        let vitals = VitalsVector::default();

        fsm.handle_event(&TelemetryEvent::CpuUsage(90.0));
        for _ in 0..60 {
            fsm.update(Duration::from_secs(1), &vitals);
        }
        assert_ne!(fsm.current_state, PetState::Distressed);

        fsm.update(Duration::from_secs(1), &vitals);
        assert_eq!(fsm.current_state, PetState::Distressed);

        fsm.handle_event(&TelemetryEvent::CpuUsage(50.0));
        fsm.update(Duration::from_secs(1), &vitals);
        assert_eq!(fsm.current_state, PetState::Idle);
    }
}

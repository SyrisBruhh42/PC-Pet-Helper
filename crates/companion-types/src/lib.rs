use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum TelemetryEvent {
    CpuUsage(f32),
    IdleTime(u32), // Idle seconds
    BatteryLevel(Option<u8>),
    ActiveWindow {
        class: String,
        title: String,
    },
    DevCommit {
        repo: String,
        hash: String,
    },
    UserClick {
        x: i32,
        y: i32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PetState {
    Idle,
    Walking,
    Working,
    Sleeping,
    Distressed,
    Celebrating,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VitalsVector {
    pub energy: f32,
    pub hunger: f32,
    pub focus: f32,
    pub affection: f32,
    pub stress: f32,
}

impl Default for VitalsVector {
    fn default() -> Self {
        Self {
            energy: 100.0,
            hunger: 0.0,
            focus: 50.0,
            affection: 80.0,
            stress: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderFrameState {
    pub state: PetState,
    pub vitals: VitalsVector,
    pub dialogue: Option<String>,
    pub timestamp_ms: u64,
}

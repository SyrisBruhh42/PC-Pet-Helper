use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum TelemetryEvent {
    CpuUsage(f32),
    IdleTime(u32), // Idle seconds
    BatteryLevel(Option<u8>), // None if desktop / no battery, Some(0..=100)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderFrameState {
    pub state: PetState,
    pub vitals: VitalsVector,
    pub dialogue: Option<String>,
    pub timestamp_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_event_serialization() {
        let cpu_event = TelemetryEvent::CpuUsage(45.2);
        let json = serde_json::to_string(&cpu_event).unwrap();
        assert_eq!(json, r#"{"type":"CpuUsage","payload":45.2}"#);

        let dev_commit = TelemetryEvent::DevCommit {
            repo: "my-repo".to_string(),
            hash: "1234567".to_string(),
        };
        let commit_json = serde_json::to_string(&dev_commit).unwrap();
        assert_eq!(
            commit_json,
            r#"{"type":"DevCommit","payload":{"repo":"my-repo","hash":"1234567"}}"#
        );
    }
}

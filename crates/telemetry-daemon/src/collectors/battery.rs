use std::fs;
use std::path::Path;
use tracing::warn;

pub struct BatteryCollector {
    sys_path: String,
}

impl BatteryCollector {
    pub fn new() -> Self {
        Self {
            sys_path: "/sys/class/power_supply".to_string(),
        }
    }

    pub fn with_path(path: &str) -> Self {
        Self {
            sys_path: path.to_string(),
        }
    }

    pub fn sample(&self) -> Option<u8> {
        let dir = Path::new(&self.sys_path);
        if !dir.exists() || !dir.is_dir() {
            return None;
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read power_supply dir {}: {}", self.sys_path, e);
                return None;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if name_str.starts_with("BAT") {
                let capacity_file = path.join("capacity");
                if capacity_file.exists() {
                    if let Ok(content) = fs::read_to_string(&capacity_file) {
                        if let Ok(val) = content.trim().parse::<u8>() {
                            return Some(val.min(100));
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_battery_collector_with_mock() {
        let temp_dir = TempDir::new().unwrap();
        let bat0 = temp_dir.path().join("BAT0");
        fs::create_dir_all(&bat0).unwrap();
        fs::write(bat0.join("capacity"), "85\n").unwrap();

        let collector = BatteryCollector::with_path(temp_dir.path().to_str().unwrap());
        assert_eq!(collector.sample(), Some(85));
    }

    #[test]
    fn test_battery_collector_missing() {
        let temp_dir = TempDir::new().unwrap();
        let collector = BatteryCollector::with_path(temp_dir.path().to_str().unwrap());
        assert_eq!(collector.sample(), None);
    }
}

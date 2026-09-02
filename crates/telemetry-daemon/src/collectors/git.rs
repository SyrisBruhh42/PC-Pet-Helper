use companion_types::TelemetryEvent;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use tracing::{info, warn};

pub fn parse_git_log_head(git_dir: &Path) -> Option<TelemetryEvent> {
    // git_dir points to `.git` folder or its parent repository folder
    let repo_dir = if git_dir.file_name() == Some(std::ffi::OsStr::new(".git")) {
        git_dir.parent()?
    } else {
        git_dir
    };

    let repo_name = repo_dir.file_name()?.to_string_lossy().to_string();
    let head_log_path = repo_dir.join(".git").join("logs").join("HEAD");

    if !head_log_path.exists() {
        return None;
    }

    let file = fs::File::open(head_log_path).ok()?;
    let reader = BufReader::new(file);

    let last_line = reader.lines().filter_map(|l| l.ok()).last()?;
    let parts: Vec<&str> = last_line.split_whitespace().collect();
    if parts.len() >= 2 {
        let hash = parts[1].to_string();
        return Some(TelemetryEvent::DevCommit {
            repo: repo_name,
            hash,
        });
    }

    None
}

pub fn get_watch_roots(config_path: Option<&Path>) -> Vec<PathBuf> {
    if let Some(cfg) = config_path {
        if cfg.exists() {
            if let Ok(content) = fs::read_to_string(cfg) {
                if let Ok(roots) = serde_json::from_str::<Vec<String>>(&content) {
                    return roots.into_iter().map(PathBuf::from).collect();
                }
            }
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let mut default_roots = Vec::new();

    let proj_dir = PathBuf::from(&home).join("Projects");
    if proj_dir.exists() && proj_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(proj_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join(".git").exists() {
                    default_roots.push(path);
                }
            }
        }
    }

    let dev_dir = PathBuf::from(&home).join("dev");
    if dev_dir.exists() && dev_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dev_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join(".git").exists() {
                    default_roots.push(path);
                }
            }
        }
    }

    default_roots
}

pub struct GitCollector {
    tx: tokio::sync::mpsc::UnboundedSender<TelemetryEvent>,
    roots: Vec<PathBuf>,
}

impl GitCollector {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<TelemetryEvent>, config_path: Option<&Path>) -> Self {
        let roots = get_watch_roots(config_path);
        Self { tx, roots }
    }

    pub fn start_watching(self) -> Option<RecommendedWatcher> {
        let (std_tx, std_rx): (Sender<notify::Result<Event>>, Receiver<notify::Result<Event>>) = channel();

        let mut watcher = match RecommendedWatcher::new(std_tx, notify::Config::default()) {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to create notify watcher: {}", e);
                return None;
            }
        };

        for root in &self.roots {
            let head_log = root.join(".git").join("logs").join("HEAD");
            let path_to_watch = if head_log.exists() {
                head_log
            } else {
                root.clone()
            };

            if let Err(e) = watcher.watch(&path_to_watch, RecursiveMode::Recursive) {
                warn!("Failed to watch {}: {}", path_to_watch.display(), e);
            } else {
                info!("Watching git log: {}", path_to_watch.display());
            }
        }

        let tokio_tx = self.tx;
        let roots = self.roots;

        tokio::task::spawn_blocking(move || {
            while let Ok(res) = std_rx.recv() {
                match res {
                    Ok(event) => {
                        if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                            for path in event.paths {
                                for root in &roots {
                                    if path.starts_with(root) {
                                        if let Some(telemetry_event) = parse_git_log_head(root) {
                                            let _ = tokio_tx.send(telemetry_event);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => warn!("Watch error: {}", e),
                }
            }
        });

        Some(watcher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_git_log_head() {
        let temp_dir = TempDir::new().unwrap();
        let repo_dir = temp_dir.path().join("my-cool-project");
        let git_logs = repo_dir.join(".git").join("logs");
        fs::create_dir_all(&git_logs).unwrap();

        let head_log = git_logs.join("HEAD");
        fs::write(
            &head_log,
            "0000000000000000000000000000000000000000 a1b2c3d4e5f67890 User <user@example.com> 1600000000 +0000\tcommit: initial commit\n",
        )
        .unwrap();

        let event = parse_git_log_head(&repo_dir).unwrap();
        assert_eq!(
            event,
            TelemetryEvent::DevCommit {
                repo: "my-cool-project".to_string(),
                hash: "a1b2c3d4e5f67890".to_string(),
            }
        );
    }
}

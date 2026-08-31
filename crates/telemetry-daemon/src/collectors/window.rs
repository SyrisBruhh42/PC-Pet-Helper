use tracing::warn;
use zbus::Connection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveWindowInfo {
    pub class: String,
    pub title: String,
}

pub struct WindowCollector {
    conn: Option<Connection>,
}

impl WindowCollector {
    pub async fn new() -> Self {
        let conn = Connection::session().await.ok();
        Self { conn }
    }

    pub fn with_connection(conn: Connection) -> Self {
        Self { conn: Some(conn) }
    }

    pub fn stub() -> Self {
        Self { conn: None }
    }

    pub async fn sample(&self) -> ActiveWindowInfo {
        let conn = match &self.conn {
            Some(c) => c,
            None => {
                return ActiveWindowInfo {
                    class: "unknown".to_string(),
                    title: "unknown".to_string(),
                }
            }
        };

        // Try direct call to KWin or ActivityManager if available
        let active_win = conn
            .call_method(
                Some("org.kde.KWin"),
                "/KWin",
                Some("org.kde.KWin"),
                "activeWindow",
                &(),
            )
            .await;

        if let Ok(reply) = active_win {
            if let Ok((class, title)) = reply.body().deserialize::<(String, String)>() {
                return ActiveWindowInfo { class, title };
            }
        }

        // Fallback transient script or default stub output
        warn!("KWin active window query unavailable, falling back to stub");
        ActiveWindowInfo {
            class: "unknown".to_string(),
            title: "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_window_collector_stub() {
        let collector = WindowCollector::stub();
        let info = collector.sample().await;
        assert_eq!(info.class, "unknown");
        assert_eq!(info.title, "unknown");
    }
}

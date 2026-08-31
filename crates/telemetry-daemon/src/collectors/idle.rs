use tracing::warn;
use zbus::Connection;

pub struct IdleCollector {
    conn: Option<Connection>,
}

impl IdleCollector {
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

    pub async fn sample(&self) -> u32 {
        let conn = match &self.conn {
            Some(c) => c,
            None => return 0,
        };

        let result = conn
            .call_method(
                Some("org.freedesktop.ScreenSaver"),
                "/org/freedesktop/ScreenSaver",
                Some("org.freedesktop.ScreenSaver"),
                "GetSessionIdleTime",
                &(),
            )
            .await;

        match result {
            Ok(reply) => match reply.body().deserialize::<u32>() {
                Ok(seconds) => seconds,
                Err(e) => {
                    warn!("Failed to deserialize GetSessionIdleTime reply: {}", e);
                    0
                }
            },
            Err(e) => {
                warn!("DBus call org.freedesktop.ScreenSaver.GetSessionIdleTime failed: {}", e);
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_idle_collector_stub() {
        let collector = IdleCollector::stub();
        assert_eq!(collector.sample().await, 0);
    }
}

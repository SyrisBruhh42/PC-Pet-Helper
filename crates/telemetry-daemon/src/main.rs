use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Telemetry Daemon started");
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

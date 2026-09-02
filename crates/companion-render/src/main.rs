use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Companion Render started");
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

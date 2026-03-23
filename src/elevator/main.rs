mod controller;

use std::time::Duration;

use tokio::time::sleep;

use crate::controller::Controller;

#[tokio::main]
async fn main() {
    let c = Controller::new(1);
    let _ = c.request(0, 3).await;
    let _ = c.request(3, 8).await;
    let _ = c.request(8, 5).await;
    let _ = c.request(5, 10).await;
    let _ = c.request(10, 6).await;

    sleep(Duration::from_secs(10)).await;
}

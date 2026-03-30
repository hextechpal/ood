mod controller;

use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

use crate::controller::Controller;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let c = Controller::new(2);

    let _ = c.request(0, 3).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let _ = c.request(0, 2).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(3, 8).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(8, 4).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(4, 0).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(4, 6).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(6, 9).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(9, 2).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(2, 5).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(5, 1).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("State : {:?}", c.state().await);

    let _ = c.request(8, 5).await;
    let _ = c.request(5, 10).await;
    info!("State : {:?}", c.state().await);

    sleep(Duration::from_secs(10)).await;
}

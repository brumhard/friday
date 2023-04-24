#![warn(clippy::pedantic)]

#[cfg_attr(not(feature = "axum"), path = "own.rs")]
#[cfg_attr(feature = "axum", path = "axum.rs")]
mod server;

#[tokio::main]
async fn main() {
    server::main().await;
}

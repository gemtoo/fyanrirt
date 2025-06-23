#[macro_use]
extern crate log;
mod args;
mod engine;
mod misc;
mod tracing;

#[tokio::main]
async fn main() {
    tracing::init();
    engine::run().await.unwrap();
}

mod config;
mod handler;
mod model;
mod server;
mod service;
mod utils;

#[tokio::main]
async fn main() {
    server::run().await;
}

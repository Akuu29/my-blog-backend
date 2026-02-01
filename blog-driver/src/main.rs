mod config;
mod error;
mod handler;
mod middleware;
mod model;
mod server;
mod service;
mod utils;

#[tokio::main]
async fn main() {
    server::run().await;
}

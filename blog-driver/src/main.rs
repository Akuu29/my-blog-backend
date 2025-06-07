mod handler;
mod model;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    server::run().await;
}

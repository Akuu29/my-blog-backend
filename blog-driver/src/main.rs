mod handler;
mod model;
mod server;

#[tokio::main]
async fn main() {
    server::run().await;
}

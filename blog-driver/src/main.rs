mod handler;
mod setup;

#[tokio::main]
async fn main() {
    setup::create_server().await;
}

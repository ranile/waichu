use backend;

#[tokio::main]
async fn main() {
    backend::start().await;
}

use nightctl::cli;

#[tokio::main]
async fn main() {
    cli::run().await;
}

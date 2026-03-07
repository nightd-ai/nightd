use nightd::cli;

#[tokio::main]
async fn main() {
    // For now, just run CLI (will be updated in Phase 7)
    cli::run().await;
}

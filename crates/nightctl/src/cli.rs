use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nightctl")]
#[command(about = "CLI for the nightd daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the status of the daemon
    Status,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => match get_status().await {
            Ok(status) => println!("Status: {}", status),
            Err(e) => eprintln!("Error: {}", e),
        },
    }
}

async fn get_status() -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get("http://127.0.0.1:3000/status").await?;
    let status: serde_json::Value = response.json().await?;

    status["status"]
        .as_str()
        .map(|s: &str| s.to_string())
        .ok_or_else(|| "Invalid response format".into())
}

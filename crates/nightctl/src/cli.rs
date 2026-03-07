use clap::{Parser, Subcommand};
use reqwest;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "nightctl")]
#[command(about = "Nightctl CLI - command line interface for nightd")]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the status of the nightd daemon
    Status,
}

#[derive(Deserialize)]
struct StatusResponse {
    status: String,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            if let Err(e) = check_status().await {
                eprintln!("Error checking status: {}", e);
                std::process::exit(1);
            }
        }
    }
}

async fn check_status() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    match client
        .get("http://localhost:8000/status")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let data: StatusResponse = response.json().await?;
                println!("Daemon is running: {}", data.status);
                Ok(())
            } else {
                Err(format!("HTTP error: {}", response.status()).into())
            }
        }
        Err(e) if e.is_connect() => {
            println!("Daemon is stopped");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

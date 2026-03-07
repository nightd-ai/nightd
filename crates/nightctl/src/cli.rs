use clap::{Parser, Subcommand};
use reqwest;
use serde::{Deserialize, Serialize};

const DAEMON_URL: &str = "http://localhost:8000";

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
    /// Check the status of the nightd daemon and running tasks
    Status,

    /// Run a new task with a prompt for the AI agent
    Run {
        /// The prompt to send to the agent
        prompt: String,
    },

    /// List recent tasks
    List {
        /// Filter by status (pending, running, completed, failed)
        #[arg(short, long)]
        status: Option<String>,

        /// Number of tasks to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
}

#[derive(Deserialize)]
struct StatusResponse {
    status: String,
    running_tasks: i64,
    pending_tasks: i64,
    failed_tasks: i64,
}

#[derive(Serialize)]
struct CreateTaskRequest {
    prompt: String,
}

#[derive(Deserialize)]
struct CreateTaskResponse {
    task_id: String,
    status: String,
}

#[derive(Deserialize)]
struct TaskDto {
    id: String,
    prompt: String,
    status: String,
    pub _response: Option<String>,
    pub _exit_code: Option<i32>,
    created_at: String,
    pub _started_at: Option<String>,
    pub _completed_at: Option<String>,
}

#[derive(Deserialize)]
struct TaskListResponse {
    tasks: Vec<TaskDto>,
    total: usize,
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
        Commands::Run { prompt } => {
            if let Err(e) = run_task(&prompt).await {
                eprintln!("Error running task: {}", e);
                std::process::exit(1);
            }
        }
        Commands::List { status, limit } => {
            if let Err(e) = list_tasks(status.as_deref(), limit).await {
                eprintln!("Error listing tasks: {}", e);
                std::process::exit(1);
            }
        }
    }
}

async fn check_status() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    match client
        .get(format!("{}/status", DAEMON_URL))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let data: StatusResponse = response.json().await?;
                println!("Daemon is running: {}", data.status);
                println!("  Running tasks: {}", data.running_tasks);
                println!("  Pending tasks: {}", data.pending_tasks);
                println!("  Failed tasks: {}", data.failed_tasks);
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

async fn run_task(prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let request = CreateTaskRequest {
        prompt: prompt.to_string(),
    };

    let response = client
        .post(format!("{}/tasks", DAEMON_URL))
        .json(&request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if response.status().is_success() {
        let data: CreateTaskResponse = response.json().await?;
        println!("Task created: {}", data.task_id);
        println!("Status: {}", data.status);
        Ok(())
    } else {
        Err(format!("Failed to create task: HTTP {}", response.status()).into())
    }
}

async fn list_tasks(status: Option<&str>, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut url = format!("{}/tasks?limit={}", DAEMON_URL, limit);
    if let Some(s) = status {
        url.push_str(&format!("&status={}", s));
    }

    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if response.status().is_success() {
        let data: TaskListResponse = response.json().await?;

        if data.tasks.is_empty() {
            println!("No tasks found.");
            return Ok(());
        }

        // Print header
        println!("{:<36} {:<12} {:<20} Prompt", "ID", "Status", "Created");
        println!("{}", "-".repeat(100));

        // Print tasks
        for task in data.tasks {
            let prompt_preview = if task.prompt.len() > 40 {
                format!("{}...", &task.prompt[..37])
            } else {
                task.prompt.clone()
            };

            let created = task.created_at.chars().take(19).collect::<String>();

            println!(
                "{:<36} {:<12} {:<20} {}",
                task.id, task.status, created, prompt_preview
            );
        }

        println!("\nTotal: {}", data.total);
        Ok(())
    } else if response.status().as_u16() == 503 {
        println!("Daemon is not running.");
        Ok(())
    } else {
        Err(format!("Failed to list tasks: HTTP {}", response.status()).into())
    }
}

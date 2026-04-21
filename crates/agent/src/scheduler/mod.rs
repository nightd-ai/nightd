//! Session scheduler for managing agent execution.

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use dashmap::DashMap;
use sacp::{
    AcpAgentToClientNotification, AcpAgentToClientRequest, ContentBlock, Error as AcpError,
    ErrorCode, InitializeRequest, JrConnection, NewSessionRequest, PromptRequest, TextContent,
    VERSION,
};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::process::Child;
use tokio::task::JoinHandle;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing;
use uuid::Uuid;

use crate::{AgentId, Result, SessionHandle, SessionRequest, SessionStatus};

/// Agent runtime handle for managing an agent process or component.
#[derive(Debug)]
pub enum AgentRuntime {
    /// Process-based agent with a running child process.
    Process(Child),
    /// Component-based agent (for testing).
    Component(Box<dyn AgentComponent + Send + Sync>),
}

impl Clone for AgentRuntime {
    fn clone(&self) -> Self {
        match self {
            // Child cannot be cloned, so we return an error variant
            // In practice, this should not be called for processes
            AgentRuntime::Process(_) => {
                panic!("Cannot clone AgentRuntime::Process")
            }
            AgentRuntime::Component(_) => {
                panic!("Cannot clone AgentRuntime::Component")
            }
        }
    }
}

/// Trait for component-based agents.
///
/// This trait allows for testing agents without spawning external processes.
pub trait AgentComponent: std::fmt::Debug + Send + Sync {
    /// Handle an incoming request from the client.
    fn handle_request(&self, request: AcpAgentToClientRequest) -> Result<serde_json::Value>;
}

/// Scheduler for managing agent session execution.
#[derive(Debug, Clone)]
pub struct Scheduler {
    db: SqlitePool,
    agents: Arc<DashMap<AgentId, AgentRuntime>>,
    _background_task: Arc<JoinHandle<Result<()>>>,
}

impl Scheduler {
    /// Create a new scheduler and run migrations.
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        // Run migrations
        sqlx::migrate!("./src/db/migrations").run(&pool).await?;

        let agents: Arc<DashMap<AgentId, AgentRuntime>> = Arc::new(DashMap::new());
        let agents_clone = Arc::clone(&agents);

        // Spawn background task for session scheduling
        let background_task = tokio::spawn(async move { Self::scheduler_loop(agents_clone).await });

        Ok(Self {
            db: pool,
            agents,
            _background_task: Arc::new(background_task),
        })
    }

    /// Background scheduler loop to process pending sessions.
    async fn scheduler_loop(agents: Arc<DashMap<AgentId, AgentRuntime>>) -> Result<()> {
        // Create a local SQLite connection for the scheduler loop
        // We'll use a separate pool or connection for each execution task
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:nightd.db".to_string());
        let pool = SqlitePool::connect(&db_url).await?;

        loop {
            // Query for pending sessions ordered by priority (highest first), then by creation time
            let pending_sessions: Vec<(Vec<u8>, String, String, String)> = sqlx::query_as(
                r#"
                SELECT id, agent_id, workspace_path, prompt 
                FROM sessions 
                WHERE status = 'pending' 
                ORDER BY priority DESC, created_at ASC
                LIMIT 10
                "#,
            )
            .fetch_all(&pool)
            .await?;

            for (id_bytes, agent_id_str, workspace_path, prompt) in pending_sessions {
                let Ok(session_id) = Uuid::from_slice(&id_bytes) else {
                    tracing::error!("Invalid session ID in database");
                    continue;
                };

                let agent_id = AgentId::new(agent_id_str);
                let agents_clone = Arc::clone(&agents);
                let pool_clone = pool.clone();

                // Spawn a task to execute the session
                tokio::spawn(async move {
                    if let Err(e) = Self::execute_session(
                        session_id,
                        agent_id,
                        workspace_path.into(),
                        prompt,
                        agents_clone,
                        pool_clone,
                    )
                    .await
                    {
                        tracing::error!(session_id = %session_id, error = %e, "Session execution failed");
                    }
                });
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Execute a single session with ACP communication.
    async fn execute_session(
        session_id: Uuid,
        agent_id: AgentId,
        workspace_path: PathBuf,
        prompt: String,
        agents: Arc<DashMap<AgentId, AgentRuntime>>,
        pool: SqlitePool,
    ) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        // Update session status to 'running' and set started_at
        let result = sqlx::query(
            r#"
            UPDATE sessions 
            SET status = 'running', started_at = ?1 
            WHERE id = ?2 AND status = 'pending'
            "#,
        )
        .bind(now)
        .bind(session_id.as_bytes().as_slice())
        .execute(&pool)
        .await?;

        if result.rows_affected() == 0 {
            // Session was already processed or cancelled
            return Ok(());
        }

        tracing::info!(session_id = %session_id, agent_id = %agent_id, "Starting session execution");

        // Get or start the agent runtime
        let agent_runtime = Self::get_or_start_agent(&agent_id, &agents)?;

        // Execute the ACP session
        let execution_result: Result<String> = match agent_runtime {
            AgentRuntime::Process(mut child) => {
                // For process-based agents, connect via stdio
                let stdin = child
                    .stdin
                    .take()
                    .ok_or_else(|| crate::Error::Acp("Failed to get stdin".to_string()))?;
                let stdout = child
                    .stdout
                    .take()
                    .ok_or_else(|| crate::Error::Acp("Failed to get stdout".to_string()))?;

                // Convert tokio streams to futures-compatible streams
                let stdin = stdin.compat_write();
                let stdout = stdout.compat();

                // Create ACP connection
                let connection = JrConnection::new(stdin, stdout);

                Self::run_acp_session(connection, &workspace_path, &prompt).await
            }
            AgentRuntime::Component(component) => {
                // For component-based agents, we need to use a different approach
                // since they don't have stdio streams
                Self::run_component_session(component, &workspace_path, &prompt).await
            }
        };

        // Update session status based on execution result
        match execution_result {
            Ok(result) => {
                let finished_at = OffsetDateTime::now_utc();
                sqlx::query(
                    r#"
                    UPDATE sessions 
                    SET status = 'completed', result = ?1, finished_at = ?2 
                    WHERE id = ?3
                    "#,
                )
                .bind(&result)
                .bind(finished_at)
                .bind(session_id.as_bytes().as_slice())
                .execute(&pool)
                .await?;

                tracing::info!(session_id = %session_id, "Session completed successfully");
            }
            Err(e) => {
                let finished_at = OffsetDateTime::now_utc();
                let error_msg = format!("{}", e);
                sqlx::query(
                    r#"
                    UPDATE sessions 
                    SET status = 'failed', error = ?1, finished_at = ?2 
                    WHERE id = ?3
                    "#,
                )
                .bind(&error_msg)
                .bind(finished_at)
                .bind(session_id.as_bytes().as_slice())
                .execute(&pool)
                .await?;

                tracing::error!(session_id = %session_id, error = %error_msg, "Session failed");
            }
        }

        Ok(())
    }

    /// Get or start an agent runtime.
    fn get_or_start_agent(
        agent_id: &AgentId,
        agents: &Arc<DashMap<AgentId, AgentRuntime>>,
    ) -> Result<AgentRuntime> {
        // Check if agent exists
        if let Some(entry) = agents.get(agent_id) {
            match entry.value() {
                AgentRuntime::Process(_) => {
                    // Since we can't check the process status without &mut,
                    // we'll just return an error indicating the agent is already registered.
                    // The caller should handle checking if the process is still alive.
                    return Err(crate::Error::Acp(
                        "Agent already registered. Use is_agent_running to check status."
                            .to_string(),
                    ));
                }
                AgentRuntime::Component(_) => {
                    // Components are always "running"
                    return Err(crate::Error::Acp(
                        "Component-based agents cannot be restarted".to_string(),
                    ));
                }
            }
        }

        // Agent not found - we need to start it
        // For now, return an error indicating the agent needs to be registered first
        Err(crate::Error::Acp(format!(
            "Agent '{}' not found. Please register the agent first.",
            agent_id
        )))
    }

    /// Run an ACP session over a JrConnection.
    async fn run_acp_session<OB, IB, H>(
        connection: JrConnection<OB, IB, H>,
        workspace_path: &std::path::Path,
        prompt: &str,
    ) -> Result<String>
    where
        OB: futures::AsyncWrite + Send + Unpin + 'static,
        IB: futures::AsyncRead + Send + Unpin + 'static,
        H: sacp::JrHandler,
    {
        // Use a shared result variable
        let result_text = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let result_text_clone = result_text.clone();

        // Build the connection with handlers
        let connection = connection
            .on_receive_notification(async |notif: AcpAgentToClientNotification, _cx| {
                // Handle session notifications (streaming updates)
                tracing::debug!(?notif, "Received agent notification");
                Ok(())
            })
            .on_receive_request(async |req: AcpAgentToClientRequest, request_cx| {
                // Handle agent requests (e.g., file operations, tool calls)
                tracing::debug!(?req, "Received agent request");
                // For now, respond with an error as we don't implement these
                request_cx.respond_with_error(AcpError::new(ErrorCode::METHOD_NOT_FOUND))
            });

        // Run the ACP session directly (not in a spawned task to avoid Send issues)
        let run_result: std::result::Result<(), AcpError> = connection
            .with_client(async |cx| {
                // Initialize the connection
                let init_request = InitializeRequest {
                    protocol_version: VERSION,
                    client_capabilities: Default::default(),
                    client_info: None,
                    meta: None,
                };

                let init_response = cx.send_request(init_request).block_task().await?;

                tracing::debug!(?init_response, "ACP connection initialized");

                // Create a new session
                let new_session = NewSessionRequest {
                    cwd: workspace_path.to_path_buf(),
                    mcp_servers: vec![], // No MCP servers for now
                    meta: None,
                };

                let session_response = cx.send_request(new_session).block_task().await?;

                let session_id = session_response.session_id;
                tracing::debug!(session_id = %session_id, "Session created");

                // Send the prompt
                let prompt_request = PromptRequest {
                    session_id: session_id.clone(),
                    prompt: vec![ContentBlock::Text(TextContent {
                        text: prompt.to_string(),
                        annotations: None,
                        meta: None,
                    })],
                    meta: None,
                };

                let prompt_response = cx.send_request(prompt_request).block_task().await?;

                tracing::debug!(?prompt_response, "Prompt completed");

                // Collect the result from the session
                let text = format!(
                    "Session completed with stop reason: {:?}",
                    prompt_response.stop_reason
                );

                // Store the result
                if let Ok(mut guard) = result_text_clone.lock() {
                    *guard = text;
                }

                Ok(())
            })
            .await;

        if let Err(e) = run_result {
            return Err(crate::Error::Acp(format!("ACP session error: {}", e)));
        }

        // Extract the result
        let result = result_text
            .lock()
            .map_err(|_| crate::Error::Acp("Failed to lock result".to_string()))?;

        Ok(result.clone())
    }

    /// Run a session with a component-based agent.
    async fn run_component_session(
        _component: Box<dyn AgentComponent + Send + Sync>,
        _workspace_path: &PathBuf,
        _prompt: &str,
    ) -> Result<String> {
        // For component-based agents, we use the handle_request method directly
        // This is a simplified implementation
        // In a real implementation, we'd need to handle the full ACP flow

        // For now, return a placeholder result
        Ok("Component-based session completed (placeholder)".to_string())
    }

    /// Register a process-based agent.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the agent
    /// * `command` - Command to spawn the agent process
    pub async fn register_agent(&self, id: AgentId, command: impl AsRef<OsStr>) -> Result<()> {
        tracing::info!(agent_id = %id, "Registering process-based agent");

        // Start the agent process
        let child = tokio::process::Command::new(&command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.agents.insert(id, AgentRuntime::Process(child));

        Ok(())
    }

    /// Register a component-based agent (for testing).
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the agent
    /// * `component` - Component implementing the agent behavior
    pub async fn register_agent_component(
        &self,
        id: AgentId,
        component: impl AgentComponent + 'static,
    ) -> Result<()> {
        tracing::info!(agent_id = %id, "Registering component-based agent");

        self.agents
            .insert(id, AgentRuntime::Component(Box::new(component)));

        Ok(())
    }

    /// Schedule a new session.
    ///
    /// Creates a session in the database and returns a handle.
    ///
    /// # Arguments
    /// * `request` - Session request with workspace, prompt, agent, and priority
    pub async fn schedule(&self, request: SessionRequest) -> Result<SessionHandle> {
        let id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            INSERT INTO sessions (id, agent_id, workspace_path, prompt, status, created_at, priority)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(id.as_bytes().as_slice())
        .bind(&request.agent.0)
        .bind(request.workspace.to_string_lossy().as_ref())
        .bind(&request.prompt)
        .bind(SessionStatus::Pending)
        .bind(now)
        .bind(request.priority)
        .execute(&self.db)
        .await?;

        tracing::info!(session_id = %id, agent_id = %request.agent, "Scheduled new session");

        Ok(SessionHandle::with_db(id, self.db.clone()))
    }

    /// Get a session by ID.
    ///
    /// Returns `None` if the session doesn't exist.
    pub async fn get(&self, id: Uuid) -> Result<Option<SessionHandle>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT id FROM sessions WHERE id = ?1")
            .bind(id.as_bytes().as_slice())
            .fetch_optional(&self.db)
            .await?;

        Ok(row.map(|_| SessionHandle::with_db(id, self.db.clone())))
    }

    /// List all sessions.
    ///
    /// Returns handles for all sessions ordered by creation time (newest first).
    pub async fn list(&self) -> Result<Vec<SessionHandle>> {
        let rows: Vec<(Vec<u8>,)> =
            sqlx::query_as("SELECT id FROM sessions ORDER BY created_at DESC")
                .fetch_all(&self.db)
                .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(id_bytes,)| {
                Uuid::from_slice(&id_bytes)
                    .ok()
                    .map(|id| SessionHandle::with_db(id, self.db.clone()))
            })
            .collect())
    }

    /// List all pending sessions.
    ///
    /// Returns handles for pending sessions ordered by priority (highest first),
    /// then by creation time (oldest first).
    pub async fn list_pending(&self) -> Result<Vec<SessionHandle>> {
        let rows: Vec<(Vec<u8>,)> = sqlx::query_as(
            "SELECT id FROM sessions WHERE status = 'pending' ORDER BY priority DESC, created_at ASC"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(id_bytes,)| {
                Uuid::from_slice(&id_bytes)
                    .ok()
                    .map(|id| SessionHandle::with_db(id, self.db.clone()))
            })
            .collect())
    }

    /// List all running sessions.
    ///
    /// Returns handles for running sessions ordered by creation time.
    pub async fn list_running(&self) -> Result<Vec<SessionHandle>> {
        let rows: Vec<(Vec<u8>,)> = sqlx::query_as(
            "SELECT id FROM sessions WHERE status = 'running' ORDER BY created_at ASC",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(id_bytes,)| {
                Uuid::from_slice(&id_bytes)
                    .ok()
                    .map(|id| SessionHandle::with_db(id, self.db.clone()))
            })
            .collect())
    }

    /// List sessions by agent ID.
    ///
    /// Returns handles for all sessions associated with the given agent.
    pub async fn list_by_agent(&self, agent_id: &AgentId) -> Result<Vec<SessionHandle>> {
        let rows: Vec<(Vec<u8>,)> =
            sqlx::query_as("SELECT id FROM sessions WHERE agent_id = ?1 ORDER BY created_at DESC")
                .bind(&agent_id.0)
                .fetch_all(&self.db)
                .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(id_bytes,)| {
                Uuid::from_slice(&id_bytes)
                    .ok()
                    .map(|id| SessionHandle::with_db(id, self.db.clone()))
            })
            .collect())
    }

    /// Create a session builder for the fluent API.
    ///
    /// # Arguments
    /// * `workspace` - Path to the workspace directory
    /// * `prompt` - The prompt or task description
    pub fn session(
        &self,
        workspace: impl Into<PathBuf>,
        prompt: impl Into<String>,
    ) -> SessionBuilder<'_> {
        SessionBuilder::new(self, workspace.into(), prompt.into())
    }
}

/// Builder for creating sessions with a fluent API.
#[derive(Debug)]
pub struct SessionBuilder<'a> {
    scheduler: &'a Scheduler,
    workspace: PathBuf,
    prompt: String,
    agent: Option<AgentId>,
    priority: i32,
}

impl<'a> SessionBuilder<'a> {
    /// Create a new session builder.
    fn new(scheduler: &'a Scheduler, workspace: PathBuf, prompt: String) -> Self {
        Self {
            scheduler,
            workspace,
            prompt,
            agent: None,
            priority: 0,
        }
    }

    /// Set the agent ID for this session.
    pub fn agent(mut self, agent: impl Into<AgentId>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    /// Set the priority for this session.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Schedule the session and return a handle.
    pub async fn schedule(self) -> Result<SessionHandle> {
        let request = SessionRequest {
            workspace: self.workspace,
            prompt: self.prompt,
            agent: self.agent.unwrap_or_else(|| AgentId::new("default")),
            priority: self.priority,
        };

        self.scheduler.schedule(request).await
    }
}

// Re-export AgentComponent trait for convenience
pub use AgentComponent as Component;

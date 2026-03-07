use crate::acp::AcpClient;
use crate::db;
use sqlx::SqlitePool;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub(crate) enum WorkerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("ACP error: {0}")]
    Acp(#[from] crate::acp::AcpError),
}

pub(crate) type Result<T> = std::result::Result<T, WorkerError>;

#[derive(Clone)]
pub(crate) struct Worker {
    db_pool: SqlitePool,
    concurrency: usize,
    acp_client: AcpClient,
}

impl Worker {
    pub(crate) async fn new(db_pool: SqlitePool, concurrency: usize) -> Result<Self> {
        let acp_client = AcpClient::new()?;

        Ok(Self {
            db_pool,
            concurrency,
            acp_client,
        })
    }

    pub(crate) async fn run(&self) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(self.concurrency));

        info!("Worker started with concurrency: {}", self.concurrency);

        loop {
            // Acquire a permit to limit concurrency
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("Semaphore should not be closed");

            // Try to get a pending task
            match db::get_next_pending(&self.db_pool).await? {
                Some(task) => {
                    info!("Processing task {}: {}", task.id, task.prompt);

                    let client = self.acp_client.clone();
                    let pool = self.db_pool.clone();

                    // Spawn a new task to process
                    tokio::spawn(async move {
                        process_task(task, client, pool, permit).await;
                    });
                }
                None => {
                    // No pending tasks, sleep briefly
                    drop(permit); // Release the permit since we're not using it
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}

async fn process_task(
    task: crate::models::Task,
    client: AcpClient,
    pool: SqlitePool,
    _permit: OwnedSemaphorePermit,
) {
    let task_id = task.id;

    // Mark as running
    if let Err(e) = db::mark_task_running(&pool, &task_id).await {
        error!("Failed to mark task {} as running: {}", task_id, e);
        return;
    }

    info!("Task {} marked as running, executing prompt...", task_id);

    // Execute the prompt via ACP
    match client.execute_prompt(&task.prompt).await {
        Ok(response) => {
            info!("Task {} completed successfully", task_id);

            if let Err(e) = db::complete_task(&pool, &task_id, &response, 0).await {
                error!("Failed to mark task {} as completed: {}", task_id, e);
            }
        }
        Err(e) => {
            warn!("Task {} failed: {}", task_id, e);

            let error_msg = e.to_string();
            if let Err(e) = db::fail_task(&pool, &task_id, &error_msg).await {
                error!("Failed to mark task {} as failed: {}", task_id, e);
            }
        }
    }
}

// For testing - Worker with MockAcpClient
#[cfg(test)]
pub(crate) struct TestWorker {
    db_pool: SqlitePool,
    concurrency: usize,
}

#[cfg(test)]
impl TestWorker {
    pub(crate) fn new(db_pool: SqlitePool, concurrency: usize) -> Self {
        Self {
            db_pool,
            concurrency,
        }
    }

    pub(crate) async fn run_with_mock<F>(&self, mock_fn: F) -> Result<()>
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let mock_fn = Arc::new(mock_fn);

        loop {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("Semaphore should not be closed");

            match db::get_next_pending(&self.db_pool).await? {
                Some(task) => {
                    let pool = self.db_pool.clone();
                    let mock_fn = mock_fn.clone();
                    let task_id = task.id;
                    let prompt = task.prompt.clone();

                    tokio::spawn(async move {
                        db::mark_task_running(&pool, &task_id).await.ok();
                        let response = mock_fn(&prompt);
                        db::complete_task(&pool, &task_id, &response, 0).await.ok();
                        drop(permit);
                    });
                }
                None => {
                    drop(permit);
                    sleep(Duration::from_millis(100)).await;
                    break; // For tests, exit after processing
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_worker_processes_pending_tasks() {
        let pool = db::create_test_pool().await;

        // Create some tasks
        db::create_task(&pool, "task 1").await.unwrap();
        db::create_task(&pool, "task 2").await.unwrap();

        let worker = TestWorker::new(pool.clone(), 2);

        // Run with mock that echoes the prompt
        worker
            .run_with_mock(|prompt| format!("Response: {}", prompt))
            .await
            .unwrap();

        // Give tasks time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify tasks were processed
        let completed = db::count_tasks_by_status(&pool, crate::models::TaskStatus::Completed)
            .await
            .unwrap();
        assert_eq!(completed, 2);

        let pending = db::count_tasks_by_status(&pool, crate::models::TaskStatus::Pending)
            .await
            .unwrap();
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn test_concurrency_limits_execution() {
        let pool = db::create_test_pool().await;

        // Create many tasks
        for i in 0..10 {
            db::create_task(&pool, &format!("task {}", i))
                .await
                .unwrap();
        }

        let worker = TestWorker::new(pool.clone(), 2); // Only 2 concurrent

        let execution_count = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let max_concurrent = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let current_concurrent = Arc::new(std::sync::atomic::AtomicI32::new(0));

        let count_clone = execution_count.clone();
        let max_clone = max_concurrent.clone();
        let current_clone = current_concurrent.clone();

        worker
            .run_with_mock(move |prompt| {
                let current = current_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                let max = max_clone.load(std::sync::atomic::Ordering::SeqCst);
                if current > max {
                    max_clone.store(current, std::sync::atomic::Ordering::SeqCst);
                }

                count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                std::thread::sleep(std::time::Duration::from_millis(50));

                current_clone.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

                format!("Done: {}", prompt)
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify max concurrent never exceeded limit
        let max = max_concurrent.load(std::sync::atomic::Ordering::SeqCst);
        assert!(max <= 2, "Max concurrent was {}, should be <= 2", max);
    }
}

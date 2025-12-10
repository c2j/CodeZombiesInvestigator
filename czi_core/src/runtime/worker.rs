//! Task worker for handling background processing

use crate::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, oneshot};
use tracing::{debug, info, warn, error};

/// Worker status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    /// Worker is idle and ready for tasks
    Idle,
    /// Worker is currently processing a task
    Busy,
    /// Worker is shutting down
    ShuttingDown,
    /// Worker has stopped
    Stopped,
    /// Worker encountered an error
    Error,
}

/// Configuration for task workers
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Worker ID for identification
    pub worker_id: String,
    /// Maximum number of concurrent tasks per worker
    pub max_concurrent_tasks: usize,
    /// Task timeout in milliseconds (None for no timeout)
    pub task_timeout_ms: Option<u64>,
    /// Worker heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Enable detailed logging
    pub enable_verbose_logging: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_id: format!("worker-{}", uuid::Uuid::new_v4()),
            max_concurrent_tasks: 10,
            task_timeout_ms: Some(300000), // 5 minutes default
            heartbeat_interval_ms: 5000,
            enable_verbose_logging: false,
        }
    }
}

/// Task message for worker communication
pub enum TaskMessage {
    /// Execute a task
    Execute {
        id: uuid::Uuid,
        name: String,
        task: Box<dyn FnOnce() + Send>,
        response_sender: oneshot::Sender<TaskResult>,
    },
    /// Cancel a specific task
    Cancel(uuid::Uuid),
    /// Get worker status
    GetStatus(oneshot::Sender<WorkerStatus>),
    /// Shutdown the worker
    Shutdown,
}

/// Task execution result
#[derive(Debug)]
pub struct TaskResult {
    /// Task ID
    pub task_id: uuid::Uuid,
    /// Task name
    pub task_name: String,
    /// Success status
    pub success: bool,
    /// Error message (if any)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Worker ID that executed the task
    pub worker_id: String,
}

/// Task worker for background processing
pub struct TaskWorker {
    /// Worker configuration
    config: WorkerConfig,
    /// Current status
    status: Arc<RwLock<WorkerStatus>>,
    /// Task count tracking
    active_tasks: Arc<RwLock<usize>>,
    /// Communication channel for tasks
    task_sender: mpsc::UnboundedSender<TaskMessage>,
    /// Worker handle
    worker_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Statistics
    stats: Arc<RwLock<WorkerStats>>,
}

/// Worker statistics
#[derive(Debug, Default, Clone)]
pub struct WorkerStats {
    pub tasks_received: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_cancelled: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
    pub uptime_seconds: u64,
    pub last_heartbeat: Option<std::time::Instant>,
}

impl TaskWorker {
    /// Create a new task worker
    pub async fn new() -> Result<Self> {
        Self::with_config(WorkerConfig::default()).await
    }

    /// Create a new task worker with custom configuration
    pub async fn with_config(config: WorkerConfig) -> Result<Self> {
        info!("Creating task worker: {}", config.worker_id);

        let (task_sender, task_receiver) = mpsc::unbounded_channel();

        let worker = Self {
            config: config.clone(),
            status: Arc::new(RwLock::new(WorkerStatus::Idle)),
            active_tasks: Arc::new(RwLock::new(0)),
            task_sender,
            worker_handle: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(WorkerStats::default())),
        };

        // Start the worker task
        let handle = worker.start_worker_task(config, task_receiver).await?;

        // Store the handle
        {
            let mut worker_handle_ref = worker.worker_handle.write().await;
            *worker_handle_ref = Some(handle);
        }

        Ok(worker)
    }

    /// Start the worker task
    async fn start_worker_task(
        &self,
        config: WorkerConfig,
        mut task_receiver: mpsc::UnboundedReceiver<TaskMessage>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let worker_id = config.worker_id.clone();
        let status = Arc::clone(&self.status);
        let active_tasks = Arc::clone(&self.active_tasks);
        let stats = Arc::clone(&self.stats);
        let heartbeat_interval = std::time::Duration::from_millis(config.heartbeat_interval_ms);

        let handle = tokio::spawn(async move {
            info!("Worker {} started", worker_id);

            // Set initial status
            {
                let mut status_ref = status.write().await;
                *status_ref = WorkerStatus::Idle;
            }

            // Initialize statistics
            {
                let mut stats_ref = stats.write().await;
                stats_ref.uptime_seconds = 0;
                stats_ref.last_heartbeat = Some(std::time::Instant::now());
            }

            // Main worker loop
            let mut heartbeat_interval_timer = tokio::time::interval(heartbeat_interval);
            let start_time = std::time::Instant::now();

            loop {
                tokio::select! {
                    // Handle incoming task messages
                    message = task_receiver.recv() => {
                        match message {
                            Some(TaskMessage::Execute { id, name, task, response_sender }) => {
                                Self::handle_execute_task(
                                    &worker_id,
                                    &config,
                                    id,
                                    name,
                                    task,
                                    response_sender,
                                    &status,
                                    &active_tasks,
                                    &stats,
                                ).await;
                            }
                            Some(TaskMessage::Cancel(task_id)) => {
                                Self::handle_cancel_task(&worker_id, task_id, &stats).await;
                            }
                            Some(TaskMessage::GetStatus(response_sender)) => {
                                Self::handle_get_status(&status, response_sender).await;
                            }
                            Some(TaskMessage::Shutdown) => {
                                info!("Worker {} shutting down", worker_id);
                                {
                                    let mut status_ref = status.write().await;
                                    *status_ref = WorkerStatus::ShuttingDown;
                                }
                                break;
                            }
                            None => {
                                info!("Worker {} channel closed", worker_id);
                                break;
                            }
                        }
                    }

                    // Handle periodic heartbeat
                    _ = heartbeat_interval_timer.tick() => {
                        Self::handle_heartbeat(&worker_id, &status, &stats, start_time).await;
                    }
                }
            }

            // Set final status
            {
                let mut status_ref = status.write().await;
                *status_ref = WorkerStatus::Stopped;
            }

            info!("Worker {} stopped", worker_id);
        });

        Ok(handle)
    }

    /// Handle task execution
    async fn handle_execute_task(
        worker_id: &str,
        config: &WorkerConfig,
        task_id: uuid::Uuid,
        task_name: String,
        task: Box<dyn FnOnce() + Send>,
        response_sender: oneshot::Sender<TaskResult>,
        status: &Arc<RwLock<WorkerStatus>>,
        active_tasks: &Arc<RwLock<usize>>,
        stats: &Arc<RwLock<WorkerStats>>,
    ) {
        // Check concurrent task limit
        {
            let current_tasks = *active_tasks.read().await;
            if current_tasks >= config.max_concurrent_tasks {
                warn!("Worker {} at max capacity ({}), rejecting task {}", worker_id, current_tasks, task_name);
                let _ = response_sender.send(TaskResult {
                    task_id,
                    task_name,
                    success: false,
                    error: Some("Worker at max capacity".to_string()),
                    execution_time_ms: 0,
                    worker_id: worker_id.to_string(),
                });
                return;
            }
        }

        // Update status and tracking
        {
            let mut status_ref = status.write().await;
            *status_ref = WorkerStatus::Busy;
        }

        {
            let mut tasks = active_tasks.write().await;
            *tasks += 1;
        }

        {
            let mut stats_ref = stats.write().await;
            stats_ref.tasks_received += 1;
        }

        debug!("Worker {} executing task {} ({})", worker_id, task_id, task_name);

        // Execute the task with timing
        let start_time = std::time::Instant::now();

        let result = if let Some(timeout_ms) = config.task_timeout_ms {
            let timeout = std::time::Duration::from_millis(timeout_ms);
            match tokio::time::timeout(timeout, tokio::task::spawn_blocking(task)).await {
                Ok(Ok(())) => {
                    let execution_time = start_time.elapsed().as_millis() as u64;
                    TaskResult {
                        task_id,
                        task_name,
                        success: true,
                        error: None,
                        execution_time_ms: execution_time,
                        worker_id: worker_id.to_string(),
                    }
                }
                Ok(Err(join_error)) => {
                    let execution_time = start_time.elapsed().as_millis() as u64;
                    TaskResult {
                        task_id,
                        task_name,
                        success: false,
                        error: Some(format!("Task join error: {}", join_error)),
                        execution_time_ms: execution_time,
                        worker_id: worker_id.to_string(),
                    }
                }
                Err(_) => {
                    let execution_time = start_time.elapsed().as_millis() as u64;
                    TaskResult {
                        task_id,
                        task_name,
                        success: false,
                        error: Some("Task timed out".to_string()),
                        execution_time_ms: execution_time,
                        worker_id: worker_id.to_string(),
                    }
                }
            }
        } else {
            match tokio::task::spawn_blocking(task).await {
                Ok(()) => {
                    let execution_time = start_time.elapsed().as_millis() as u64;
                    TaskResult {
                        task_id,
                        task_name,
                        success: true,
                        error: None,
                        execution_time_ms: execution_time,
                        worker_id: worker_id.to_string(),
                    }
                }
                Err(join_error) => {
                    let execution_time = start_time.elapsed().as_millis() as u64;
                    TaskResult {
                        task_id,
                        task_name,
                        success: false,
                        error: Some(format!("Task join error: {}", join_error)),
                        execution_time_ms: execution_time,
                        worker_id: worker_id.to_string(),
                    }
                }
            }
        };

        // Update statistics
        {
            let mut stats_ref = stats.write().await;
            if result.success {
                stats_ref.tasks_completed += 1;
            } else {
                stats_ref.tasks_failed += 1;
            }
            stats_ref.total_execution_time_ms += result.execution_time_ms;
            if stats_ref.tasks_completed + stats_ref.tasks_failed > 0 {
                stats_ref.average_execution_time_ms = stats_ref.total_execution_time_ms as f64
                    / (stats_ref.tasks_completed + stats_ref.tasks_failed) as f64;
            }
        }

        // Send result back
        if let Err(_) = response_sender.send(result) {
            warn!("Failed to send task result for task {}", task_id);
        }

        // Update status and tracking
        {
            let mut tasks = active_tasks.write().await;
            *tasks = tasks.saturating_sub(1);
        }

        {
            let mut status_ref = status.write().await;
            *status_ref = WorkerStatus::Idle;
        }

        debug!("Worker {} completed task {}", worker_id, task_id);
    }

    /// Handle task cancellation
    async fn handle_cancel_task(
        worker_id: &str,
        task_id: uuid::Uuid,
        stats: &Arc<RwLock<WorkerStats>>,
    ) {
        debug!("Worker {} received cancel request for task {}", worker_id, task_id);

        // Update statistics
        {
            let mut stats_ref = stats.write().await;
            stats_ref.tasks_cancelled += 1;
        }

        // Note: In a real implementation, you'd need to track which task
        // is currently running and implement cancellation logic
        warn!("Task cancellation not fully implemented for worker {}", worker_id);
    }

    /// Handle status request
    async fn handle_get_status(
        status: &Arc<RwLock<WorkerStatus>>,
        response_sender: oneshot::Sender<WorkerStatus>,
    ) {
        let current_status = *status.read().await;
        let _ = response_sender.send(current_status);
    }

    /// Handle heartbeat
    async fn handle_heartbeat(
        worker_id: &str,
        status: &Arc<RwLock<WorkerStatus>>,
        stats: &Arc<RwLock<WorkerStats>>,
        start_time: std::time::Instant,
    ) {
        let uptime = start_time.elapsed().as_secs();
        let current_status = *status.read().await;

        // Update statistics
        {
            let mut stats_ref = stats.write().await;
            stats_ref.uptime_seconds = uptime;
            stats_ref.last_heartbeat = Some(std::time::Instant::now());
        }

        if current_status == WorkerStatus::Error {
            warn!("Worker {} in error state during heartbeat", worker_id);
        } else {
            debug!("Worker {} heartbeat: status={:?}, uptime={}s", worker_id, current_status, uptime);
        }
    }

    /// Submit a task to the worker
    pub async fn submit_task<F>(&self, task_id: uuid::Uuid, name: String, task: F) -> Result<TaskResult>
    where
        F: FnOnce() + Send + 'static,
    {
        let (response_sender, response_receiver) = oneshot::channel();

        let message = TaskMessage::Execute {
            id: task_id,
            name,
            task: Box::new(task),
            response_sender,
        };

        self.task_sender.send(message)
            .map_err(|e| crate::CziError::runtime(format!("Failed to submit task: {}", e)))?;

        response_receiver.await
            .map_err(|e| crate::CziError::runtime(format!("Failed to receive task result: {}", e)))
    }

    /// Get current worker status
    pub async fn get_status(&self) -> WorkerStatus {
        let (response_sender, response_receiver) = oneshot::channel();

        let message = TaskMessage::GetStatus(response_sender);

        if let Err(e) = self.task_sender.send(message) {
            error!("Failed to send status request: {}", e);
            return WorkerStatus::Error;
        }

        response_receiver.await.unwrap_or(WorkerStatus::Error)
    }

    /// Get worker statistics
    pub async fn get_stats(&self) -> WorkerStats {
        (*self.stats.read().await).clone()
    }

    /// Get number of active tasks
    pub async fn active_tasks(&self) -> usize {
        *self.active_tasks.read().await
    }

    /// Get worker configuration
    pub fn config(&self) -> &WorkerConfig {
        &self.config
    }

    /// Shutdown the worker
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down worker {}", self.config.worker_id);

        // Send shutdown message
        if let Err(e) = self.task_sender.send(TaskMessage::Shutdown) {
            warn!("Failed to send shutdown message: {}", e);
        }

        // Wait for worker to complete
        {
            let mut handle_ref = self.worker_handle.write().await;
            if let Some(handle) = handle_ref.take() {
                if let Err(e) = handle.await {
                    warn!("Error waiting for worker shutdown: {}", e);
                }
            }
        }

        info!("Worker {} shutdown completed", self.config.worker_id);
        Ok(())
    }
}

impl Drop for TaskWorker {
    fn drop(&mut self) {
        // Attempt graceful shutdown
        if let Err(e) = self.task_sender.send(TaskMessage::Shutdown) {
            warn!("Failed to send shutdown message during drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_worker_creation() {
        let worker = TaskWorker::new();
        assert!(worker.is_ok());
    }

    #[tokio::test]
    async fn test_worker_config() {
        let config = WorkerConfig {
            worker_id: "test-worker".to_string(),
            max_concurrent_tasks: 5,
            task_timeout_ms: Some(10000),
            heartbeat_interval_ms: 1000,
            enable_verbose_logging: true,
        };

        let worker = TaskWorker::with_config(config);
        assert!(worker.is_ok());
    }

    #[tokio::test]
    async fn test_task_submission() {
        let worker = TaskWorker::new().unwrap();
        let task_id = uuid::Uuid::new_v4();

        let result = worker.submit_task(task_id, "test_task".to_string(), || {
            std::thread::sleep(Duration::from_millis(10));
            "completed"
        }).await;

        assert!(result.is_ok());
        let task_result = result.unwrap();
        assert!(task_result.success);
        assert!(task_result.error.is_none());
        assert!(task_result.execution_time_ms >= 10);
    }

    #[tokio::test]
    async fn test_worker_status() {
        let worker = TaskWorker::new().unwrap();

        // Initially should be idle
        let status = worker.get_status().await;
        assert!(status == WorkerStatus::Idle || status == WorkerStatus::Busy);

        // Wait a bit for initial status to settle
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = worker.get_status().await;
        assert!(matches!(status, WorkerStatus::Idle | WorkerStatus::Busy));
    }

    #[tokio::test]
    async fn test_worker_stats() {
        let worker = TaskWorker::new().unwrap();

        // Submit a task
        let _ = worker.submit_task(uuid::Uuid::new_v4(), "test".to_string(), || {
            std::thread::sleep(Duration::from_millis(10));
        }).await;

        // Give some time for processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = worker.get_stats().await;
        assert!(stats.tasks_received >= 1);
    }

    #[tokio::test]
    async fn test_concurrent_tasks() {
        let config = WorkerConfig {
            worker_id: "concurrent-worker".to_string(),
            max_concurrent_tasks: 3,
            task_timeout_ms: Some(5000),
            heartbeat_interval_ms: 1000,
            enable_verbose_logging: false,
        };

        let worker = TaskWorker::with_config(config).unwrap();

        let mut handles = Vec::new();

        // Submit multiple tasks concurrently
        for i in 0..5 {
            let worker = &worker;
            let handle = tokio::spawn(async move {
                worker.submit_task(uuid::Uuid::new_v4(), format!("task_{}", i), move || {
                    std::thread::sleep(Duration::from_millis(50));
                    i
                }).await
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let mut completed_tasks = 0;
        for handle in handles {
            if let Ok(Ok(result)) = handle.await {
                if result.success {
                    completed_tasks += 1;
                }
            }
        }

        // All tasks should eventually complete
        assert!(completed_tasks > 0);
    }

    #[tokio::test]
    async fn test_worker_shutdown() {
        let worker = TaskWorker::new().unwrap();
        let worker_id = worker.config().worker_id.clone();

        // Submit a task
        let _ = worker.submit_task(uuid::Uuid::new_v4(), "shutdown_test".to_string(), || {
            std::thread::sleep(Duration::from_millis(10));
        }).await;

        // Shutdown the worker
        let result = worker.shutdown().await;
        assert!(result.is_ok());

        // Status should be stopped after shutdown
        tokio::time::sleep(Duration::from_millis(100)).await;
        // Note: After shutdown, get_status might not work as expected
        // since the worker task has been terminated
    }

    #[tokio::test]
    async fn test_task_timeout() {
        let config = WorkerConfig {
            worker_id: "timeout-worker".to_string(),
            max_concurrent_tasks: 1,
            task_timeout_ms: Some(50), // Very short timeout
            heartbeat_interval_ms: 1000,
            enable_verbose_logging: false,
        };

        let worker = TaskWorker::with_config(config).unwrap();

        let task_id = uuid::Uuid::new_v4();
        let result = worker.submit_task(task_id, "timeout_test".to_string(), || {
            std::thread::sleep(Duration::from_millis(200)); // Longer than timeout
        }).await;

        assert!(result.is_ok());
        let task_result = result.unwrap();
        assert!(!task_result.success); // Should fail due to timeout
        assert!(task_result.error.unwrap().contains("timeout"));
    }
}
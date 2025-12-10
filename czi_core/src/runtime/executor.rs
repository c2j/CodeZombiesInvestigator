//! Task executor for high-performance async operations

use crate::Result;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::runtime::Handle;
use tokio::sync::{oneshot, RwLock};
use tracing::{debug, error, info, warn};

/// Configuration for the executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Task timeout in milliseconds (None for no timeout)
    pub task_timeout_ms: Option<u64>,
    /// Enable task cancellation
    pub enable_cancellation: bool,
    /// Enable detailed metrics
    pub enable_metrics: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            task_timeout_ms: Some(300000), // 5 minutes default
            enable_cancellation: true,
            enable_metrics: true,
        }
    }
}

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending execution
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with error
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task timed out
    TimedOut,
}

/// Task result wrapper
#[derive(Debug)]
pub struct TaskResult<T> {
    /// Task status
    pub status: TaskStatus,
    /// Result value (if successful)
    pub value: Option<T>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Task ID
    pub task_id: uuid::Uuid,
}

/// High-performance async task executor
pub struct CziExecutor {
    /// Executor configuration
    config: ExecutorConfig,
    /// Tokio runtime handle
    runtime_handle: Handle,
    /// Active tasks tracking
    active_tasks: Arc<RwLock<std::collections::HashMap<uuid::Uuid, TaskInfo>>>,
    /// Statistics
    stats: Arc<RwLock<ExecutorStats>>,
}

/// Information about an active task
#[derive(Debug)]
struct TaskInfo {
    /// Task name for debugging
    name: String,
    /// Current status
    status: TaskStatus,
    /// Start timestamp
    start_time: std::time::Instant,
    /// Optional timeout
    timeout: Option<std::time::Duration>,
}

/// Executor statistics
#[derive(Debug, Default, Clone)]
struct ExecutorStats {
    tasks_submitted: u64,
    tasks_completed: u64,
    tasks_failed: u64,
    tasks_cancelled: u64,
    tasks_timed_out: u64,
    total_execution_time_ms: u64,
    average_execution_time_ms: f64,
    current_active_tasks: usize,
}

impl CziExecutor {
    /// Create a new executor with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ExecutorConfig::default())
    }

    /// Create a new executor with custom configuration
    pub fn with_config(config: ExecutorConfig) -> Result<Self> {
        info!("Creating CZI executor with max concurrent tasks: {}", config.max_concurrent_tasks);

        let runtime_handle = Handle::current();

        Ok(Self {
            config,
            runtime_handle,
            active_tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            stats: Arc::new(RwLock::new(ExecutorStats::default())),
        })
    }

    /// Execute a synchronous task
    pub async fn execute<F, T>(&self, name: String, task: F) -> TaskResult<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let task_id = uuid::Uuid::new_v4();
        let start_time = std::time::Instant::now();

        // Record task submission
        {
            let mut stats = self.stats.write().await;
            stats.tasks_submitted += 1;
            stats.current_active_tasks += 1;
        }

        // Add to active tasks
        {
            let mut active = self.active_tasks.write().await;
            active.insert(task_id, TaskInfo {
                name: name.clone(),
                status: TaskStatus::Pending,
                start_time,
                timeout: self.config.task_timeout_ms.map(std::time::Duration::from_millis),
            });
        }

        debug!("Executing task '{}' with ID {}", name, task_id);

        // Execute task with timeout handling
        let result = if let Some(timeout_ms) = self.config.task_timeout_ms {
            let timeout = std::time::Duration::from_millis(timeout_ms);
            match tokio::time::timeout(timeout, self.runtime_handle.spawn_blocking(task)).await {
                Ok(Ok(result)) => TaskResult {
                    status: TaskStatus::Completed,
                    value: Some(result),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    task_id,
                },
                Ok(Err(join_error)) => TaskResult {
                    status: TaskStatus::Failed,
                    value: None,
                    error: Some(format!("Task join error: {}", join_error)),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    task_id,
                },
                Err(_) => TaskResult {
                    status: TaskStatus::TimedOut,
                    value: None,
                    error: Some("Task timed out".to_string()),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    task_id,
                },
            }
        } else {
            match self.runtime_handle.spawn_blocking(task).await {
                Ok(result) => TaskResult {
                    status: TaskStatus::Completed,
                    value: Some(result),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    task_id,
                },
                Err(join_error) => TaskResult {
                    status: TaskStatus::Failed,
                    value: None,
                    error: Some(format!("Task join error: {}", join_error)),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    task_id,
                },
            }
        };

        // Update task tracking
        self.update_task_stats(&result).await;

        // Remove from active tasks
        {
            let mut active = self.active_tasks.write().await;
            active.remove(&task_id);
            if let Ok(mut stats) = self.stats.try_write() {
                stats.current_active_tasks = stats.current_active_tasks.saturating_sub(1);
            }
        }

        debug!("Task '{}' completed with status {:?}", name, result.status);

        result
    }

    /// Execute an async task
    pub async fn execute_async<F, Fut, T>(&self, name: String, future: F) -> TaskResult<T>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let task_id = uuid::Uuid::new_v4();
        let start_time = std::time::Instant::now();

        // Record task submission
        {
            let mut stats = self.stats.write().await;
            stats.tasks_submitted += 1;
            stats.current_active_tasks += 1;
        }

        // Add to active tasks
        {
            let mut active = self.active_tasks.write().await;
            active.insert(task_id, TaskInfo {
                name: name.clone(),
                status: TaskStatus::Pending,
                start_time,
                timeout: self.config.task_timeout_ms.map(std::time::Duration::from_millis),
            });
        }

        debug!("Executing async task '{}' with ID {}", name, task_id);

        // Execute async task with timeout handling
        let task_future = future();
        let result = if let Some(timeout_ms) = self.config.task_timeout_ms {
            let timeout = std::time::Duration::from_millis(timeout_ms);
            match tokio::time::timeout(timeout, task_future).await {
                Ok(result) => TaskResult {
                    status: TaskStatus::Completed,
                    value: Some(result),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    task_id,
                },
                Err(_) => TaskResult {
                    status: TaskStatus::TimedOut,
                    value: None,
                    error: Some("Async task timed out".to_string()),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    task_id,
                },
            }
        } else {
            let result = task_future.await;
            TaskResult {
                status: TaskStatus::Completed,
                value: Some(result),
                error: None,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                task_id,
            }
        };

        // Update task tracking
        self.update_task_stats(&result).await;

        // Remove from active tasks
        {
            let mut active = self.active_tasks.write().await;
            active.remove(&task_id);
            if let Ok(mut stats) = self.stats.try_write() {
                stats.current_active_tasks = stats.current_active_tasks.saturating_sub(1);
            }
        }

        debug!("Async task '{}' completed with status {:?}", name, result.status);

        result
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: uuid::Uuid) -> bool {
        debug!("Cancelling task {}", task_id);

        // Update task status
        {
            let mut active = self.active_tasks.write().await;
            if let Some(task_info) = active.get_mut(&task_id) {
                task_info.status = TaskStatus::Cancelled;
                return true;
            }
        }

        false
    }

    /// Get status of a specific task
    pub async fn get_task_status(&self, task_id: uuid::Uuid) -> Option<TaskStatus> {
        let active = self.active_tasks.read().await;
        active.get(&task_id).map(|info| info.status)
    }

    /// Get current executor statistics
    pub async fn get_stats(&self) -> ExecutorStats {
        (*self.stats.read().await).clone()
    }

    /// Get number of active tasks
    pub async fn active_tasks_count(&self) -> usize {
        self.active_tasks.read().await.len()
    }

    /// Update task statistics based on result
    async fn update_task_stats<T>(&self, result: &TaskResult<T>) {
        let mut stats = self.stats.write().await;

        match result.status {
            TaskStatus::Completed => {
                stats.tasks_completed += 1;
                stats.total_execution_time_ms += result.execution_time_ms;
                stats.average_execution_time_ms = stats.total_execution_time_ms as f64 / stats.tasks_completed as f64;
            }
            TaskStatus::Failed => {
                stats.tasks_failed += 1;
            }
            TaskStatus::Cancelled => {
                stats.tasks_cancelled += 1;
            }
            TaskStatus::TimedOut => {
                stats.tasks_timed_out += 1;
            }
            _ => {}
        }
    }

    /// Get executor configuration
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }

    /// Cancel all active tasks
    pub async fn cancel_all_tasks(&self) -> usize {
        info!("Cancelling all active tasks");

        let cancelled_count = {
            let mut active = self.active_tasks.write().await;
            let count = active.len();
            for (_, task_info) in active.iter_mut() {
                task_info.status = TaskStatus::Cancelled;
            }
            count
        };

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.tasks_cancelled += cancelled_count as u64;
        }

        info!("Cancelled {} tasks", cancelled_count);
        cancelled_count
    }
}

impl Default for CziExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create default executor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = CziExecutor::new();
        assert!(executor.is_ok());
    }

    #[tokio::test]
    async fn test_sync_task_execution() {
        let executor = CziExecutor::new().unwrap();

        let result = executor.execute("test_task".to_string(), || {
            std::thread::sleep(Duration::from_millis(10));
            42
        }).await;

        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.value, Some(42));
        assert!(result.error.is_none());
        assert!(result.execution_time_ms >= 10);
    }

    #[tokio::test]
    async fn test_async_task_execution() {
        let executor = CziExecutor::new().unwrap();

        let result = executor.execute_async("async_test".to_string(), || async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "async_result".to_string()
        }).await;

        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.value, Some("async_result".to_string()));
    }

    #[tokio::test]
    async fn test_task_timeout() {
        let config = ExecutorConfig {
            max_concurrent_tasks: 10,
            task_timeout_ms: Some(50), // 50ms timeout
            enable_cancellation: true,
            enable_metrics: false,
        };

        let executor = CziExecutor::with_config(config).unwrap();

        let result = executor.execute("timeout_test".to_string(), || {
            std::thread::sleep(Duration::from_millis(200)); // Longer than timeout
            "should_not_reach".to_string()
        }).await;

        assert_eq!(result.status, TaskStatus::TimedOut);
        assert!(result.value.is_none());
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let executor = CziExecutor::new().unwrap();

        let task_id = {
            // Start a long-running task
            let result = executor.execute("cancellable_task".to_string(), || {
                std::thread::sleep(Duration::from_millis(1000));
                "done"
            }).await;
            result.task_id
        };

        // Cancel the task quickly
        let cancelled = executor.cancel_task(task_id).await;
        assert!(cancelled);

        // Check task status
        let status = executor.get_task_status(task_id).await;
        assert_eq!(status, Some(TaskStatus::Cancelled));
    }

    #[tokio::test]
    async fn test_concurrent_tasks() {
        let executor = CziExecutor::new().unwrap();

        let mut handles = Vec::new();

        // Execute multiple tasks concurrently
        for i in 0..10 {
            let executor = executor.clone();
            let handle = tokio::spawn(async move {
                executor.execute(format!("task_{}", i), move || {
                    std::thread::sleep(Duration::from_millis(10));
                    i
                }).await
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }

        // Verify all tasks completed
        assert_eq!(results.len(), 10);
        for result in results {
            assert_eq!(result.status, TaskStatus::Completed);
        }
    }

    #[tokio::test]
    async fn test_executor_stats() {
        let executor = CziExecutor::new().unwrap();

        // Execute some tasks
        let _ = executor.execute("task1".to_string(), || 1).await;
        let _ = executor.execute("task2".to_string(), || 2).await;

        // Check statistics
        let stats = executor.get_stats().await;
        assert_eq!(stats.tasks_submitted, 2);
        assert_eq!(stats.tasks_completed, 2);
        assert_eq!(stats.tasks_failed, 0);
        assert!(stats.average_execution_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_cancel_all_tasks() {
        let executor = CziExecutor::new().unwrap();

        // Start some long-running tasks
        let _ = executor.execute("long1".to_string(), || {
            std::thread::sleep(Duration::from_millis(500));
        }).await.task_id;

        let _ = executor.execute("long2".to_string(), || {
            std::thread::sleep(Duration::from_millis(500));
        }).await.task_id;

        // Cancel all tasks
        let cancelled_count = executor.cancel_all_tasks().await;
        assert!(cancelled_count > 0);
    }

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_concurrent_tasks, 100);
        assert_eq!(config.task_timeout_ms, Some(300000));
        assert!(config.enable_cancellation);
        assert!(config.enable_metrics);
    }

    // Helper to implement Clone for executor (needed for concurrent tests)
    impl Clone for CziExecutor {
        fn clone(&self) -> Self {
            Self {
                config: self.config.clone(),
                runtime_handle: self.runtime_handle.clone(),
                active_tasks: Arc::clone(&self.active_tasks),
                stats: Arc::clone(&self.stats),
            }
        }
    }
}
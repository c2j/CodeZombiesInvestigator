//! Async task runtime management for CodeZombiesInvestigator
//!
//! Provides Tokio-based async runtime with task orchestration,
//! concurrency control, and resource management.

pub mod task_pool;
pub mod scheduler;
pub mod worker;
pub mod monitor;
pub mod executor;

pub use task_pool::{TaskPool, TaskPoolConfig};
pub use scheduler::{TaskScheduler, TaskPriority, TaskSchedulerConfig};
pub use worker::{TaskWorker, WorkerConfig, WorkerStatus};
pub use monitor::{RuntimeMonitor, RuntimeMetrics, ResourceMonitor};
pub use executor::{CziExecutor, ExecutorConfig};

use crate::Result;
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};
use tracing::{info, warn};

/// Main async runtime manager for CZI operations
pub struct CziRuntime {
    /// Tokio runtime instance
    runtime: Arc<Runtime>,
    /// Task scheduler for managing concurrent operations
    scheduler: Arc<TaskScheduler>,
    /// Task pool for managing worker threads
    task_pool: Arc<TaskPool>,
    /// Runtime monitor for performance tracking
    monitor: Arc<RuntimeMonitor>,
    /// Configuration
    config: RuntimeConfig,
}

/// Configuration for the async runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Number of worker threads (None = use CPU count)
    pub worker_threads: Option<usize>,
    /// Maximum concurrent tasks per worker
    pub max_concurrent_tasks: usize,
    /// Task queue size limit
    pub queue_size: usize,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// Thread stack size in bytes
    pub thread_stack_size: Option<usize>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // Use CPU count
            max_concurrent_tasks: 100,
            queue_size: 1000,
            enable_monitoring: true,
            monitoring_interval_ms: 5000,
            thread_stack_size: Some(2 * 1024 * 1024), // 2MB
        }
    }
}

impl CziRuntime {
    /// Create a new runtime with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(RuntimeConfig::default())
    }

    /// Create a new runtime with custom configuration
    pub fn with_config(config: RuntimeConfig) -> Result<Self> {
        info!("Initializing CZI async runtime with config: {:?}", config);

        // Build Tokio runtime
        let mut builder = Builder::new_multi_thread();

        if let Some(worker_threads) = config.worker_threads {
            builder.worker_threads(worker_threads);
        }

        if let Some(stack_size) = config.thread_stack_size {
            builder.thread_stack_size(stack_size);
        }

        // Enable I/O and time drivers
        builder.enable_io()
              .enable_time();

        let runtime = Arc::new(
            builder.build()
                .map_err(|e| crate::CziError::runtime(format!("Failed to create Tokio runtime: {}", e)))?
        );

        // Create task scheduler
        let scheduler = Arc::new(TaskScheduler::with_config(TaskSchedulerConfig {
            max_queue_size: config.queue_size,
            enable_metrics: config.enable_monitoring,
        })?);

        // Create task pool
        let task_pool = Arc::new(TaskPool::with_config(TaskPoolConfig {
            worker_threads: config.worker_threads.unwrap_or_else(num_cpus::get),
            max_concurrent_tasks: config.max_concurrent_tasks,
        })?);

        // Create runtime monitor
        let monitor = Arc::new(RuntimeMonitor::new(config.monitoring_interval_ms)?);

        info!("CZI async runtime initialized successfully");

        Ok(Self {
            runtime,
            scheduler,
            task_pool,
            monitor,
            config,
        })
    }

    /// Get the Tokio runtime handle
    pub fn runtime(&self) -> Arc<Runtime> {
        Arc::clone(&self.runtime)
    }

    /// Get the task scheduler
    pub fn scheduler(&self) -> Arc<TaskScheduler> {
        Arc::clone(&self.scheduler)
    }

    /// Get the task pool
    pub fn task_pool(&self) -> Arc<TaskPool> {
        Arc::clone(&self.task_pool)
    }

    /// Get the runtime monitor
    pub fn monitor(&self) -> Arc<RuntimeMonitor> {
        Arc::clone(&self.monitor)
    }

    /// Get runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Execute an async task with the runtime
    pub async fn execute<F, R>(&self, task: F) -> Result<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        // This method allows executing synchronous functions in async context
        // For truly async operations, use runtime.spawn() directly
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Run the task in a blocking thread pool
        tokio::task::spawn_blocking(move || {
            let result = task();
            let _ = tx.send(result);
        });

        rx.await
            .map_err(|e| crate::CziError::runtime(format!("Task execution failed: {}", e)))
    }

    /// Schedule a task with priority
    pub async fn schedule_task<F, R>(&self,
        name: String,
        priority: TaskPriority,
        task: F
    ) -> Result<uuid::Uuid>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let task_id = uuid::Uuid::new_v4();
        self.scheduler.schedule_task(task_id, name, priority, task).await
    }

    /// Get current runtime metrics
    pub fn get_metrics(&self) -> RuntimeMetrics {
        self.monitor.get_current_metrics()
    }

    /// Start background monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        if self.config.enable_monitoring {
            self.monitor.start().await?;
            info!("Runtime monitoring started");
        } else {
            warn!("Monitoring is disabled in configuration");
        }
        Ok(())
    }

    /// Stop background monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        self.monitor.stop().await?;
        info!("Runtime monitoring stopped");
        Ok(())
    }

    /// Shutdown the runtime gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down CZI async runtime");

        // Stop monitoring first
        if let Err(e) = self.stop_monitoring().await {
            warn!("Failed to stop monitoring: {}", e);
        }

        // Drain task queue
        if let Err(e) = self.scheduler.drain().await {
            warn!("Failed to drain scheduler: {}", e);
        }

        // Shutdown task pool
        if let Err(e) = self.task_pool.shutdown().await {
            warn!("Failed to shutdown task pool: {}", e);
        }

        info!("CZI async runtime shutdown completed");
        Ok(())
    }

    /// Get runtime statistics
    pub async fn get_stats(&self) -> RuntimeStats {
        RuntimeStats {
            uptime_seconds: self.monitor.uptime_seconds(),
            tasks_completed: self.scheduler.tasks_completed(),
            tasks_failed: self.scheduler.tasks_failed(),
            current_queue_size: self.scheduler.queue_size().await,
            active_workers: self.task_pool.active_workers().await,
            memory_usage_mb: self.monitor.memory_usage_mb(),
        }
    }
}

/// Runtime statistics snapshot
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    /// Runtime uptime in seconds
    pub uptime_seconds: u64,
    /// Total tasks completed
    pub tasks_completed: u64,
    /// Total tasks failed
    pub tasks_failed: u64,
    /// Current task queue size
    pub current_queue_size: usize,
    /// Number of active workers
    pub active_workers: usize,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
}

impl Default for CziRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create default runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = CziRuntime::new();
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = RuntimeConfig {
            worker_threads: Some(2),
            max_concurrent_tasks: 50,
            queue_size: 500,
            enable_monitoring: false,
            monitoring_interval_ms: 1000,
            thread_stack_size: Some(1024 * 1024), // 1MB
        };

        let runtime = CziRuntime::with_config(config);
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_task_execution() {
        let runtime = CziRuntime::new().unwrap();

        // Test synchronous task execution
        let result = runtime.execute(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_task_scheduling() {
        let runtime = CziRuntime::new().unwrap();

        // Schedule a task
        let task_id = runtime.schedule_task(
            "test_task".to_string(),
            TaskPriority::Normal,
            || {
                std::thread::sleep(Duration::from_millis(10));
                "test_result"
            }
        ).await;

        assert!(task_id.is_ok());

        // Wait a bit for task to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = runtime.get_stats();
        assert!(stats.tasks_completed > 0);
    }

    #[tokio::test]
    async fn test_runtime_metrics() {
        let runtime = CziRuntime::new().unwrap();

        let metrics = runtime.get_metrics();
        assert!(metrics.timestamp > 0);
    }

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert!(config.worker_threads.is_none());
        assert_eq!(config.max_concurrent_tasks, 100);
        assert_eq!(config.queue_size, 1000);
        assert!(config.enable_monitoring);
    }
}
//! Task pool for managing concurrent operations

use crate::Result;
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Configuration for task pool
#[derive(Debug, Clone)]
pub struct TaskPoolConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    /// Maximum concurrent tasks per worker
    pub max_concurrent_tasks: usize,
}

impl Default for TaskPoolConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            max_concurrent_tasks: 50,
        }
    }
}

/// Pool for managing concurrent task execution
pub struct TaskPool {
    /// Semaphore to limit concurrent tasks
    semaphore: Arc<Semaphore>,
    /// Configuration
    config: TaskPoolConfig,
    /// Active workers counter
    active_workers: Arc<RwLock<usize>>,
    /// Shutdown flag
    shutdown: Arc<RwLock<bool>>,
}

impl TaskPool {
    /// Create a new task pool with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(TaskPoolConfig::default())
    }

    /// Create a new task pool with custom configuration
    pub fn with_config(config: TaskPoolConfig) -> Result<Self> {
        info!("Creating task pool with {} workers, max concurrent tasks: {}",
              config.worker_threads, config.max_concurrent_tasks);

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));

        Ok(Self {
            semaphore,
            config,
            active_workers: Arc::new(RwLock::new(0)),
            shutdown: Arc::new(RwLock::new(false)),
        })
    }

    /// Execute a task in the pool
    pub async fn execute<F, R>(&self, task: F) -> Result<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        // Check if shutdown is in progress
        {
            let shutdown = self.shutdown.read().await;
            if *shutdown {
                return Err(crate::CziError::runtime("Task pool is shutting down".to_string()));
            }
        }

        // Acquire semaphore permit
        let permit = self.semaphore.acquire().await
            .map_err(|e| crate::CziError::runtime(format!("Failed to acquire task permit: {}", e)))?;

        // Increment active workers
        {
            let mut active = self.active_workers.write().await;
            *active += 1;
            debug!("Active workers: {}", *active);
        }

        // Clone Arc for worker tracking
        let active_workers_clone = Arc::clone(&self.active_workers);

        // Execute the task in a blocking task
        let task_result = tokio::task::spawn_blocking(move || {
            // Execute the task
            let result = task();

            // Decrement active workers
            {
                let mut active = active_workers_clone.blocking_write();
                *active = active.saturating_sub(1);
            }

            result
        }).await
            .map_err(|e| crate::CziError::runtime(format!("Task execution failed: {}", e)))?;

        // The permit is automatically released when dropped
        drop(permit);

        Ok(task_result)
    }

    /// Execute an async task in the pool
    pub async fn execute_async<F, R>(&self, task: F) -> Result<R>
    where
        F: std::future::Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        // Check if shutdown is in progress
        {
            let shutdown = self.shutdown.read().await;
            if *shutdown {
                return Err(crate::CziError::runtime("Task pool is shutting down".to_string()));
            }
        }

        // Acquire semaphore permit
        let permit = self.semaphore.acquire().await
            .map_err(|e| crate::CziError::runtime(format!("Failed to acquire task permit: {}", e)))?;

        // Increment active workers
        {
            let mut active = self.active_workers.write().await;
            *active += 1;
            debug!("Active workers: {}", *active);
        }

        // Execute async task
        let result = task.await;

        // Release permit and decrement active workers
        drop(permit);
        {
            let mut active = self.active_workers.write().await;
            *active = active.saturating_sub(1);
        }

        Ok(result)
    }

    /// Spawn a background task without waiting for result
    pub async fn spawn<F, R>(&self, name: String, task: F) -> Result<JoinHandle<Option<R>>>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        debug!("Spawning background task: {}", name);

        let semaphore = Arc::clone(&self.semaphore);
        let shutdown = Arc::clone(&self.shutdown);

        let handle = tokio::spawn(async move {
            // Check shutdown status
            {
                let shutdown_flag = shutdown.read().await;
                if *shutdown_flag {
                    warn!("Task {} cancelled due to shutdown", name);
                    return None;
                }
            }

            // Acquire permit
            let permit = semaphore.acquire().await;
            if let Err(e) = permit {
                warn!("Task {} failed to acquire permit: {}", name, e);
                return None;
            }
            let permit = permit.unwrap();

            debug!("Task {} started execution", name);

            // Execute task
            let result = tokio::task::spawn_blocking(move || {
                // Execute actual task
                task()
            }).await;

            debug!("Task {} completed execution", name);
            drop(permit);

            match result {
                Ok(value) => Some(value),
                Err(e) => {
                    warn!("Task {} execution error: {}", name, e);
                    None
                }
            }
        });

        Ok(handle)
    }

    /// Get current number of active workers
    pub async fn active_workers(&self) -> usize {
        *self.active_workers.read().await
    }

    /// Get task pool configuration
    pub fn config(&self) -> &TaskPoolConfig {
        &self.config
    }

    /// Get semaphore available permits
    pub async fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Shutdown the task pool gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down task pool");

        // Set shutdown flag
        {
            let mut shutdown = self.shutdown.write().await;
            *shutdown = true;
        }

        // Wait for all active tasks to complete
        loop {
            let active = *self.active_workers.read().await;
            if active == 0 {
                break;
            }
            debug!("Waiting for {} active tasks to complete", active);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("Task pool shutdown completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_task_pool_creation() {
        let pool = TaskPool::new();
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_task_execution() {
        let pool = TaskPool::new().unwrap();

        let result = pool.execute(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_async_task_execution() {
        let pool = TaskPool::new().unwrap();

        let result = pool.execute_async(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            24
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 24);
    }

    #[tokio::test]
    async fn test_concurrent_tasks() {
        let pool = TaskPool::with_config(TaskPoolConfig {
            worker_threads: 2,
            max_concurrent_tasks: 5,
        }).unwrap();

        let mut handles = Vec::new();

        // Spawn multiple tasks
        for i in 0..10 {
            let handle = pool.spawn(format!("task_{}", i), move || {
                std::thread::sleep(Duration::from_millis(50));
                i
            }).await.unwrap();
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
        }

        // Give some time for tasks to finish
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Check that no workers are active
        let active = pool.active_workers().await;
        assert_eq!(active, 0);
    }

    #[tokio::test]
    async fn test_task_limits() {
        let pool = TaskPool::with_config(TaskPoolConfig {
            worker_threads: 2,
            max_concurrent_tasks: 3, // Limit concurrency
        }).unwrap();

        let active_before = pool.active_workers().await;
        let available_before = pool.available_permits().await;

        // Start tasks that will take time
        let mut handles = Vec::new();
        for _ in 0..5 {
            let handle = pool.spawn("test".to_string(), || {
                std::thread::sleep(Duration::from_millis(100));
            }).await.unwrap();
            handles.push(handle);
        }

        // Give some time for tasks to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        let active_during = pool.active_workers().await;
        let available_during = pool.available_permits().await;

        // Should not exceed max concurrent tasks
        assert!(available_during <= available_before);
        assert!(active_during >= active_before);

        // Wait for completion
        for handle in handles {
            let _ = handle.await;
        }
    }

    #[tokio::test]
    async fn test_shutdown() {
        let pool = TaskPool::new().unwrap();

        // Start some long-running tasks
        let _ = pool.spawn("test1".to_string(), || {
            std::thread::sleep(Duration::from_millis(100));
        }).await;

        let _ = pool.spawn("test2".to_string(), || {
            std::thread::sleep(Duration::from_millis(100));
        }).await;

        // Shutdown should wait for tasks to complete
        let result = pool.shutdown().await;
        assert!(result.is_ok());

        // Should not accept new tasks after shutdown
        let result = pool.execute(|| 42).await;
        assert!(result.is_err());
    }
}
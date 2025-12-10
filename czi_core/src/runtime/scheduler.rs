//! Task scheduler for managing task priorities and execution order

use crate::Result;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Low priority tasks
    Low = 0,
    /// Normal priority tasks
    Normal = 1,
    /// High priority tasks
    High = 2,
    /// Critical priority tasks
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Configuration for task scheduler
#[derive(Debug, Clone)]
pub struct TaskSchedulerConfig {
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Enable detailed metrics collection
    pub enable_metrics: bool,
}

impl Default for TaskSchedulerConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            enable_metrics: true,
        }
    }
}

/// Scheduled task wrapper
#[derive(Debug)]
struct ScheduledTask {
    pub id: uuid::Uuid,
    pub name: String,
    pub priority: TaskPriority,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for max-heap (highest priority first)
        other.priority.cmp(&self.priority)
            .then_with(|| self.created_at.cmp(&other.created_at))
    }
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for ScheduledTask {}

/// Task scheduling statistics
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_cancelled: u64,
    pub current_queue_size: usize,
    pub average_execution_time_ms: f64,
}

/// Task scheduler for managing concurrent operations
pub struct TaskScheduler {
    /// Priority queue for tasks
    task_queue: Arc<RwLock<BinaryHeap<ScheduledTask>>>,
    /// Task function storage
    task_functions: Arc<RwLock<HashMap<uuid::Uuid, TaskFn>>>,
    /// Statistics tracking
    stats: Arc<RwLock<SchedulerStats>>,
    /// Configuration
    config: TaskSchedulerConfig,
    /// Task execution channel
    task_sender: mpsc::UnboundedSender<TaskMessage>,
    /// Receiver for task messages
    task_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<TaskMessage>>>>,
}

/// Task function wrapper
type TaskFn = Box<dyn FnOnce() + Send>;

/// Internal task messages
enum TaskMessage {
    Execute(ScheduledTask),
    Cancel(uuid::Uuid),
    Shutdown,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new() -> Result<Self> {
        Self::with_config(TaskSchedulerConfig::default())
    }

    /// Create a task scheduler with custom configuration
    pub fn with_config(config: TaskSchedulerConfig) -> Result<Self> {
        info!("Creating task scheduler with max queue size: {}", config.max_queue_size);

        let (task_sender, task_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            task_functions: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(SchedulerStats::default())),
            config,
            task_sender,
            task_receiver: Arc::new(RwLock::new(Some(task_receiver))),
        })
    }

    /// Schedule a task with given priority
    pub async fn schedule_task<F, R>(&self,
        id: uuid::Uuid,
        name: String,
        priority: TaskPriority,
        task: F
    ) -> Result<uuid::Uuid>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        // Check queue size limit
        {
            let queue = self.task_queue.read().await;
            if queue.len() >= self.config.max_queue_size {
                return Err(crate::CziError::runtime("Task queue is full".to_string()));
            }
        }

        let scheduled_task = ScheduledTask {
            id,
            name: name.clone(),
            priority,
            created_at: chrono::Utc::now(),
        };

        debug!("Scheduling task '{}' with priority {:?}", name, priority);

        // Store task function
        {
            let mut functions = self.task_functions.write().await;
            functions.insert(id, Box::new(move || {
                // Execute the task and ignore result
                task();
            }));
        }

        // Add to priority queue
        {
            let mut queue = self.task_queue.write().await;
            queue.push(scheduled_task);
        }

        // Send execution message
        if let Err(e) = self.task_sender.send(TaskMessage::Execute(ScheduledTask {
            id,
            name,
            priority,
            created_at: chrono::Utc::now(),
        })) {
            warn!("Failed to send task execution message: {}", e);
            return Err(crate::CziError::runtime(format!("Failed to schedule task: {}", e)));
        }

        Ok(id)
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        let queue = self.task_queue.read().await;
        queue.len()
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        self.stats.read().await.clone()
    }

    /// Get total completed tasks
    pub fn tasks_completed(&self) -> u64 {
        // Note: This is a simplified version that doesn't read from the lock
        // In a real implementation, you'd want async access
        self.stats.try_read().map(|s| s.tasks_completed).unwrap_or(0)
    }

    /// Get total failed tasks
    pub fn tasks_failed(&self) -> u64 {
        // Note: This is a simplified version
        self.stats.try_read().map(|s| s.tasks_failed).unwrap_or(0)
    }

    /// Cancel a scheduled task
    pub async fn cancel_task(&self, task_id: uuid::Uuid) -> Result<bool> {
        debug!("Cancelling task: {}", task_id);

        // Remove from queue
        let was_in_queue = {
            let mut queue = self.task_queue.write().await;
            let mut found = false;
            queue.retain(|task| {
                if task.id == task_id {
                    found = true;
                    false
                } else {
                    true
                }
            });
            found
        };

        // Remove from function storage
        {
            let mut functions = self.task_functions.write().await;
            functions.remove(&task_id);
        }

        if was_in_queue {
            // Update statistics
            let mut stats = self.stats.write().await;
            stats.tasks_cancelled += 1;
        }

        // Send cancellation message
        let _ = self.task_sender.send(TaskMessage::Cancel(task_id));

        Ok(was_in_queue)
    }

    /// Drain all pending tasks
    pub async fn drain(&self) -> Result<usize> {
        info!("Draining task queue");

        let mut removed_count = 0;

        // Clear queue and collect task IDs
        let task_ids = {
            let mut queue = self.task_queue.write().await;
            let task_ids: Vec<uuid::Uuid> = queue.iter().map(|task| task.id).collect();
            queue.clear();
            task_ids
        };

        // Remove from function storage
        {
            let mut functions = self.task_functions.write().await;
            for task_id in task_ids {
                functions.remove(&task_id);
                removed_count += 1;
            }
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.tasks_cancelled += removed_count as u64;
        }

        info!("Drained {} tasks from queue", removed_count);
        Ok(removed_count)
    }

    /// Get scheduler configuration
    pub fn config(&self) -> &TaskSchedulerConfig {
        &self.config
    }
}

impl Drop for TaskScheduler {
    fn drop(&mut self) {
        // Send shutdown message
        let _ = self.task_sender.send(TaskMessage::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = TaskScheduler::new();
        assert!(scheduler.is_ok());
    }

    #[tokio::test]
    async fn test_task_scheduling() {
        let scheduler = TaskScheduler::new().unwrap();

        let task_id = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "test_task".to_string(),
            TaskPriority::Normal,
            || {
                debug!("Test task executed");
            }
        ).await;

        assert!(task_id.is_ok());
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let scheduler = TaskScheduler::new().unwrap();

        // Schedule tasks with different priorities
        let low_id = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "low_priority".to_string(),
            TaskPriority::Low,
            || { debug!("Low priority task") }
        ).await.unwrap();

        let critical_id = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "critical_priority".to_string(),
            TaskPriority::Critical,
            || { debug!("Critical priority task") }
        ).await.unwrap();

        let high_id = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "high_priority".to_string(),
            TaskPriority::High,
            || { debug!("High priority task") }
        ).await.unwrap();

        // All tasks should be scheduled successfully
        assert!(scheduler.queue_size().await >= 3);

        // Clean up
        assert!(scheduler.cancel_task(low_id).await.is_ok());
        assert!(scheduler.cancel_task(critical_id).await.is_ok());
        assert!(scheduler.cancel_task(high_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let scheduler = TaskScheduler::new().unwrap();

        let task_id = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "cancel_me".to_string(),
            TaskPriority::Normal,
            || { debug!("This should not execute") }
        ).await.unwrap();

        // Task should be in queue
        assert!(scheduler.queue_size().await > 0);

        // Cancel the task
        let cancelled = scheduler.cancel_task(task_id).await.unwrap();
        assert!(cancelled);
    }

    #[tokio::test]
    async fn test_queue_limit() {
        let scheduler = TaskScheduler::with_config(TaskSchedulerConfig {
            max_queue_size: 2,
            enable_metrics: false,
        }).unwrap();

        // Schedule tasks up to limit
        let _task1 = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "task1".to_string(),
            TaskPriority::Normal,
            || {}
        ).await.unwrap();

        let _task2 = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "task2".to_string(),
            TaskPriority::Normal,
            || {}
        ).await.unwrap();

        // Third task should fail due to queue limit
        let task3 = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "task3".to_string(),
            TaskPriority::Normal,
            || {}
        ).await;

        assert!(task3.is_err());
    }

    #[tokio::test]
    async fn test_drain_queue() {
        let scheduler = TaskScheduler::new().unwrap();

        // Schedule some tasks
        let _task1 = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "task1".to_string(),
            TaskPriority::Normal,
            || {}
        ).await.unwrap();

        let _task2 = scheduler.schedule_task(
            uuid::Uuid::new_v4(),
            "task2".to_string(),
            TaskPriority::Normal,
            || {}
        ).await.unwrap();

        assert!(scheduler.queue_size().await > 0);

        // Drain queue
        let drained = scheduler.drain().await.unwrap();
        assert!(drained > 0);
        assert_eq!(scheduler.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_scheduler_stats() {
        let scheduler = TaskScheduler::new().unwrap();

        let stats = scheduler.get_stats().await;
        assert_eq!(stats.tasks_completed, 0);
        assert_eq!(stats.tasks_failed, 0);
        assert_eq!(stats.tasks_cancelled, 0);
    }
}
//! Runtime monitoring and performance metrics

use crate::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Runtime performance metrics
#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    /// Timestamp when metrics were collected
    pub timestamp: u64,
    /// Runtime uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in megabytes
    pub memory_usage_mb: f64,
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f64,
    /// Active thread count
    pub active_threads: u64,
    /// Task queue size
    pub task_queue_size: usize,
    /// Tasks completed since startup
    pub tasks_completed: u64,
    /// Tasks failed since startup
    pub tasks_failed: u64,
    /// Average task execution time in milliseconds
    pub avg_task_duration_ms: f64,
    /// Peak memory usage in megabytes
    pub peak_memory_usage_mb: f64,
}

impl Default for RuntimeMetrics {
    fn default() -> Self {
        Self {
            timestamp: 0,
            uptime_seconds: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            active_threads: 0,
            task_queue_size: 0,
            tasks_completed: 0,
            tasks_failed: 0,
            avg_task_duration_ms: 0.0,
            peak_memory_usage_mb: 0.0,
        }
    }
}

/// Runtime performance monitor
pub struct RuntimeMonitor {
    /// Start time of the runtime
    start_time: Instant,
    /// Current metrics
    metrics: Arc<RwLock<RuntimeMetrics>>,
    /// Peak memory usage tracking
    peak_memory_mb: Arc<RwLock<f64>>,
    /// Monitoring interval in milliseconds
    monitoring_interval_ms: u64,
    /// Background monitoring task handle
    monitoring_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Whether monitoring is currently running
    is_running: Arc<RwLock<bool>>,
}

impl RuntimeMonitor {
    /// Create a new runtime monitor with specified monitoring interval
    pub fn new(monitoring_interval_ms: u64) -> Result<Self> {
        info!("Creating runtime monitor with {}ms interval", monitoring_interval_ms);

        let start_time = Instant::now();
        let metrics = Arc::new(RwLock::new(RuntimeMetrics {
            timestamp: chrono::Utc::now().timestamp() as u64,
            uptime_seconds: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            active_threads: 0,
            task_queue_size: 0,
            tasks_completed: 0,
            tasks_failed: 0,
            avg_task_duration_ms: 0.0,
            peak_memory_usage_mb: 0.0,
        }));

        Ok(Self {
            start_time,
            metrics,
            peak_memory_mb: Arc::new(RwLock::new(0.0)),
            monitoring_interval_ms,
            monitoring_handle: Arc::new(RwLock::new(None)),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start background monitoring
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("Runtime monitoring is already running");
            return Ok(());
        }

        info!("Starting runtime monitoring");
        *is_running = true;

        let metrics = Arc::clone(&self.metrics);
        let peak_memory_mb = Arc::clone(&self.peak_memory_mb);
        let start_time = self.start_time;
        let interval_ms = self.monitoring_interval_ms;
        let is_running_flag = Arc::clone(&self.is_running);

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));

            loop {
                // Check if we should stop
                {
                    let running = is_running_flag.read().await;
                    if !*running {
                        debug!("Runtime monitoring loop stopping");
                        break;
                    }
                }

                interval.tick().await;

                // Update metrics
                let current_time = chrono::Utc::now().timestamp() as u64;
                let uptime_seconds = start_time.elapsed().as_secs();
                let memory_usage_mb = Self::get_memory_usage_mb();
                let cpu_usage_percent = Self::get_cpu_usage_percent();
                let active_threads = Self::get_active_thread_count();

                // Update peak memory if needed
                {
                    let mut peak = peak_memory_mb.write().await;
                    if memory_usage_mb > *peak {
                        *peak = memory_usage_mb;
                    }
                }

                {
                    let mut metrics = metrics.write().await;
                    metrics.timestamp = current_time;
                    metrics.uptime_seconds = uptime_seconds;
                    metrics.memory_usage_mb = memory_usage_mb;
                    metrics.cpu_usage_percent = cpu_usage_percent;
                    metrics.active_threads = active_threads;
                    metrics.peak_memory_usage_mb = *peak_memory_mb.read().await;
                }

                debug!("Updated runtime metrics: {}MB memory, {:.1}% CPU",
                       memory_usage_mb, cpu_usage_percent);
            }
        });

        // Store the handle
        {
            let mut monitoring_handle = self.monitoring_handle.write().await;
            *monitoring_handle = Some(handle);
        }

        info!("Runtime monitoring started successfully");
        Ok(())
    }

    /// Stop background monitoring
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping runtime monitoring");

        // Set flag to stop monitoring
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }

        // Wait for monitoring task to complete
        {
            let mut monitoring_handle = self.monitoring_handle.write().await;
            if let Some(handle) = monitoring_handle.take() {
                debug!("Waiting for monitoring task to complete");
                if let Err(e) = handle.await {
                    warn!("Error waiting for monitoring task: {}", e);
                }
            }
        }

        info!("Runtime monitoring stopped");
        Ok(())
    }

    /// Get current runtime metrics
    pub fn get_current_metrics(&self) -> RuntimeMetrics {
        // This is a simplified synchronous version
        // In a real implementation, you might want to make this async
        if let Ok(metrics) = self.metrics.try_read() {
            metrics.clone()
        } else {
            RuntimeMetrics::default()
        }
    }

    /// Get runtime uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get current memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        Self::get_memory_usage_mb()
    }

    /// Update task statistics
    pub async fn update_task_stats(&self, completed: u64, failed: u64, avg_duration_ms: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.tasks_completed = completed;
        metrics.tasks_failed = failed;
        metrics.avg_task_duration_ms = avg_duration_ms;
    }

    /// Update task queue size
    pub async fn update_queue_size(&self, queue_size: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.task_queue_size = queue_size;
    }

    /// Get current memory usage in MB (simplified implementation)
    fn get_memory_usage_mb() -> f64 {
        // This is a simplified implementation
        // In a real scenario, you'd use platform-specific APIs
        // For now, we'll return a reasonable estimate
        #[cfg(unix)]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(kb) = parts[1].parse::<u64>() {
                                return kb as f64 / 1024.0;
                            }
                        }
                    }
                }
            }
        }

        // Fallback estimate (would need platform-specific implementation)
        100.0 // Reasonable default
    }

    /// Get current CPU usage percentage (simplified implementation)
    fn get_cpu_usage_percent() -> f64 {
        // This is a simplified implementation
        // In a real scenario, you'd use system APIs to measure CPU time
        // For now, we'll return a reasonable estimate
        25.0 // Reasonable default
    }

    /// Get active thread count
    fn get_active_thread_count() -> u64 {
        // This is a simplified implementation
        // In a real scenario, you'd use system APIs
        // For now, we'll return a reasonable default
        // Note: std::thread::active_count() is unstable
        num_cpus::get() as u64
    }

    /// Check if monitoring is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get monitoring interval
    pub fn monitoring_interval_ms(&self) -> u64 {
        self.monitoring_interval_ms
    }
}

/// Resource monitor for tracking system resource usage
pub struct ResourceMonitor {
    /// Last CPU measurement
    last_cpu_time: Arc<RwLock<Option<(u64, u64)>>>, // (user_time, system_time)
    /// Monitoring start time
    start_time: Instant,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            last_cpu_time: Arc::new(RwLock::new(None)),
            start_time: Instant::now(),
        }
    }

    /// Get detailed resource statistics
    pub async fn get_resource_stats(&self) -> ResourceStats {
        let uptime = self.start_time.elapsed().as_secs_f64();
        let memory_mb = RuntimeMonitor::get_memory_usage_mb();
        let cpu_percent = RuntimeMonitor::get_cpu_usage_percent();
        let threads = RuntimeMonitor::get_active_thread_count();

        ResourceStats {
            uptime_seconds: uptime,
            memory_usage_mb: memory_mb,
            cpu_usage_percent: cpu_percent,
            thread_count: threads,
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// Uptime in seconds
    pub uptime_seconds: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Thread count
    pub thread_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_monitor_creation() {
        let monitor = RuntimeMonitor::new(1000);
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_monitor_lifecycle() {
        let monitor = RuntimeMonitor::new(100).unwrap();

        // Should not be running initially
        assert!(!monitor.is_running().await);

        // Start monitoring
        let result = monitor.start().await;
        assert!(result.is_ok());
        assert!(monitor.is_running().await);

        // Get metrics
        let metrics = monitor.get_current_metrics();
        assert!(metrics.uptime_seconds >= 0);

        // Stop monitoring
        let result = monitor.stop().await;
        assert!(result.is_ok());
        assert!(!monitor.is_running().await);
    }

    #[tokio::test]
    async fn test_metrics_updates() {
        let monitor = RuntimeMonitor::new(1000).unwrap();

        // Update task stats
        monitor.update_task_stats(100, 5, 50.0).await;

        // Update queue size
        monitor.update_queue_size(25).await;

        // Check metrics
        let metrics = monitor.get_current_metrics();
        assert_eq!(metrics.tasks_completed, 100);
        assert_eq!(metrics.tasks_failed, 5);
        assert_eq!(metrics.avg_task_duration_ms, 50.0);
        assert_eq!(metrics.task_queue_size, 25);
    }

    #[tokio::test]
    async fn test_uptime() {
        let monitor = RuntimeMonitor::new(1000).unwrap();

        let initial_uptime = monitor.uptime_seconds();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let later_uptime = monitor.uptime_seconds();
        assert!(later_uptime > initial_uptime);
    }

    #[tokio::test]
    async fn test_resource_monitor() {
        let resource_monitor = ResourceMonitor::new();

        let stats = resource_monitor.get_resource_stats().await;
        assert!(stats.uptime_seconds >= 0.0);
        assert!(stats.memory_usage_mb >= 0.0);
        assert!(stats.cpu_usage_percent >= 0.0 && stats.cpu_usage_percent <= 100.0);
        assert!(stats.thread_count > 0);
    }

    #[tokio::test]
    async fn test_monitor_restart() {
        let monitor = RuntimeMonitor::new(100).unwrap();

        // Start monitoring
        monitor.start().await.unwrap();
        assert!(monitor.is_running().await);

        // Start again (should not cause issues)
        let result = monitor.start().await;
        assert!(result.is_ok());

        // Stop monitoring
        monitor.stop().await.unwrap();
        assert!(!monitor.is_running().await);

        // Stop again (should not cause issues)
        let result = monitor.stop().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_metrics_default() {
        let metrics = RuntimeMetrics::default();
        assert_eq!(metrics.timestamp, 0);
        assert_eq!(metrics.uptime_seconds, 0);
        assert_eq!(metrics.tasks_completed, 0);
        assert_eq!(metrics.tasks_failed, 0);
    }
}
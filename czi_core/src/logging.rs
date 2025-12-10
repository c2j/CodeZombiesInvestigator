//! Logging and performance monitoring for CodeZombiesInvestigator

use std::time::{Duration, Instant};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// Initialize logging with structured output
pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("czi=info,info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// Performance timer for measuring operation duration
#[derive(Debug)]
pub struct OperationTimer {
    name: String,
    start: Instant,
    threshold: Duration,
}

impl OperationTimer {
    /// Create a new timer with a given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            threshold: Duration::from_millis(100), // Default threshold
        }
    }

    /// Create a new timer with a custom threshold
    pub fn with_threshold(name: impl Into<String>, threshold: Duration) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            threshold,
        }
    }

    /// Complete the timer and log the duration
    pub fn finish(self) -> Duration {
        let duration = self.start.elapsed();

        if duration > self.threshold {
            info!(
                operation = %self.name,
                duration_ms = duration.as_millis(),
                threshold_ms = self.threshold.as_millis(),
                "Operation completed (exceeded threshold)"
            );
        } else {
            info!(
                operation = %self.name,
                duration_ms = duration.as_millis(),
                "Operation completed"
            );
        }

        duration
    }

    /// Get the elapsed time without finishing the timer
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let duration = self.start.elapsed();
            info!(
                operation = %self.name,
                duration_ms = duration.as_millis(),
                "Operation completed (implicit)"
            );
        }
    }
}

/// Performance metrics collector
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    operations: Vec<OperationMetric>,
}

#[derive(Debug)]
struct OperationMetric {
    name: String,
    duration: Duration,
    timestamp: Instant,
}

impl PerformanceMetrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an operation duration
    pub fn record(&mut self, name: impl Into<String>, duration: Duration) {
        self.operations.push(OperationMetric {
            name: name.into(),
            duration,
            timestamp: Instant::now(),
        });
    }

    /// Get average duration for an operation
    pub fn average_duration(&self, name: &str) -> Option<Duration> {
        let durations: Vec<Duration> = self.operations
            .iter()
            .filter(|op| op.name == name)
            .map(|op| op.duration)
            .collect();

        if durations.is_empty() {
            None
        } else {
            let sum: Duration = durations.iter().sum();
            Some(sum / durations.len() as u32)
        }
    }

    /// Get total operations recorded
    pub fn total_operations(&self) -> usize {
        self.operations.len()
    }

    /// Get operations by name
    pub fn operations_by_name(&self, name: &str) -> Vec<Duration> {
        self.operations
            .iter()
            .filter(|op| op.name == name)
            .map(|op| op.duration)
            .collect()
    }

    /// Get all operation names
    pub fn operation_names(&self) -> Vec<&str> {
        self.operations
            .iter()
            .map(|op| op.name.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Performance Metrics Report\n");
        report.push_str("===========================\n\n");

        let names = self.operation_names();
        for name in names {
            let durations = self.operations_by_name(name);
            if let Some(avg) = self.average_duration(name) {
                let min = durations.iter().min().unwrap();
                let max = durations.iter().max().unwrap();

                report.push_str(&format!(
                    "{}: avg={:.2}ms, min={:.2}ms, max={:.2}ms, count={}\n",
                    name,
                    avg.as_millis(),
                    min.as_millis(),
                    max.as_millis(),
                    durations.len()
                ));
            }
        }

        report
    }
}

/// Structured logging macros for common patterns
#[macro_export]
macro_rules! log_operation_start {
    ($operation:expr) => {
        tracing::info!(
            operation = $operation,
            "Starting operation"
        );
    };
    ($operation:expr, $($key:tt = $value:expr),+) => {
        tracing::info!(
            operation = $operation,
            $($key = $value),+,
            "Starting operation"
        );
    };
}

#[macro_export]
macro_rules! log_operation_complete {
    ($operation:expr, $duration:expr) => {
        tracing::info!(
            operation = $operation,
            duration_ms = $duration.as_millis(),
            "Operation completed"
        );
    };
    ($operation:expr, $duration:expr, $($key:tt = $value:expr),+) => {
        tracing::info!(
            operation = $operation,
            duration_ms = $duration.as_millis(),
            $($key = $value),+,
            "Operation completed"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr) => {
        tracing::error!(
            error = %$error,
            error_category = $error.category(),
            "Operation failed"
        );
    };
    ($error:expr, $($key:tt = $value:expr),+) => {
        tracing::error!(
            error = %$error,
            error_category = $error.category(),
            $($key = $value),+,
            "Operation failed"
        );
    };
}

/// Initialize logging for tests
#[cfg(test)]
pub fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_timer() {
        init_test_logging();

        let timer = OperationTimer::new("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        let duration = timer.finish();

        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record("op1", Duration::from_millis(100));
        metrics.record("op1", Duration::from_millis(200));
        metrics.record("op2", Duration::from_millis(50));

        assert_eq!(metrics.total_operations(), 3);

        let avg = metrics.average_duration("op1").unwrap();
        assert_eq!(avg, Duration::from_millis(150));

        let report = metrics.generate_report();
        assert!(report.contains("op1"));
        assert!(report.contains("op2"));
    }

    #[test]
    fn test_logging_macros() {
        init_test_logging();

        log_operation_start!("test_operation");
        let timer = OperationTimer::new("test_operation");
        std::thread::sleep(Duration::from_millis(1));
        let duration = timer.finish();
        log_operation_complete!("test_operation", duration);

        let error = crate::error::CziError::config("test error");
        log_error!(error);
    }
}
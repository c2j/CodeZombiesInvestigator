//! Base traits and utilities for the CodeZombiesInvestigator core services
//!
//! This module provides common traits, structs, and utilities that are used
//! across different services in the system, including error handling patterns,
//! service lifecycle management, and common utilities.

use crate::{Result, CziError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn, error, trace};

/// Common service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,

    /// Operation timeout in seconds
    pub operation_timeout_secs: u64,

    /// Whether to enable debug logging
    pub debug_enabled: bool,

    /// Custom service-specific configuration
    pub custom_config: HashMap<String, serde_json::Value>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "default_service".to_string(),
            version: "1.0.0".to_string(),
            max_concurrent_operations: 10,
            operation_timeout_secs: 30,
            debug_enabled: false,
            custom_config: HashMap::new(),
        }
    }
}

/// Service lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    /// Service is being initialized
    Initializing,

    /// Service is ready to accept requests
    Ready,

    /// Service is currently processing requests
    Active,

    /// Service is being shutdown
    Shutting,

    /// Service has been shut down
    Shutdown,
}

impl ServiceState {
    /// Check if service is in an active state
    pub fn is_active(&self) -> bool {
        matches!(self, ServiceState::Ready | ServiceState::Active)
    }

    /// Check if service can accept new requests
    pub fn can_accept_requests(&self) -> bool {
        matches!(self, ServiceState::Ready | ServiceState::Active)
    }
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Service name
    pub service_name: String,

    /// Current service state
    pub state: ServiceState,

    /// Whether the service is healthy
    pub is_healthy: bool,

    /// Health check timestamp
    pub check_timestamp: DateTime<Utc>,

    /// Additional health information
    pub details: HashMap<String, String>,

    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
}

impl ServiceHealth {
    /// Create a new health status
    pub fn new(service_name: String, state: ServiceState) -> Self {
        Self {
            is_healthy: state.is_active(),
            service_name,
            state,
            check_timestamp: Utc::now(),
            details: HashMap::new(),
            response_time_ms: None,
        }
    }

    /// Add health detail
    pub fn add_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Set response time
    pub fn with_response_time(mut self, response_time_ms: u64) -> Self {
        self.response_time_ms = Some(response_time_ms);
        self
    }
}

/// Service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Service name
    pub service_name: String,

    /// Total operations performed
    pub total_operations: u64,

    /// Successful operations
    pub successful_operations: u64,

    /// Failed operations
    pub failed_operations: u64,

    /// Average operation time in milliseconds
    pub avg_operation_time_ms: f64,

    /// Operations per second
    pub operations_per_second: f64,

    /// Current concurrent operations
    pub current_concurrent_operations: usize,

    /// Peak concurrent operations
    pub peak_concurrent_operations: usize,

    /// Last reset timestamp
    pub last_reset_timestamp: DateTime<Utc>,

    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl ServiceMetrics {
    /// Create new metrics for a service
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            avg_operation_time_ms: 0.0,
            operations_per_second: 0.0,
            current_concurrent_operations: 0,
            peak_concurrent_operations: 0,
            last_reset_timestamp: Utc::now(),
            custom_metrics: HashMap::new(),
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.successful_operations as f64 / self.total_operations as f64) * 100.0
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self, duration_ms: u64) {
        self.total_operations += 1;
        self.successful_operations += 1;
        self.update_average_time(duration_ms);
    }

    /// Record a failed operation
    pub fn record_failure(&mut self, duration_ms: u64) {
        self.total_operations += 1;
        self.failed_operations += 1;
        self.update_average_time(duration_ms);
    }

    /// Update average operation time
    fn update_average_time(&mut self, duration_ms: u64) {
        if self.total_operations == 1 {
            self.avg_operation_time_ms = duration_ms as f64;
        } else {
            let total_time = self.avg_operation_time_ms * (self.total_operations - 1) as f64 + duration_ms as f64;
            self.avg_operation_time_ms = total_time / self.total_operations as f64;
        }
    }

    /// Reset metrics
    pub fn reset(&mut self) {
        *self = Self::new(self.service_name.clone());
    }
}

/// Base trait for all services
#[async_trait]
pub trait Service: Send + Sync {
    /// Get the service name
    fn name(&self) -> &str;

    /// Get the service version
    fn version(&self) -> &str;

    /// Initialize the service
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the service
    async fn shutdown(&mut self) -> Result<()>;

    /// Get the current service state
    fn state(&self) -> ServiceState;

    /// Get service configuration
    fn config(&self) -> &ServiceConfig;

    /// Get service metrics
    fn metrics(&self) -> &ServiceMetrics;

    /// Perform health check
    async fn health_check(&self) -> Result<ServiceHealth>;

    /// Check if the service can accept requests
    fn can_accept_requests(&self) -> bool {
        self.state().can_accept_requests()
    }
}

/// Base trait for analysis services
#[async_trait]
pub trait AnalysisService: Service {
    /// Get supported file types
    fn supported_file_types(&self) -> Vec<String>;

    /// Check if a file type is supported
    fn supports_file_type(&self, file_path: &str) -> bool {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        self.supported_file_types()
            .iter()
            .any(|supported| supported == extension)
    }

    /// Validate service configuration
    fn validate_config(&self) -> Result<()> {
        let config = self.config();

        if config.name.is_empty() {
            return Err(CziError::validation_error("Service name cannot be empty"));
        }

        if config.max_concurrent_operations == 0 {
            return Err(CziError::validation_error(
                "Max concurrent operations must be greater than 0"
            ));
        }

        if config.operation_timeout_secs == 0 {
            return Err(CziError::validation_error(
                "Operation timeout must be greater than 0"
            ));
        }

        Ok(())
    }
}

/// Base implementation for common service functionality
pub struct BaseService {
    /// Service configuration
    config: ServiceConfig,

    /// Current service state
    state: Arc<RwLock<ServiceState>>,

    /// Service metrics
    metrics: Arc<RwLock<ServiceMetrics>>,

    /// Concurrency limiter
    semaphore: Arc<Semaphore>,

    /// Initialization timestamp
    initialized_at: Option<DateTime<Utc>>,

    /// Shutdown timestamp
    shutdown_at: Option<DateTime<Utc>>,
}

impl BaseService {
    /// Create a new base service with the given configuration
    pub fn new(config: ServiceConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
        let state = Arc::new(RwLock::new(ServiceState::Initializing));
        let metrics = Arc::new(RwLock::new(ServiceMetrics::new(config.name.clone())));

        Self {
            config,
            state,
            metrics,
            semaphore,
            initialized_at: None,
            shutdown_at: None,
        }
    }

    /// Initialize the base service
    pub async fn initialize_base(&mut self) -> Result<()> {
        info!("Initializing service: {}", self.config.name);

        // Validate configuration
        self.validate_base_config()?;

        // Set state to ready
        {
            let mut state = self.state.write().map_err(|e| {
                CziError::internal_error(&format!("Failed to acquire state lock: {}", e))
            })?;
            *state = ServiceState::Ready;
        }

        self.initialized_at = Some(Utc::now());

        info!("Service initialized successfully: {}", self.config.name);
        Ok(())
    }

    /// Shutdown the base service
    pub async fn shutdown_base(&mut self) -> Result<()> {
        info!("Shutting down service: {}", self.config.name);

        // Set state to shutting
        {
            let mut state = self.state.write().map_err(|e| {
                CziError::internal_error(&format!("Failed to acquire state lock: {}", e))
            })?;
            *state = ServiceState::Shutting;
        }

        // Wait for all operations to complete
        let permits = self.semaphore.available_permits();
        let max_permits = self.semaphore.max_permits();
        if permits < max_permits {
            debug!(
                "Waiting for {} operations to complete",
                max_permits - permits
            );
            // In a real implementation, you might want to wait with a timeout
        }

        // Set state to shutdown
        {
            let mut state = self.state.write().map_err(|e| {
                CziError::internal_error(&format!("Failed to acquire state lock: {}", e))
            })?;
            *state = ServiceState::Shutdown;
        }

        self.shutdown_at = Some(Utc::now());

        info!("Service shut down successfully: {}", self.config.name);
        Ok(())
    }

    /// Acquire a permit for an operation
    pub async fn acquire_operation_permit(&self) -> Result<tokio::sync::SemaphorePermit<'_>> {
        let permit = self.semaphore.acquire().await.map_err(|e| {
            CziError::internal_error(&format!("Failed to acquire operation permit: {}", e))
        })?;

        // Update concurrent operations count
        {
            let mut metrics = self.metrics.write().map_err(|e| {
                CziError::internal_error(&format!("Failed to acquire metrics lock: {}", e))
            })?;
            metrics.current_concurrent_operations += 1;
            metrics.peak_concurrent_operations = metrics.peak_concurrent_operations
                .max(metrics.current_concurrent_operations);
        }

        Ok(permit)
    }

    /// Execute an operation with metrics tracking
    pub async fn execute_operation<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        let start_time = Instant::now();
        let _permit = self.acquire_operation_permit().await?;

        let result = operation();

        let duration = start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;

        // Update metrics
        {
            let mut metrics = self.metrics.write().map_err(|e| {
                CziError::internal_error(&format!("Failed to acquire metrics lock: {}", e))
            })?;
            metrics.current_concurrent_operations = metrics.current_concurrent_operations.saturating_sub(1);

            match &result {
                Ok(_) => {
                    metrics.record_success(duration_ms);
                    trace!("Operation completed successfully in {}ms", duration_ms);
                }
                Err(e) => {
                    metrics.record_failure(duration_ms);
                    warn!("Operation failed in {}ms: {}", duration_ms, e);
                }
            }
        }

        result
    }

    /// Get current state
    pub fn get_state(&self) -> ServiceState {
        self.state.read()
            .map(|guard| *guard)
            .unwrap_or(ServiceState::Shutdown)
    }

    /// Get configuration
    pub fn get_config(&self) -> &ServiceConfig {
        &self.config
    }

    /// Get metrics
    pub fn get_metrics(&self) -> ServiceMetrics {
        self.metrics.read()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| ServiceMetrics::new(self.config.name.clone()))
    }

    /// Perform basic health check
    pub async fn perform_health_check(&self) -> Result<ServiceHealth> {
        let start_time = Instant::now();

        let state = self.get_state();
        let metrics = self.get_metrics();

        let is_healthy = state.is_active() && metrics.failed_operations < metrics.total_operations / 2;

        let mut health = ServiceHealth::new(self.config.name.clone(), state);
        health.is_healthy = is_healthy;
        health.response_time_ms = Some(start_time.elapsed().as_millis() as u64);

        // Add health details
        health.details.insert("total_operations".to_string(), metrics.total_operations.to_string());
        health.details.insert("success_rate".to_string(), format!("{:.2}%", metrics.success_rate()));
        health.details.insert("current_concurrent".to_string(), metrics.current_concurrent_operations.to_string());

        if let Some(initialized_at) = self.initialized_at {
            health.details.insert("uptime_seconds".to_string(),
                (Utc::now() - initialized_at).num_seconds().to_string());
        }

        Ok(health)
    }

    /// Validate base configuration
    fn validate_base_config(&self) -> Result<()> {
        if self.config.name.is_empty() {
            return Err(CziError::validation_error("Service name cannot be empty"));
        }

        if self.config.max_concurrent_operations == 0 {
            return Err(CziError::validation_error(
                "Max concurrent operations must be greater than 0"
            ));
        }

        if self.config.operation_timeout_secs == 0 {
            return Err(CziError::validation_error(
                "Operation timeout must be greater than 0"
            ));
        }

        Ok(())
    }
}

/// Utility functions for services
pub mod utils {
    use super::*;

    /// Create a default service configuration
    pub fn default_service_config(name: &str, version: &str) -> ServiceConfig {
        ServiceConfig {
            name: name.to_string(),
            version: version.to_string(),
            max_concurrent_operations: 10,
            operation_timeout_secs: 30,
            debug_enabled: false,
            custom_config: HashMap::new(),
        }
    }

    /// Create a service configuration for high-performance scenarios
    pub fn high_performance_config(name: &str, version: &str) -> ServiceConfig {
        ServiceConfig {
            name: name.to_string(),
            version: version.to_string(),
            max_concurrent_operations: 50,
            operation_timeout_secs: 60,
            debug_enabled: false,
            custom_config: HashMap::new(),
        }
    }

    /// Create a service configuration for debugging
    pub fn debug_config(name: &str, version: &str) -> ServiceConfig {
        ServiceConfig {
            name: name.to_string(),
            version: version.to_string(),
            max_concurrent_operations: 1,
            operation_timeout_secs: 300, // 5 minutes
            debug_enabled: true,
            custom_config: HashMap::new(),
        }
    }

    /// Measure execution time of a function
    pub fn measure_time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Execute a function with timeout
    pub async fn with_timeout<F, T>(
        future: F,
        timeout_secs: u64,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        match tokio::time::timeout(Duration::from_secs(timeout_secs), future).await {
            Ok(result) => result,
            Err(_) => Err(CziError::timeout_error(&format!(
                "Operation timed out after {} seconds", timeout_secs
            ))),
        }
    }

    /// Retry an operation with exponential backoff
    pub async fn retry_with_backoff<F, T, E>(
        operation: F,
        max_attempts: usize,
        initial_delay_ms: u64,
    ) -> std::result::Result<T, E>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, E>> + Send>>,
        E: std::fmt::Display,
    {
        let mut delay = initial_delay_ms;

        for attempt in 1..=max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(e);
                    }

                    warn!("Attempt {} failed: {}. Retrying in {}ms", attempt, e, delay);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }

        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert_eq!(config.name, "default_service");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.max_concurrent_operations, 10);
        assert_eq!(config.operation_timeout_secs, 30);
        assert!(!config.debug_enabled);
    }

    #[test]
    fn test_service_state_transitions() {
        assert!(ServiceState::Ready.is_active());
        assert!(ServiceState::Active.is_active());
        assert!(!ServiceState::Initializing.is_active());
        assert!(!ServiceState::Shutting.is_active());
        assert!(!ServiceState::Shutdown.is_active());

        assert!(ServiceState::Ready.can_accept_requests());
        assert!(ServiceState::Active.can_accept_requests());
        assert!(!ServiceState::Initializing.can_accept_requests());
    }

    #[test]
    fn test_service_health_creation() {
        let health = ServiceHealth::new(
            "test_service".to_string(),
            ServiceState::Ready,
        );

        assert_eq!(health.service_name, "test_service");
        assert_eq!(health.state, ServiceState::Ready);
        assert!(health.is_healthy);
        assert!(health.check_timestamp <= Utc::now());
        assert!(health.details.is_empty());
        assert!(health.response_time_ms.is_none());
    }

    #[test]
    fn test_service_health_with_details() {
        let health = ServiceHealth::new(
            "test_service".to_string(),
            ServiceState::Ready,
        )
        .add_detail("test_key".to_string(), "test_value".to_string())
        .with_response_time(42);

        assert_eq!(health.details.get("test_key"), Some(&"test_value".to_string()));
        assert_eq!(health.response_time_ms, Some(42));
    }

    #[test]
    fn test_service_metrics_new() {
        let metrics = ServiceMetrics::new("test_service".to_string());
        assert_eq!(metrics.service_name, "test_service");
        assert_eq!(metrics.total_operations, 0);
        assert_eq!(metrics.successful_operations, 0);
        assert_eq!(metrics.failed_operations, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_service_metrics_record_operations() {
        let mut metrics = ServiceMetrics::new("test_service".to_string());

        metrics.record_success(100);
        assert_eq!(metrics.total_operations, 1);
        assert_eq!(metrics.successful_operations, 1);
        assert_eq!(metrics.failed_operations, 0);
        assert_eq!(metrics.success_rate(), 100.0);
        assert_eq!(metrics.avg_operation_time_ms, 100.0);

        metrics.record_failure(200);
        assert_eq!(metrics.total_operations, 2);
        assert_eq!(metrics.successful_operations, 1);
        assert_eq!(metrics.failed_operations, 1);
        assert_eq!(metrics.success_rate(), 50.0);
        assert_eq!(metrics.avg_operation_time_ms, 150.0);
    }

    #[test]
    fn test_service_metrics_reset() {
        let mut metrics = ServiceMetrics::new("test_service".to_string());
        metrics.record_success(100);
        metrics.reset();

        assert_eq!(metrics.total_operations, 0);
        assert_eq!(metrics.successful_operations, 0);
        assert_eq!(metrics.failed_operations, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_base_service_initialization() {
        let config = utils::default_service_config("test_service", "1.0.0");
        let mut service = BaseService::new(config);

        let result = service.initialize_base().await;
        assert!(result.is_ok());
        assert_eq!(service.get_state(), ServiceState::Ready);
        assert!(service.initialized_at.is_some());
    }

    #[tokio::test]
    async fn test_base_service_shutdown() {
        let config = utils::default_service_config("test_service", "1.0.0");
        let mut service = BaseService::new(config);

        // Initialize first
        service.initialize_base().await.unwrap();

        // Then shutdown
        let result = service.shutdown_base().await;
        assert!(result.is_ok());
        assert_eq!(service.get_state(), ServiceState::Shutdown);
        assert!(service.shutdown_at.is_some());
    }

    #[test]
    fn test_utils_configurations() {
        let default_config = utils::default_service_config("test", "1.0");
        assert_eq!(default_config.max_concurrent_operations, 10);

        let hp_config = utils::high_performance_config("test", "1.0");
        assert_eq!(hp_config.max_concurrent_operations, 50);

        let debug_config = utils::debug_config("test", "1.0");
        assert_eq!(debug_config.max_concurrent_operations, 1);
        assert!(debug_config.debug_enabled);
    }

    #[test]
    fn test_measure_time() {
        let (result, duration) = utils::measure_time(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 10);
    }
}
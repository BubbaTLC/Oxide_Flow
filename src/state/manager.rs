use crate::state::backend::{
    BackendConfig, BackendHealth, CleanupResult, FileBackend, LockInfo, MemoryBackend, StateBackend,
};
use crate::state::types::{ErrorRecord, PipelineState, StateError, StepState};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Configuration for the StateManager
#[derive(Debug, Clone)]
pub struct StateManagerConfig {
    /// Backend configuration
    pub backend: BackendConfig,

    /// Default lock timeout in milliseconds
    pub default_lock_timeout_ms: u64,

    /// Worker ID for this instance
    pub worker_id: String,

    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,

    /// Maximum number of retries for state operations
    pub max_retries: u64,

    /// Cleanup interval in hours
    pub cleanup_interval_hours: u64,

    /// Maximum age for state files in hours before cleanup
    pub max_state_age_hours: u64,
}

impl Default for StateManagerConfig {
    fn default() -> Self {
        Self {
            backend: BackendConfig::Memory { persistent: false },
            default_lock_timeout_ms: 30000, // 30 seconds
            worker_id: format!("worker_{}", Uuid::new_v4()),
            heartbeat_interval_ms: 5000, // 5 seconds
            max_retries: 3,
            cleanup_interval_hours: 24, // Daily cleanup
            max_state_age_hours: 168,   // 7 days
        }
    }
}

/// High-level state manager providing pipeline state management operations
pub struct StateManager {
    backend: Arc<dyn StateBackend>,
    config: StateManagerConfig,
}

impl StateManager {
    /// Create a new StateManager with the given configuration
    pub async fn new(config: StateManagerConfig) -> Result<Self, StateError> {
        let backend: Arc<dyn StateBackend> = match &config.backend {
            BackendConfig::File { .. } => Arc::new(FileBackend::new(config.backend.clone())?),
            BackendConfig::Memory { .. } => Arc::new(MemoryBackend::new()),
            BackendConfig::Redis { .. } => {
                return Err(StateError::BackendError {
                    details: "Redis backend not yet implemented".to_string(),
                });
            }
        };

        Ok(Self { backend, config })
    }

    /// Create a new StateManager with memory backend (for testing)
    pub fn new_memory() -> Self {
        let config = StateManagerConfig {
            backend: BackendConfig::Memory { persistent: false },
            ..Default::default()
        };

        Self {
            backend: Arc::new(MemoryBackend::new()),
            config,
        }
    }

    /// Initialize a new pipeline state
    pub async fn initialize_pipeline(
        &self,
        pipeline_id: &str,
        run_id: Option<String>,
    ) -> Result<PipelineState, StateError> {
        let run_id = run_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let mut state = PipelineState::new(pipeline_id.to_string(), run_id);

        // Set worker ID if configured
        state.worker_id = Some(self.config.worker_id.clone());

        // Save initial state
        self.save_state(&state).await?;

        Ok(state)
    }

    /// Load pipeline state by ID
    pub async fn load_state(&self, pipeline_id: &str) -> Result<PipelineState, StateError> {
        self.backend.load_state(pipeline_id).await
    }

    /// Save pipeline state with retry logic
    pub async fn save_state(&self, state: &PipelineState) -> Result<(), StateError> {
        self.retry_operation(|| async { self.backend.save_state(state).await })
            .await
    }

    /// Update pipeline state with a closure
    pub async fn update_state<F, R>(&self, pipeline_id: &str, updater: F) -> Result<R, StateError>
    where
        F: FnOnce(&mut PipelineState) -> R,
    {
        let mut state = self.load_state(pipeline_id).await?;
        let result = updater(&mut state);
        self.save_state(&state).await?;
        Ok(result)
    }

    /// Update pipeline state with locking
    pub async fn update_state_locked<F, R>(
        &self,
        pipeline_id: &str,
        updater: F,
    ) -> Result<R, StateError>
    where
        F: FnOnce(&mut PipelineState) -> R,
    {
        let _lock = self
            .acquire_lock(pipeline_id, self.config.default_lock_timeout_ms)
            .await?;

        let mut state = self.load_state(pipeline_id).await?;
        let result = updater(&mut state);
        self.save_state(&state).await?;

        Ok(result)
    }

    /// Delete pipeline state
    pub async fn delete_state(&self, pipeline_id: &str) -> Result<(), StateError> {
        self.backend.delete_state(pipeline_id).await
    }

    /// List all pipeline IDs
    pub async fn list_pipelines(&self) -> Result<Vec<String>, StateError> {
        self.backend.list_pipelines().await
    }

    /// Acquire a lock on pipeline state
    pub async fn acquire_lock(
        &self,
        pipeline_id: &str,
        timeout_ms: u64,
    ) -> Result<StateManagerLock, StateError> {
        let lock_info = self
            .backend
            .acquire_lock(pipeline_id, &self.config.worker_id, timeout_ms)
            .await?;

        Ok(StateManagerLock {
            pipeline_id: pipeline_id.to_string(),
            worker_id: self.config.worker_id.clone(),
            backend: Arc::clone(&self.backend),
            lock_info,
        })
    }

    /// Check if a pipeline is locked
    pub async fn is_locked(&self, pipeline_id: &str) -> Result<Option<LockInfo>, StateError> {
        self.backend.is_locked(pipeline_id).await
    }

    /// Force release a lock (admin operation)
    pub async fn force_release_lock(&self, pipeline_id: &str) -> Result<(), StateError> {
        self.backend.force_release_lock(pipeline_id).await
    }

    /// Update heartbeat for a pipeline
    pub async fn update_heartbeat(&self, pipeline_id: &str) -> Result<(), StateError> {
        self.update_state(pipeline_id, |state| {
            state.update_heartbeat();
        })
        .await
    }

    /// Add an error to pipeline state
    pub async fn add_error(&self, pipeline_id: &str, error: ErrorRecord) -> Result<(), StateError> {
        self.update_state(pipeline_id, |state| {
            state.add_error(error);
        })
        .await
    }

    /// Update step state
    pub async fn update_step_state(
        &self,
        pipeline_id: &str,
        step_id: &str,
        step_state: StepState,
    ) -> Result<(), StateError> {
        self.update_state(pipeline_id, |state| {
            state.step_states.insert(step_id.to_string(), step_state);
            state.increment_version();
        })
        .await
    }

    /// Get step state
    pub async fn get_step_state(
        &self,
        pipeline_id: &str,
        step_id: &str,
    ) -> Result<Option<StepState>, StateError> {
        let state = self.load_state(pipeline_id).await?;
        Ok(state.step_states.get(step_id).cloned())
    }

    /// Update processing progress
    pub async fn update_progress(
        &self,
        pipeline_id: &str,
        records_processed: u64,
        data_size_processed: u64,
        last_processed_id: Option<String>,
    ) -> Result<(), StateError> {
        self.update_state(pipeline_id, |state| {
            state.records_processed += records_processed;
            state.data_size_processed += data_size_processed;

            if let Some(id) = last_processed_id {
                state.last_processed_id = id;
            }

            state.last_success_timestamp = Utc::now();
            state.increment_version();
        })
        .await
    }

    /// Check for stale pipelines and clean them up
    pub async fn find_stale_pipelines(
        &self,
        stale_threshold_ms: u64,
    ) -> Result<Vec<String>, StateError> {
        let pipeline_ids = self.list_pipelines().await?;
        let mut stale_pipelines = Vec::new();

        for pipeline_id in pipeline_ids {
            if let Ok(state) = self.load_state(&pipeline_id).await {
                if state.is_stale(stale_threshold_ms) {
                    stale_pipelines.push(pipeline_id);
                }
            }
        }

        Ok(stale_pipelines)
    }

    /// Perform health check on the backend
    pub async fn health_check(&self) -> Result<BackendHealth, StateError> {
        self.backend.health_check().await
    }

    /// Cleanup old state and expired locks
    pub async fn cleanup(&self) -> Result<CleanupResult, StateError> {
        self.backend.cleanup(self.config.max_state_age_hours).await
    }

    /// Start automatic heartbeat for a pipeline
    pub async fn start_heartbeat(&self, pipeline_id: String) -> HeartbeatHandle {
        let manager = StateManager {
            backend: Arc::clone(&self.backend),
            config: self.config.clone(),
        };

        let interval_ms = self.config.heartbeat_interval_ms;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));

            loop {
                interval.tick().await;

                if let Err(e) = manager.update_heartbeat(&pipeline_id).await {
                    eprintln!("Heartbeat failed for pipeline {pipeline_id}: {e}");
                    break;
                }
            }
        });

        HeartbeatHandle { handle }
    }

    /// Get configuration
    pub fn config(&self) -> &StateManagerConfig {
        &self.config
    }

    /// Retry an operation with exponential backoff
    async fn retry_operation<F, Fut, T>(&self, operation: F) -> Result<T, StateError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, StateError>>,
    {
        let mut retries = 0;
        let max_retries = self.config.max_retries;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    retries += 1;

                    if retries >= max_retries {
                        return Err(e);
                    }

                    // Exponential backoff: 100ms, 200ms, 400ms, etc.
                    let delay_ms = 100 * (1 << retries);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }
}

/// RAII lock guard for pipeline state
pub struct StateManagerLock {
    pipeline_id: String,
    worker_id: String,
    backend: Arc<dyn StateBackend>,
    lock_info: LockInfo,
}

impl StateManagerLock {
    /// Get the lock information
    pub fn lock_info(&self) -> &LockInfo {
        &self.lock_info
    }

    /// Get the pipeline ID
    pub fn pipeline_id(&self) -> &str {
        &self.pipeline_id
    }

    /// Check if the lock is still valid
    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.lock_info.expires_at {
            Utc::now() < expires_at
        } else {
            true // No expiration means valid
        }
    }
}

impl Drop for StateManagerLock {
    fn drop(&mut self) {
        // Release lock on drop (fire and forget)
        let backend = Arc::clone(&self.backend);
        let pipeline_id = self.pipeline_id.clone();
        let worker_id = self.worker_id.clone();

        tokio::spawn(async move {
            if let Err(e) = backend.release_lock(&pipeline_id, &worker_id).await {
                eprintln!("Failed to release lock for pipeline {pipeline_id}: {e}");
            }
        });
    }
}

/// Handle for a background heartbeat task
pub struct HeartbeatHandle {
    handle: tokio::task::JoinHandle<()>,
}

impl HeartbeatHandle {
    /// Stop the heartbeat
    pub fn stop(self) {
        self.handle.abort();
    }

    /// Check if the heartbeat is still running
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }
}

/// Trait for state change observers
#[async_trait]
pub trait StateObserver: Send + Sync {
    /// Called when pipeline state changes
    async fn on_state_change(
        &self,
        pipeline_id: &str,
        old_state: Option<&PipelineState>,
        new_state: &PipelineState,
    );

    /// Called when an error is added to pipeline state
    async fn on_error(&self, pipeline_id: &str, error: &ErrorRecord);

    /// Called when a pipeline lock is acquired
    async fn on_lock_acquired(&self, pipeline_id: &str, worker_id: &str);

    /// Called when a pipeline lock is released
    async fn on_lock_released(&self, pipeline_id: &str, worker_id: &str);
}

/// StateManager with observer support
pub struct ObservableStateManager {
    manager: StateManager,
    observers: Vec<Arc<dyn StateObserver>>,
}

impl ObservableStateManager {
    /// Create a new observable state manager
    pub fn new(manager: StateManager) -> Self {
        Self {
            manager,
            observers: Vec::new(),
        }
    }

    /// Add an observer
    pub fn add_observer(&mut self, observer: Arc<dyn StateObserver>) {
        self.observers.push(observer);
    }

    /// Get the underlying state manager
    pub fn manager(&self) -> &StateManager {
        &self.manager
    }

    /// Save state with observer notifications
    pub async fn save_state_observed(
        &self,
        old_state: Option<&PipelineState>,
        new_state: &PipelineState,
    ) -> Result<(), StateError> {
        // Save the state
        self.manager.save_state(new_state).await?;

        // Notify observers
        for observer in &self.observers {
            observer
                .on_state_change(&new_state.pipeline_id, old_state, new_state)
                .await;
        }

        Ok(())
    }

    /// Add error with observer notifications
    pub async fn add_error_observed(
        &self,
        pipeline_id: &str,
        error: ErrorRecord,
    ) -> Result<(), StateError> {
        // Notify observers
        for observer in &self.observers {
            observer.on_error(pipeline_id, &error).await;
        }

        // Add the error
        self.manager.add_error(pipeline_id, error).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::{ErrorRecord, ErrorType, PipelineStatus};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_state_manager_creation() {
        let manager = StateManager::new_memory();
        assert!(manager.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_pipeline_initialization() {
        let manager = StateManager::new_memory();

        let state = manager
            .initialize_pipeline("test_pipeline", None)
            .await
            .unwrap();

        assert_eq!(state.pipeline_id, "test_pipeline");
        assert!(!state.run_id.is_empty());
        assert_eq!(state.status, PipelineStatus::Pending);
        assert_eq!(state.version, 1);

        // Verify state was saved
        let loaded_state = manager.load_state("test_pipeline").await.unwrap();
        assert_eq!(loaded_state.pipeline_id, state.pipeline_id);
        assert_eq!(loaded_state.run_id, state.run_id);
    }

    #[tokio::test]
    async fn test_state_updates() {
        let manager = StateManager::new_memory();
        let state = manager
            .initialize_pipeline("test_pipeline", Some("run_123".to_string()))
            .await
            .unwrap();

        // Update progress
        manager
            .update_progress("test_pipeline", 100, 1024, Some("record_100".to_string()))
            .await
            .unwrap();

        let updated_state = manager.load_state("test_pipeline").await.unwrap();
        assert_eq!(updated_state.records_processed, 100);
        assert_eq!(updated_state.data_size_processed, 1024);
        assert_eq!(updated_state.last_processed_id, "record_100");
        assert!(updated_state.version > state.version);
    }

    #[tokio::test]
    async fn test_state_locking() {
        let backend: Arc<dyn StateBackend> = Arc::new(MemoryBackend::new());

        let config1 = StateManagerConfig {
            backend: BackendConfig::Memory { persistent: false },
            worker_id: "worker_1".to_string(),
            ..Default::default()
        };

        let config2 = StateManagerConfig {
            backend: BackendConfig::Memory { persistent: false },
            worker_id: "worker_2".to_string(),
            ..Default::default()
        };

        let manager1 = StateManager {
            backend: Arc::clone(&backend),
            config: config1,
        };

        let manager2 = StateManager {
            backend: Arc::clone(&backend),
            config: config2,
        };

        manager1
            .initialize_pipeline("test_pipeline", None)
            .await
            .unwrap();

        // Acquire lock with manager1
        let _lock = manager1.acquire_lock("test_pipeline", 5000).await.unwrap();

        // Verify pipeline is locked
        let lock_info = manager1.is_locked("test_pipeline").await.unwrap();
        assert!(lock_info.is_some());

        // Try to acquire lock with manager2 (should timeout)
        let lock_result = manager2.acquire_lock("test_pipeline", 100).await;
        assert!(matches!(lock_result, Err(StateError::LockTimeout { .. })));

        // Drop the lock (automatic release)
        drop(_lock);

        // Small delay to allow async drop to complete
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should be able to acquire lock now with manager2
        let _lock2 = manager2.acquire_lock("test_pipeline", 1000).await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling() {
        let manager = StateManager::new_memory();
        manager
            .initialize_pipeline("test_pipeline", None)
            .await
            .unwrap();

        let error = ErrorRecord::processing_error(
            "step_1".to_string(),
            "Processing failed".to_string(),
            "Invalid data format".to_string(),
            true,
        );

        manager
            .add_error("test_pipeline", error.clone())
            .await
            .unwrap();

        let state = manager.load_state("test_pipeline").await.unwrap();
        assert_eq!(state.errors.len(), 1);
        assert_eq!(state.errors[0].message, "Processing failed");
        assert_eq!(state.errors[0].error_type, ErrorType::Processing);
    }

    #[tokio::test]
    async fn test_step_state_management() {
        let manager = StateManager::new_memory();
        manager
            .initialize_pipeline("test_pipeline", None)
            .await
            .unwrap();

        let mut step_state = StepState::new("step_1".to_string(), "read_file".to_string());
        step_state.start();
        step_state.records_processed = 50;

        manager
            .update_step_state("test_pipeline", "step_1", step_state.clone())
            .await
            .unwrap();

        let retrieved_step = manager
            .get_step_state("test_pipeline", "step_1")
            .await
            .unwrap();

        assert!(retrieved_step.is_some());
        let step = retrieved_step.unwrap();
        assert_eq!(step.step_id, "step_1");
        assert_eq!(step.records_processed, 50);
        assert!(step.is_running());
    }

    #[tokio::test]
    async fn test_heartbeat_functionality() {
        let manager = StateManager::new_memory();
        let state = manager
            .initialize_pipeline("test_pipeline", None)
            .await
            .unwrap();

        let initial_heartbeat = state.last_heartbeat;

        // Wait a bit then update heartbeat
        tokio::time::sleep(Duration::from_millis(10)).await;
        manager.update_heartbeat("test_pipeline").await.unwrap();

        let updated_state = manager.load_state("test_pipeline").await.unwrap();
        assert!(updated_state.last_heartbeat > initial_heartbeat);
    }

    #[tokio::test]
    async fn test_stale_pipeline_detection() {
        let manager = StateManager::new_memory();
        manager
            .initialize_pipeline("test_pipeline", None)
            .await
            .unwrap();

        // Fresh pipeline should not be stale
        let stale_pipelines = manager.find_stale_pipelines(1000).await.unwrap();
        assert!(stale_pipelines.is_empty());

        // Set an old heartbeat manually
        manager
            .update_state("test_pipeline", |state| {
                state.last_heartbeat = Utc::now() - chrono::Duration::seconds(10);
            })
            .await
            .unwrap();

        // Should now be detected as stale
        let stale_pipelines = manager.find_stale_pipelines(5000).await.unwrap();
        assert_eq!(stale_pipelines, vec!["test_pipeline"]);
    }

    #[tokio::test]
    async fn test_file_backend_integration() {
        let temp_dir = TempDir::new().unwrap();

        let config = StateManagerConfig {
            backend: BackendConfig::File {
                base_path: temp_dir.path().to_path_buf(),
                format: crate::state::backend::SerializationFormat::Json,
                atomic_writes: true,
                lock_timeout_ms: 5000,
            },
            ..Default::default()
        };

        let manager = StateManager::new(config).await.unwrap();

        // Test basic operations with file backend
        let state = manager
            .initialize_pipeline("file_test", Some("run_file".to_string()))
            .await
            .unwrap();

        assert_eq!(state.pipeline_id, "file_test");
        assert_eq!(state.run_id, "run_file");

        // Verify persistence by creating a new manager
        let config2 = StateManagerConfig {
            backend: BackendConfig::File {
                base_path: temp_dir.path().to_path_buf(),
                format: crate::state::backend::SerializationFormat::Json,
                atomic_writes: true,
                lock_timeout_ms: 5000,
            },
            ..Default::default()
        };

        let manager2 = StateManager::new(config2).await.unwrap();
        let loaded_state = manager2.load_state("file_test").await.unwrap();
        assert_eq!(loaded_state.pipeline_id, "file_test");
        assert_eq!(loaded_state.run_id, "run_file");
    }

    #[tokio::test]
    async fn test_observable_state_manager() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        #[derive(Default)]
        struct TestObserver {
            state_changes: AtomicUsize,
            errors: AtomicUsize,
        }

        #[async_trait]
        impl StateObserver for TestObserver {
            async fn on_state_change(
                &self,
                _pipeline_id: &str,
                _old_state: Option<&PipelineState>,
                _new_state: &PipelineState,
            ) {
                self.state_changes.fetch_add(1, Ordering::SeqCst);
            }

            async fn on_error(&self, _pipeline_id: &str, _error: &ErrorRecord) {
                self.errors.fetch_add(1, Ordering::SeqCst);
            }

            async fn on_lock_acquired(&self, _pipeline_id: &str, _worker_id: &str) {}
            async fn on_lock_released(&self, _pipeline_id: &str, _worker_id: &str) {}
        }

        let manager = StateManager::new_memory();
        let mut observable = ObservableStateManager::new(manager);

        let observer = Arc::new(TestObserver::default());
        observable.add_observer(observer.clone() as Arc<dyn StateObserver>);

        // Initialize pipeline through the observable manager
        let state = observable
            .manager()
            .initialize_pipeline("test_pipeline", None)
            .await
            .unwrap();

        // Save state with observation
        observable.save_state_observed(None, &state).await.unwrap();

        // Add error with observation
        let error = ErrorRecord::config_error("Test error".to_string(), "Test context".to_string());
        observable
            .add_error_observed("test_pipeline", error)
            .await
            .unwrap();

        // Verify observer was called
        assert_eq!(observer.state_changes.load(Ordering::SeqCst), 1);
        assert_eq!(observer.errors.load(Ordering::SeqCst), 1);
    }
}

use crate::state::types::{PipelineState, StateError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use fs4::tokio::AsyncFileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Configuration for different state backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendConfig {
    /// File-based backend configuration
    File {
        /// Base directory for state files
        base_path: PathBuf,
        /// File format for serialization
        format: SerializationFormat,
        /// Whether to use atomic writes (via temp files)
        atomic_writes: bool,
        /// Lock timeout in milliseconds
        lock_timeout_ms: u64,
    },

    /// Memory-based backend (for testing)
    Memory {
        /// Whether to persist state across restarts
        persistent: bool,
    },

    /// Redis backend configuration (future feature)
    #[allow(dead_code)]
    Redis {
        connection_string: String,
        key_prefix: String,
        ttl_seconds: Option<u64>,
    },
}

/// Supported serialization formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SerializationFormat {
    Json,
    Yaml,
    #[allow(dead_code)]
    Bincode, // Future binary format option
}

/// Information about a state lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub pipeline_id: String,
    pub worker_id: String,
    pub locked_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub lock_version: u64,
}

/// State backend trait for different persistence mechanisms
#[async_trait]
pub trait StateBackend: Send + Sync {
    /// Load pipeline state by pipeline ID
    async fn load_state(&self, pipeline_id: &str) -> Result<PipelineState, StateError>;

    /// Save pipeline state with version control
    async fn save_state(&self, state: &PipelineState) -> Result<(), StateError>;

    /// Delete pipeline state
    async fn delete_state(&self, pipeline_id: &str) -> Result<(), StateError>;

    /// List all pipeline IDs that have state
    async fn list_pipelines(&self) -> Result<Vec<String>, StateError>;

    /// Acquire an exclusive lock on pipeline state
    async fn acquire_lock(
        &self,
        pipeline_id: &str,
        worker_id: &str,
        timeout_ms: u64,
    ) -> Result<LockInfo, StateError>;

    /// Release a previously acquired lock
    async fn release_lock(&self, pipeline_id: &str, worker_id: &str) -> Result<(), StateError>;

    /// Check if a pipeline is currently locked
    async fn is_locked(&self, pipeline_id: &str) -> Result<Option<LockInfo>, StateError>;

    /// Force release a lock (admin operation)
    async fn force_release_lock(&self, pipeline_id: &str) -> Result<(), StateError>;

    /// Get backend health status
    async fn health_check(&self) -> Result<BackendHealth, StateError>;

    /// Cleanup expired locks and stale state
    async fn cleanup(&self, max_age_hours: u64) -> Result<CleanupResult, StateError>;
}

/// Health status of a state backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendHealth {
    pub backend_type: String,
    pub healthy: bool,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    pub metrics: HashMap<String, f64>,
}

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub expired_locks_removed: u64,
    pub stale_states_removed: u64,
    pub total_states_checked: u64,
    pub cleanup_duration_ms: u64,
    pub errors: Vec<String>,
}

/// File-based state backend implementation
pub struct FileBackend {
    base_path: PathBuf,
    format: SerializationFormat,
    atomic_writes: bool,
    #[allow(dead_code)] // Used for future timeout configuration
    lock_timeout_ms: u64,
}

impl FileBackend {
    /// Create a new file backend
    pub fn new(config: BackendConfig) -> Result<Self, StateError> {
        match config {
            BackendConfig::File {
                base_path,
                format,
                atomic_writes,
                lock_timeout_ms,
            } => Ok(Self {
                base_path,
                format,
                atomic_writes,
                lock_timeout_ms,
            }),
            _ => Err(StateError::InvalidState {
                details: "FileBackend requires File configuration".to_string(),
            }),
        }
    }

    /// Get the state file path for a pipeline
    fn state_file_path(&self, pipeline_id: &str) -> PathBuf {
        let extension = match self.format {
            SerializationFormat::Json => "json",
            SerializationFormat::Yaml => "yaml",
            SerializationFormat::Bincode => "bin",
        };

        self.base_path
            .join("states")
            .join(format!("{pipeline_id}.{extension}"))
    }

    /// Get the lock file path for a pipeline
    fn lock_file_path(&self, pipeline_id: &str) -> PathBuf {
        self.base_path
            .join("locks")
            .join(format!("{pipeline_id}.lock"))
    }

    /// Serialize state to bytes
    fn serialize_state(&self, state: &PipelineState) -> Result<Vec<u8>, StateError> {
        match self.format {
            SerializationFormat::Json => serde_json::to_vec_pretty(state).map_err(StateError::from),
            SerializationFormat::Yaml => serde_yaml::to_string(state)
                .map(|s| s.into_bytes())
                .map_err(StateError::from),
            SerializationFormat::Bincode => Err(StateError::SerializationError {
                details: "Bincode format not yet implemented".to_string(),
            }),
        }
    }

    /// Deserialize state from bytes
    fn deserialize_state(&self, data: &[u8]) -> Result<PipelineState, StateError> {
        match self.format {
            SerializationFormat::Json => serde_json::from_slice(data).map_err(StateError::from),
            SerializationFormat::Yaml => {
                let text = String::from_utf8(data.to_vec()).map_err(|e| {
                    StateError::SerializationError {
                        details: format!("Invalid UTF-8: {e}"),
                    }
                })?;
                serde_yaml::from_str(&text).map_err(StateError::from)
            }
            SerializationFormat::Bincode => Err(StateError::SerializationError {
                details: "Bincode format not yet implemented".to_string(),
            }),
        }
    }

    /// Ensure directories exist
    async fn ensure_directories(&self) -> Result<(), StateError> {
        let states_dir = self.base_path.join("states");
        let locks_dir = self.base_path.join("locks");

        fs::create_dir_all(&states_dir).await?;
        fs::create_dir_all(&locks_dir).await?;

        Ok(())
    }

    /// Write data to file atomically (if enabled)
    async fn write_file_atomic(&self, path: &PathBuf, data: &[u8]) -> Result<(), StateError> {
        if self.atomic_writes {
            // Write to temporary file first, then rename
            let temp_path = path.with_extension(format!(
                "{}.tmp",
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("tmp")
            ));

            fs::write(&temp_path, data).await?;
            fs::rename(&temp_path, path).await?;
        } else {
            fs::write(path, data).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl StateBackend for FileBackend {
    async fn load_state(&self, pipeline_id: &str) -> Result<PipelineState, StateError> {
        self.ensure_directories().await?;

        let file_path = self.state_file_path(pipeline_id);

        if !file_path.exists() {
            return Err(StateError::PipelineNotFound {
                pipeline_id: pipeline_id.to_string(),
            });
        }

        let data = fs::read(&file_path).await?;
        self.deserialize_state(&data)
    }

    async fn save_state(&self, state: &PipelineState) -> Result<(), StateError> {
        self.ensure_directories().await?;

        let file_path = self.state_file_path(&state.pipeline_id);
        let data = self.serialize_state(state)?;

        self.write_file_atomic(&file_path, &data).await?;

        Ok(())
    }

    async fn delete_state(&self, pipeline_id: &str) -> Result<(), StateError> {
        let file_path = self.state_file_path(pipeline_id);

        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }

        // Also remove lock file if it exists
        let lock_path = self.lock_file_path(pipeline_id);
        if lock_path.exists() {
            fs::remove_file(&lock_path).await?;
        }

        Ok(())
    }

    async fn list_pipelines(&self) -> Result<Vec<String>, StateError> {
        self.ensure_directories().await?;

        let states_dir = self.base_path.join("states");
        let mut pipeline_ids = Vec::new();

        let mut entries = fs::read_dir(&states_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                // Remove the file extension to get pipeline ID
                if let Some(dot_pos) = file_name.rfind('.') {
                    let pipeline_id = &file_name[..dot_pos];
                    pipeline_ids.push(pipeline_id.to_string());
                }
            }
        }

        pipeline_ids.sort();
        Ok(pipeline_ids)
    }

    async fn acquire_lock(
        &self,
        pipeline_id: &str,
        worker_id: &str,
        timeout_ms: u64,
    ) -> Result<LockInfo, StateError> {
        self.ensure_directories().await?;

        let lock_path = self.lock_file_path(pipeline_id);
        let lock_info = LockInfo {
            pipeline_id: pipeline_id.to_string(),
            worker_id: worker_id.to_string(),
            locked_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::milliseconds(timeout_ms as i64)),
            lock_version: 1,
        };

        // Try to acquire the lock with timeout
        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        loop {
            // Try to create and lock the file
            match tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&lock_path)
                .await
            {
                Ok(mut file) => {
                    // Try to acquire exclusive lock
                    match file.try_lock_exclusive() {
                        Ok(true) => {
                            // Write lock info to file
                            let lock_data = serde_json::to_vec(&lock_info)?;
                            file.write_all(&lock_data).await?;
                            file.flush().await?;

                            return Ok(lock_info);
                        }
                        Ok(false) | Err(_) => {
                            // Lock is held by another process or other error
                            if start_time.elapsed() >= timeout_duration {
                                return Err(StateError::LockTimeout { timeout_ms });
                            }

                            // Short delay before retry
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                    }
                }
                Err(e) => {
                    return Err(StateError::IoError {
                        details: format!("Failed to open lock file: {e}"),
                    });
                }
            }
        }
    }

    async fn release_lock(&self, pipeline_id: &str, worker_id: &str) -> Result<(), StateError> {
        let lock_path = self.lock_file_path(pipeline_id);

        if !lock_path.exists() {
            return Ok(()); // Lock already released
        }

        // Verify the lock is owned by this worker
        if let Some(lock_info) = self.is_locked(pipeline_id).await? {
            if lock_info.worker_id != worker_id {
                return Err(StateError::LockAlreadyHeld {
                    worker_id: lock_info.worker_id,
                });
            }
        }

        // Remove the lock file
        fs::remove_file(&lock_path).await?;

        Ok(())
    }

    async fn is_locked(&self, pipeline_id: &str) -> Result<Option<LockInfo>, StateError> {
        let lock_path = self.lock_file_path(pipeline_id);

        if !lock_path.exists() {
            return Ok(None);
        }

        // Try to read the lock file
        match fs::read(&lock_path).await {
            Ok(data) => {
                match serde_json::from_slice::<LockInfo>(&data) {
                    Ok(lock_info) => {
                        // Check if lock has expired
                        if let Some(expires_at) = lock_info.expires_at {
                            if Utc::now() > expires_at {
                                // Lock has expired, remove it
                                let _ = fs::remove_file(&lock_path).await;
                                return Ok(None);
                            }
                        }

                        Ok(Some(lock_info))
                    }
                    Err(_) => {
                        // Invalid lock file, remove it
                        let _ = fs::remove_file(&lock_path).await;
                        Ok(None)
                    }
                }
            }
            Err(_) => Ok(None),
        }
    }

    async fn force_release_lock(&self, pipeline_id: &str) -> Result<(), StateError> {
        let lock_path = self.lock_file_path(pipeline_id);

        if lock_path.exists() {
            fs::remove_file(&lock_path).await?;
        }

        Ok(())
    }

    async fn health_check(&self) -> Result<BackendHealth, StateError> {
        let start_time = std::time::Instant::now();

        // Try to ensure directories exist as a basic health check
        let health_result = self.ensure_directories().await;
        let response_time_ms = start_time.elapsed().as_millis() as u64;

        match health_result {
            Ok(()) => {
                let mut metrics = HashMap::new();
                metrics.insert("response_time_ms".to_string(), response_time_ms as f64);

                // Get disk usage information if possible
                if let Ok(states_dir) = fs::read_dir(&self.base_path.join("states")).await {
                    let mut state_count = 0;
                    let mut total_size = 0;

                    let mut entries = states_dir;
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        if let Ok(metadata) = entry.metadata().await {
                            state_count += 1;
                            total_size += metadata.len();
                        }
                    }

                    metrics.insert("state_files_count".to_string(), state_count as f64);
                    metrics.insert("total_state_size_bytes".to_string(), total_size as f64);
                }

                Ok(BackendHealth {
                    backend_type: "file".to_string(),
                    healthy: true,
                    last_check: Utc::now(),
                    response_time_ms,
                    error_message: None,
                    metrics,
                })
            }
            Err(e) => Ok(BackendHealth {
                backend_type: "file".to_string(),
                healthy: false,
                last_check: Utc::now(),
                response_time_ms,
                error_message: Some(e.to_string()),
                metrics: HashMap::new(),
            }),
        }
    }

    async fn cleanup(&self, max_age_hours: u64) -> Result<CleanupResult, StateError> {
        let start_time = std::time::Instant::now();
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        let mut result = CleanupResult {
            expired_locks_removed: 0,
            stale_states_removed: 0,
            total_states_checked: 0,
            cleanup_duration_ms: 0,
            errors: Vec::new(),
        };

        self.ensure_directories().await?;

        // Clean up expired locks
        let locks_dir = self.base_path.join("locks");
        if let Ok(mut entries) = fs::read_dir(&locks_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                // Try to read and check lock expiration
                if let Ok(data) = fs::read(&path).await {
                    if let Ok(lock_info) = serde_json::from_slice::<LockInfo>(&data) {
                        if let Some(expires_at) = lock_info.expires_at {
                            if expires_at < cutoff_time {
                                if let Err(e) = fs::remove_file(&path).await {
                                    result
                                        .errors
                                        .push(format!("Failed to remove expired lock: {e}"));
                                } else {
                                    result.expired_locks_removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check state files for staleness
        let states_dir = self.base_path.join("states");
        if let Ok(mut entries) = fs::read_dir(&states_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                result.total_states_checked += 1;

                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified_time) = metadata.modified() {
                        let modified_datetime = DateTime::<Utc>::from(modified_time);

                        if modified_datetime < cutoff_time {
                            // This is a stale state file
                            if let Err(e) = fs::remove_file(entry.path()).await {
                                result
                                    .errors
                                    .push(format!("Failed to remove stale state: {e}"));
                            } else {
                                result.stale_states_removed += 1;
                            }
                        }
                    }
                }
            }
        }

        result.cleanup_duration_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }
}

/// Memory-based backend for testing and development
pub struct MemoryBackend {
    states: std::sync::Arc<tokio::sync::RwLock<HashMap<String, PipelineState>>>,
    locks: std::sync::Arc<tokio::sync::RwLock<HashMap<String, LockInfo>>>,
}

impl MemoryBackend {
    /// Create a new memory backend
    pub fn new() -> Self {
        Self {
            states: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            locks: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StateBackend for MemoryBackend {
    async fn load_state(&self, pipeline_id: &str) -> Result<PipelineState, StateError> {
        let states = self.states.read().await;

        states
            .get(pipeline_id)
            .cloned()
            .ok_or_else(|| StateError::PipelineNotFound {
                pipeline_id: pipeline_id.to_string(),
            })
    }

    async fn save_state(&self, state: &PipelineState) -> Result<(), StateError> {
        let mut states = self.states.write().await;
        states.insert(state.pipeline_id.clone(), state.clone());
        Ok(())
    }

    async fn delete_state(&self, pipeline_id: &str) -> Result<(), StateError> {
        let mut states = self.states.write().await;
        let mut locks = self.locks.write().await;

        states.remove(pipeline_id);
        locks.remove(pipeline_id);

        Ok(())
    }

    async fn list_pipelines(&self) -> Result<Vec<String>, StateError> {
        let states = self.states.read().await;
        let mut pipeline_ids: Vec<String> = states.keys().cloned().collect();
        pipeline_ids.sort();
        Ok(pipeline_ids)
    }

    async fn acquire_lock(
        &self,
        pipeline_id: &str,
        worker_id: &str,
        timeout_ms: u64,
    ) -> Result<LockInfo, StateError> {
        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        loop {
            {
                let mut locks = self.locks.write().await;

                // Check if lock exists and is still valid
                if let Some(existing_lock) = locks.get(pipeline_id) {
                    if let Some(expires_at) = existing_lock.expires_at {
                        if Utc::now() > expires_at {
                            // Lock has expired, remove it
                            locks.remove(pipeline_id);
                        } else {
                            // Lock is still valid
                            if start_time.elapsed() >= timeout_duration {
                                return Err(StateError::LockTimeout { timeout_ms });
                            }

                            // Release the lock temporarily and wait
                            drop(locks);
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                            continue;
                        }
                    }
                }

                // Acquire the lock
                let lock_info = LockInfo {
                    pipeline_id: pipeline_id.to_string(),
                    worker_id: worker_id.to_string(),
                    locked_at: Utc::now(),
                    expires_at: Some(
                        Utc::now() + chrono::Duration::milliseconds(timeout_ms as i64),
                    ),
                    lock_version: 1,
                };

                locks.insert(pipeline_id.to_string(), lock_info.clone());
                return Ok(lock_info);
            }
        }
    }

    async fn release_lock(&self, pipeline_id: &str, worker_id: &str) -> Result<(), StateError> {
        let mut locks = self.locks.write().await;

        if let Some(lock_info) = locks.get(pipeline_id) {
            if lock_info.worker_id != worker_id {
                return Err(StateError::LockAlreadyHeld {
                    worker_id: lock_info.worker_id.clone(),
                });
            }
        }

        locks.remove(pipeline_id);
        Ok(())
    }

    async fn is_locked(&self, pipeline_id: &str) -> Result<Option<LockInfo>, StateError> {
        let mut locks = self.locks.write().await;

        if let Some(lock_info) = locks.get(pipeline_id) {
            // Check if lock has expired
            if let Some(expires_at) = lock_info.expires_at {
                if Utc::now() > expires_at {
                    locks.remove(pipeline_id);
                    return Ok(None);
                }
            }

            Ok(Some(lock_info.clone()))
        } else {
            Ok(None)
        }
    }

    async fn force_release_lock(&self, pipeline_id: &str) -> Result<(), StateError> {
        let mut locks = self.locks.write().await;
        locks.remove(pipeline_id);
        Ok(())
    }

    async fn health_check(&self) -> Result<BackendHealth, StateError> {
        let start_time = std::time::Instant::now();

        // Basic health check - try to access the data structures
        let states_count = self.states.read().await.len();
        let locks_count = self.locks.read().await.len();

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        let mut metrics = HashMap::new();
        metrics.insert("states_count".to_string(), states_count as f64);
        metrics.insert("locks_count".to_string(), locks_count as f64);
        metrics.insert("response_time_ms".to_string(), response_time_ms as f64);

        Ok(BackendHealth {
            backend_type: "memory".to_string(),
            healthy: true,
            last_check: Utc::now(),
            response_time_ms,
            error_message: None,
            metrics,
        })
    }

    async fn cleanup(&self, _max_age_hours: u64) -> Result<CleanupResult, StateError> {
        let start_time = std::time::Instant::now();

        // Clean up expired locks
        let mut locks = self.locks.write().await;
        let mut expired_count = 0;
        let mut expired_keys = Vec::new();

        for (pipeline_id, lock_info) in locks.iter() {
            if let Some(expires_at) = lock_info.expires_at {
                if Utc::now() > expires_at {
                    expired_keys.push(pipeline_id.clone());
                }
            }
        }

        for key in expired_keys {
            locks.remove(&key);
            expired_count += 1;
        }

        let states_count = self.states.read().await.len();

        Ok(CleanupResult {
            expired_locks_removed: expired_count,
            stale_states_removed: 0, // Memory backend doesn't remove stale states automatically
            total_states_checked: states_count as u64,
            cleanup_duration_ms: start_time.elapsed().as_millis() as u64,
            errors: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::PipelineState;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_backend_basic_operations() {
        let backend = MemoryBackend::new();
        let state = PipelineState::new("test_pipeline".to_string(), "run_123".to_string());

        // Save state
        assert!(backend.save_state(&state).await.is_ok());

        // Load state
        let loaded_state = backend.load_state("test_pipeline").await.unwrap();
        assert_eq!(loaded_state.pipeline_id, "test_pipeline");
        assert_eq!(loaded_state.run_id, "run_123");

        // List pipelines
        let pipelines = backend.list_pipelines().await.unwrap();
        assert_eq!(pipelines, vec!["test_pipeline"]);

        // Delete state
        assert!(backend.delete_state("test_pipeline").await.is_ok());

        // Verify deletion
        let load_result = backend.load_state("test_pipeline").await;
        assert!(matches!(
            load_result,
            Err(StateError::PipelineNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn test_memory_backend_locking() {
        let backend = MemoryBackend::new();

        // Acquire lock
        let lock_info = backend
            .acquire_lock("test_pipeline", "worker_1", 5000)
            .await
            .unwrap();

        assert_eq!(lock_info.pipeline_id, "test_pipeline");
        assert_eq!(lock_info.worker_id, "worker_1");

        // Check if locked
        let is_locked = backend.is_locked("test_pipeline").await.unwrap();
        assert!(is_locked.is_some());

        // Try to acquire lock with different worker (should fail)
        let lock_result = backend.acquire_lock("test_pipeline", "worker_2", 100).await;
        assert!(matches!(lock_result, Err(StateError::LockTimeout { .. })));

        // Release lock
        assert!(backend
            .release_lock("test_pipeline", "worker_1")
            .await
            .is_ok());

        // Verify lock is released
        let is_locked = backend.is_locked("test_pipeline").await.unwrap();
        assert!(is_locked.is_none());
    }

    #[tokio::test]
    async fn test_file_backend_configuration() {
        let temp_dir = TempDir::new().unwrap();

        let config = BackendConfig::File {
            base_path: temp_dir.path().to_path_buf(),
            format: SerializationFormat::Json,
            atomic_writes: true,
            lock_timeout_ms: 5000,
        };

        let backend = FileBackend::new(config).unwrap();
        assert_eq!(backend.format, SerializationFormat::Json);
        assert!(backend.atomic_writes);
        assert_eq!(backend.lock_timeout_ms, 5000);
    }

    #[tokio::test]
    async fn test_file_backend_basic_operations() {
        let temp_dir = TempDir::new().unwrap();

        let config = BackendConfig::File {
            base_path: temp_dir.path().to_path_buf(),
            format: SerializationFormat::Json,
            atomic_writes: true,
            lock_timeout_ms: 5000,
        };

        let backend = FileBackend::new(config).unwrap();
        let state = PipelineState::new("test_pipeline".to_string(), "run_123".to_string());

        // Save state
        assert!(backend.save_state(&state).await.is_ok());

        // Verify file exists
        let state_file = backend.state_file_path("test_pipeline");
        assert!(state_file.exists());

        // Load state
        let loaded_state = backend.load_state("test_pipeline").await.unwrap();
        assert_eq!(loaded_state.pipeline_id, "test_pipeline");
        assert_eq!(loaded_state.run_id, "run_123");

        // List pipelines
        let pipelines = backend.list_pipelines().await.unwrap();
        assert_eq!(pipelines, vec!["test_pipeline"]);

        // Delete state
        assert!(backend.delete_state("test_pipeline").await.is_ok());
        assert!(!state_file.exists());
    }

    #[tokio::test]
    async fn test_serialization_formats() {
        let temp_dir = TempDir::new().unwrap();
        let state = PipelineState::new("test_pipeline".to_string(), "run_123".to_string());

        // Test JSON format
        let json_config = BackendConfig::File {
            base_path: temp_dir.path().join("json"),
            format: SerializationFormat::Json,
            atomic_writes: false,
            lock_timeout_ms: 1000,
        };

        let json_backend = FileBackend::new(json_config).unwrap();
        let json_data = json_backend.serialize_state(&state).unwrap();
        let json_restored = json_backend.deserialize_state(&json_data).unwrap();
        assert_eq!(json_restored.pipeline_id, state.pipeline_id);

        // Test YAML format
        let yaml_config = BackendConfig::File {
            base_path: temp_dir.path().join("yaml"),
            format: SerializationFormat::Yaml,
            atomic_writes: false,
            lock_timeout_ms: 1000,
        };

        let yaml_backend = FileBackend::new(yaml_config).unwrap();
        let yaml_data = yaml_backend.serialize_state(&state).unwrap();
        let yaml_restored = yaml_backend.deserialize_state(&yaml_data).unwrap();
        assert_eq!(yaml_restored.pipeline_id, state.pipeline_id);
    }

    #[tokio::test]
    async fn test_backend_health_check() {
        // Test memory backend health
        let memory_backend = MemoryBackend::new();
        let memory_health = memory_backend.health_check().await.unwrap();
        assert!(memory_health.healthy);
        assert_eq!(memory_health.backend_type, "memory");

        // Test file backend health
        let temp_dir = TempDir::new().unwrap();
        let config = BackendConfig::File {
            base_path: temp_dir.path().to_path_buf(),
            format: SerializationFormat::Json,
            atomic_writes: false,
            lock_timeout_ms: 1000,
        };

        let file_backend = FileBackend::new(config).unwrap();
        let file_health = file_backend.health_check().await.unwrap();
        assert!(file_health.healthy);
        assert_eq!(file_health.backend_type, "file");
    }
}

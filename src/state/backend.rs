use crate::state::types::{PipelineState, StateError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use fs4::tokio::AsyncFileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

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

    // Production hardening methods

    /// Validate the integrity of a stored state
    async fn validate_state(&self, pipeline_id: &str) -> Result<ValidationResult, StateError>;

    /// Create a backup of pipeline state
    async fn backup_state(&self, pipeline_id: &str) -> Result<BackupResult, StateError>;

    /// Restore pipeline state from backup
    async fn restore_state(&self, pipeline_id: &str, backup_id: &str) -> Result<(), StateError>;

    /// List available backups for a pipeline
    async fn list_backups(&self, pipeline_id: &str) -> Result<Vec<BackupInfo>, StateError>;

    /// Attempt to repair a corrupted state file
    async fn repair_state(&self, pipeline_id: &str) -> Result<RepairResult, StateError>;

    /// Get detailed storage metrics and diagnostics
    async fn get_diagnostics(&self) -> Result<BackendDiagnostics, StateError>;

    /// Verify backend integrity (check all state files)
    async fn verify_integrity(&self) -> Result<IntegrityReport, StateError>;
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

/// Result of state validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub corruption_detected: bool,
    pub validation_errors: Vec<String>,
    pub checksum_match: bool,
    pub file_size_bytes: u64,
    pub last_modified: DateTime<Utc>,
}

/// Information about a state backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub backup_id: String,
    pub pipeline_id: String,
    pub created_at: DateTime<Utc>,
    pub file_size_bytes: u64,
    pub backup_type: BackupType,
    pub state_version: u64,
}

/// Type of backup created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Automatic,
    Manual,
    PreRepair,
    PreUpgrade,
}

/// Result of a backup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub backup_id: String,
    pub backup_path: String,
    pub file_size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub checksum: String,
}

/// Result of a state repair operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    pub success: bool,
    pub backup_created: bool,
    pub backup_id: Option<String>,
    pub repairs_made: Vec<String>,
    pub issues_found: Vec<String>,
    pub manual_intervention_required: bool,
}

/// Comprehensive backend diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendDiagnostics {
    pub backend_type: String,
    pub total_states: u64,
    pub total_locks: u64,
    pub total_backups: u64,
    pub storage_used_bytes: u64,
    pub storage_available_bytes: u64,
    pub average_state_size_bytes: u64,
    pub oldest_state: Option<DateTime<Utc>>,
    pub newest_state: Option<DateTime<Utc>>,
    pub performance_metrics: HashMap<String, f64>,
    pub health_issues: Vec<String>,
}

/// Result of integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub total_files_checked: u64,
    pub corrupted_files: Vec<String>,
    pub missing_files: Vec<String>,
    pub permission_errors: Vec<String>,
    pub checksum_mismatches: Vec<String>,
    pub repair_recommendations: Vec<String>,
    pub overall_health: f64, // 0.0 to 1.0
}

/// File-based state backend implementation
pub struct FileBackend {
    base_path: PathBuf,
    format: SerializationFormat,
    atomic_writes: bool,
    #[allow(dead_code)] // Used for future timeout configuration
    lock_timeout_ms: u64,

    // Performance optimization features
    cache: std::sync::Arc<tokio::sync::RwLock<HashMap<String, CachedState>>>,
    cache_enabled: bool,
    cache_max_size: usize,
    performance_metrics: std::sync::Arc<tokio::sync::RwLock<PerformanceMetrics>>,
}

/// Cached state with metadata
#[derive(Debug, Clone)]
struct CachedState {
    state: PipelineState,
    #[allow(dead_code)] // Used for cache expiration (future feature)
    cached_at: DateTime<Utc>,
    access_count: u64,
    last_accessed: DateTime<Utc>,
}

/// Performance metrics for the backend
#[derive(Debug, Clone, Default)]
struct PerformanceMetrics {
    total_reads: u64,
    total_writes: u64,
    cache_hits: u64,
    cache_misses: u64,
    avg_read_time_ms: f64,
    avg_write_time_ms: f64,
    avg_serialization_time_ms: f64,
    avg_deserialization_time_ms: f64,
    total_bytes_read: u64,
    total_bytes_written: u64,
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
                cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
                cache_enabled: true, // Enable by default
                cache_max_size: 100, // Default cache size
                performance_metrics: std::sync::Arc::new(tokio::sync::RwLock::new(
                    PerformanceMetrics::default(),
                )),
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

    // Cache management methods for performance optimization

    /// Check cache for a state
    async fn get_from_cache(&self, pipeline_id: &str) -> Option<PipelineState> {
        if !self.cache_enabled {
            return None;
        }

        let mut cache = self.cache.write().await;

        if let Some(cached) = cache.get_mut(pipeline_id) {
            cached.access_count += 1;
            cached.last_accessed = Utc::now();

            // Update metrics
            let mut metrics = self.performance_metrics.write().await;
            metrics.cache_hits += 1;

            Some(cached.state.clone())
        } else {
            // Update metrics
            let mut metrics = self.performance_metrics.write().await;
            metrics.cache_misses += 1;

            None
        }
    }

    /// Store state in cache
    async fn store_in_cache(&self, pipeline_id: &str, state: &PipelineState) {
        if !self.cache_enabled {
            return;
        }

        let mut cache = self.cache.write().await;

        // Check if cache is full and needs cleanup
        if cache.len() >= self.cache_max_size {
            self.evict_least_recently_used(&mut cache).await;
        }

        let cached_state = CachedState {
            state: state.clone(),
            cached_at: Utc::now(),
            access_count: 1,
            last_accessed: Utc::now(),
        };

        cache.insert(pipeline_id.to_string(), cached_state);
    }

    /// Evict least recently used item from cache
    async fn evict_least_recently_used(&self, cache: &mut HashMap<String, CachedState>) {
        if cache.is_empty() {
            return;
        }

        // Find the least recently used item
        let mut oldest_key = String::new();
        let mut oldest_time = Utc::now();

        for (key, cached_state) in cache.iter() {
            if cached_state.last_accessed < oldest_time {
                oldest_time = cached_state.last_accessed;
                oldest_key = key.clone();
            }
        }

        if !oldest_key.is_empty() {
            cache.remove(&oldest_key);
        }
    }

    /// Invalidate cache entry
    async fn invalidate_cache(&self, pipeline_id: &str) {
        if !self.cache_enabled {
            return;
        }

        let mut cache = self.cache.write().await;
        cache.remove(pipeline_id);
    }

    /// Clear entire cache
    #[allow(dead_code)] // Used for maintenance operations (future CLI command)
    async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Record performance metrics
    async fn record_read_metrics(&self, duration_ms: f64, bytes_read: u64) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_reads += 1;
        metrics.total_bytes_read += bytes_read;

        // Update running average
        metrics.avg_read_time_ms = ((metrics.avg_read_time_ms * (metrics.total_reads - 1) as f64)
            + duration_ms)
            / metrics.total_reads as f64;
    }

    /// Record write performance metrics
    async fn record_write_metrics(&self, duration_ms: f64, bytes_written: u64) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_writes += 1;
        metrics.total_bytes_written += bytes_written;

        // Update running average
        metrics.avg_write_time_ms =
            ((metrics.avg_write_time_ms * (metrics.total_writes - 1) as f64) + duration_ms)
                / metrics.total_writes as f64;
    }

    /// Record serialization performance metrics
    async fn record_serialization_metrics(&self, duration_ms: f64) {
        let mut metrics = self.performance_metrics.write().await;

        // Use total writes as the count for serialization operations
        metrics.avg_serialization_time_ms = ((metrics.avg_serialization_time_ms
            * (metrics.total_writes.saturating_sub(1)) as f64)
            + duration_ms)
            / metrics.total_writes.max(1) as f64;
    }

    /// Record deserialization performance metrics
    async fn record_deserialization_metrics(&self, duration_ms: f64) {
        let mut metrics = self.performance_metrics.write().await;

        // Use total reads as the count for deserialization operations
        metrics.avg_deserialization_time_ms = ((metrics.avg_deserialization_time_ms
            * (metrics.total_reads.saturating_sub(1)) as f64)
            + duration_ms)
            / metrics.total_reads.max(1) as f64;
    }
}

#[async_trait]
impl StateBackend for FileBackend {
    async fn load_state(&self, pipeline_id: &str) -> Result<PipelineState, StateError> {
        let start_time = std::time::Instant::now();

        // Check cache first
        if let Some(cached_state) = self.get_from_cache(pipeline_id).await {
            return Ok(cached_state);
        }

        self.ensure_directories().await?;

        let file_path = self.state_file_path(pipeline_id);

        if !file_path.exists() {
            return Err(StateError::PipelineNotFound {
                pipeline_id: pipeline_id.to_string(),
            });
        }

        let data = fs::read(&file_path).await?;
        let bytes_read = data.len() as u64;

        let deserialize_start = std::time::Instant::now();
        let state = self.deserialize_state(&data)?;
        let deserialize_duration = deserialize_start.elapsed().as_millis() as f64;

        // Record performance metrics
        let total_duration = start_time.elapsed().as_millis() as f64;
        self.record_read_metrics(total_duration, bytes_read).await;
        self.record_deserialization_metrics(deserialize_duration)
            .await;

        // Store in cache for future use
        self.store_in_cache(pipeline_id, &state).await;

        Ok(state)
    }

    async fn save_state(&self, state: &PipelineState) -> Result<(), StateError> {
        let start_time = std::time::Instant::now();

        self.ensure_directories().await?;

        let file_path = self.state_file_path(&state.pipeline_id);

        let serialize_start = std::time::Instant::now();
        let data = self.serialize_state(state)?;
        let serialize_duration = serialize_start.elapsed().as_millis() as f64;

        let bytes_written = data.len() as u64;

        self.write_file_atomic(&file_path, &data).await?;

        // Record performance metrics
        let total_duration = start_time.elapsed().as_millis() as f64;
        self.record_write_metrics(total_duration, bytes_written)
            .await;
        self.record_serialization_metrics(serialize_duration).await;

        // Update cache with new state
        self.store_in_cache(&state.pipeline_id, state).await;

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

        // Invalidate cache entry
        self.invalidate_cache(pipeline_id).await;

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

    // Production hardening methods implementation

    async fn validate_state(&self, pipeline_id: &str) -> Result<ValidationResult, StateError> {
        let file_path = self.state_file_path(pipeline_id);

        if !file_path.exists() {
            return Err(StateError::PipelineNotFound {
                pipeline_id: pipeline_id.to_string(),
            });
        }

        let _start_time = std::time::Instant::now();
        let mut validation_errors = Vec::new();
        let mut corruption_detected = false;

        // Get file metadata
        let metadata = fs::metadata(&file_path).await?;
        let file_size_bytes = metadata.len();
        let last_modified = DateTime::<Utc>::from(metadata.modified()?);

        // Try to read and parse the state
        let data_result = fs::read(&file_path).await;
        let checksum_match = match data_result {
            Ok(data) => {
                // Check if the data can be deserialized
                match self.deserialize_state(&data) {
                    Ok(state) => {
                        // Validate state integrity
                        match state.validate() {
                            Ok(()) => true,
                            Err(errors) => {
                                validation_errors.extend(errors);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        validation_errors.push(format!("Deserialization failed: {e}"));
                        corruption_detected = true;
                        false
                    }
                }
            }
            Err(e) => {
                validation_errors.push(format!("File read failed: {e}"));
                corruption_detected = true;
                false
            }
        };

        // Check for zero-sized files (likely corruption)
        if file_size_bytes == 0 {
            validation_errors.push("State file is empty".to_string());
            corruption_detected = true;
        }

        Ok(ValidationResult {
            valid: validation_errors.is_empty() && !corruption_detected,
            corruption_detected,
            validation_errors,
            checksum_match,
            file_size_bytes,
            last_modified,
        })
    }

    async fn backup_state(&self, pipeline_id: &str) -> Result<BackupResult, StateError> {
        let source_path = self.state_file_path(pipeline_id);

        if !source_path.exists() {
            return Err(StateError::PipelineNotFound {
                pipeline_id: pipeline_id.to_string(),
            });
        }

        // Create backup directory if it doesn't exist
        let backup_dir = self.base_path.join("backups").join(pipeline_id);
        fs::create_dir_all(&backup_dir).await?;

        // Generate backup ID with timestamp
        let backup_id = format!("backup_{}", Utc::now().format("%Y%m%d_%H%M%S_%3f"));
        let backup_path = backup_dir.join(format!(
            "{}.{}",
            backup_id,
            match self.format {
                SerializationFormat::Json => "json",
                SerializationFormat::Yaml => "yaml",
                SerializationFormat::Bincode => "bin",
            }
        ));

        // Copy the state file to backup location
        fs::copy(&source_path, &backup_path).await?;

        // Get file metadata
        let metadata = fs::metadata(&backup_path).await?;
        let file_size_bytes = metadata.len();
        let created_at = Utc::now();

        // Create checksum for backup verification
        let data = fs::read(&backup_path).await?;
        let checksum = format!("{:x}", md5::compute(&data));

        Ok(BackupResult {
            backup_id,
            backup_path: backup_path.to_string_lossy().to_string(),
            file_size_bytes,
            created_at,
            checksum,
        })
    }

    async fn restore_state(&self, pipeline_id: &str, backup_id: &str) -> Result<(), StateError> {
        let backup_dir = self.base_path.join("backups").join(pipeline_id);
        let backup_path = backup_dir.join(format!(
            "{}.{}",
            backup_id,
            match self.format {
                SerializationFormat::Json => "json",
                SerializationFormat::Yaml => "yaml",
                SerializationFormat::Bincode => "bin",
            }
        ));

        if !backup_path.exists() {
            return Err(StateError::BackupFailed {
                details: format!("Backup not found: {backup_id}"),
            });
        }

        // Validate backup before restoring
        let data = fs::read(&backup_path).await?;
        let _state = self.deserialize_state(&data)?; // Validate it can be parsed

        // Create backup of current state before restoring
        if self.state_file_path(pipeline_id).exists() {
            let _ = self.backup_state(pipeline_id).await; // Best effort backup
        }

        // Restore the backup
        let target_path = self.state_file_path(pipeline_id);
        self.ensure_directories().await?;
        fs::copy(&backup_path, &target_path).await?;

        Ok(())
    }

    async fn list_backups(&self, pipeline_id: &str) -> Result<Vec<BackupInfo>, StateError> {
        let backup_dir = self.base_path.join("backups").join(pipeline_id);

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                if file_name.starts_with("backup_") {
                    let metadata = entry.metadata().await?;
                    let created_at = DateTime::<Utc>::from(metadata.modified()?);

                    // Try to read and get state version
                    let state_version = if let Ok(data) = fs::read(&path).await {
                        if let Ok(state) = self.deserialize_state(&data) {
                            state.version
                        } else {
                            0 // Corrupted backup
                        }
                    } else {
                        0
                    };

                    backups.push(BackupInfo {
                        backup_id: file_name.to_string(),
                        pipeline_id: pipeline_id.to_string(),
                        created_at,
                        file_size_bytes: metadata.len(),
                        backup_type: BackupType::Automatic, // Default, could be enhanced
                        state_version,
                    });
                }
            }
        }

        // Sort by creation time, newest first
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    async fn repair_state(&self, pipeline_id: &str) -> Result<RepairResult, StateError> {
        let mut repairs_made = Vec::new();
        let mut issues_found = Vec::new();
        let mut manual_intervention_required = false;

        // First, create a backup before attempting repairs
        let backup_result = self.backup_state(pipeline_id).await;
        let (backup_created, backup_id) = match backup_result {
            Ok(result) => (true, Some(result.backup_id)),
            Err(_) => {
                issues_found.push("Could not create backup before repair".to_string());
                (false, None)
            }
        };

        // Validate current state
        let validation = self.validate_state(pipeline_id).await?;

        if validation.valid {
            return Ok(RepairResult {
                success: true,
                backup_created,
                backup_id,
                repairs_made: vec!["No repairs needed - state is valid".to_string()],
                issues_found,
                manual_intervention_required: false,
            });
        }

        issues_found.extend(validation.validation_errors.clone());

        // Attempt repairs based on detected issues
        let file_path = self.state_file_path(pipeline_id);

        if validation.corruption_detected {
            // Try to restore from the most recent backup
            let backups = self.list_backups(pipeline_id).await?;

            if let Some(latest_backup) = backups.first() {
                match self
                    .restore_state(pipeline_id, &latest_backup.backup_id)
                    .await
                {
                    Ok(()) => {
                        repairs_made
                            .push(format!("Restored from backup: {}", latest_backup.backup_id));
                    }
                    Err(e) => {
                        issues_found.push(format!("Failed to restore from backup: {e}"));
                        manual_intervention_required = true;
                    }
                }
            } else {
                issues_found.push("No backups available for restoration".to_string());
                manual_intervention_required = true;
            }
        } else {
            // Try to fix validation errors by reconstructing state
            if let Ok(data) = fs::read(&file_path).await {
                if let Ok(mut state) = self.deserialize_state(&data) {
                    // Fix common validation issues
                    let now = Utc::now();

                    if state.pipeline_id.is_empty() {
                        state.pipeline_id = pipeline_id.to_string();
                        repairs_made.push("Fixed empty pipeline_id".to_string());
                    }

                    if state.run_id.is_empty() {
                        state.run_id = format!("repaired_{}", Uuid::new_v4());
                        repairs_made.push("Generated new run_id".to_string());
                    }

                    if state.version == 0 {
                        state.version = 1;
                        repairs_made.push("Fixed invalid version".to_string());
                    }

                    // Update timestamps if they're invalid
                    if state.started_at > now {
                        state.started_at = now - chrono::Duration::hours(1);
                        repairs_made.push("Fixed invalid start time".to_string());
                    }

                    if state.last_heartbeat < state.started_at {
                        state.last_heartbeat = now;
                        repairs_made.push("Fixed invalid heartbeat time".to_string());
                    }

                    // Save the repaired state
                    if let Err(e) = self.save_state(&state).await {
                        issues_found.push(format!("Failed to save repaired state: {e}"));
                        manual_intervention_required = true;
                    }
                } else {
                    manual_intervention_required = true;
                }
            } else {
                manual_intervention_required = true;
            }
        }

        let success = !manual_intervention_required && !repairs_made.is_empty();

        Ok(RepairResult {
            success,
            backup_created,
            backup_id,
            repairs_made,
            issues_found,
            manual_intervention_required,
        })
    }

    async fn get_diagnostics(&self) -> Result<BackendDiagnostics, StateError> {
        self.ensure_directories().await?;

        let mut total_states = 0;
        let mut total_locks = 0;
        let mut total_backups = 0;
        let mut storage_used_bytes = 0;
        let mut oldest_state: Option<DateTime<Utc>> = None;
        let mut newest_state: Option<DateTime<Utc>> = None;
        let mut state_sizes = Vec::new();
        let mut health_issues = Vec::new();

        // Analyze states directory
        let states_dir = self.base_path.join("states");
        if let Ok(mut entries) = fs::read_dir(&states_dir).await {
            while let Some(entry) = entries.next_entry().await? {
                total_states += 1;

                if let Ok(metadata) = entry.metadata().await {
                    let size = metadata.len();
                    storage_used_bytes += size;
                    state_sizes.push(size);

                    if let Ok(modified) = metadata.modified() {
                        let modified_dt = DateTime::<Utc>::from(modified);

                        match oldest_state {
                            None => oldest_state = Some(modified_dt),
                            Some(current_oldest) => {
                                if modified_dt < current_oldest {
                                    oldest_state = Some(modified_dt);
                                }
                            }
                        }

                        match newest_state {
                            None => newest_state = Some(modified_dt),
                            Some(current_newest) => {
                                if modified_dt > current_newest {
                                    newest_state = Some(modified_dt);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Analyze locks directory
        let locks_dir = self.base_path.join("locks");
        if let Ok(mut entries) = fs::read_dir(&locks_dir).await {
            while let Some(_entry) = entries.next_entry().await? {
                total_locks += 1;
            }
        }

        // Analyze backups directory
        let backups_dir = self.base_path.join("backups");
        if let Ok(mut entries) = fs::read_dir(&backups_dir).await {
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    // Count files in each pipeline's backup directory
                    if let Ok(mut backup_entries) = fs::read_dir(entry.path()).await {
                        while let Some(_backup_entry) = backup_entries.next_entry().await? {
                            total_backups += 1;
                        }
                    }
                }
            }
        }

        // Calculate average state size
        let average_state_size_bytes = if state_sizes.is_empty() {
            0
        } else {
            state_sizes.iter().sum::<u64>() / state_sizes.len() as u64
        };

        // Get available disk space
        let storage_available_bytes = match fs::metadata(&self.base_path).await {
            Ok(_) => {
                // Simple estimation - in production this could use statvfs or similar
                1_000_000_000_u64 // 1GB default assumption
            }
            Err(e) => {
                health_issues.push(format!("Could not check disk space: {e}"));
                0
            }
        };

        // Performance metrics including cache and I/O statistics
        let mut performance_metrics = HashMap::new();
        performance_metrics.insert(
            "avg_state_size_kb".to_string(),
            average_state_size_bytes as f64 / 1024.0,
        );
        performance_metrics.insert(
            "total_storage_mb".to_string(),
            storage_used_bytes as f64 / (1024.0 * 1024.0),
        );

        // Add cache and performance metrics
        let cache = self.cache.read().await;
        let metrics = self.performance_metrics.read().await;

        performance_metrics.insert("cache_size".to_string(), cache.len() as f64);
        performance_metrics.insert("cache_max_size".to_string(), self.cache_max_size as f64);
        performance_metrics.insert(
            "cache_hit_rate".to_string(),
            if metrics.cache_hits + metrics.cache_misses > 0 {
                metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64
            } else {
                0.0
            },
        );

        performance_metrics.insert("total_reads".to_string(), metrics.total_reads as f64);
        performance_metrics.insert("total_writes".to_string(), metrics.total_writes as f64);
        performance_metrics.insert("avg_read_time_ms".to_string(), metrics.avg_read_time_ms);
        performance_metrics.insert("avg_write_time_ms".to_string(), metrics.avg_write_time_ms);
        performance_metrics.insert(
            "avg_serialization_time_ms".to_string(),
            metrics.avg_serialization_time_ms,
        );
        performance_metrics.insert(
            "avg_deserialization_time_ms".to_string(),
            metrics.avg_deserialization_time_ms,
        );
        performance_metrics.insert(
            "total_bytes_read".to_string(),
            metrics.total_bytes_read as f64,
        );
        performance_metrics.insert(
            "total_bytes_written".to_string(),
            metrics.total_bytes_written as f64,
        );

        // Cache efficiency warnings
        let cache_hit_rate = performance_metrics.get("cache_hit_rate").unwrap_or(&0.0);
        if *cache_hit_rate < 0.5 && metrics.total_reads > 10 {
            health_issues.push("Low cache hit rate - consider increasing cache size".to_string());
        }

        if metrics.avg_read_time_ms > 100.0 {
            health_issues.push("High average read time - consider optimizing I/O".to_string());
        }

        if metrics.avg_write_time_ms > 200.0 {
            health_issues.push("High average write time - consider optimizing I/O".to_string());
        }

        Ok(BackendDiagnostics {
            backend_type: "file".to_string(),
            total_states,
            total_locks,
            total_backups,
            storage_used_bytes,
            storage_available_bytes,
            average_state_size_bytes,
            oldest_state,
            newest_state,
            performance_metrics,
            health_issues,
        })
    }

    async fn verify_integrity(&self) -> Result<IntegrityReport, StateError> {
        let mut total_files_checked = 0;
        let mut corrupted_files = Vec::new();
        let mut missing_files = Vec::new();
        let mut permission_errors = Vec::new();
        let mut checksum_mismatches = Vec::new();
        let mut repair_recommendations = Vec::new();

        self.ensure_directories().await?;

        // Check all state files
        let states_dir = self.base_path.join("states");
        if let Ok(mut entries) = fs::read_dir(&states_dir).await {
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                total_files_checked += 1;

                // Extract pipeline ID from filename
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if let Some(dot_pos) = file_name.rfind('.') {
                        let pipeline_id = &file_name[..dot_pos];

                        // Validate this state file
                        match self.validate_state(pipeline_id).await {
                            Ok(validation) => {
                                if validation.corruption_detected {
                                    corrupted_files.push(path.to_string_lossy().to_string());
                                    repair_recommendations
                                        .push(format!("Run repair on pipeline: {pipeline_id}"));
                                }

                                if !validation.checksum_match {
                                    checksum_mismatches.push(path.to_string_lossy().to_string());
                                }
                            }
                            Err(StateError::PermissionDenied { .. }) => {
                                permission_errors.push(path.to_string_lossy().to_string());
                            }
                            Err(StateError::PipelineNotFound { .. }) => {
                                missing_files.push(path.to_string_lossy().to_string());
                            }
                            Err(_) => {
                                corrupted_files.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        // Calculate overall health score
        let total_issues = corrupted_files.len()
            + missing_files.len()
            + permission_errors.len()
            + checksum_mismatches.len();

        let overall_health = if total_files_checked == 0 {
            1.0
        } else {
            1.0 - (total_issues as f64 / total_files_checked as f64)
        };

        // Add general recommendations
        if !corrupted_files.is_empty() {
            repair_recommendations
                .push("Consider running automated repair on corrupted files".to_string());
        }

        if !missing_files.is_empty() {
            repair_recommendations.push("Check for accidental file deletion".to_string());
        }

        if overall_health < 0.8 {
            repair_recommendations.push("Consider full backup and restoration".to_string());
        }

        Ok(IntegrityReport {
            total_files_checked: total_files_checked as u64,
            corrupted_files,
            missing_files,
            permission_errors,
            checksum_mismatches,
            repair_recommendations,
            overall_health,
        })
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

    // Production hardening methods - simplified for memory backend

    async fn validate_state(&self, pipeline_id: &str) -> Result<ValidationResult, StateError> {
        let states = self.states.read().await;

        match states.get(pipeline_id) {
            Some(state) => {
                let validation_errors = match state.validate() {
                    Ok(()) => Vec::new(),
                    Err(errors) => errors,
                };

                Ok(ValidationResult {
                    valid: validation_errors.is_empty(),
                    corruption_detected: false, // Memory backend can't be corrupted in traditional sense
                    validation_errors,
                    checksum_match: true, // Always true for memory backend
                    file_size_bytes: state.estimated_memory_usage() as u64,
                    last_modified: state.metadata.updated_at,
                })
            }
            None => Err(StateError::PipelineNotFound {
                pipeline_id: pipeline_id.to_string(),
            }),
        }
    }

    async fn backup_state(&self, pipeline_id: &str) -> Result<BackupResult, StateError> {
        // Memory backend doesn't support traditional backups
        // But we can simulate it by cloning the state
        let states = self.states.read().await;

        match states.get(pipeline_id) {
            Some(state) => {
                let backup_id = format!("memory_backup_{}", Utc::now().format("%Y%m%d_%H%M%S_%3f"));
                let state_data = serde_json::to_vec(state).map_err(StateError::from)?;
                let checksum = format!("{:x}", md5::compute(&state_data));

                Ok(BackupResult {
                    backup_id,
                    backup_path: "memory://backup".to_string(),
                    file_size_bytes: state_data.len() as u64,
                    created_at: Utc::now(),
                    checksum,
                })
            }
            None => Err(StateError::PipelineNotFound {
                pipeline_id: pipeline_id.to_string(),
            }),
        }
    }

    async fn restore_state(&self, _pipeline_id: &str, _backup_id: &str) -> Result<(), StateError> {
        // Memory backend doesn't support traditional restore
        Err(StateError::BackendError {
            details: "Memory backend does not support state restoration".to_string(),
        })
    }

    async fn list_backups(&self, _pipeline_id: &str) -> Result<Vec<BackupInfo>, StateError> {
        // Memory backend doesn't maintain backups
        Ok(Vec::new())
    }

    async fn repair_state(&self, pipeline_id: &str) -> Result<RepairResult, StateError> {
        let validation = self.validate_state(pipeline_id).await?;

        if validation.valid {
            Ok(RepairResult {
                success: true,
                backup_created: false,
                backup_id: None,
                repairs_made: vec!["No repairs needed - state is valid".to_string()],
                issues_found: Vec::new(),
                manual_intervention_required: false,
            })
        } else {
            // For memory backend, we can try to fix basic validation issues
            let mut states = self.states.write().await;

            if let Some(state) = states.get_mut(pipeline_id) {
                let mut repairs_made = Vec::new();

                // Fix basic issues
                if state.pipeline_id.is_empty() {
                    state.pipeline_id = pipeline_id.to_string();
                    repairs_made.push("Fixed empty pipeline_id".to_string());
                }

                if state.run_id.is_empty() {
                    state.run_id = format!("repaired_{}", uuid::Uuid::new_v4());
                    repairs_made.push("Generated new run_id".to_string());
                }

                if state.version == 0 {
                    state.version = 1;
                    repairs_made.push("Fixed invalid version".to_string());
                }

                state.increment_version();

                Ok(RepairResult {
                    success: !repairs_made.is_empty(),
                    backup_created: false,
                    backup_id: None,
                    repairs_made,
                    issues_found: validation.validation_errors,
                    manual_intervention_required: false,
                })
            } else {
                Err(StateError::PipelineNotFound {
                    pipeline_id: pipeline_id.to_string(),
                })
            }
        }
    }

    async fn get_diagnostics(&self) -> Result<BackendDiagnostics, StateError> {
        let states = self.states.read().await;
        let locks = self.locks.read().await;

        let total_states = states.len() as u64;
        let total_locks = locks.len() as u64;

        let mut total_memory = 0;
        let mut oldest_state: Option<DateTime<Utc>> = None;
        let mut newest_state: Option<DateTime<Utc>> = None;

        for state in states.values() {
            total_memory += state.estimated_memory_usage();

            let state_time = state.metadata.created_at;
            match oldest_state {
                None => oldest_state = Some(state_time),
                Some(current_oldest) => {
                    if state_time < current_oldest {
                        oldest_state = Some(state_time);
                    }
                }
            }

            match newest_state {
                None => newest_state = Some(state_time),
                Some(current_newest) => {
                    if state_time > current_newest {
                        newest_state = Some(state_time);
                    }
                }
            }
        }

        let average_state_size_bytes = if total_states > 0 {
            total_memory as u64 / total_states
        } else {
            0
        };

        let mut performance_metrics = HashMap::new();
        performance_metrics.insert(
            "memory_usage_mb".to_string(),
            total_memory as f64 / (1024.0 * 1024.0),
        );
        performance_metrics.insert(
            "avg_state_size_bytes".to_string(),
            average_state_size_bytes as f64,
        );

        Ok(BackendDiagnostics {
            backend_type: "memory".to_string(),
            total_states,
            total_locks,
            total_backups: 0, // Memory backend doesn't maintain backups
            storage_used_bytes: total_memory as u64,
            storage_available_bytes: u64::MAX, // Essentially unlimited for memory
            average_state_size_bytes,
            oldest_state,
            newest_state,
            performance_metrics,
            health_issues: Vec::new(),
        })
    }

    async fn verify_integrity(&self) -> Result<IntegrityReport, StateError> {
        let states = self.states.read().await;
        let total_files_checked = states.len() as u64;
        let mut corrupted_files = Vec::new();

        // Check each state for validation errors
        for (pipeline_id, state) in states.iter() {
            if state.validate().is_err() {
                corrupted_files.push(format!("memory://{pipeline_id}"));
            }
        }

        let overall_health = if total_files_checked == 0 {
            1.0
        } else {
            1.0 - (corrupted_files.len() as f64 / total_files_checked as f64)
        };

        let repair_recommendations = if corrupted_files.is_empty() {
            Vec::new()
        } else {
            vec!["Run repair command on corrupted states".to_string()]
        };

        Ok(IntegrityReport {
            total_files_checked,
            corrupted_files,
            missing_files: Vec::new(), // Memory backend can't have missing files
            permission_errors: Vec::new(), // Memory backend doesn't have permission issues
            checksum_mismatches: Vec::new(), // Memory backend doesn't use checksums
            repair_recommendations,
            overall_health,
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

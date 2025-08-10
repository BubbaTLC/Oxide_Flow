use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Represents the current status of a pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PipelineStatus {
    /// Pipeline is ready to run but hasn't started
    Pending,
    /// Pipeline is currently running
    Running { started_at: DateTime<Utc> },
    /// Pipeline completed successfully
    Completed { completed_at: DateTime<Utc> },
    /// Pipeline failed with an error
    Failed {
        failed_at: DateTime<Utc>,
        error: String,
    },
    /// Pipeline was paused manually
    Paused { paused_at: DateTime<Utc> },
}

/// Represents the current status of a pipeline step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    /// Step is waiting to be executed
    Pending,
    /// Step is currently running
    Running { started_at: DateTime<Utc> },
    /// Step completed successfully
    Completed { completed_at: DateTime<Utc> },
    /// Step failed with an error
    Failed {
        error: String,
        failed_at: DateTime<Utc>,
    },
    /// Step was skipped due to continue_on_error or conditional logic
    Skipped { reason: String },
}

/// Core pipeline state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    // Identity and versioning
    pub pipeline_id: String,
    pub run_id: String,
    pub version: u64, // For optimistic concurrency control

    // Progress tracking
    pub last_processed_id: String,
    pub batch_number: u64,
    pub records_processed: u64,
    pub records_failed: u64,
    pub data_size_processed: u64, // bytes

    // Execution state
    pub current_step: String,
    pub step_states: HashMap<String, StepState>,
    pub status: PipelineStatus,

    // Timing and metadata
    pub started_at: DateTime<Utc>,
    pub last_success_timestamp: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,

    // Error tracking
    pub errors: Vec<ErrorRecord>,
    pub retry_count: u64,

    // Worker coordination (for future distributed features)
    pub worker_id: Option<String>,
    pub last_heartbeat: DateTime<Utc>,

    // Metadata
    pub metadata: StateMetadata,
}

/// State information for an individual pipeline step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepState {
    pub step_id: String,
    pub step_name: String,
    pub status: StepStatus,
    pub last_processed_id: String,
    pub records_processed: u64,
    pub processing_time_ms: u64,
    pub worker_id: Option<String>,
    pub last_heartbeat: DateTime<Utc>,

    // Step-specific metadata
    pub retry_count: u64,
    pub error_count: u64,
    pub config_hash: Option<String>, // Hash of step configuration
}

/// Error record for tracking pipeline and step failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub error_id: String,
    pub step_id: Option<String>, // None for pipeline-level errors
    pub error_type: ErrorType,
    pub message: String,
    pub context: String,
    pub timestamp: DateTime<Utc>,
    pub retryable: bool,
    pub stack_trace: Option<String>,
}

/// Types of errors that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorType {
    /// Configuration or validation error
    Configuration,
    /// Network or I/O error
    Network,
    /// Data processing error
    Processing,
    /// Resource exhaustion (memory, disk, etc.)
    Resource,
    /// Unknown or unexpected error
    Unknown,
}

/// Metadata about the state itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schema_version: String,
    pub state_backend: String,
    pub checkpoint_count: u64,
    pub last_checkpoint_at: DateTime<Utc>,

    // Optional fields for enhanced metadata
    pub pipeline_name: Option<String>,
    pub pipeline_version: Option<String>,
    pub environment: Option<String>,
    pub tags: HashMap<String, String>,
}

/// Errors that can occur during state management operations
#[derive(Error, Debug)]
pub enum StateError {
    #[error("Pipeline not found: {pipeline_id}")]
    PipelineNotFound { pipeline_id: String },

    #[error("State file not found: {path}")]
    StateFileNotFound { path: String },

    #[error("Lock already held by worker: {worker_id}")]
    LockAlreadyHeld { worker_id: String },

    #[error("Lock acquisition timeout after {timeout_ms}ms")]
    LockTimeout { timeout_ms: u64 },

    #[error("Version conflict: expected {expected}, found {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    #[error("Serialization error: {details}")]
    SerializationError { details: String },

    #[error("I/O error: {details}")]
    IoError { details: String },

    #[error("Backend error: {details}")]
    BackendError { details: String },

    #[error("Invalid state: {details}")]
    InvalidState { details: String },

    #[error("Worker not found: {worker_id}")]
    WorkerNotFound { worker_id: String },

    // Production hardening error types
    #[error("State file corrupted: {path}, reason: {reason}")]
    StateCorrupted { path: String, reason: String },

    #[error("Backup operation failed: {details}")]
    BackupFailed { details: String },

    #[error("Recovery operation failed: {details}")]
    RecoveryFailed { details: String },

    #[error("State validation failed: {validation_errors:?}")]
    ValidationFailed { validation_errors: Vec<String> },

    #[error("File system error: {operation}, path: {path}, error: {error}")]
    FileSystemError {
        operation: String,
        path: String,
        error: String,
    },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Disk space insufficient: {required_bytes} bytes needed, {available_bytes} available")]
    InsufficientDiskSpace {
        required_bytes: u64,
        available_bytes: u64,
    },

    #[error("Maximum retries exceeded: {max_retries} for operation: {operation}")]
    MaxRetriesExceeded { max_retries: u32, operation: String },
}

impl PipelineState {
    /// Create a new pipeline state
    pub fn new(pipeline_id: String, run_id: String) -> Self {
        let now = Utc::now();

        Self {
            pipeline_id: pipeline_id.clone(),
            run_id,
            version: 1,
            last_processed_id: String::new(),
            batch_number: 0,
            records_processed: 0,
            records_failed: 0,
            data_size_processed: 0,
            current_step: String::new(),
            step_states: HashMap::new(),
            status: PipelineStatus::Pending,
            started_at: now,
            last_success_timestamp: now,
            estimated_completion: None,
            errors: Vec::new(),
            retry_count: 0,
            worker_id: None,
            last_heartbeat: now,
            metadata: StateMetadata {
                created_at: now,
                updated_at: now,
                schema_version: "1.0.0".to_string(),
                state_backend: "file".to_string(),
                checkpoint_count: 0,
                last_checkpoint_at: now,
                pipeline_name: None,
                pipeline_version: None,
                environment: None,
                tags: HashMap::new(),
            },
        }
    }

    /// Update the state version for optimistic concurrency control
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.metadata.updated_at = Utc::now();
    }

    /// Add an error to the state
    pub fn add_error(&mut self, error: ErrorRecord) {
        self.errors.push(error);
        self.increment_version();
    }

    /// Update the heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        self.increment_version();
    }

    /// Check if the state is stale (no heartbeat for specified duration)
    pub fn is_stale(&self, stale_threshold_ms: u64) -> bool {
        let stale_threshold = chrono::Duration::milliseconds(stale_threshold_ms as i64);
        Utc::now() - self.last_heartbeat > stale_threshold
    }

    /// Get the current pipeline duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        (Utc::now() - self.started_at).num_milliseconds() as u64
    }

    /// Estimate memory usage of this state (for optimization)
    pub fn estimated_memory_usage(&self) -> usize {
        // Basic estimation - could be refined
        std::mem::size_of::<Self>()
            + self.pipeline_id.len()
            + self.run_id.len()
            + self.last_processed_id.len()
            + self.current_step.len()
            + self.step_states.len() * 500 // Rough estimate per step
            + self.errors.len() * 200 // Rough estimate per error
    }

    /// Validate the integrity and consistency of the pipeline state
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Basic field validation
        if self.pipeline_id.is_empty() {
            errors.push("Pipeline ID cannot be empty".to_string());
        }

        if self.run_id.is_empty() {
            errors.push("Run ID cannot be empty".to_string());
        }

        if self.version == 0 {
            errors.push("Version must be greater than 0".to_string());
        }

        // Status consistency checks
        match &self.status {
            PipelineStatus::Running { started_at } => {
                if started_at > &Utc::now() {
                    errors.push("Pipeline start time cannot be in the future".to_string());
                }
                if self.current_step.is_empty() {
                    errors.push("Running pipeline must have a current step".to_string());
                }
            }
            PipelineStatus::Completed { completed_at } => {
                if completed_at < &self.started_at {
                    errors.push("Completion time cannot be before start time".to_string());
                }
                if completed_at > &Utc::now() {
                    errors.push("Completion time cannot be in the future".to_string());
                }
            }
            PipelineStatus::Failed { failed_at, .. } => {
                if failed_at < &self.started_at {
                    errors.push("Failure time cannot be before start time".to_string());
                }
                if failed_at > &Utc::now() {
                    errors.push("Failure time cannot be in the future".to_string());
                }
            }
            _ => {} // Other statuses are fine
        }

        // Step state consistency checks
        for (step_id, step_state) in &self.step_states {
            if step_state.step_id != *step_id {
                errors.push(format!(
                    "Step ID mismatch: key '{}' vs state '{}'",
                    step_id, step_state.step_id
                ));
            }

            // Validate step status consistency
            match &step_state.status {
                StepStatus::Completed { completed_at } => {
                    if step_state.records_processed == 0 && step_state.processing_time_ms == 0 {
                        errors.push(format!(
                            "Completed step '{step_id}' should have processing metrics"
                        ));
                    }
                    if completed_at > &Utc::now() {
                        errors.push(format!(
                            "Step '{step_id}' completion time cannot be in the future"
                        ));
                    }
                }
                StepStatus::Failed { failed_at, .. } => {
                    if failed_at > &Utc::now() {
                        errors.push(format!(
                            "Step '{step_id}' failure time cannot be in the future"
                        ));
                    }
                }
                _ => {}
            }
        }

        // Data consistency checks
        let step_records_total: u64 = self.step_states.values().map(|s| s.records_processed).sum();

        if step_records_total > 0 && self.records_processed == 0 {
            errors.push("Total records processed should reflect step totals".to_string());
        }

        // Timestamp consistency
        if self.last_success_timestamp < self.started_at {
            errors.push("Last success timestamp cannot be before start time".to_string());
        }

        if self.last_heartbeat < self.started_at {
            errors.push("Last heartbeat cannot be before start time".to_string());
        }

        // Error validation
        for (idx, error) in self.errors.iter().enumerate() {
            if error.timestamp < self.started_at {
                errors.push(format!(
                    "Error {idx} timestamp cannot be before pipeline start"
                ));
            }
            if error.message.is_empty() {
                errors.push(format!("Error {idx} message cannot be empty"));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Create a checksum/hash of the state for corruption detection
    pub fn checksum(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash key fields that shouldn't change unexpectedly
        self.pipeline_id.hash(&mut hasher);
        self.run_id.hash(&mut hasher);
        self.version.hash(&mut hasher);
        self.records_processed.hash(&mut hasher);
        self.batch_number.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    /// Check if the state appears to be corrupted by validating critical fields
    pub fn is_corrupted(&self) -> bool {
        // Basic corruption checks
        self.pipeline_id.is_empty()
            || self.run_id.is_empty()
            || self.version == 0
            || self.validate().is_err()
    }
}

impl StepState {
    /// Create a new step state
    pub fn new(step_id: String, step_name: String) -> Self {
        let now = Utc::now();

        Self {
            step_id,
            step_name,
            status: StepStatus::Pending,
            last_processed_id: String::new(),
            records_processed: 0,
            processing_time_ms: 0,
            worker_id: None,
            last_heartbeat: now,
            retry_count: 0,
            error_count: 0,
            config_hash: None,
        }
    }

    /// Mark the step as started
    pub fn start(&mut self) {
        self.status = StepStatus::Running {
            started_at: Utc::now(),
        };
        self.last_heartbeat = Utc::now();
    }

    /// Mark the step as completed
    pub fn complete(&mut self) {
        self.status = StepStatus::Completed {
            completed_at: Utc::now(),
        };
        self.last_heartbeat = Utc::now();
    }

    /// Mark the step as failed
    pub fn fail(&mut self, error: String) {
        self.status = StepStatus::Failed {
            error,
            failed_at: Utc::now(),
        };
        self.error_count += 1;
        self.last_heartbeat = Utc::now();
    }

    /// Check if the step is currently running
    pub fn is_running(&self) -> bool {
        matches!(self.status, StepStatus::Running { .. })
    }

    /// Check if the step has completed successfully
    pub fn is_completed(&self) -> bool {
        matches!(self.status, StepStatus::Completed { .. })
    }

    /// Check if the step has failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, StepStatus::Failed { .. })
    }
}

impl ErrorRecord {
    /// Create a new error record
    pub fn new(
        step_id: Option<String>,
        error_type: ErrorType,
        message: String,
        context: String,
        retryable: bool,
    ) -> Self {
        Self {
            error_id: Uuid::new_v4().to_string(),
            step_id,
            error_type,
            message,
            context,
            timestamp: Utc::now(),
            retryable,
            stack_trace: None,
        }
    }

    /// Create a configuration error
    pub fn config_error(message: String, context: String) -> Self {
        Self::new(None, ErrorType::Configuration, message, context, false)
    }

    /// Create a processing error
    pub fn processing_error(
        step_id: String,
        message: String,
        context: String,
        retryable: bool,
    ) -> Self {
        Self::new(
            Some(step_id),
            ErrorType::Processing,
            message,
            context,
            retryable,
        )
    }

    /// Create a network error
    pub fn network_error(step_id: String, message: String, context: String) -> Self {
        Self::new(Some(step_id), ErrorType::Network, message, context, true)
    }
}

// Implement From for common error types
impl From<std::io::Error> for StateError {
    fn from(err: std::io::Error) -> Self {
        StateError::IoError {
            details: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for StateError {
    fn from(err: serde_json::Error) -> Self {
        StateError::SerializationError {
            details: err.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for StateError {
    fn from(err: serde_yaml::Error) -> Self {
        StateError::SerializationError {
            details: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_state_creation() {
        let state = PipelineState::new("test_pipeline".to_string(), "run_123".to_string());

        assert_eq!(state.pipeline_id, "test_pipeline");
        assert_eq!(state.run_id, "run_123");
        assert_eq!(state.version, 1);
        assert_eq!(state.records_processed, 0);
        assert_eq!(state.status, PipelineStatus::Pending);
        assert!(state.step_states.is_empty());
        assert!(state.errors.is_empty());
    }

    #[test]
    fn test_pipeline_state_version_increment() {
        let mut state = PipelineState::new("test".to_string(), "run".to_string());
        let initial_version = state.version;
        let initial_updated_at = state.metadata.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        state.increment_version();

        assert_eq!(state.version, initial_version + 1);
        assert!(state.metadata.updated_at > initial_updated_at);
    }

    #[test]
    fn test_step_state_lifecycle() {
        let mut step = StepState::new("step_1".to_string(), "read_file".to_string());

        assert_eq!(step.step_id, "step_1");
        assert_eq!(step.step_name, "read_file");
        assert_eq!(step.status, StepStatus::Pending);

        step.start();
        assert!(step.is_running());
        assert!(!step.is_completed());
        assert!(!step.is_failed());

        step.complete();
        assert!(!step.is_running());
        assert!(step.is_completed());
        assert!(!step.is_failed());
    }

    #[test]
    fn test_step_state_failure() {
        let mut step = StepState::new("step_1".to_string(), "parse_json".to_string());

        step.start();
        step.fail("Invalid JSON format".to_string());

        assert!(!step.is_running());
        assert!(!step.is_completed());
        assert!(step.is_failed());
        assert_eq!(step.error_count, 1);
    }

    #[test]
    fn test_error_record_creation() {
        let error = ErrorRecord::config_error(
            "Missing required field".to_string(),
            "pipeline validation".to_string(),
        );

        assert!(error.error_id.len() > 0);
        assert_eq!(error.step_id, None);
        assert_eq!(error.error_type, ErrorType::Configuration);
        assert!(!error.retryable);

        let processing_error = ErrorRecord::processing_error(
            "step_1".to_string(),
            "Data transformation failed".to_string(),
            "invalid data format".to_string(),
            true,
        );

        assert_eq!(processing_error.step_id, Some("step_1".to_string()));
        assert_eq!(processing_error.error_type, ErrorType::Processing);
        assert!(processing_error.retryable);
    }

    #[test]
    fn test_pipeline_state_add_error() {
        let mut state = PipelineState::new("test".to_string(), "run".to_string());
        let initial_version = state.version;

        let error = ErrorRecord::config_error("Test error".to_string(), "Unit test".to_string());

        state.add_error(error);

        assert_eq!(state.errors.len(), 1);
        assert_eq!(state.version, initial_version + 1);
        assert_eq!(state.errors[0].message, "Test error");
    }

    #[test]
    fn test_state_staleness() {
        let mut state = PipelineState::new("test".to_string(), "run".to_string());

        // Fresh state should not be stale
        assert!(!state.is_stale(1000));

        // Manually set an old heartbeat
        state.last_heartbeat = Utc::now() - chrono::Duration::seconds(10);

        // Should be stale with 5 second threshold
        assert!(state.is_stale(5000));
        assert!(!state.is_stale(15000));
    }

    #[test]
    fn test_state_memory_estimation() {
        let state = PipelineState::new("test_pipeline".to_string(), "run_123".to_string());
        let memory_usage = state.estimated_memory_usage();

        // Should be a reasonable estimate (at least basic struct size)
        assert!(memory_usage > 100);
        assert!(memory_usage < 10000); // Reasonable upper bound for empty state
    }

    #[test]
    fn test_state_serialization() {
        let state = PipelineState::new("test_pipeline".to_string(), "run_123".to_string());

        // Test JSON serialization
        let json_result = serde_json::to_string(&state);
        assert!(json_result.is_ok());

        let json_str = json_result.unwrap();
        let deserialized: Result<PipelineState, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let restored_state = deserialized.unwrap();
        assert_eq!(restored_state.pipeline_id, state.pipeline_id);
        assert_eq!(restored_state.run_id, state.run_id);
        assert_eq!(restored_state.version, state.version);
    }

    #[test]
    fn test_state_yaml_serialization() {
        let mut state = PipelineState::new("yaml_test".to_string(), "run_yaml".to_string());

        // Add some data to make it more interesting
        let error = ErrorRecord::processing_error(
            "step_1".to_string(),
            "YAML test error".to_string(),
            "testing context".to_string(),
            true,
        );
        state.add_error(error);

        // Test YAML serialization
        let yaml_result = serde_yaml::to_string(&state);
        assert!(yaml_result.is_ok());

        let yaml_str = yaml_result.unwrap();
        let deserialized: Result<PipelineState, _> = serde_yaml::from_str(&yaml_str);
        assert!(deserialized.is_ok());

        let restored_state = deserialized.unwrap();
        assert_eq!(restored_state.pipeline_id, state.pipeline_id);
        assert_eq!(restored_state.errors.len(), 1);
        assert_eq!(restored_state.errors[0].message, "YAML test error");
    }
}

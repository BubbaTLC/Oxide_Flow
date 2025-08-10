pub mod backend;
pub mod cli;
pub mod manager;
pub mod pipeline_tracker;
pub mod types;

// Re-export common types for convenience
pub use backend::{
    BackendConfig, BackendHealth, CleanupResult, FileBackend, LockInfo, MemoryBackend,
    SerializationFormat, StateBackend,
};
pub use manager::{
    HeartbeatHandle, ObservableStateManager, StateManager, StateManagerConfig, StateManagerLock,
    StateObserver,
};
pub use types::{
    ErrorRecord, ErrorType, PipelineState, PipelineStatus, StateError, StateMetadata, StepState,
    StepStatus,
};

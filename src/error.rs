use thiserror::Error;

#[derive(Error, Debug)]
pub enum OxiError {
    #[error("Failed to read from stdin")]
    StdInReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Failed to parse CSV: {0}")]
    CsvParseError(#[from] csv::Error),

    #[error("Type conversion error: {0}")]
    TypeConversionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    #[error("Oxi execution error: {0}")]
    ExecutionError(String),

    #[error("Unknown Oxi: {0}")]
    UnknownOxi(String),

    #[error("Oxi chaining error: {0}")]
    ChainingError(String),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    // Enhanced errors with context for better debugging
    #[error("JSON operation failed: {operation} - {details}")]
    JsonOperationError { operation: String, details: String },

    #[error("Type mismatch in pipeline: expected {expected}, got {actual} at step '{step}'")]
    TypeMismatch {
        expected: String,
        actual: String,
        step: String,
    },

    #[error("Schema validation failed: {details}")]
    ValidationError { details: String },

    #[error("Query operation failed: {query} - {error}")]
    QueryError { query: String, error: String },

    #[error("Data format incompatible: {source_format} cannot be converted to {target_format}")]
    FormatIncompatible {
        source_format: String,
        target_format: String,
    },

    // Processing limit errors for batch processing and resource management
    #[error("Batch size limit exceeded: {actual_size} > {max_size} in '{oxi_name}'")]
    BatchSizeExceeded {
        actual_size: usize,
        max_size: usize,
        oxi_name: String,
    },

    #[error("Memory limit exceeded: {actual_mb}MB > {max_mb}MB in '{oxi_name}'")]
    MemoryLimitExceeded {
        actual_mb: usize,
        max_mb: usize,
        oxi_name: String,
    },

    #[error("Processing timeout: {actual_ms}ms > {max_ms}ms in '{oxi_name}'")]
    ProcessingTimeout {
        actual_ms: u64,
        max_ms: u64,
        oxi_name: String,
    },

    #[error("Unsupported input type: {oxi_name} does not support {input_type}")]
    UnsupportedInputType {
        oxi_name: String,
        input_type: String,
    },
}

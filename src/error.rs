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
}

# Oxi SDK Template

This document provides a template and guidelines for creating new Oxis for Oxide Flow.

## Directory Structure

Each Oxi should be organized in its own module under the `src/oxis` directory:

```
src/oxis/
  └── your_oxi/
      ├── mod.rs        # Exports the oxi module
      └── oxi.rs        # Contains the Oxi implementation
```

# Oxi SDK - Enhanced Development Guide

This document provides a comprehensive guide for creating new Oxis using the enhanced Oxide Flow SDK with processing limits, type safety, and robust error handling.

## Overview

The Oxide Flow Oxi SDK provides:
- **Processing Limits**: Built-in resource management and constraints
- **Type Safety**: Input validation and type checking
- **Enhanced Error Handling**: Context-aware errors with specific details
- **Batch Awareness**: Support for both streaming and batch processing
- **Memory Management**: Automatic memory usage estimation and limits

## Directory Structure

Each Oxi should be organized in its own module under the `src/oxis` directory:

```
src/oxis/
  └── your_oxi/
      ├── mod.rs        # Exports the oxi module
      └── oxi.rs        # Contains the Oxi implementation
```

## Enhanced Template Implementation

Here's a complete template using the new enhanced SDK:

### mod.rs

```rust
pub mod oxi;
```

### oxi.rs

```rust
use crate::oxis::prelude::*;

/// YourOxi does something useful with data processing
#[derive(Debug)]
pub struct YourOxi {
    config: YourOxiConfig,
}

#[derive(Debug, Deserialize)]
pub struct YourOxiConfig {
    pub processing_mode: String,
    pub enable_validation: bool,
    pub max_items: Option<usize>,
}

impl Default for YourOxiConfig {
    fn default() -> Self {
        Self {
            processing_mode: "normal".to_string(),
            enable_validation: true,
            max_items: None,
        }
    }
}

#[async_trait]
impl Oxi for YourOxi {
    fn name(&self) -> &str {
        "your_oxi"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
            type: object
            properties:
              processing_mode:
                type: string
                enum: ["normal", "fast", "thorough"]
                description: "Processing mode for the operation"
                default: "normal"
              enable_validation:
                type: boolean
                description: "Whether to enable input validation"
                default: true
              max_items:
                type: integer
                description: "Maximum number of items to process (optional)"
                minimum: 1
        "#).unwrap()
    }

    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: Some(5_000),     // Handle up to 5K records
            max_memory_mb: Some(128),        // Use up to 128MB
            max_processing_time_ms: Some(10_000), // 10 second timeout
            supported_input_types: vec![
                OxiDataType::Json,           // Primary input type
                OxiDataType::Text,           // Secondary input type
                // OxiDataType::Binary,      // Uncomment if binary supported
                // OxiDataType::Empty,       // Uncomment if empty input supported
            ],
        }
    }

    fn validate_input(&self, input: &OxiData) -> Result<(), OxiError> {
        if !self.config.enable_validation {
            return Ok(());
        }

        match input {
            OxiData::Json(json) => {
                // Example: validate JSON structure
                if json.is_null() {
                    return Err(OxiError::ValidationError {
                        details: "JSON input cannot be null".to_string(),
                    });
                }

                // Example: check for required fields
                if let Some(obj) = json.as_object() {
                    if !obj.contains_key("data") {
                        return Err(OxiError::ValidationError {
                            details: "JSON input must contain 'data' field".to_string(),
                        });
                    }
                }
                Ok(())
            }
            OxiData::Text(text) => {
                // Example: validate text is not empty
                if text.trim().is_empty() {
                    return Err(OxiError::ValidationError {
                        details: "Text input cannot be empty".to_string(),
                    });
                }
                Ok(())
            }
            _ => {
                // This should be caught by supported_input_types, but double-check
                Err(OxiError::UnsupportedInputType {
                    oxi_name: self.name().to_string(),
                    input_type: input.get_data_type().to_string(),
                })
            }
        }
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Parse configuration with defaults
        let oxi_config: YourOxiConfig = if config.values.is_empty() {
            YourOxiConfig::default()
        } else {
            let yaml_value = serde_yaml::Value::Mapping(
                config.values.iter()
                    .map(|(k, v)| (serde_yaml::Value::String(k.clone()), v.clone()))
                    .collect()
            );
            serde_yaml::from_value(yaml_value).unwrap_or_else(|_| YourOxiConfig::default())
        };

        // Process based on input type and configuration
        match input {
            OxiData::Text(text) => {
                let processed_text = match oxi_config.processing_mode.as_str() {
                    "fast" => text.to_uppercase(),
                    "thorough" => format!("PROCESSED[{}]: {}", self.name(), text),
                    _ => format!("Processed: {}", text),
                };
                Ok(OxiData::Text(processed_text))
            },
            OxiData::Json(mut json) => {
                // Handle JSON processing
                match oxi_config.processing_mode.as_str() {
                    "fast" => {
                        // Fast processing - minimal changes
                        if let Some(obj) = json.as_object_mut() {
                            obj.insert("processed".to_string(),
                                      serde_json::Value::Bool(true));
                        }
                    },
                    "thorough" => {
                        // Thorough processing - add metadata
                        if let Some(obj) = json.as_object_mut() {
                            obj.insert("processed_by".to_string(),
                                      serde_json::Value::String(self.name().to_string()));
                            obj.insert("processing_mode".to_string(),
                                      serde_json::Value::String(oxi_config.processing_mode.clone()));
                            obj.insert("timestamp".to_string(),
                                      serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
                        }
                    },
                    _ => {
                        // Normal processing
                        if let Some(obj) = json.as_object_mut() {
                            obj.insert("processed".to_string(),
                                      serde_json::Value::Bool(true));
                        }
                    }
                }

                // Check max_items limit if processing an array
                if let Some(max_items) = oxi_config.max_items {
                    if let Some(arr) = json.as_array() {
                        if arr.len() > max_items {
                            return Err(anyhow::anyhow!(
                                "Input array has {} items, but max_items is set to {}",
                                arr.len(), max_items
                            ));
                        }
                    }
                }

                Ok(OxiData::Json(json))
            },
            _ => {
                // This shouldn't happen due to input validation
                Err(anyhow::anyhow!("Unsupported input type"))
            }
        }
    }
}

impl YourOxi {
    pub fn new(config: YourOxiConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self {
            config: YourOxiConfig::default(),
        }
    }

    pub fn with_mode(mode: &str) -> Self {
        Self {
            config: YourOxiConfig {
                processing_mode: mode.to_string(),
                ..YourOxiConfig::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_text_processing() {
        let oxi = YourOxi::with_defaults();
        let input = OxiData::Text("test input".to_string());
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Text(text) = result {
            assert!(text.contains("Processed: test input"));
        } else {
            panic!("Expected text output");
        }
    }

    #[tokio::test]
    async fn test_json_processing() {
        let oxi = YourOxi::with_defaults();
        let input = OxiData::Json(json!({"data": "test", "value": 42}));
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Json(json) = result {
            assert_eq!(json["processed"], true);
            assert_eq!(json["data"], "test");
            assert_eq!(json["value"], 42);
        } else {
            panic!("Expected JSON output");
        }
    }

    #[tokio::test]
    async fn test_processing_limits() {
        let oxi = YourOxi::with_defaults();
        let limits = oxi.processing_limits();

        assert_eq!(limits.max_batch_size, Some(5_000));
        assert_eq!(limits.max_memory_mb, Some(128));
        assert_eq!(limits.max_processing_time_ms, Some(10_000));
        assert!(limits.supported_input_types.contains(&OxiDataType::Json));
        assert!(limits.supported_input_types.contains(&OxiDataType::Text));
    }

    #[tokio::test]
    async fn test_input_validation() {
        let oxi = YourOxi::with_defaults();

        // Test valid JSON input
        let valid_json = OxiData::Json(json!({"data": "test"}));
        assert!(oxi.validate_input(&valid_json).is_ok());

        // Test invalid JSON input (null)
        let invalid_json = OxiData::Json(serde_json::Value::Null);
        assert!(oxi.validate_input(&invalid_json).is_err());

        // Test valid text input
        let valid_text = OxiData::Text("some text".to_string());
        assert!(oxi.validate_input(&valid_text).is_ok());

        // Test invalid text input (empty)
        let invalid_text = OxiData::Text("   ".to_string());
        assert!(oxi.validate_input(&invalid_text).is_err());
    }

    #[tokio::test]
    async fn test_processing_modes() {
        let config = OxiConfig::default();

        // Test fast mode
        let fast_oxi = YourOxi::with_mode("fast");
        let input = OxiData::Text("hello".to_string());
        let result = fast_oxi.process(input, &config).await.unwrap();
        if let OxiData::Text(text) = result {
            assert_eq!(text, "HELLO");
        }

        // Test thorough mode
        let thorough_oxi = YourOxi::with_mode("thorough");
        let input = OxiData::Text("hello".to_string());
        let result = thorough_oxi.process(input, &config).await.unwrap();
        if let OxiData::Text(text) = result {
            assert!(text.contains("PROCESSED[your_oxi]: hello"));
        }
    }

    #[tokio::test]
    async fn test_memory_limit_enforcement() {
        let oxi = YourOxi::with_defaults();

        // Create large JSON data that would exceed memory limit
        let large_string = "x".repeat(200 * 1024 * 1024); // 200MB string
        let large_json = OxiData::Json(json!({"data": large_string}));

        // This should fail during process() due to memory limits
        let result = oxi.process(OxiDataWithSchema::from_data(large_json), &OxiConfig::default()).await;
        assert!(result.is_err());

        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("Memory limit exceeded"));
    }
}
```

## Enhanced Best Practices

### 1. **Processing Limits Configuration**
- Set realistic limits based on your Oxi's capabilities
- Consider the typical data sizes your Oxi will handle
- Be conservative with memory limits to prevent OOM errors
- Specify only the input types you actually support

```rust
fn processing_limits(&self) -> ProcessingLimits {
    ProcessingLimits {
        max_batch_size: Some(10_000),    // Realistic for your processing
        max_memory_mb: Some(256),        // Conservative memory usage
        max_processing_time_ms: Some(30_000), // Reasonable timeout
        supported_input_types: vec![
            OxiDataType::Json,           // Only what you support
            OxiDataType::Text,
        ],
    }
}
```

### 2. **Input Validation Strategy**
- Validate beyond just type checking
- Provide clear, actionable error messages
- Check for required fields or data structures
- Validate ranges and constraints early

```rust
fn validate_input(&self, input: &OxiData) -> Result<(), OxiError> {
    match input {
        OxiData::Json(json) => {
            // Check structure
            if !json.is_object() {
                return Err(OxiError::ValidationError {
                    details: "JSON input must be an object".to_string(),
                });
            }

            // Check required fields
            let obj = json.as_object().unwrap();
            if !obj.contains_key("required_field") {
                return Err(OxiError::ValidationError {
                    details: "Missing required field 'required_field'".to_string(),
                });
            }

            Ok(())
        }
        _ => Ok(())
    }
}
```

### 3. **Enhanced Error Handling**
- Use the new context-aware error types
- Include specific details in error messages
- Handle expected failures gracefully
- Propagate unexpected errors with context

```rust
// Good error handling examples
return Err(anyhow::anyhow!(OxiError::ValidationError {
    details: format!("Field 'count' must be between 1 and 1000, got {}", count),
}));

return Err(anyhow::anyhow!(OxiError::JsonOperationError {
    operation: "field_extraction".to_string(),
    details: format!("Field '{}' not found in JSON object", field_name),
}));
```

### 4. **Configuration Management**
- Define a struct for your Oxi's configuration
- Provide sensible defaults
- Use proper deserialization with error handling
- Validate configuration values

```rust
#[derive(Debug, Deserialize)]
pub struct YourOxiConfig {
    pub mode: ProcessingMode,
    pub threshold: f64,
    pub options: HashMap<String, String>,
}

impl Default for YourOxiConfig {
    fn default() -> Self {
        Self {
            mode: ProcessingMode::Normal,
            threshold: 0.5,
            options: HashMap::new(),
        }
    }
}
```

### 5. **Comprehensive Testing**
- Test all supported input types
- Test processing limits enforcement
- Test error conditions and edge cases
- Test configuration variations
- Include performance tests for large data

```rust
#[tokio::test]
async fn test_batch_size_limits() {
    let oxi = YourOxi::with_defaults();

    // Create data that exceeds batch size limit
    let large_array: Vec<serde_json::Value> = (0..10_000).map(|i| json!({"id": i})).collect();
    let large_input = OxiData::Json(serde_json::Value::Array(large_array));

    let result = oxi.process(OxiDataWithSchema::from_data(large_input), &OxiConfig::default()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Batch size limit exceeded"));
}
```

### 6. **Memory-Aware Processing**
- Use the `estimated_memory_usage()` method to check data size
- Process data in chunks for large datasets
- Implement streaming for very large inputs
- Monitor memory usage during processing

```rust
async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
    let memory_usage = input.estimated_memory_usage();

    // Log memory usage for monitoring
    log::debug!("Processing {} bytes of data", memory_usage);

    // Handle large datasets differently
    if memory_usage > 100 * 1024 * 1024 { // 100MB
        // Use streaming processing
        self.process_streaming(input, config).await
    } else {
        // Use normal processing
        self.process_normal(input, config).await
    }
}
```

### 7. **Batch-Aware Processing**
- Use `is_batch()` to detect batch data
- Handle both single items and batches
- Optimize processing for batch vs streaming

```rust
async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
    match input {
        OxiData::Json(json) if input.is_batch() => {
            // Batch processing optimization
            let array = input.as_array()?;
            let processed: Vec<serde_json::Value> = array
                .into_iter()
                .map(|item| self.process_single_item(item))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(OxiData::Json(serde_json::Value::Array(processed)))
        }
        _ => {
            // Single item processing
            self.process_single(input, config).await
        }
    }
}
```

## SDK Features Reference

### ProcessingLimits
```rust
pub struct ProcessingLimits {
    pub max_batch_size: Option<usize>,           // Max items in a batch
    pub max_memory_mb: Option<usize>,            // Max memory usage in MB
    pub max_processing_time_ms: Option<u64>,     // Max processing time in ms
    pub supported_input_types: Vec<OxiDataType>, // Supported input types
}
```

### OxiDataType Enum
```rust
pub enum OxiDataType {
    Json,    // JSON data structures
    Text,    // String/text data
    Binary,  // Binary data (files, images, etc.)
    Empty,   // Empty/null data
}
```

### Enhanced Error Types
```rust
// Context-aware errors with specific details
OxiError::TypeMismatch { expected, actual, step }
OxiError::BatchSizeExceeded { actual_size, max_size, oxi_name }
OxiError::MemoryLimitExceeded { actual_mb, max_mb, oxi_name }
OxiError::ProcessingTimeout { actual_ms, max_ms, oxi_name }
OxiError::UnsupportedInputType { oxi_name, input_type }
OxiError::ValidationError { details }
OxiError::JsonOperationError { operation, details }
```

### OxiData Enhanced Methods
```rust
// Type detection and conversion
input.get_data_type() -> OxiDataType
input.to_json() -> Result<serde_json::Value>
input.as_array() -> Result<Vec<serde_json::Value>>

// Batch processing support
input.is_batch() -> bool
input.estimated_memory_usage() -> usize
```

## Configuration Schema

Use YAML Schema format for your configuration schema. Common types include:

- `string`: Text values
- `boolean`: True/false values
- `integer`: Whole numbers
- `number`: Floating point numbers
- `array`: Lists of values
- `object`: Nested configuration objects

Example:

```yaml
type: object
properties:
  stringOption:
    type: string
    description: "A string option"
    default: "default value"
  numberOption:
    type: number
    description: "A number option"
    default: 42
  booleanOption:
    type: boolean
    description: "A boolean option"
    default: false
  enumOption:
    type: string
    enum: ["option1", "option2", "option3"]
    description: "An option with predefined values"
    default: "option1"
  arrayOption:
    type: array
    items:
      type: string
    description: "An array of strings"
    default: []
```

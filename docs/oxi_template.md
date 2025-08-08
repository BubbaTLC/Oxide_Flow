# Oxi Development Template

This document provides a template and guidelines for creating new Oxis for Oxide Flow using the enhanced Oxi SDK.

## Directory Structure

Each Oxi should be organized in its own module under the `src/oxis` directory:

```
src/oxis/
  └── your_oxi/
      ├── mod.rs        # Exports the oxi module
      └── oxi.rs        # Contains the Oxi implementation
```

## Template Implementation

Here's a template for implementing a new Oxi using the enhanced SDK:

### mod.rs

```rust
pub mod oxi;
```

### oxi.rs

```rust
use crate::oxis::prelude::*;

/// YourOxi does something useful with data
pub struct YourOxi {
    config: YourOxiConfig,
}

#[derive(Debug, Deserialize)]
pub struct YourOxiConfig {
    pub option1: String,
    pub option2: bool,
    pub option3: Option<i32>,
}

impl Default for YourOxiConfig {
    fn default() -> Self {
        Self {
            option1: "default_value".to_string(),
            option2: false,
            option3: None,
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
              option1:
                type: string
                description: "Description of option1"
                default: "default_value"
              option2:
                type: boolean
                description: "Description of option2"
                default: false
              option3:
                type: integer
                description: "Optional integer parameter"
        "#).unwrap()
    }

    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: Some(5_000),     // Your Oxi can handle 5K records
            max_memory_mb: Some(128),        // Use up to 128MB
            max_processing_time_ms: Some(10_000), // 10 second timeout
            supported_input_types: vec![
                OxiDataType::Json,           // Supports JSON input
                OxiDataType::Text,           // Supports text input
                // OxiDataType::Binary,      // Uncomment if binary supported
                // OxiDataType::Empty,       // Uncomment if empty supported
            ],
        }
    }

    fn validate_input(&self, input: &OxiData) -> Result<(), OxiError> {
        // Custom input validation beyond type checking
        match input {
            OxiData::Json(json) => {
                // Example: ensure JSON has required fields
                if json.get("required_field").is_none() {
                    return Err(OxiError::ValidationError {
                        details: "Missing required_field in JSON input".to_string(),
                    });
                }
                Ok(())
            }
            OxiData::Text(text) => {
                // Example: ensure text is not empty
                if text.trim().is_empty() {
                    return Err(OxiError::ValidationError {
                        details: "Text input cannot be empty".to_string(),
                    });
                }
                Ok(())
            }
            _ => Ok(()) // Accept other supported types
        }
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Parse configuration
        let config: YourOxiConfig = if config.values.is_empty() {
            YourOxiConfig::default()
        } else {
            let yaml_value = serde_yaml::Value::Mapping(
                config.values.iter()
                    .map(|(k, v)| (serde_yaml::Value::String(k.clone()), v.clone()))
                    .collect()
            );
            serde_yaml::from_value(yaml_value)?
        };

        // Process based on input type
        match input {
            OxiData::Text(text) => {
                // Process text data
                let processed_text = if config.option2 {
                    text.to_uppercase()
                } else {
                    format!("{}: {}", config.option1, text)
                };
                Ok(OxiData::Text(processed_text))
            },
            OxiData::Json(json) => {
                // Process JSON data
                let mut processed_json = json;

                // Example: add metadata
                if let Some(obj) = processed_json.as_object_mut() {
                    obj.insert("processed_by".to_string(),
                              serde_json::Value::String(self.name().to_string()));
                    obj.insert("option1".to_string(),
                              serde_json::Value::String(config.option1.clone()));
                }

                Ok(OxiData::Json(processed_json))
            },
            _ => {
                // This shouldn't happen due to supported_input_types validation
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_your_oxi_text_processing() {
        let oxi = YourOxi::with_defaults();
        let input = OxiData::Text("test input".to_string());
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Text(text) = result {
            assert!(text.contains("default_value: test input"));
        } else {
            panic!("Expected text output");
        }
    }

    #[tokio::test]
    async fn test_your_oxi_json_processing() {
        let oxi = YourOxi::with_defaults();
        let input = OxiData::Json(json!({"data": "test"}));
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Json(json) = result {
            assert_eq!(json["processed_by"], "your_oxi");
            assert_eq!(json["data"], "test");
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
        let valid_json = OxiData::Json(json!({"required_field": "value"}));
        assert!(oxi.validate_input(&valid_json).is_ok());

        // Test invalid JSON input
        let invalid_json = OxiData::Json(json!({"other_field": "value"}));
        assert!(oxi.validate_input(&invalid_json).is_err());

        // Test valid text input
        let valid_text = OxiData::Text("some text".to_string());
        assert!(oxi.validate_input(&valid_text).is_ok());

        // Test invalid text input
        let invalid_text = OxiData::Text("   ".to_string());
        assert!(oxi.validate_input(&invalid_text).is_err());
    }

    #[tokio::test]
    async fn test_memory_limit_enforcement() {
        let oxi = YourOxi::with_defaults();

        // Create large JSON data that exceeds memory limit
        let large_string = "x".repeat(200 * 1024 * 1024); // 200MB string
        let large_json = OxiData::Json(json!({"data": large_string}));

        // This should fail during process() due to memory limits
        let result = oxi.process(OxiDataWithSchema::from_data(large_json), &OxiConfig::default()).await;
        assert!(result.is_err());

        // Check that it's specifically a memory limit error
        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("Memory limit exceeded"));
    }
}
```

## Best Practices

### 1. Configuration Handling
- Define a struct for your Oxi's configuration
- Provide sensible defaults
- Use serde for YAML deserialization
- Validate configuration in your schema

### 2. Processing Limits
- Set realistic limits based on your Oxi's capabilities
- Consider the typical data sizes your Oxi will handle
- Be conservative with memory limits
- Specify only the input types you actually support

### 3. Input Validation
- Validate beyond just type checking
- Provide clear error messages
- Check for required fields or data structures
- Validate ranges and constraints

### 4. Error Handling
- Use the enhanced error types for better debugging
- Include context in error messages
- Handle expected failures gracefully
- Propagate unexpected errors with context

### 5. Testing
- Test all supported input types
- Test processing limits enforcement
- Test error conditions
- Test configuration variations
- Include performance tests for large data

### 6. Documentation
- Document what your Oxi does
- Explain configuration options
- Provide usage examples
- Document performance characteristics

## Configuration Schema Format

Use YAML Schema format for your configuration schema:

```yaml
type: object
properties:
  string_option:
    type: string
    description: "A string option"
    default: "default value"
  number_option:
    type: number
    description: "A number option"
    default: 42
    minimum: 0
    maximum: 100
  boolean_option:
    type: boolean
    description: "A boolean option"
    default: false
  enum_option:
    type: string
    enum: ["option1", "option2", "option3"]
    description: "An option with predefined values"
    default: "option1"
  array_option:
    type: array
    items:
      type: string
    description: "An array of strings"
    default: []
  object_option:
    type: object
    properties:
      nested_field:
        type: string
        default: "nested_value"
    description: "A nested object"
required:
  - string_option  # Mark required fields
```

## Integration Steps

1. **Create your Oxi module** following the directory structure
2. **Implement the Oxi trait** with all required methods
3. **Add to the registry** in `src/oxis/mod.rs`:
   ```rust
   pub mod your_oxi;
   // ... in the registry function ...
   "your_oxi" => Box::new(your_oxi::oxi::YourOxi::with_defaults()),
   ```
4. **Write comprehensive tests** covering all functionality
5. **Update documentation** with usage examples

This enhanced template provides a solid foundation for building robust, well-tested Oxis that integrate seamlessly with the Oxide Flow processing limits and error handling system.

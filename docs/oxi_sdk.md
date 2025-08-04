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

## Template Implementation

Here's a template for implementing a new Oxi:

### mod.rs

```rust
pub mod oxi;
```

### oxi.rs

```rust
use crate::oxis::prelude::*;
use async_trait::async_trait;

/// YourOxi does something useful
pub struct YourOxi;

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
                default: "default value"
              option2:
                type: boolean
                description: "Description of option2"
                default: false
        "#).unwrap()
    }
    
    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Get configuration options
        let option1 = config.get_string_or("option1", "default value");
        let option2 = config.get_bool_or("option2", false);
        
        // Process based on input type
        match input {
            OxiData::Text(text) => {
                // Process text data
                let processed_text = format!("Processed: {}", text);
                Ok(OxiData::Text(processed_text))
            },
            OxiData::Structured(data) => {
                // Process structured data
                // ... your processing logic here ...
                Ok(OxiData::Structured(data))
            },
            OxiData::Binary(bytes) => {
                // Process binary data
                // ... your processing logic here ...
                Ok(OxiData::Binary(bytes))
            },
            OxiData::Tabular(rows) => {
                // Process tabular data
                // ... your processing logic here ...
                Ok(OxiData::Tabular(rows))
            },
            OxiData::Empty => {
                // Handle empty input
                Ok(OxiData::Empty)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_your_oxi() {
        let oxi = YourOxi;
        let input = OxiData::Text("test input".to_string());
        let config = OxiConfig::default();
        
        let result = oxi.process(input, &config).await.unwrap();
        
        if let OxiData::Text(text) = result {
            assert!(text.contains("Processed: test input"));
        } else {
            panic!("Expected text output");
        }
    }
}
```

## Best Practices

1. **Clear Naming**: Choose descriptive names for your Oxis that reflect their purpose.

2. **Documentation**: Include comprehensive documentation for your Oxi, explaining its purpose, inputs, outputs, and configuration options.

3. **Configuration**: Define a clear configuration schema with sensible defaults and descriptive help text.

4. **Error Handling**: Use proper error handling with descriptive error messages.

5. **Testing**: Include comprehensive tests for your Oxi to ensure it works as expected.

6. **Input/Output Contracts**: Clearly document the expected input and output types.

7. **Performance**: Consider performance implications, especially for operations that might process large amounts of data.

8. **Composition**: Design Oxis to be composable with other Oxis in a pipeline.

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

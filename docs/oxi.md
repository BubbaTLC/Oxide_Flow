# Oxi

## Overview

An Oxi is a plugin that extends the functionality of Oxide Flow. It allows for flexible data transformations and integrations, similar to how pipes (`|`) are used in Unix systems. Oxis can handle both input and output data, making them versatile tools for managing data flows.

## Oxi Interface

The Oxi trait defines the interface that all Oxis must implement:

```rust
#[async_trait]
pub trait Oxi {
    /// Get the name of this Oxi
    fn name(&self) -> &str;
    
    /// Get the configuration schema for this Oxi
    fn config_schema(&self) -> serde_yaml::Value;
    
    /// Process data and produce output
    async fn process(&self, 
                     input: types::OxiData, 
                     config: &types::OxiConfig) -> anyhow::Result<types::OxiData>;
                     
    /// Run this Oxi with the given input and configuration
    async fn run(&self, 
                 input: Option<types::OxiData>,
                 config: Option<types::OxiConfig>) -> anyhow::Result<types::OxiData>;
}
```

## Data Types

Oxis can work with multiple data types:

- **Text**: String data for logs, simple text, etc.
- **Structured**: JSON, YAML, or other structured data
- **Binary**: Raw binary data for files, images, etc.
- **Tabular**: Table-like data (e.g., CSV)
- **Empty**: Represents no data (initialization state)

## Configuration

Each Oxi can specify its configuration schema using standard YAML Schema format. The configuration is passed to the Oxi during execution, allowing for customizable behavior.

## Creating an Oxi

To create a new Oxi:

1. Create a new module in `src/oxis/<name>/oxi.rs`
2. Implement the Oxi trait for your struct
3. Add the module to `src/oxis/<name>/mod.rs`
4. Register the module in `src/oxis/mod.rs`

## Example

```rust
use crate::oxis::prelude::*;
use async_trait::async_trait;

pub struct MyOxi;

#[async_trait]
impl Oxi for MyOxi {
    fn name(&self) -> &str {
        "my_oxi"
    }
    
    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
            type: object
            properties:
              option:
                type: string
                description: "An example option"
                default: "default value"
        "#).unwrap()
    }
    
    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Process the input and return the output
        let text = input.as_text()?;
        let option = config.get_string_or("option", "default value");
        
        let output = format!("{} - {}", text, option);
        
        Ok(OxiData::Text(output))
    }
}
```
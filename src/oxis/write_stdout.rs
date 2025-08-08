use crate::oxis::prelude::*;

pub struct WriteStdOut;

#[async_trait]
impl Oxi for WriteStdOut {
    fn name(&self) -> &str {
        "write_stdout"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
            type: object
            properties:
              format:
                type: string
                enum: [auto, text, json, yaml]
                description: "Output format"
                default: auto
        "#,
        )
        .unwrap()
    }

    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: Some(100_000), // Can handle large batches for output
            max_memory_mb: Some(512),      // 512MB for large output formatting
            max_processing_time_ms: Some(10_000), // 10 second timeout
            supported_input_types: vec![
                OxiDataType::Json,
                OxiDataType::Text,
                OxiDataType::Binary,
                OxiDataType::Empty,
            ],
        }
    }

    fn validate_input(&self, input: &OxiData) -> Result<(), OxiError> {
        // write_stdout can handle any input type, so validation is minimal
        match &input.data {
            Data::Binary(data) if data.len() > 100 * 1024 * 1024 => {
                // Warn about very large binary data (>100MB)
                Err(OxiError::ValidationError {
                    details: format!("Binary data is very large ({} bytes). Consider using binary: false or streaming output.", data.len()),
                })
            }
            _ => Ok(()),
        }
    }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Passthrough
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        let format = config.get_string_or("format", "auto");

        match format.as_str() {
            "text" => {
                let text = input
                    .data()
                    .as_text()
                    .map_err(|e| OxiError::ValidationError {
                        details: format!("Failed to get text data: {e}"),
                    })?;
                println!("{text}");
            }
            "json" => {
                let value = input
                    .data()
                    .as_json()
                    .map_err(|e| OxiError::ValidationError {
                        details: format!("Failed to get JSON data: {e}"),
                    })?;
                let json = serde_json::to_string_pretty(&value).map_err(|e| {
                    OxiError::ValidationError {
                        details: format!("Failed to serialize JSON: {e}"),
                    }
                })?;
                println!("{json}");
            }
            _ => {
                // Auto-detect based on input type
                match &input.data {
                    Data::Text(text) => println!("{text}"),
                    Data::Json(value) => {
                        let json = serde_json::to_string_pretty(&value).map_err(|e| {
                            OxiError::ValidationError {
                                details: format!("Failed to serialize JSON: {e}"),
                            }
                        })?;
                        println!("{json}");
                    }
                    Data::Binary(data) => {
                        println!("<Binary data: {} bytes>", data.len());
                    }
                    Data::Empty => {}
                }
            }
        }

        // Return the input data unchanged (passthrough schema strategy)
        Ok(input)
    }
}

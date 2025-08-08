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
        match input {
            OxiData::Binary(data) if data.len() > 100 * 1024 * 1024 => {
                // Warn about very large binary data (>100MB)
                Err(OxiError::ValidationError {
                    details: format!("Binary data is very large ({} bytes). Consider using binary: false or streaming output.", data.len()),
                })
            }
            _ => Ok(()),
        }
    }

    async fn process_data(&self, data: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        let format = config.get_string_or("format", "auto");

        match format.as_str() {
            "text" => {
                let text = data.as_text()?;
                println!("{text}");
            }
            "json" => {
                let value = data.as_json()?;
                let json = serde_json::to_string_pretty(&value)?;
                println!("{json}");
            }
            _ => {
                // Auto-detect based on input type
                match &data {
                    OxiData::Text(text) => println!("{text}"),
                    OxiData::Json(value) => {
                        let json = serde_json::to_string_pretty(&value)?;
                        println!("{json}");
                    }
                    OxiData::Binary(data) => {
                        println!("<Binary data: {} bytes>", data.len());
                    }
                    OxiData::Empty => {}
                }
            }
        }

        // Return the input data unchanged
        Ok(data)
    }
}

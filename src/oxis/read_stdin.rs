use crate::oxis::prelude::*;

pub struct ReadStdIn;

#[async_trait]
impl Oxi for ReadStdIn {
    fn name(&self) -> &str {
        "read_stdin"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
            type: object
            properties:
              binary:
                type: boolean
                description: "Whether to read input as binary"
                default: false
        "#,
        )
        .unwrap()
    }

    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: None,                 // stdin is single input, no batching
            max_memory_mb: Some(64),              // Limit stdin reads to 64MB
            max_processing_time_ms: Some(30_000), // 30 second timeout for reading
            supported_input_types: vec![
                OxiDataType::Empty, // Typically starts with empty input
            ],
        }
    }

    fn validate_input(&self, input: &OxiData) -> Result<(), OxiError> {
        // read_stdin typically starts the pipeline, so should accept empty input
        match input {
            OxiData::Empty => Ok(()),
            _ => Err(OxiError::TypeMismatch {
                expected: "Empty (stdin reader starts pipeline)".to_string(),
                actual: input.data_type().to_string(),
                step: self.name().to_string(),
            }),
        }
    }

    async fn process_data(&self, _data: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        use tokio::io::{self, AsyncReadExt};

        let is_binary = config.get_bool_or("binary", false);

        if is_binary {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer).await?;
            Ok(OxiData::Binary(buffer))
        } else {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).await?;
            Ok(OxiData::Text(buffer))
        }
    }
}

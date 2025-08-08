pub mod cli;
pub mod config;
pub mod config_resolver;
pub mod error;
pub mod oxis;
pub mod pipeline;
pub mod pipeline_manager;
pub mod project;
pub mod schema;
pub mod types;

use async_trait::async_trait;

/// The Oxi trait defines the interface for all Oxide Flow plugins.
/// Each Oxi can process data and pass it to the next Oxi in a chain.
#[async_trait]
pub trait Oxi {
    /// Get the name of this Oxi
    fn name(&self) -> &str;

    /// Get the configuration schema for this Oxi
    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    }

    /// Core processing logic - implement this in your Oxi
    async fn process_data(
        &self,
        data: types::OxiData,
        config: &types::OxiConfig,
    ) -> anyhow::Result<types::OxiData>;

    /// Optional: Set processing limits (memory, batch size, etc.)
    fn processing_limits(&self) -> types::ProcessingLimits {
        types::ProcessingLimits::default()
    }

    /// Optional: Validate input data before processing
    fn validate_input(&self, _input: &types::OxiData) -> Result<(), error::OxiError> {
        Ok(()) // Default: accept all inputs
    }

    /// Determine output schema given input schema and configuration
    fn output_schema(
        &self,
        input_schema: Option<&types::OxiSchema>,
        _config: &types::OxiConfig,
    ) -> anyhow::Result<types::OxiSchema> {
        // Default: pass through input schema or return empty schema
        Ok(input_schema
            .cloned()
            .unwrap_or_else(types::OxiSchema::empty))
    }

    /// Single process method that handles both data and optional schema
    async fn process(
        &self,
        input: types::OxiDataWithSchema,
        config: &types::OxiConfig,
    ) -> anyhow::Result<types::OxiDataWithSchema> {
        // Apply processing limits and validation
        let limits = self.processing_limits();

        // Validate input type
        let input_type = input.data.get_data_type();
        if !limits.supported_input_types.is_empty()
            && !limits.supported_input_types.contains(&input_type)
        {
            return Err(anyhow::anyhow!(
                "This Oxi does not support {} data",
                input_type
            ));
        }

        // Apply custom input validation
        self.validate_input(&input.data)?;

        // Check memory limits
        let estimated_memory = input.data.estimated_memory_usage();
        if let Some(max_memory) = limits.max_memory_mb {
            if estimated_memory > max_memory * 1024 * 1024 {
                return Err(anyhow::anyhow!(
                    "Memory limit exceeded: {} bytes exceeds {}MB",
                    estimated_memory,
                    max_memory
                ));
            }
        }

        // Check batch size limits
        if let Some(max_batch_size) = limits.max_batch_size {
            if input.data.is_batch() && input.data.batch_size() > max_batch_size {
                return Err(anyhow::anyhow!(
                    "Batch size limit exceeded: {} exceeds limit of {}",
                    input.data.batch_size(),
                    max_batch_size
                ));
            }
        }

        // Validate input data if schema is present
        if let Some(schema) = &input.schema {
            schema
                .validate_data(&input.data)
                .map_err(anyhow::Error::from)?;
        }

        // Process the actual data
        let output_data = self.process_data(input.data, config).await?;

        // Calculate output schema
        let output_schema = self.output_schema(input.schema.as_ref(), config)?;

        Ok(types::OxiDataWithSchema::new(output_data, output_schema))
    }
}

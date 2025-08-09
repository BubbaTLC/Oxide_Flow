pub mod cli;
pub mod config;
pub mod config_resolver;
pub mod error;
pub mod oxis;
pub mod pipeline;
pub mod pipeline_manager;
pub mod project;
pub mod schema;
pub mod state;
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
    /// All Oxis now process schema-aware OxiData
    async fn process(
        &self,
        input: types::OxiData,
        config: &types::OxiConfig,
    ) -> Result<types::OxiData, error::OxiError>;

    /// Declare how this Oxi handles schemas
    fn schema_strategy(&self) -> types::SchemaStrategy;

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
}

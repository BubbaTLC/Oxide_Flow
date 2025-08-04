pub mod cli;
pub mod config;
pub mod config_resolver;
pub mod error;
pub mod oxis;
pub mod pipeline;
pub mod project;
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

    /// Process data and produce output
    async fn process(
        &self,
        input: types::OxiData,
        config: &types::OxiConfig,
    ) -> anyhow::Result<types::OxiData>;

    /// Run this Oxi with the given input and configuration
    async fn run(
        &self,
        input: Option<types::OxiData>,
        config: Option<types::OxiConfig>,
    ) -> anyhow::Result<types::OxiData> {
        let input = input.unwrap_or_else(types::OxiData::empty);
        let config = config.unwrap_or_else(types::OxiConfig::default);
        self.process(input, &config).await
    }
}

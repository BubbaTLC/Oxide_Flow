use crate::types::OxiConfig;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Configuration errors that can occur during loading and validation
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
}

/// Schema for an individual Oxi configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxiConfigSchema {
    /// Description of the Oxi
    pub description: Option<String>,

    /// Properties this Oxi accepts
    pub properties: HashMap<String, PropertySchema>,

    /// Required properties
    #[serde(default)]
    pub required: Vec<String>,
}

/// Schema for a property in an Oxi configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// Type of the property
    #[serde(rename = "type")]
    pub property_type: String,

    /// Description of the property
    pub description: Option<String>,

    /// Default value if not specified
    pub default: Option<serde_yaml::Value>,

    /// For enum types, the allowed values
    pub enum_values: Option<Vec<String>>,

    /// For nested objects, their properties
    pub properties: Option<HashMap<String, PropertySchema>>,
}

/// Global configuration for Oxide Flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Version of the configuration format
    pub version: String,

    /// Global settings that apply to all Oxis
    #[serde(default)]
    pub global: GlobalConfig,

    /// Pipeline definitions
    #[serde(default)]
    pub pipelines: HashMap<String, PipelineConfig>,

    /// Default Oxi configurations
    #[serde(default)]
    pub defaults: HashMap<String, serde_yaml::Value>,
}

/// Global configuration settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,

    /// Base directory for relative paths
    #[serde(default)]
    pub base_dir: Option<String>,

    /// Environment variables to include
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Pipeline name
    pub name: String,

    /// Pipeline description
    pub description: Option<String>,

    /// List of Oxis to execute in order
    pub oxis: Vec<OxiInstanceConfig>,
}

/// Configuration for a specific Oxi instance in a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxiInstanceConfig {
    /// Oxi type/name to use
    pub oxi: String,

    /// Configuration for this Oxi
    #[serde(default)]
    pub config: serde_yaml::Value,

    /// Alias for this Oxi instance (optional)
    pub alias: Option<String>,

    /// Conditionals for this Oxi
    #[serde(default)]
    pub when: Option<String>,
}

impl Config {
    /// Load configuration from a YAML file
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;

        // Parse YAML
        let mut config: Config = serde_yaml::from_str(&content)?;

        // Process includes
        config = process_includes(config, Path::new(path))?;

        // Process environment variables and other dynamic references
        config = process_config(config)?;

        // Validate configuration
        validate_config(&config)?;

        Ok(config)
    }

    /// Get a specific pipeline by name
    pub fn get_pipeline(&self, name: &str) -> Option<&PipelineConfig> {
        self.pipelines.get(name)
    }

    /// Get default configuration for an Oxi
    pub fn get_oxi_defaults(&self, oxi_name: &str) -> Option<&serde_yaml::Value> {
        self.defaults.get(oxi_name)
    }

    /// Merge default and instance configurations for an Oxi
    pub fn merge_oxi_config(
        &self,
        oxi_name: &str,
        instance_config: &serde_yaml::Value,
    ) -> serde_yaml::Value {
        if let Some(defaults) = self.get_oxi_defaults(oxi_name) {
            merge_yaml_values(defaults, instance_config)
        } else {
            instance_config.clone()
        }
    }
}

/// Substitute environment variables in the configuration
/// Environment variables are specified as ${ENV_VAR} or ${ENV_VAR:-default}
/// Also supports references to other pipeline step outputs using ${step_id.property.path}
pub fn substitute_env_vars(content: &str) -> Result<String, ConfigError> {
    let mut result = String::with_capacity(content.len());

    // Use regex to better handle complex variable references
    let re = Regex::new(r"\$\{([^{}]+?)(:-([^{}]+))?\}").unwrap();
    let mut last_end = 0;

    for cap in re.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let var_expr = cap.get(1).unwrap().as_str();
        let default_value = cap.get(3).map(|m| m.as_str());

        // Add text before this match
        result.push_str(&content[last_end..full_match.start()]);
        last_end = full_match.end();

        // Handle different reference types
        if var_expr.contains('.') {
            // This is likely a reference to a pipeline step output
            // For now, just keep it as is (will be resolved later in the pipeline execution)
            result.push_str(full_match.as_str());
        } else {
            // This is an environment variable reference
            let value = match env::var(var_expr) {
                Ok(val) => val,
                Err(_) => {
                    if let Some(default) = default_value {
                        default.to_string()
                    } else {
                        return Err(ConfigError::EnvVarNotFound(var_expr.to_string()));
                    }
                }
            };

            result.push_str(&value);
        }
    }

    // Add any remaining text after the last match
    result.push_str(&content[last_end..]);
    Ok(result)
}

/// Process any included configurations
fn process_includes(config: Config, _base_path: &Path) -> Result<Config, ConfigError> {
    // Implementation depends on your include mechanism
    // This is a placeholder for the include processing logic

    Ok(config)
}

/// Process a loaded configuration by applying environment variable substitution
/// and resolving other dynamic references
pub fn process_config(mut config: Config) -> Result<Config, ConfigError> {
    // Apply environment variable substitution to the entire config
    let config_value = serde_yaml::to_value(&config).map_err(ConfigError::YamlError)?;
    let mut processed_value = config_value;
    process_env_vars_in_yaml(&mut processed_value)?;

    // Deserialize back to Config
    config = serde_yaml::from_value(processed_value).map_err(ConfigError::YamlError)?;

    Ok(config)
}

/// Validate the configuration
fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Check version
    if config.version != "1.0" {
        return Err(ConfigError::ValidationError(format!(
            "Unsupported configuration version: {}",
            config.version
        )));
    }

    // Validate pipelines
    for (name, pipeline) in &config.pipelines {
        // Check that each Oxi in the pipeline exists
        for oxi_instance in &pipeline.oxis {
            // Here you would check that the Oxi exists and its configuration is valid
            // This would typically involve loading the Oxi's schema and validating against it
            // For now we'll just do a basic check
            if oxi_instance.oxi.is_empty() {
                return Err(ConfigError::ValidationError(format!(
                    "Oxi name is empty in pipeline '{name}'"
                )));
            }
        }
    }

    Ok(())
}

/// Merge two YAML values, with the right value taking precedence
fn merge_yaml_values(base: &serde_yaml::Value, overlay: &serde_yaml::Value) -> serde_yaml::Value {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(base_map), serde_yaml::Value::Mapping(overlay_map)) => {
            let mut result = base_map.clone();

            for (key, value) in overlay_map {
                if let Some(base_value) = base_map.get(key) {
                    // If both are mappings, merge recursively
                    if base_value.is_mapping() && value.is_mapping() {
                        result.insert(key.clone(), merge_yaml_values(base_value, value));
                    } else {
                        // Otherwise, overlay value takes precedence
                        result.insert(key.clone(), value.clone());
                    }
                } else {
                    // Key doesn't exist in base, add it
                    result.insert(key.clone(), value.clone());
                }
            }

            serde_yaml::Value::Mapping(result)
        }
        // For non-mapping values, overlay takes precedence
        (_, overlay_value) => overlay_value.clone(),
    }
}

/// Process a YAML value recursively to substitute environment variables
pub fn process_env_vars_in_yaml(value: &mut serde_yaml::Value) -> Result<(), ConfigError> {
    match value {
        serde_yaml::Value::String(s) => {
            *s = substitute_env_vars(s)?;
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq.iter_mut() {
                process_env_vars_in_yaml(item)?;
            }
        }
        serde_yaml::Value::Mapping(map) => {
            for (_, val) in map.iter_mut() {
                process_env_vars_in_yaml(val)?;
            }
        }
        _ => {} // Other types (numbers, booleans, null) don't need substitution
    }
    Ok(())
}

/// Context for resolving step references during pipeline execution
#[derive(Debug, Clone)]
pub struct PipelineContext {
    /// Step outputs indexed by step alias
    pub step_outputs: HashMap<String, serde_yaml::Value>,

    /// Step metadata indexed by step alias
    pub step_metadata: HashMap<String, HashMap<String, serde_yaml::Value>>,
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineContext {
    /// Create a new empty pipeline context
    pub fn new() -> Self {
        PipelineContext {
            step_outputs: HashMap::new(),
            step_metadata: HashMap::new(),
        }
    }

    /// Add output from a step
    pub fn add_step_output(&mut self, alias: &str, output: serde_yaml::Value) {
        self.step_outputs.insert(alias.to_string(), output);
    }

    /// Add metadata from a step
    pub fn add_step_metadata(&mut self, alias: &str, metadata: HashMap<String, serde_yaml::Value>) {
        self.step_metadata.insert(alias.to_string(), metadata);
    }

    /// Resolve step references in a string (e.g., ${reader.output.path})
    pub fn resolve_step_references(&self, input: &str) -> Result<String, ConfigError> {
        let re = Regex::new(r"\$\{([a-zA-Z0-9_]+)\.([a-zA-Z0-9_.]+)\}").unwrap();
        let mut result = input.to_string();

        // We need to process from right to left to avoid offset issues
        let mut replacements = Vec::new();
        for cap in re.captures_iter(input) {
            let full_match = cap.get(0).unwrap();
            let step_alias = cap.get(1).unwrap().as_str();
            let property_path = cap.get(2).unwrap().as_str();

            let replacement_value = self.resolve_property_path(step_alias, property_path)?;
            replacements.push((full_match.start(), full_match.end(), replacement_value));
        }

        // Apply replacements from right to left to preserve positions
        replacements.reverse();
        for (start, end, replacement) in replacements {
            result.replace_range(start..end, &replacement);
        }

        Ok(result)
    }

    /// Resolve a property path like "output.data.users[0].name"
    fn resolve_property_path(
        &self,
        step_alias: &str,
        property_path: &str,
    ) -> Result<String, ConfigError> {
        let parts: Vec<&str> = property_path.split('.').collect();

        if parts.is_empty() {
            return Err(ConfigError::ValidationError(format!(
                "Invalid property path: {property_path}"
            )));
        }

        // Handle special metadata references
        if parts[0] == "metadata" && parts.len() > 1 {
            // First try step metadata
            if let Some(metadata) = self.step_metadata.get(step_alias) {
                if let Some(value) = metadata.get(parts[1]) {
                    // Navigate through remaining parts if any
                    let mut current_value = Some(value);
                    for part in &parts[2..] {
                        if let Some(value) = current_value {
                            current_value = match value {
                                serde_yaml::Value::Mapping(map) => {
                                    map.get(serde_yaml::Value::String(part.to_string()))
                                }
                                serde_yaml::Value::Sequence(seq) => {
                                    if let Ok(index) = part.parse::<usize>() {
                                        seq.get(index)
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };
                        } else {
                            break;
                        }
                    }

                    return match current_value {
                        Some(serde_yaml::Value::String(s)) => Ok(s.clone()),
                        Some(serde_yaml::Value::Number(n)) => Ok(n.to_string()),
                        Some(serde_yaml::Value::Bool(b)) => Ok(b.to_string()),
                        Some(other) => serde_yaml::to_string(other)
                            .map_err(ConfigError::YamlError)
                            .map(|s| s.trim().to_string()),
                        None => Err(ConfigError::ValidationError(format!(
                            "Property path '{property_path}' not found in step metadata '{step_alias}'"
                        ))),
                    };
                }
            }

            // Fall through to check step output metadata
        }

        // Get the step output
        let step_output = self.step_outputs.get(step_alias).ok_or_else(|| {
            ConfigError::ValidationError(format!("Step '{step_alias}' not found"))
        })?;

        // Start navigation from the step output
        let mut current_value = Some(step_output);

        // Navigate through all the path parts
        for part in &parts {
            if let Some(value) = current_value {
                current_value = match value {
                    serde_yaml::Value::Mapping(map) => {
                        map.get(serde_yaml::Value::String(part.to_string()))
                    }
                    serde_yaml::Value::Sequence(seq) => {
                        // Handle array indexing like "users[0]"
                        if let Ok(index) = part.parse::<usize>() {
                            seq.get(index)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
            } else {
                break;
            }
        }

        // Convert the final value to string
        match current_value {
            Some(serde_yaml::Value::String(s)) => Ok(s.clone()),
            Some(serde_yaml::Value::Number(n)) => Ok(n.to_string()),
            Some(serde_yaml::Value::Bool(b)) => Ok(b.to_string()),
            Some(other) => {
                // For complex objects, serialize to YAML string
                serde_yaml::to_string(other)
                    .map_err(ConfigError::YamlError)
                    .map(|s| s.trim().to_string())
            }
            None => Err(ConfigError::ValidationError(format!(
                "Property path '{property_path}' not found in step '{step_alias}'"
            ))),
        }
    }

    /// Resolve config references in an OxiConfig
    pub fn resolve_config_references(&self, config: &OxiConfig) -> Result<OxiConfig, ConfigError> {
        let mut resolved_config = config.clone();

        // Process each value in the config
        for (_key, value) in resolved_config.values.iter_mut() {
            if let serde_yaml::Value::String(s) = value {
                let resolved_string = self.resolve_step_references(s)?;
                *value = serde_yaml::Value::String(resolved_string);
            }
        }

        Ok(resolved_config)
    }
}

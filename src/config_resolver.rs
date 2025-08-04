use crate::types::OxiData;
use regex::Regex;
use std::collections::HashMap;
use std::env;

/// Resolves dynamic references in configuration values
pub struct ConfigResolver {
    /// Environment variables cache
    env_vars: HashMap<String, String>,

    /// Step outputs from previous pipeline steps
    step_outputs: HashMap<String, OxiData>,
}

impl ConfigResolver {
    /// Create a new ConfigResolver
    pub fn new() -> Self {
        Self {
            env_vars: HashMap::new(),
            step_outputs: HashMap::new(),
        }
    }

    /// Add a step output for future reference
    pub fn add_step_output(&mut self, step_id: String, output: OxiData) {
        self.step_outputs.insert(step_id, output);
    }

    /// Resolve all dynamic references in a configuration value
    pub fn resolve_value(&self, value: &serde_yaml::Value) -> anyhow::Result<serde_yaml::Value> {
        match value {
            serde_yaml::Value::String(s) => {
                let resolved = self.resolve_string_references(s)?;
                Ok(serde_yaml::Value::String(resolved))
            }
            serde_yaml::Value::Mapping(map) => {
                let mut resolved_map = serde_yaml::Mapping::new();
                for (key, val) in map {
                    let resolved_val = self.resolve_value(val)?;
                    resolved_map.insert(key.clone(), resolved_val);
                }
                Ok(serde_yaml::Value::Mapping(resolved_map))
            }
            serde_yaml::Value::Sequence(seq) => {
                let mut resolved_seq = Vec::new();
                for item in seq {
                    resolved_seq.push(self.resolve_value(item)?);
                }
                Ok(serde_yaml::Value::Sequence(resolved_seq))
            }
            // For other types (Number, Bool, Null), return as-is
            _ => Ok(value.clone()),
        }
    }

    /// Resolve string references like ${ENV_VAR} and ${step.metadata.path}
    fn resolve_string_references(&self, text: &str) -> anyhow::Result<String> {
        let mut result = text.to_string();

        // Environment variable substitution: ${ENV_VAR}
        result = self.resolve_env_vars(&result)?;

        // Step reference substitution: ${step_id.field.path}
        result = self.resolve_step_references(&result)?;

        Ok(result)
    }

    /// Resolve environment variable references
    fn resolve_env_vars(&self, text: &str) -> anyhow::Result<String> {
        // Support both ${VAR} and ${VAR:-default} syntax
        let env_regex = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)(?::(-)?([^}]*))?\}").unwrap();
        let mut result = text.to_string();

        for cap in env_regex.captures_iter(text) {
            let full_match = &cap[0];
            let var_name = &cap[1];
            let has_default = cap.get(2).is_some();
            let default_value = cap.get(3).map(|m| m.as_str()).unwrap_or("");

            // Try to get from cache first, then from environment
            let value = if let Some(cached_value) = self.env_vars.get(var_name) {
                cached_value.clone()
            } else {
                match env::var(var_name) {
                    Ok(val) => val,
                    Err(_) => {
                        if has_default {
                            default_value.to_string()
                        } else {
                            return Err(anyhow::anyhow!(
                                "Environment variable '{}' not found",
                                var_name
                            ));
                        }
                    }
                }
            };

            result = result.replace(full_match, &value);
        }

        Ok(result)
    }

    /// Resolve step output references like ${reader.metadata.path}
    fn resolve_step_references(&self, text: &str) -> anyhow::Result<String> {
        let step_regex =
            Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)(\.([a-zA-Z0-9_.]+))?\}").unwrap();
        let mut result = text.to_string();

        for cap in step_regex.captures_iter(text) {
            let full_match = &cap[0];
            let step_id = &cap[1];
            let field_path = cap.get(3).map(|m| m.as_str());

            // Get step output
            let step_output = self
                .step_outputs
                .get(step_id)
                .ok_or_else(|| anyhow::anyhow!("Step '{}' output not found", step_id))?;

            // Extract the value based on field path
            let value = if let Some(path) = field_path {
                self.extract_field_from_output(step_output, path)?
            } else {
                // If no field path, convert the entire output to string
                step_output.to_text()?
            };

            result = result.replace(full_match, &value);
        }

        Ok(result)
    }

    /// Extract a specific field from step output using dot notation
    fn extract_field_from_output(
        &self,
        output: &OxiData,
        field_path: &str,
    ) -> anyhow::Result<String> {
        match output {
            OxiData::Json(json_value) => {
                let fields: Vec<&str> = field_path.split('.').collect();
                let mut current = json_value;

                for field in fields {
                    current = current.get(field).ok_or_else(|| {
                        anyhow::anyhow!("Field '{}' not found in JSON output", field)
                    })?;
                }

                // Convert final value to string
                match current {
                    serde_json::Value::String(s) => Ok(s.clone()),
                    serde_json::Value::Number(n) => Ok(n.to_string()),
                    serde_json::Value::Bool(b) => Ok(b.to_string()),
                    _ => Ok(current.to_string()),
                }
            }
            OxiData::Text(_) => {
                // For text data, we could implement simple field extraction
                // For now, return an error suggesting JSON format
                anyhow::bail!(
                    "Field extraction from text data not supported. Use JSON output format."
                )
            }
            _ => {
                anyhow::bail!("Field extraction not supported for this data type")
            }
        }
    }

    /// Cache an environment variable
    pub fn cache_env_var(&mut self, name: String, value: String) {
        self.env_vars.insert(name, value);
    }

    /// Load common environment variables into cache
    pub fn load_common_env_vars(&mut self) {
        let common_vars = [
            "HOME",
            "PATH",
            "USER",
            "PWD",
            "SHELL",
            "LOG_LEVEL",
            "OUTPUT_FORMAT",
            "DEBUG",
        ];

        for var in &common_vars {
            if let Ok(value) = env::var(var) {
                self.env_vars.insert(var.to_string(), value);
            }
        }
    }
}

impl Default for ConfigResolver {
    fn default() -> Self {
        let mut resolver = Self::new();
        resolver.load_common_env_vars();
        resolver
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_var_substitution() {
        env::set_var("TEST_VAR", "test_value");

        let resolver = ConfigResolver::default();
        let result = resolver
            .resolve_string_references("Path: ${TEST_VAR}/data")
            .unwrap();
        assert_eq!(result, "Path: test_value/data");

        // Test with default value
        let result = resolver
            .resolve_string_references("Path: ${MISSING_VAR:-default_value}/data")
            .unwrap();
        assert_eq!(result, "Path: default_value/data");

        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_step_reference_substitution() {
        let mut resolver = ConfigResolver::new();

        // Add a JSON step output
        let json_data = serde_json::json!({
            "metadata": {
                "path": "/some/file.json",
                "size": 1024
            }
        });
        resolver.add_step_output("reader".to_string(), OxiData::Json(json_data));

        let result = resolver
            .resolve_string_references("Output: ${reader.metadata.path}")
            .unwrap();
        assert_eq!(result, "Output: /some/file.json");

        let result = resolver
            .resolve_string_references("Size: ${reader.metadata.size}")
            .unwrap();
        assert_eq!(result, "Size: 1024");
    }

    #[test]
    fn test_nested_config_resolution() {
        env::set_var("BASE_PATH", "/data");

        let resolver = ConfigResolver::default();

        let config = serde_yaml::from_str(
            r#"
            input:
              path: "${BASE_PATH}/input.json"
              options:
                format: "json"
                encoding: "utf-8"
        "#,
        )
        .unwrap();

        let resolved = resolver.resolve_value(&config).unwrap();

        if let serde_yaml::Value::Mapping(map) = &resolved {
            if let Some(serde_yaml::Value::Mapping(input)) = map.get("input") {
                if let Some(serde_yaml::Value::String(path)) = input.get("path") {
                    assert_eq!(path, "/data/input.json");
                }
            }
        }

        env::remove_var("BASE_PATH");
    }
}

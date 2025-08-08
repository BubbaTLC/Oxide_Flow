use crate::types::OxiConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Schema validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required property: {property}")]
    MissingProperty { property: String },

    #[error("Invalid property type for '{property}': expected {expected}, got {actual}")]
    InvalidType {
        property: String,
        expected: String,
        actual: String,
    },

    #[error("Invalid property value for '{property}': {message}")]
    InvalidValue { property: String, message: String },

    #[error("Unknown property: {property}")]
    UnknownProperty { property: String },

    #[error("Schema validation failed: {message}")]
    ValidationFailed { message: String },
}

/// Schema for validating Oxi configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxiSchema {
    /// Type of the schema (always "object" for Oxi configs)
    #[serde(rename = "type")]
    pub schema_type: String,

    /// Description of the Oxi
    pub description: Option<String>,

    /// Properties this Oxi accepts
    pub properties: HashMap<String, PropertySchema>,

    /// Required properties
    #[serde(default)]
    pub required: Vec<String>,

    /// Additional properties allowed
    #[serde(default = "default_additional_properties")]
    pub additional_properties: bool,
}

fn default_additional_properties() -> bool {
    false
}

/// Schema for a single property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// Type of the property
    #[serde(rename = "type")]
    pub property_type: String,

    /// Description of the property
    pub description: Option<String>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_yaml::Value>,

    /// Enum values (for string types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Minimum value (for numeric types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    /// Maximum value (for numeric types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    /// Pattern (for string types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl OxiSchema {
    /// Create a new schema from YAML
    pub fn from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ValidationError> {
        serde_yaml::from_value(yaml.clone()).map_err(|e| ValidationError::ValidationFailed {
            message: format!("Failed to parse schema: {e}"),
        })
    }

    /// Validate an OxiConfig against this schema
    pub fn validate(&self, config: &OxiConfig) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check required properties
        for required_prop in &self.required {
            if !config.values.contains_key(required_prop) {
                errors.push(ValidationError::MissingProperty {
                    property: required_prop.clone(),
                });
            }
        }

        // Validate each property in the config
        for (key, value) in &config.values {
            if let Some(prop_schema) = self.properties.get(key) {
                if let Err(prop_errors) = self.validate_property(key, value, prop_schema) {
                    errors.extend(prop_errors);
                }
            } else if !self.additional_properties {
                errors.push(ValidationError::UnknownProperty {
                    property: key.clone(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate a single property
    fn validate_property(
        &self,
        property_name: &str,
        value: &serde_yaml::Value,
        schema: &PropertySchema,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Type validation
        let actual_type = self.get_yaml_type(value);
        if actual_type != schema.property_type && schema.property_type != "any" {
            errors.push(ValidationError::InvalidType {
                property: property_name.to_string(),
                expected: schema.property_type.clone(),
                actual: actual_type,
            });
            return Err(errors);
        }

        // Value-specific validations
        match schema.property_type.as_str() {
            "string" => {
                if let Some(serde_yaml::Value::String(_s)) = value
                    .as_str()
                    .map(|s| serde_yaml::Value::String(s.to_string()))
                {
                    // Enum validation
                    if let Some(enum_values) = &schema.enum_values {
                        if let Some(string_val) = value.as_str() {
                            if !enum_values.contains(&string_val.to_string()) {
                                errors.push(ValidationError::InvalidValue {
                                    property: property_name.to_string(),
                                    message: format!("Must be one of: {}", enum_values.join(", ")),
                                });
                            }
                        }
                    }

                    // Pattern validation
                    if let Some(pattern) = &schema.pattern {
                        if let Some(string_val) = value.as_str() {
                            if let Ok(regex) = regex::Regex::new(pattern) {
                                if !regex.is_match(string_val) {
                                    errors.push(ValidationError::InvalidValue {
                                        property: property_name.to_string(),
                                        message: format!("Must match pattern: {pattern}"),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            "number" | "integer" => {
                if let Some(num) = value.as_f64() {
                    // Minimum validation
                    if let Some(min) = schema.minimum {
                        if num < min {
                            errors.push(ValidationError::InvalidValue {
                                property: property_name.to_string(),
                                message: format!("Must be >= {min}"),
                            });
                        }
                    }

                    // Maximum validation
                    if let Some(max) = schema.maximum {
                        if num > max {
                            errors.push(ValidationError::InvalidValue {
                                property: property_name.to_string(),
                                message: format!("Must be <= {max}"),
                            });
                        }
                    }
                }
            }
            _ => {} // Other types don't need special validation yet
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the YAML type name for a value
    fn get_yaml_type(&self, value: &serde_yaml::Value) -> String {
        match value {
            serde_yaml::Value::String(_) => "string".to_string(),
            serde_yaml::Value::Number(_) => "number".to_string(),
            serde_yaml::Value::Bool(_) => "boolean".to_string(),
            serde_yaml::Value::Sequence(_) => "array".to_string(),
            serde_yaml::Value::Mapping(_) => "object".to_string(),
            serde_yaml::Value::Null => "null".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

/// Registry of Oxi schemas for validation
pub struct SchemaRegistry {
    schemas: HashMap<String, OxiSchema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Register a schema for an Oxi
    pub fn register(&mut self, oxi_name: String, schema: OxiSchema) {
        self.schemas.insert(oxi_name, schema);
    }

    /// Get schema for an Oxi
    pub fn get_schema(&self, oxi_name: &str) -> Option<&OxiSchema> {
        self.schemas.get(oxi_name)
    }

    /// Validate a config against the registered schema
    pub fn validate(&self, oxi_name: &str, config: &OxiConfig) -> Result<(), Vec<ValidationError>> {
        if let Some(schema) = self.get_schema(oxi_name) {
            schema.validate(config)
        } else {
            // If no schema registered, allow any configuration
            Ok(())
        }
    }

    /// Load built-in schemas for core Oxis
    pub fn with_builtin_schemas() -> Self {
        let mut registry = Self::new();
        registry.load_builtin_schemas();
        registry
    }

    fn load_builtin_schemas(&mut self) {
        // ReadFile schema
        let read_file_schema = OxiSchema {
            schema_type: "object".to_string(),
            description: Some("Read content from a file".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert(
                    "path".to_string(),
                    PropertySchema {
                        property_type: "string".to_string(),
                        description: Some("Path to the file to read".to_string()),
                        default: None,
                        enum_values: None,
                        minimum: None,
                        maximum: None,
                        pattern: None,
                    },
                );
                props.insert(
                    "encoding".to_string(),
                    PropertySchema {
                        property_type: "string".to_string(),
                        description: Some("File encoding".to_string()),
                        default: Some(serde_yaml::Value::String("utf-8".to_string())),
                        enum_values: Some(vec!["utf-8".to_string(), "ascii".to_string()]),
                        minimum: None,
                        maximum: None,
                        pattern: None,
                    },
                );
                props
            },
            required: vec!["path".to_string()],
            additional_properties: false,
        };
        self.register("read_file".to_string(), read_file_schema);

        // WriteFile schema
        let write_file_schema = OxiSchema {
            schema_type: "object".to_string(),
            description: Some("Write content to a file".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert(
                    "path".to_string(),
                    PropertySchema {
                        property_type: "string".to_string(),
                        description: Some("Path to the output file".to_string()),
                        default: None,
                        enum_values: None,
                        minimum: None,
                        maximum: None,
                        pattern: None,
                    },
                );
                props.insert(
                    "create_dirs".to_string(),
                    PropertySchema {
                        property_type: "boolean".to_string(),
                        description: Some(
                            "Create parent directories if they don't exist".to_string(),
                        ),
                        default: Some(serde_yaml::Value::Bool(true)),
                        enum_values: None,
                        minimum: None,
                        maximum: None,
                        pattern: None,
                    },
                );
                props
            },
            required: vec!["path".to_string()],
            additional_properties: false,
        };
        self.register("write_file".to_string(), write_file_schema);

        // FormatCsv schema
        let format_csv_schema = OxiSchema {
            schema_type: "object".to_string(),
            description: Some("Format JSON data as CSV".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert(
                    "delimiter".to_string(),
                    PropertySchema {
                        property_type: "string".to_string(),
                        description: Some("CSV field delimiter".to_string()),
                        default: Some(serde_yaml::Value::String(",".to_string())),
                        enum_values: None,
                        minimum: None,
                        maximum: None,
                        pattern: Some(r"^.{1}$".to_string()), // Single character
                    },
                );
                props.insert(
                    "headers".to_string(),
                    PropertySchema {
                        property_type: "boolean".to_string(),
                        description: Some("Include headers in output".to_string()),
                        default: Some(serde_yaml::Value::Bool(true)),
                        enum_values: None,
                        minimum: None,
                        maximum: None,
                        pattern: None,
                    },
                );
                props
            },
            required: vec![],
            additional_properties: false,
        };
        self.register("format_csv".to_string(), format_csv_schema);
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::with_builtin_schemas()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validation_success() {
        let registry = SchemaRegistry::default();

        let mut config = OxiConfig::default();
        config.values.insert(
            "path".to_string(),
            serde_yaml::Value::String("test.txt".to_string()),
        );

        let result = registry.validate("read_file", &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_schema_validation_missing_required() {
        let registry = SchemaRegistry::default();
        let config = OxiConfig::default(); // Missing required "path"

        let result = registry.validate("read_file", &config);
        assert!(result.is_err());

        if let Err(errors) = result {
            assert_eq!(errors.len(), 1);
            match &errors[0] {
                ValidationError::MissingProperty { property } => {
                    assert_eq!(property, "path");
                }
                _ => panic!("Expected MissingProperty error"),
            }
        }
    }

    #[test]
    fn test_schema_validation_invalid_type() {
        let registry = SchemaRegistry::default();

        let mut config = OxiConfig::default();
        config.values.insert(
            "path".to_string(),
            serde_yaml::Value::Number(serde_yaml::Number::from(42)),
        );

        let result = registry.validate("read_file", &config);
        assert!(result.is_err());
    }
}

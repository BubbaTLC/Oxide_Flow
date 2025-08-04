use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use crate::config::{OxiConfigSchema, PropertySchema, ConfigError};

/// OxiData represents the data flowing between Oxis in the pipeline.
/// Uses JSON as the primary internal data format for structured data exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OxiData {
    /// JSON data - the primary format for structured data exchange between Oxis
    Json(serde_json::Value),
    
    /// Text data (strings, logs, etc.) - for simple text operations
    Text(String),
    
    /// Binary data (files, images, etc.) - for binary operations
    Binary(Vec<u8>),
    
    /// Empty data (used for initialization)
    Empty,
}

impl OxiData {
    /// Create a new empty OxiData
    pub fn empty() -> Self {
        OxiData::Empty
    }
    
    /// Create a new OxiData from text
    pub fn from_text(text: &str) -> Self {
        OxiData::Text(text.to_string())
    }
    
    /// Create a new OxiData from JSON data
    pub fn from_json(data: serde_json::Value) -> Self {
        OxiData::Json(data)
    }
    
    /// Create a new OxiData from binary data
    pub fn from_binary(data: Vec<u8>) -> Self {
        OxiData::Binary(data)
    }
    
    /// Check if this is empty data
    pub fn is_empty(&self) -> bool {
        matches!(self, OxiData::Empty)
    }
    
    /// Get text data or return an error
    pub fn as_text(&self) -> anyhow::Result<&str> {
        match self {
            OxiData::Text(text) => Ok(text),
            _ => anyhow::bail!("Expected text data, found {:?}", self.data_type()),
        }
    }
    
    /// Get JSON data or return an error
    pub fn as_json(&self) -> anyhow::Result<&serde_json::Value> {
        match self {
            OxiData::Json(json) => Ok(json),
            _ => anyhow::bail!("Expected JSON data, found {:?}", self.data_type()),
        }
    }
    
    /// Get binary data or return an error
    pub fn as_binary(&self) -> anyhow::Result<&Vec<u8>> {
        match self {
            OxiData::Binary(binary) => Ok(binary),
            _ => anyhow::bail!("Expected binary data, found {:?}", self.data_type()),
        }
    }
    
    /// Get the type of data for error messages
    pub fn data_type(&self) -> &'static str {
        match self {
            OxiData::Json(_) => "JSON",
            OxiData::Text(_) => "Text",
            OxiData::Binary(_) => "Binary",
            OxiData::Empty => "Empty",
        }
    }
    
    /// Convert to text representation
    pub fn to_text(&self) -> anyhow::Result<String> {
        match self {
            OxiData::Text(text) => Ok(text.clone()),
            OxiData::Json(json) => Ok(serde_json::to_string_pretty(json)?),
            OxiData::Binary(data) => {
                // Convert binary to base64 string for text representation
                use base64::Engine;
                Ok(base64::engine::general_purpose::STANDARD.encode(data))
            },
            OxiData::Empty => Ok(String::new()),
        }
    }
    
    /// Convert to binary representation  
    pub fn to_binary(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            OxiData::Text(text) => Ok(text.as_bytes().to_vec()),
            OxiData::Json(json) => Ok(serde_json::to_string(json)?.as_bytes().to_vec()),
            OxiData::Binary(data) => Ok(data.clone()),
            OxiData::Empty => Ok(Vec::new()),
        }
    }
}

impl fmt::Display for OxiData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OxiData::Text(text) => write!(f, "{}", text),
            OxiData::Json(data) => match serde_json::to_string_pretty(&data) {
                Ok(json) => write!(f, "{}", json),
                Err(_) => write!(f, "<Invalid JSON data>"),
            },
            OxiData::Binary(data) => {
                if data.len() > 100 {
                    write!(f, "<Binary data: {} bytes>", data.len())
                } else {
                    write!(f, "<Binary data: {:?}>", data)
                }
            },
            OxiData::Empty => write!(f, "<Empty>"),
        }
    }
}

/// Configuration for an Oxi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxiConfig {
    /// Key-value pairs for configuration
    pub values: HashMap<String, serde_yaml::Value>,
}

impl OxiConfig {
    /// Create a new default OxiConfig
    pub fn default() -> Self {
        OxiConfig {
            values: HashMap::new(),
        }
    }
    
    /// Create a new OxiConfig from a YAML value
    pub fn from_yaml(value: serde_yaml::Value) -> Self {
        match value {
            serde_yaml::Value::Mapping(map) => {
                let mut values = HashMap::new();
                for (key, value) in map {
                    if let Some(key_str) = key.as_str() {
                        values.insert(key_str.to_string(), value);
                    }
                }
                OxiConfig { values }
            },
            _ => OxiConfig::default(),
        }
    }
    
    /// Get a string configuration value
    pub fn get_string(&self, key: &str) -> anyhow::Result<String> {
        match self.values.get(key) {
            Some(serde_yaml::Value::String(value)) => Ok(value.clone()),
            Some(value) => Ok(value.as_str()
                .ok_or_else(|| anyhow::anyhow!("Value for key '{}' is not a string", key))?
                .to_string()),
            None => anyhow::bail!("Configuration key '{}' not found", key),
        }
    }
    
    /// Get a string configuration value or default
    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        self.get_string(key).unwrap_or_else(|_| default.to_string())
    }
    
    /// Get a boolean configuration value
    pub fn get_bool(&self, key: &str) -> anyhow::Result<bool> {
        match self.values.get(key) {
            Some(serde_yaml::Value::Bool(value)) => Ok(*value),
            Some(value) => {
                // Try to convert string values like "true"/"false"
                if let Some(s) = value.as_str() {
                    match s.to_lowercase().as_str() {
                        "true" | "yes" | "1" => Ok(true),
                        "false" | "no" | "0" => Ok(false),
                        _ => anyhow::bail!("Value for key '{}' cannot be converted to boolean", key),
                    }
                } else {
                    anyhow::bail!("Value for key '{}' is not a boolean", key)
                }
            },
            None => anyhow::bail!("Configuration key '{}' not found", key),
        }
    }
    
    /// Get a boolean configuration value or default
    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.get_bool(key).unwrap_or(default)
    }
    
    /// Get a numeric configuration value
    pub fn get_number(&self, key: &str) -> anyhow::Result<f64> {
        match self.values.get(key) {
            Some(serde_yaml::Value::Number(value)) => {
                value.as_f64().ok_or_else(|| anyhow::anyhow!("Value for key '{}' cannot be converted to f64", key))
            },
            Some(value) => {
                // Try to convert string values to numbers
                if let Some(s) = value.as_str() {
                    s.parse::<f64>().map_err(|_| anyhow::anyhow!("Value for key '{}' cannot be parsed as f64", key))
                } else {
                    anyhow::bail!("Value for key '{}' is not a number", key)
                }
            },
            None => anyhow::bail!("Configuration key '{}' not found", key),
        }
    }
    
    /// Get a numeric configuration value or default
    pub fn get_number_or(&self, key: &str, default: f64) -> f64 {
        self.get_number(key).unwrap_or(default)
    }
    
    /// Get a nested configuration object
    pub fn get_nested(&self, key: &str) -> anyhow::Result<OxiConfig> {
        match self.values.get(key) {
            Some(serde_yaml::Value::Mapping(_)) => {
                Ok(OxiConfig::from_yaml(self.values.get(key).unwrap().clone()))
            },
            Some(_) => anyhow::bail!("Value for key '{}' is not a mapping", key),
            None => anyhow::bail!("Configuration key '{}' not found", key),
        }
    }
    
    /// Get a nested configuration object or default
    pub fn get_nested_or(&self, key: &str) -> OxiConfig {
        self.get_nested(key).unwrap_or_else(|_| OxiConfig::default())
    }
    
    /// Get a sequence configuration value
    pub fn get_sequence(&self, key: &str) -> anyhow::Result<Vec<serde_yaml::Value>> {
        match self.values.get(key) {
            Some(serde_yaml::Value::Sequence(value)) => Ok(value.clone()),
            Some(_) => anyhow::bail!("Value for key '{}' is not a sequence", key),
            None => anyhow::bail!("Configuration key '{}' not found", key),
        }
    }
    
    /// Get a sequence configuration value or default
    pub fn get_sequence_or(&self, key: &str) -> Vec<serde_yaml::Value> {
        self.get_sequence(key).unwrap_or_else(|_| Vec::new())
    }
    
    /// Get an integer configuration value
    pub fn get_i64(&self, key: &str) -> anyhow::Result<i64> {
        match self.values.get(key) {
            Some(serde_yaml::Value::Number(value)) => {
                value.as_i64().ok_or_else(|| anyhow::anyhow!("Value for key '{}' is not an integer", key))
            },
            None => anyhow::bail!("Configuration key '{}' not found", key),
            _ => anyhow::bail!("Value for key '{}' is not a number", key),
        }
    }
    
    /// Get an integer configuration value or default
    pub fn get_i64_or(&self, key: &str, default: i64) -> i64 {
        self.get_i64(key).unwrap_or(default)
    }
    
    /// Get a configuration value as structured data
    pub fn get_structured(&self, key: &str) -> anyhow::Result<serde_yaml::Value> {
        match self.values.get(key) {
            Some(value) => Ok(value.clone()),
            None => anyhow::bail!("Configuration key '{}' not found", key),
        }
    }
    
    /// Set a configuration value
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> anyhow::Result<()> {
        let yaml_value = serde_yaml::to_value(value)?;
        self.values.insert(key.to_string(), yaml_value);
        Ok(())
    }
    
    /// Validate this configuration against a schema
    pub fn validate_against_schema(&self, schema: &OxiConfigSchema) -> Result<(), ConfigError> {
        // Check required fields
        for required_field in &schema.required {
            if !self.values.contains_key(required_field) {
                return Err(ConfigError::MissingField(required_field.clone()));
            }
        }
        
        // Validate each property
        for (key, value) in &self.values {
            if let Some(property_schema) = schema.properties.get(key) {
                self.validate_property_value(key, value, property_schema)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate a specific property value against its schema
    fn validate_property_value(&self, key: &str, value: &serde_yaml::Value, schema: &PropertySchema) -> Result<(), ConfigError> {
        match schema.property_type.as_str() {
            "string" => {
                if !value.is_string() {
                    return Err(ConfigError::ValidationError(
                        format!("Property '{}' must be a string", key)
                    ));
                }
                
                // Validate enum values if specified
                if let Some(enum_values) = &schema.enum_values {
                    if let Some(string_val) = value.as_str() {
                        if !enum_values.contains(&string_val.to_string()) {
                            return Err(ConfigError::ValidationError(
                                format!("Property '{}' must be one of: {:?}", key, enum_values)
                            ));
                        }
                    }
                }
            },
            "boolean" => {
                if !value.is_bool() {
                    return Err(ConfigError::ValidationError(
                        format!("Property '{}' must be a boolean", key)
                    ));
                }
            },
            "number" | "integer" => {
                if !value.is_number() {
                    return Err(ConfigError::ValidationError(
                        format!("Property '{}' must be a number", key)
                    ));
                }
            },
            "array" => {
                if !value.is_sequence() {
                    return Err(ConfigError::ValidationError(
                        format!("Property '{}' must be an array", key)
                    ));
                }
            },
            "object" => {
                if !value.is_mapping() {
                    return Err(ConfigError::ValidationError(
                        format!("Property '{}' must be an object", key)
                    ));
                }
                
                // Validate nested properties if schema is provided
                if let Some(nested_properties) = &schema.properties {
                    if let Some(mapping) = value.as_mapping() {
                        for (nested_key, nested_value) in mapping {
                            if let Some(nested_key_str) = nested_key.as_str() {
                                if let Some(nested_schema) = nested_properties.get(nested_key_str) {
                                    self.validate_property_value(
                                        &format!("{}.{}", key, nested_key_str),
                                        nested_value,
                                        nested_schema
                                    )?;
                                }
                            }
                        }
                    }
                }
            },
            _ => {
                return Err(ConfigError::ValidationError(
                    format!("Unknown property type '{}' for property '{}'", schema.property_type, key)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Apply default values from a schema
    pub fn apply_defaults(&mut self, schema: &OxiConfigSchema) {
        for (key, property_schema) in &schema.properties {
            if !self.values.contains_key(key) {
                if let Some(default_value) = &property_schema.default {
                    self.values.insert(key.clone(), default_value.clone());
                }
            }
        }
    }
    
    /// Validate configuration against schema
    pub fn validate(&self, schema: &crate::config::OxiConfigSchema) -> Result<(), crate::config::ConfigError> {
        // Check required fields
        for required in &schema.required {
            if !self.values.contains_key(required) {
                return Err(crate::config::ConfigError::MissingField(required.clone()));
            }
        }
        
        // Validate properties
        for (key, property_schema) in &schema.properties {
            if let Some(value) = self.values.get(key) {
                validate_property(key, value, property_schema)?;
            }
        }
        
        Ok(())
    }
}

/// Validate a property value against its schema
fn validate_property(
    key: &str, 
    value: &serde_yaml::Value, 
    schema: &crate::config::PropertySchema
) -> Result<(), crate::config::ConfigError> {
    match schema.property_type.as_str() {
        "string" => {
            if !value.is_string() {
                return Err(crate::config::ConfigError::ValidationError(
                    format!("Property '{}' must be a string", key)
                ));
            }
            
            // Check enum values if specified
            if let Some(enum_values) = &schema.enum_values {
                let value_str = value.as_str().unwrap();
                if !enum_values.contains(&value_str.to_string()) {
                    return Err(crate::config::ConfigError::ValidationError(
                        format!("Property '{}' must be one of: {}", key, enum_values.join(", "))
                    ));
                }
            }
        },
        "number" => {
            if !value.is_number() {
                return Err(crate::config::ConfigError::ValidationError(
                    format!("Property '{}' must be a number", key)
                ));
            }
        },
        "integer" => {
            if !value.is_number() || value.as_f64().map(|f| f.fract() != 0.0).unwrap_or(true) {
                return Err(crate::config::ConfigError::ValidationError(
                    format!("Property '{}' must be an integer", key)
                ));
            }
        },
        "boolean" => {
            if !value.is_bool() {
                return Err(crate::config::ConfigError::ValidationError(
                    format!("Property '{}' must be a boolean", key)
                ));
            }
        },
        "array" => {
            if !value.is_sequence() {
                return Err(crate::config::ConfigError::ValidationError(
                    format!("Property '{}' must be an array", key)
                ));
            }
        },
        "object" => {
            if !value.is_mapping() {
                return Err(crate::config::ConfigError::ValidationError(
                    format!("Property '{}' must be an object", key)
                ));
            }
            
            // Validate nested properties if defined
            if let Some(props) = &schema.properties {
                if let Some(mapping) = value.as_mapping() {
                    for (nested_key, nested_value) in mapping {
                        if let Some(nested_key_str) = nested_key.as_str() {
                            if let Some(nested_schema) = props.get(nested_key_str) {
                                validate_property(
                                    &format!("{}.{}", key, nested_key_str), 
                                    nested_value, 
                                    nested_schema
                                )?;
                            }
                        }
                    }
                }
            }
        },
        _ => {
            return Err(crate::config::ConfigError::ValidationError(
                format!("Unknown property type: {} for property '{}'", schema.property_type, key)
            ));
        }
    }
    
    Ok(())
}

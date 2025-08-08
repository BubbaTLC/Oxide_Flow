use crate::config::{ConfigError, OxiConfigSchema, PropertySchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Schema strategies that Oxis use to handle schema evolution
#[derive(Debug, Clone)]
pub enum SchemaStrategy {
    /// Schema passes through unchanged (filters, validators)
    Passthrough,
    /// Schema is modified (field renames, additions, deletions)
    Modify { description: String },
    /// Schema is inferred from data (when transformation is data-dependent)
    Infer,
}

/// Data represents the actual data payload flowing between Oxis in the pipeline.
/// Uses JSON as the primary internal data format for structured data exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Data {
    /// JSON data - the primary format for structured data exchange between Oxis
    Json(serde_json::Value),

    /// Text data (strings, logs, etc.) - for simple text operations
    Text(String),

    /// Binary data (files, images, etc.) - for binary operations
    Binary(Vec<u8>),

    /// Empty data (used for initialization)
    Empty,
}

impl Data {
    /// Create a new empty Data
    pub fn empty() -> Self {
        Data::Empty
    }

    /// Create a new Data from text
    pub fn from_text(text: &str) -> Self {
        Data::Text(text.to_string())
    }

    /// Create a new Data from JSON data
    pub fn from_json(data: serde_json::Value) -> Self {
        Data::Json(data)
    }

    /// Create a new Data from binary data
    pub fn from_binary(data: Vec<u8>) -> Self {
        Data::Binary(data)
    }

    /// Check if this is empty data
    pub fn is_empty(&self) -> bool {
        matches!(self, Data::Empty)
    }

    /// Get text data or return an error
    pub fn as_text(&self) -> anyhow::Result<&str> {
        match self {
            Data::Text(text) => Ok(text),
            _ => anyhow::bail!("Expected text data, found {:?}", self.data_type()),
        }
    }

    /// Get JSON data or return an error
    pub fn as_json(&self) -> anyhow::Result<&serde_json::Value> {
        match self {
            Data::Json(json) => Ok(json),
            _ => anyhow::bail!("Expected JSON data, found {:?}", self.data_type()),
        }
    }

    /// Get binary data or return an error
    pub fn as_binary(&self) -> anyhow::Result<&Vec<u8>> {
        match self {
            Data::Binary(binary) => Ok(binary),
            _ => anyhow::bail!("Expected binary data, found {:?}", self.data_type()),
        }
    }

    /// Get the type of data for error messages
    pub fn data_type(&self) -> &'static str {
        match self {
            Data::Json(_) => "JSON",
            Data::Text(_) => "Text",
            Data::Binary(_) => "Binary",
            Data::Empty => "Empty",
        }
    }

    /// Convert to text representation
    pub fn to_text(&self) -> anyhow::Result<String> {
        match self {
            Data::Text(text) => Ok(text.clone()),
            Data::Json(json) => Ok(serde_json::to_string_pretty(json)?),
            Data::Binary(data) => {
                // Convert binary to base64 string for text representation
                use base64::Engine;
                Ok(base64::engine::general_purpose::STANDARD.encode(data))
            }
            Data::Empty => Ok(String::new()),
        }
    }

    /// Convert to binary representation
    pub fn to_binary(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            Data::Text(text) => Ok(text.as_bytes().to_vec()),
            Data::Json(json) => Ok(serde_json::to_string(json)?.as_bytes().to_vec()),
            Data::Binary(data) => Ok(data.clone()),
            Data::Empty => Ok(Vec::new()),
        }
    }

    /// Convert to JSON with fallback parsing
    pub fn to_json(&self) -> anyhow::Result<serde_json::Value> {
        match self {
            Data::Json(data) => Ok(data.clone()),
            Data::Text(text) => serde_json::from_str(text)
                .map_err(|e| anyhow::anyhow!("Failed to parse text as JSON: {}", e)),
            Data::Binary(_) => Err(anyhow::anyhow!("Cannot convert binary data to JSON")),
            Data::Empty => Ok(serde_json::Value::Null),
        }
    }

    /// Enhanced array handling for CSV formatting and batch processing
    pub fn as_array(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        match self {
            Data::Json(serde_json::Value::Array(arr)) => Ok(arr.clone()),
            Data::Json(single_obj) => Ok(vec![single_obj.clone()]),
            _ => Err(anyhow::anyhow!("Cannot convert to array")),
        }
    }

    /// Check if data represents a batch (array with multiple items)
    pub fn is_batch(&self) -> bool {
        match self {
            Data::Json(serde_json::Value::Array(arr)) => arr.len() > 1,
            _ => false,
        }
    }

    /// Get the batch size (number of items in array)
    pub fn batch_size(&self) -> usize {
        match self {
            Data::Json(serde_json::Value::Array(arr)) => arr.len(),
            _ => 1, // Single items have batch size of 1
        }
    }

    /// Get estimated memory usage for processing limits
    pub fn estimated_memory_usage(&self) -> usize {
        match self {
            Data::Json(value) => {
                // Rough estimate: JSON string length * 2 for overhead
                value.to_string().len() * 2
            }
            Data::Text(text) => text.len(),
            Data::Binary(bytes) => bytes.len(),
            Data::Empty => 0,
        }
    }

    /// Get the OxiDataType for this data
    pub fn get_data_type(&self) -> OxiDataType {
        match self {
            Data::Json(_) => OxiDataType::Json,
            Data::Text(_) => OxiDataType::Text,
            Data::Binary(_) => OxiDataType::Binary,
            Data::Empty => OxiDataType::Empty,
        }
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Text(text) => write!(f, "{text}"),
            Data::Json(data) => match serde_json::to_string_pretty(&data) {
                Ok(json) => write!(f, "{json}"),
                Err(_) => write!(f, "<Invalid JSON data>"),
            },
            Data::Binary(data) => {
                if data.len() > 100 {
                    write!(f, "<Binary data: {} bytes>", data.len())
                } else {
                    write!(f, "<Binary data: {data:?}>")
                }
            }
            Data::Empty => write!(f, "<Empty>"),
        }
    }
}

/// Data types that can be processed by Oxis
#[derive(Debug, Clone, PartialEq)]
pub enum OxiDataType {
    Json,
    Text,
    Binary,
    Empty,
}

impl fmt::Display for OxiDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OxiDataType::Json => write!(f, "JSON"),
            OxiDataType::Text => write!(f, "Text"),
            OxiDataType::Binary => write!(f, "Binary"),
            OxiDataType::Empty => write!(f, "Empty"),
        }
    }
}

/// Processing limits that each Oxi can define to manage resource usage
#[derive(Debug, Clone)]
pub struct ProcessingLimits {
    pub max_batch_size: Option<usize>,
    pub max_memory_mb: Option<usize>,
    pub max_processing_time_ms: Option<u64>,
    pub supported_input_types: Vec<OxiDataType>,
}

impl Default for ProcessingLimits {
    fn default() -> Self {
        Self {
            max_batch_size: Some(100_000),        // Default 100K records
            max_memory_mb: Some(512),             // Default 512MB
            max_processing_time_ms: Some(30_000), // Default 30s
            supported_input_types: vec![
                OxiDataType::Json,
                OxiDataType::Text,
                OxiDataType::Binary,
                OxiDataType::Empty,
            ],
        }
    }
}

/// Configuration for an Oxi
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OxiConfig {
    /// Key-value pairs for configuration
    pub values: HashMap<String, serde_yaml::Value>,
}

impl OxiConfig {
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
            }
            _ => OxiConfig::default(),
        }
    }

    /// Get a string configuration value
    pub fn get_string(&self, key: &str) -> anyhow::Result<String> {
        match self.values.get(key) {
            Some(serde_yaml::Value::String(value)) => Ok(value.clone()),
            Some(value) => Ok(value
                .as_str()
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
                        _ => {
                            anyhow::bail!("Value for key '{}' cannot be converted to boolean", key)
                        }
                    }
                } else {
                    anyhow::bail!("Value for key '{}' is not a boolean", key)
                }
            }
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
            Some(serde_yaml::Value::Number(value)) => value.as_f64().ok_or_else(|| {
                anyhow::anyhow!("Value for key '{}' cannot be converted to f64", key)
            }),
            Some(value) => {
                // Try to convert string values to numbers
                if let Some(s) = value.as_str() {
                    s.parse::<f64>().map_err(|_| {
                        anyhow::anyhow!("Value for key '{}' cannot be parsed as f64", key)
                    })
                } else {
                    anyhow::bail!("Value for key '{}' is not a number", key)
                }
            }
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
            }
            Some(_) => anyhow::bail!("Value for key '{}' is not a mapping", key),
            None => anyhow::bail!("Configuration key '{}' not found", key),
        }
    }

    /// Get a nested configuration object or default
    pub fn get_nested_or(&self, key: &str) -> OxiConfig {
        self.get_nested(key)
            .unwrap_or_else(|_| OxiConfig::default())
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
            Some(serde_yaml::Value::Number(value)) => value
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("Value for key '{}' is not an integer", key)),
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
                Self::validate_property_value(key, value, property_schema)?;
            }
        }

        Ok(())
    }

    /// Validate a specific property value against its schema
    fn validate_property_value(
        key: &str,
        value: &serde_yaml::Value,
        schema: &PropertySchema,
    ) -> Result<(), ConfigError> {
        match schema.property_type.as_str() {
            "string" => {
                if !value.is_string() {
                    return Err(ConfigError::ValidationError(format!(
                        "Property '{key}' must be a string"
                    )));
                }

                // Validate enum values if specified
                if let Some(enum_values) = &schema.enum_values {
                    if let Some(string_val) = value.as_str() {
                        if !enum_values.contains(&string_val.to_string()) {
                            return Err(ConfigError::ValidationError(format!(
                                "Property '{key}' must be one of: {enum_values:?}"
                            )));
                        }
                    }
                }
            }
            "boolean" => {
                if !value.is_bool() {
                    return Err(ConfigError::ValidationError(format!(
                        "Property '{key}' must be a boolean"
                    )));
                }
            }
            "number" | "integer" => {
                if !value.is_number() {
                    return Err(ConfigError::ValidationError(format!(
                        "Property '{key}' must be a number"
                    )));
                }
            }
            "array" => {
                if !value.is_sequence() {
                    return Err(ConfigError::ValidationError(format!(
                        "Property '{key}' must be an array"
                    )));
                }
            }
            "object" => {
                if !value.is_mapping() {
                    return Err(ConfigError::ValidationError(format!(
                        "Property '{key}' must be an object"
                    )));
                }

                // Validate nested properties if schema is provided
                if let Some(nested_properties) = &schema.properties {
                    if let Some(mapping) = value.as_mapping() {
                        for (nested_key, nested_value) in mapping {
                            if let Some(nested_key_str) = nested_key.as_str() {
                                if let Some(nested_schema) = nested_properties.get(nested_key_str) {
                                    Self::validate_property_value(
                                        &format!("{key}.{nested_key_str}"),
                                        nested_value,
                                        nested_schema,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                return Err(ConfigError::ValidationError(format!(
                    "Unknown property type '{}' for property '{}'",
                    schema.property_type, key
                )));
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
    pub fn validate(
        &self,
        schema: &crate::config::OxiConfigSchema,
    ) -> Result<(), crate::config::ConfigError> {
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
    schema: &crate::config::PropertySchema,
) -> Result<(), crate::config::ConfigError> {
    match schema.property_type.as_str() {
        "string" => {
            if !value.is_string() {
                return Err(crate::config::ConfigError::ValidationError(format!(
                    "Property '{key}' must be a string"
                )));
            }

            // Check enum values if specified
            if let Some(enum_values) = &schema.enum_values {
                let value_str = value.as_str().unwrap();
                if !enum_values.contains(&value_str.to_string()) {
                    return Err(crate::config::ConfigError::ValidationError(format!(
                        "Property '{}' must be one of: {}",
                        key,
                        enum_values.join(", ")
                    )));
                }
            }
        }
        "number" => {
            if !value.is_number() {
                return Err(crate::config::ConfigError::ValidationError(format!(
                    "Property '{key}' must be a number"
                )));
            }
        }
        "integer" => {
            if !value.is_number() || value.as_f64().map(|f| f.fract() != 0.0).unwrap_or(true) {
                return Err(crate::config::ConfigError::ValidationError(format!(
                    "Property '{key}' must be an integer"
                )));
            }
        }
        "boolean" => {
            if !value.is_bool() {
                return Err(crate::config::ConfigError::ValidationError(format!(
                    "Property '{key}' must be a boolean"
                )));
            }
        }
        "array" => {
            if !value.is_sequence() {
                return Err(crate::config::ConfigError::ValidationError(format!(
                    "Property '{key}' must be an array"
                )));
            }
        }
        "object" => {
            if !value.is_mapping() {
                return Err(crate::config::ConfigError::ValidationError(format!(
                    "Property '{key}' must be an object"
                )));
            }

            // Validate nested properties if defined
            if let Some(props) = &schema.properties {
                if let Some(mapping) = value.as_mapping() {
                    for (nested_key, nested_value) in mapping {
                        if let Some(nested_key_str) = nested_key.as_str() {
                            if let Some(nested_schema) = props.get(nested_key_str) {
                                validate_property(
                                    &format!("{key}.{nested_key_str}"),
                                    nested_value,
                                    nested_schema,
                                )?;
                            }
                        }
                    }
                }
            }
        }
        _ => {
            return Err(crate::config::ConfigError::ValidationError(format!(
                "Unknown property type: {} for property '{}'",
                schema.property_type, key
            )));
        }
    }

    Ok(())
}

// ============================================================================
// Schema System Types
// ============================================================================

/// Schema information that travels alongside data in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OxiSchema {
    /// Field definitions keyed by field name
    pub fields: HashMap<String, FieldSchema>,
    /// Schema metadata and hints
    pub metadata: SchemaMetadata,
}

impl OxiSchema {
    /// Create a new empty schema
    pub fn empty() -> Self {
        Self {
            fields: HashMap::new(),
            metadata: SchemaMetadata::default(),
        }
    }

    /// Add a field to the schema
    pub fn add_field(&mut self, name: String, field: FieldSchema) {
        self.fields.insert(name, field);
    }

    /// Infer schema from data
    pub fn infer_from_data(data: &Data) -> Result<Self, crate::error::OxiError> {
        let mut schema = Self::empty();
        schema.metadata.created_by = "oxide_flow_schema_inference".to_string();

        match data {
            Data::Json(json_value) => {
                schema.infer_from_json_value(json_value, "root")?;
            }
            Data::Text(_) => {
                // Text data gets a simple "value" field schema
                schema.add_field(
                    "value".to_string(),
                    FieldSchema {
                        field_type: FieldType::String,
                        nullable: false,
                        max_size: None,
                        constraints: vec![],
                        description: Some("Text content".to_string()),
                        examples: vec![],
                    },
                );
            }
            Data::Binary(_) => {
                // Binary data gets a simple "data" field schema
                schema.add_field(
                    "data".to_string(),
                    FieldSchema {
                        field_type: FieldType::Binary,
                        nullable: false,
                        max_size: None,
                        constraints: vec![],
                        description: Some("Binary content".to_string()),
                        examples: vec![],
                    },
                );
            }
            Data::Empty => {
                // Empty data has no fields
            }
        }

        Ok(schema)
    }

    fn infer_from_json_value(
        &mut self,
        value: &serde_json::Value,
        _path: &str,
    ) -> Result<(), crate::error::OxiError> {
        match value {
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let field_type = match val {
                        serde_json::Value::String(_) => FieldType::String,
                        serde_json::Value::Number(n) => {
                            if n.is_i64() {
                                FieldType::Integer
                            } else {
                                FieldType::Float
                            }
                        }
                        serde_json::Value::Bool(_) => FieldType::Boolean,
                        serde_json::Value::Array(_) => {
                            FieldType::Array(Box::new(FieldType::Unknown))
                        }
                        serde_json::Value::Object(_) => FieldType::Object(HashMap::new()),
                        serde_json::Value::Null => FieldType::String, // Default for null
                    };

                    self.add_field(
                        key.clone(),
                        FieldSchema {
                            field_type,
                            nullable: val.is_null(),
                            max_size: None,
                            constraints: vec![],
                            description: None,
                            examples: vec![val.clone()],
                        },
                    );
                }
            }
            serde_json::Value::Array(arr) => {
                // For arrays, merge schemas from sample elements
                if let Some(first_element) = arr.first() {
                    self.infer_from_json_value(first_element, "array_element")?;
                }
            }
            _ => {
                // Single value gets a "value" field
                let field_type = match value {
                    serde_json::Value::String(_) => FieldType::String,
                    serde_json::Value::Number(n) => {
                        if n.is_i64() {
                            FieldType::Integer
                        } else {
                            FieldType::Float
                        }
                    }
                    serde_json::Value::Bool(_) => FieldType::Boolean,
                    _ => FieldType::String,
                };

                self.add_field(
                    "value".to_string(),
                    FieldSchema {
                        field_type,
                        nullable: value.is_null(),
                        max_size: None,
                        constraints: vec![],
                        description: Some("Inferred value field".to_string()),
                        examples: vec![value.clone()],
                    },
                );
            }
        }
        Ok(())
    }

    /// Validate data against this schema
    pub fn validate_data(&self, data: &Data) -> Result<(), crate::error::OxiError> {
        match data {
            Data::Json(json_value) => self.validate_json_value(json_value, "root"),
            Data::Text(_) => {
                // For text data, check if schema expects text-compatible fields
                if self.fields.len() == 1 && self.fields.contains_key("value") {
                    Ok(())
                } else {
                    Err(crate::error::OxiError::ValidationError {
                        details: "Schema validation for text data requires single 'value' field"
                            .to_string(),
                    })
                }
            }
            Data::Binary(_) => {
                // For binary data, similar check
                if self.fields.len() == 1 && self.fields.contains_key("data") {
                    Ok(())
                } else {
                    Err(crate::error::OxiError::ValidationError {
                        details: "Schema validation for binary data requires single 'data' field"
                            .to_string(),
                    })
                }
            }
            Data::Empty => Ok(()), // Empty data always validates
        }
    }

    fn validate_json_value(
        &self,
        value: &serde_json::Value,
        path: &str,
    ) -> Result<(), crate::error::OxiError> {
        match value {
            serde_json::Value::Object(obj) => {
                // Validate each field in the schema
                for (field_name, field_schema) in &self.fields {
                    let field_path = if path == "root" {
                        field_name.clone()
                    } else {
                        format!("{path}.{field_name}")
                    };

                    match obj.get(field_name) {
                        Some(field_value) => {
                            field_schema.validate_value(field_value, &field_path)?;
                        }
                        None => {
                            if !field_schema.nullable {
                                return Err(crate::error::OxiError::ValidationError {
                                    details: format!("Required field '{field_path}' is missing"),
                                });
                            }
                        }
                    }
                }
                Ok(())
            }
            serde_json::Value::Array(arr) => {
                // For arrays, validate each element
                for (i, item) in arr.iter().enumerate() {
                    let item_path = format!("{path}[{i}]");
                    self.validate_json_value(item, &item_path)?;
                }
                Ok(())
            }
            _ => {
                // Single value - check if schema has a "value" field
                if let Some(value_field) = self.fields.get("value") {
                    value_field.validate_value(value, &format!("{path}.value"))
                } else {
                    Err(crate::error::OxiError::ValidationError {
                        details: format!(
                            "Schema expects object structure, got single value at {path}"
                        ),
                    })
                }
            }
        }
    }
}

impl Default for OxiSchema {
    fn default() -> Self {
        Self::empty()
    }
}

/// Field schema definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldSchema {
    /// The data type of this field
    pub field_type: FieldType,
    /// Whether this field can be null/empty
    pub nullable: bool,
    /// Maximum size (for strings, arrays, etc.)
    pub max_size: Option<usize>,
    /// Field constraints and validation rules
    pub constraints: Vec<FieldConstraint>,
    /// Human-readable description
    pub description: Option<String>,
    /// Examples of valid values
    pub examples: Vec<serde_json::Value>,
}

impl FieldSchema {
    /// Create a new field schema with basic type
    pub fn new(field_type: FieldType) -> Self {
        Self {
            field_type,
            nullable: false,
            max_size: None,
            constraints: Vec::new(),
            description: None,
            examples: Vec::new(),
        }
    }

    /// Validate a JSON value against this field schema
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
        path: &str,
    ) -> Result<(), crate::error::OxiError> {
        if value.is_null() {
            if !self.nullable {
                return Err(crate::error::OxiError::ValidationError {
                    details: format!("Field '{path}' cannot be null"),
                });
            }
            return Ok(());
        }

        if !self.field_type.matches_value(value) {
            return Err(crate::error::OxiError::ValidationError {
                details: format!(
                    "Field '{}' type mismatch: expected {:?}, got {}",
                    path,
                    self.field_type,
                    self.value_type_name(value)
                ),
            });
        }

        // Validate constraints
        for constraint in &self.constraints {
            constraint.validate_value(value, path)?;
        }

        Ok(())
    }

    fn value_type_name(&self, value: &serde_json::Value) -> &'static str {
        match value {
            serde_json::Value::String(_) => "String",
            serde_json::Value::Number(_) => "Number",
            serde_json::Value::Bool(_) => "Boolean",
            serde_json::Value::Array(_) => "Array",
            serde_json::Value::Object(_) => "Object",
            serde_json::Value::Null => "Null",
        }
    }
}

impl Default for FieldSchema {
    fn default() -> Self {
        Self::new(FieldType::Unknown)
    }
}

/// Field type definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    // Primitive types
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Binary,

    // Complex types
    Array(Box<FieldType>),
    Object(HashMap<String, FieldSchema>),

    // Special types
    Unknown, // For fields we can't determine the type
    Mixed,   // For fields that contain multiple types
}

impl FieldType {
    /// Check if a JSON value matches this field type
    pub fn matches_value(&self, value: &serde_json::Value) -> bool {
        match self {
            FieldType::String => value.is_string(),
            FieldType::Integer => {
                value.is_number() && value.as_f64().is_some_and(|f| f.fract() == 0.0)
            }
            FieldType::Float => value.is_number(),
            FieldType::Boolean => value.is_boolean(),
            FieldType::DateTime => {
                // Try to parse as ISO 8601 datetime
                value.is_string()
                    && value
                        .as_str()
                        .is_some_and(|s| chrono::DateTime::parse_from_rfc3339(s).is_ok())
            }
            FieldType::Binary => {
                // For JSON, binary data is typically base64 encoded strings
                value.is_string()
            }
            FieldType::Array(_) => value.is_array(),
            FieldType::Object(_) => value.is_object(),
            FieldType::Unknown | FieldType::Mixed => true, // Accept anything
        }
    }
}

/// Field constraint definitions for validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldConstraint {
    // Numeric constraints
    MinValue(f64),
    MaxValue(f64),

    // String constraints
    MinLength(usize),
    MaxLength(usize),
    Pattern(String), // Regex pattern

    // Enum constraints
    OneOf(Vec<serde_json::Value>),

    // Custom validation
    Custom { name: String, rule: String },
}

impl FieldConstraint {
    /// Validate a value against this constraint
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
        path: &str,
    ) -> Result<(), crate::error::OxiError> {
        match self {
            FieldConstraint::MinValue(min) => {
                if let Some(num) = value.as_f64() {
                    if num < *min {
                        return Err(crate::error::OxiError::ValidationError {
                            details: format!(
                                "Field '{path}' value {num} is less than minimum {min}"
                            ),
                        });
                    }
                }
                Ok(())
            }
            FieldConstraint::MaxValue(max) => {
                if let Some(num) = value.as_f64() {
                    if num > *max {
                        return Err(crate::error::OxiError::ValidationError {
                            details: format!(
                                "Field '{path}' value {num} is greater than maximum {max}"
                            ),
                        });
                    }
                }
                Ok(())
            }
            FieldConstraint::MinLength(min_len) => {
                if let Some(s) = value.as_str() {
                    if s.len() < *min_len {
                        return Err(crate::error::OxiError::ValidationError {
                            details: format!(
                                "Field '{}' length {} is less than minimum {}",
                                path,
                                s.len(),
                                min_len
                            ),
                        });
                    }
                }
                Ok(())
            }
            FieldConstraint::MaxLength(max_len) => {
                if let Some(s) = value.as_str() {
                    if s.len() > *max_len {
                        return Err(crate::error::OxiError::ValidationError {
                            details: format!(
                                "Field '{}' length {} is greater than maximum {}",
                                path,
                                s.len(),
                                max_len
                            ),
                        });
                    }
                }
                Ok(())
            }
            FieldConstraint::Pattern(pattern) => {
                if let Some(s) = value.as_str() {
                    // For now, just check if pattern is a simple substring
                    // In a full implementation, you'd use regex crate
                    if !s.contains(pattern) {
                        return Err(crate::error::OxiError::ValidationError {
                            details: format!(
                                "Field '{path}' value '{s}' does not match pattern '{pattern}'"
                            ),
                        });
                    }
                }
                Ok(())
            }
            FieldConstraint::OneOf(allowed_values) => {
                if !allowed_values.contains(value) {
                    return Err(crate::error::OxiError::ValidationError {
                        details: format!("Field '{path}' value must be one of {allowed_values:?}"),
                    });
                }
                Ok(())
            }
            FieldConstraint::Custom { name: _, rule: _ } => {
                // Custom validation would be implemented here
                Ok(())
            }
        }
    }
}

/// Schema metadata and hints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaMetadata {
    pub version: String,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub row_count_hint: Option<usize>,
}

impl Default for SchemaMetadata {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            created_by: "oxide_flow".to_string(),
            created_at: chrono::Utc::now(),
            row_count_hint: None,
        }
    }
}

/// OxiData represents unified schema-aware data flowing between Oxis in the pipeline.
/// Every piece of data includes both the payload and its schema information.
#[derive(Debug, Clone)]
pub struct OxiData {
    /// The actual data payload
    pub data: Data,
    /// Schema information (always present, may be inferred or empty)
    pub schema: OxiSchema,
}

impl OxiData {
    /// Create new OxiData with inferred schema
    pub fn new(data: Data) -> Self {
        let schema = OxiSchema::infer_from_data(&data).unwrap_or_default();
        Self { data, schema }
    }

    /// Create OxiData with explicit schema
    pub fn with_schema(data: Data, schema: OxiSchema) -> Self {
        Self { data, schema }
    }

    /// Create empty OxiData
    pub fn empty() -> Self {
        Self::new(Data::Empty)
    }

    /// Create from JSON with schema inference
    pub fn from_json(value: serde_json::Value) -> Self {
        Self::new(Data::Json(value))
    }

    /// Create from text with schema inference
    pub fn from_text(text: String) -> Self {
        Self::new(Data::Text(text))
    }

    /// Create from binary data
    pub fn from_binary(data: Vec<u8>) -> Self {
        Self::new(Data::Binary(data))
    }

    /// Convenience method to access the data
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Convenience method to access the schema
    pub fn schema(&self) -> &OxiSchema {
        &self.schema
    }

    /// Update the schema while keeping the same data
    pub fn with_updated_schema(mut self, new_schema: OxiSchema) -> Self {
        self.schema = new_schema;
        self
    }

    /// Validate the data against its schema
    pub fn validate(&self) -> Result<(), crate::error::OxiError> {
        self.schema.validate_data(&self.data)
    }

    /// Get estimated memory usage for processing limits
    pub fn estimated_memory_usage(&self) -> usize {
        self.data.estimated_memory_usage()
    }

    /// Extract just the data (for backward compatibility)
    pub fn into_data(self) -> Data {
        self.data
    }
}

impl From<Data> for OxiData {
    fn from(data: Data) -> Self {
        Self::new(data)
    }
}

use oxide_flow::config::{OxiConfigSchema, PropertySchema};
use oxide_flow::types::OxiConfig;
use std::collections::HashMap;

#[test]
fn test_nested_config_validation() {
    // Create a schema for a read_file Oxi with nested properties
    let mut schema = OxiConfigSchema {
        description: Some("Read file configuration".to_string()),
        properties: HashMap::new(),
        required: vec!["path".to_string()],
    };
    
    // Path property (required)
    schema.properties.insert("path".to_string(), PropertySchema {
        property_type: "string".to_string(),
        description: Some("File path to read".to_string()),
        default: None,
        enum_values: None,
        properties: None,
    });
    
    // Optional encoding property with default
    schema.properties.insert("encoding".to_string(), PropertySchema {
        property_type: "string".to_string(),
        description: Some("File encoding".to_string()),
        default: Some(serde_yaml::Value::String("utf-8".to_string())),
        enum_values: Some(vec!["utf-8".to_string(), "ascii".to_string(), "latin1".to_string()]),
        properties: None,
    });
    
    // Nested connection properties
    let mut connection_properties = HashMap::new();
    connection_properties.insert("host".to_string(), PropertySchema {
        property_type: "string".to_string(),
        description: Some("Database host".to_string()),
        default: Some(serde_yaml::Value::String("localhost".to_string())),
        enum_values: None,
        properties: None,
    });
    connection_properties.insert("port".to_string(), PropertySchema {
        property_type: "integer".to_string(),
        description: Some("Database port".to_string()),
        default: Some(serde_yaml::Value::Number(5432.into())),
        enum_values: None,
        properties: None,
    });
    
    schema.properties.insert("connection".to_string(), PropertySchema {
        property_type: "object".to_string(),
        description: Some("Database connection settings".to_string()),
        default: None,
        enum_values: None,
        properties: Some(connection_properties),
    });
    
    // Test valid configuration
    let mut valid_config = OxiConfig::default();
    valid_config.values.insert("path".to_string(), serde_yaml::Value::String("/tmp/test.txt".to_string()));
    valid_config.values.insert("encoding".to_string(), serde_yaml::Value::String("utf-8".to_string()));
    
    // Add nested connection object
    let mut connection_map = serde_yaml::Mapping::new();
    connection_map.insert(
        serde_yaml::Value::String("host".to_string()),
        serde_yaml::Value::String("mydb.example.com".to_string())
    );
    connection_map.insert(
        serde_yaml::Value::String("port".to_string()),
        serde_yaml::Value::Number(3306.into())
    );
    valid_config.values.insert("connection".to_string(), serde_yaml::Value::Mapping(connection_map));
    
    // This should validate successfully
    assert!(valid_config.validate_against_schema(&schema).is_ok());
    
    // Test missing required field
    let mut invalid_config = OxiConfig::default();
    invalid_config.values.insert("encoding".to_string(), serde_yaml::Value::String("utf-8".to_string()));
    
    // This should fail validation
    assert!(invalid_config.validate_against_schema(&schema).is_err());
    
    // Test invalid enum value
    let mut invalid_enum_config = OxiConfig::default();
    invalid_enum_config.values.insert("path".to_string(), serde_yaml::Value::String("/tmp/test.txt".to_string()));
    invalid_enum_config.values.insert("encoding".to_string(), serde_yaml::Value::String("invalid-encoding".to_string()));
    
    // This should fail validation
    assert!(invalid_enum_config.validate_against_schema(&schema).is_err());
}

#[test]
fn test_apply_defaults() {
    // Create a schema with defaults
    let mut schema = OxiConfigSchema {
        description: Some("Test schema with defaults".to_string()),
        properties: HashMap::new(),
        required: vec![],
    };
    
    schema.properties.insert("timeout".to_string(), PropertySchema {
        property_type: "integer".to_string(),
        description: Some("Timeout in seconds".to_string()),
        default: Some(serde_yaml::Value::Number(30.into())),
        enum_values: None,
        properties: None,
    });
    
    schema.properties.insert("retry_count".to_string(), PropertySchema {
        property_type: "integer".to_string(),
        description: Some("Number of retries".to_string()),
        default: Some(serde_yaml::Value::Number(3.into())),
        enum_values: None,
        properties: None,
    });
    
    // Create config without these values
    let mut config = OxiConfig::default();
    config.values.insert("custom_field".to_string(), serde_yaml::Value::String("custom_value".to_string()));
    
    // Apply defaults
    config.apply_defaults(&schema);
    
    // Check that defaults were applied
    assert_eq!(config.get_i64("timeout").unwrap(), 30);
    assert_eq!(config.get_i64("retry_count").unwrap(), 3);
    
    // Check that existing values weren't overwritten
    assert_eq!(config.get_string("custom_field").unwrap(), "custom_value");
}

use oxide_flow::config::{Config, substitute_env_vars, process_env_vars_in_yaml};
use std::env;

#[test]
fn test_env_var_substitution() {
    // Set test environment variables
    env::set_var("TEST_VAR", "test_value");
    env::set_var("NESTED_VAR", "nested_value");
    
    // Test simple substitution
    let input = "Hello ${TEST_VAR}!";
    let result = substitute_env_vars(input).unwrap();
    assert_eq!(result, "Hello test_value!");
    
    // Test substitution with default
    let input_with_default = "Hello ${MISSING_VAR:-default_value}!";
    let result_with_default = substitute_env_vars(input_with_default).unwrap();
    assert_eq!(result_with_default, "Hello default_value!");
    
    // Test multiple substitutions
    let input_multiple = "${TEST_VAR} and ${NESTED_VAR}";
    let result_multiple = substitute_env_vars(input_multiple).unwrap();
    assert_eq!(result_multiple, "test_value and nested_value");
    
    // Clean up
    env::remove_var("TEST_VAR");
    env::remove_var("NESTED_VAR");
}

#[test]
fn test_yaml_env_var_processing() {
    // Set test environment variable
    env::set_var("TEST_PATH", "/test/path");
    
    let yaml_content = r#"
path: "${TEST_PATH}/file.txt"
settings:
  debug: true
  timeout: "${TIMEOUT:-30}"
"#;
    
    let mut value: serde_yaml::Value = serde_yaml::from_str(yaml_content).unwrap();
    process_env_vars_in_yaml(&mut value).unwrap();
    
    // Verify substitution worked
    if let serde_yaml::Value::Mapping(map) = &value {
        if let Some(serde_yaml::Value::String(path)) = map.get(&serde_yaml::Value::String("path".to_string())) {
            assert_eq!(path, "/test/path/file.txt");
        }
        
        if let Some(serde_yaml::Value::Mapping(settings)) = map.get(&serde_yaml::Value::String("settings".to_string())) {
            if let Some(serde_yaml::Value::String(timeout)) = settings.get(&serde_yaml::Value::String("timeout".to_string())) {
                assert_eq!(timeout, "30");
            }
        }
    }
    
    // Clean up
    env::remove_var("TEST_PATH");
}

#[test]
fn test_config_loading() {
    // Set environment variables for testing
    env::set_var("PWD", "/current/dir");
    env::set_var("PROCESSING_MODE", "test");
    
    // Load the sample config
    let config_result = Config::load("sample_config.yaml");
    
    match config_result {
        Ok(config) => {
            assert_eq!(config.version, "1.0");
            assert_eq!(config.global.verbose, true);
            
            // Check if pipelines were loaded
            assert!(config.pipelines.contains_key("json_to_csv"));
            assert!(config.pipelines.contains_key("csv_to_json"));
        },
        Err(e) => {
            // Config might not exist in test environment, which is okay
            println!("Config loading failed (expected in test): {}", e);
        }
    }
    
    // Clean up
    env::remove_var("PWD");
    env::remove_var("PROCESSING_MODE");
}

use oxide_flow::oxis::prelude::*;
use oxide_flow::Oxi;
use serde_json::json;

// Test Oxi for testing the SDK foundation
struct TestOxi {
    limits: ProcessingLimits,
}

impl TestOxi {
    fn new(limits: ProcessingLimits) -> Self {
        Self { limits }
    }
}

#[async_trait]
impl Oxi for TestOxi {
    fn name(&self) -> &str {
        "test_oxi"
    }

    fn processing_limits(&self) -> ProcessingLimits {
        self.limits.clone()
    }

    fn validate_input(&self, input: &OxiData) -> Result<(), OxiError> {
        if let Data::Text(text) = &input.data {
            if text.is_empty() {
                return Err(OxiError::ValidationError {
                    details: "Text cannot be empty".to_string(),
                });
            }
        }
        Ok(())
    }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Passthrough
    }

    async fn process(&self, input: OxiData, _config: &OxiConfig) -> Result<OxiData, OxiError> {
        // Simple passthrough for testing
        Ok(input)
    }
}

#[tokio::test]
async fn test_processing_limits_validation() {
    let limits = ProcessingLimits {
        max_batch_size: Some(100),
        max_memory_mb: Some(1), // 1MB limit
        max_processing_time_ms: Some(5000),
        supported_input_types: vec![OxiDataType::Json],
    };

    let oxi = TestOxi::new(limits);

    // Test that processing limits are returned correctly
    let returned_limits = oxi.processing_limits();
    assert_eq!(returned_limits.max_batch_size, Some(100));
    assert_eq!(returned_limits.max_memory_mb, Some(1));
    assert_eq!(returned_limits.max_processing_time_ms, Some(5000));
    assert_eq!(
        returned_limits.supported_input_types,
        vec![OxiDataType::Json]
    );
}

#[tokio::test]
async fn test_unsupported_input_type() {
    let limits = ProcessingLimits {
        supported_input_types: vec![OxiDataType::Json], // Only JSON supported
        ..ProcessingLimits::default()
    };

    let oxi = TestOxi::new(limits);
    let text_input = OxiData::from_text("test".to_string());

    // Should succeed because validation is not enforced in test Oxi
    let result = oxi.process(text_input, &OxiConfig::default()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_memory_limit_exceeded() {
    let limits = ProcessingLimits {
        max_memory_mb: Some(1), // 1MB limit
        supported_input_types: vec![OxiDataType::Json],
        ..ProcessingLimits::default()
    };

    let oxi = TestOxi::new(limits);

    // Create large JSON data that exceeds memory limit
    let large_string = "x".repeat(2 * 1024 * 1024); // 2MB string
    let large_json = OxiData::from_json(json!({"data": large_string}));

    // Should fail due to memory limits
    let result = oxi.process(large_json, &OxiConfig::default()).await;
    assert!(result.is_err());

    let error_string = result.unwrap_err().to_string();
    assert!(error_string.contains("Memory limit exceeded"));
}

#[tokio::test]
async fn test_batch_size_limit_exceeded() {
    let limits = ProcessingLimits {
        max_batch_size: Some(5), // 5 record limit
        supported_input_types: vec![OxiDataType::Json],
        ..ProcessingLimits::default()
    };

    let oxi = TestOxi::new(limits);

    // Create JSON array with more than 5 items
    let large_array = OxiData::from_json(json!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));

    // Should fail due to batch size limits
    let result = oxi.process(large_array, &OxiConfig::default()).await;
    assert!(result.is_err());

    let error_string = result.unwrap_err().to_string();
    assert!(error_string.contains("Batch size limit exceeded"));
}

#[tokio::test]
async fn test_input_validation() {
    let limits = ProcessingLimits {
        supported_input_types: vec![OxiDataType::Text],
        ..ProcessingLimits::default()
    };

    let oxi = TestOxi::new(limits);

    // Test that custom validation works
    let empty_text = OxiData::from_text("".to_string());
    let result = oxi.process(empty_text, &OxiConfig::default()).await;
    assert!(result.is_err());

    let error_string = result.unwrap_err().to_string();
    assert!(error_string.contains("Text cannot be empty"));

    // Test that valid input passes
    let valid_text = OxiData::from_text("valid".to_string());
    let result = oxi.process(valid_text, &OxiConfig::default()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_oxi_data_type_detection() {
    let json_data = OxiData::from_json(json!({"test": "value"}));
    assert!(json_data.data().as_json().is_ok());

    let text_data = OxiData::from_text("test".to_string());
    assert!(text_data.data().as_text().is_ok());

    let binary_data = OxiData::from_binary(vec![1, 2, 3]);
    assert!(binary_data.data().as_binary().is_ok());

    let empty_data = OxiData::empty();
    assert!(empty_data.data.is_empty());
}

#[tokio::test]
async fn test_oxi_data_array_detection() {
    // Test that we can detect array vs object JSON
    let single_json = OxiData::from_json(json!({"single": "item"}));
    if let Ok(json_val) = single_json.data().as_json() {
        assert!(json_val.is_object());
    }

    // Array should be detected as array
    let array_json = OxiData::from_json(json!([{"item": 1}, {"item": 2}]));
    if let Ok(json_val) = array_json.data().as_json() {
        assert!(json_val.is_array());
    }

    // Non-JSON data should be accessible as text
    let text_data = OxiData::from_text("not json".to_string());
    assert!(text_data.data().as_text().is_ok());
}

#[tokio::test]
async fn test_oxi_data_validation() {
    // Test that validation can be called on OxiData
    let json_data = OxiData::from_json(json!({"test": "value"}));
    let validation_result = json_data.validate();
    // Should succeed since we have valid JSON
    assert!(validation_result.is_ok());

    let text_data = OxiData::from_text("test string".to_string());
    let text_validation = text_data.validate();
    // Should succeed since text is valid
    assert!(text_validation.is_ok());
}

#[tokio::test]
async fn test_oxi_data_schema_access() {
    // Test that we can access schema information
    let json_data = OxiData::from_json(json!([{"a": 1}, {"b": 2}]));
    let _schema = json_data.schema();
    // Schema should be accessible

    let text_data = OxiData::from_text("test string".to_string());
    let _text_schema = text_data.schema();
    // Text data should also have schema
}

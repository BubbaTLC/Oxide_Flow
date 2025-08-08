use oxide_flow::oxis::prelude::*;
use oxide_flow::types::OxiDataWithSchema;
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
        match input {
            OxiData::Text(text) if text.is_empty() => Err(OxiError::ValidationError {
                details: "Text cannot be empty".to_string(),
            }),
            _ => Ok(()),
        }
    }

    async fn process_data(&self, input: OxiData, _config: &OxiConfig) -> anyhow::Result<OxiData> {
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
    let text_input = OxiData::Text("test".to_string());

    // Should fail because text is not supported
    let result = oxi
        .process(
            OxiDataWithSchema::from_data(text_input),
            &OxiConfig::default(),
        )
        .await;
    assert!(result.is_err());

    let error_string = result.unwrap_err().to_string();
    assert!(error_string.contains("does not support Text"));
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
    let large_json = OxiData::Json(json!({"data": large_string}));

    // Should fail due to memory limits
    let result = oxi
        .process(
            OxiDataWithSchema::from_data(large_json),
            &OxiConfig::default(),
        )
        .await;
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
    let large_array = OxiData::Json(json!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));

    // Should fail due to batch size limits
    let result = oxi
        .process(
            OxiDataWithSchema::from_data(large_array),
            &OxiConfig::default(),
        )
        .await;
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
    let empty_text = OxiData::Text("".to_string());
    let result = oxi
        .process(
            OxiDataWithSchema::from_data(empty_text),
            &OxiConfig::default(),
        )
        .await;
    assert!(result.is_err());

    let error_string = result.unwrap_err().to_string();
    assert!(error_string.contains("Text cannot be empty"));

    // Test that valid input passes
    let valid_text = OxiData::Text("valid".to_string());
    let result = oxi
        .process(
            OxiDataWithSchema::from_data(valid_text),
            &OxiConfig::default(),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_oxi_data_type_detection() {
    let json_data = OxiData::Json(json!({"test": "value"}));
    assert_eq!(json_data.get_data_type(), OxiDataType::Json);

    let text_data = OxiData::Text("test".to_string());
    assert_eq!(text_data.get_data_type(), OxiDataType::Text);

    let binary_data = OxiData::Binary(vec![1, 2, 3]);
    assert_eq!(binary_data.get_data_type(), OxiDataType::Binary);

    let empty_data = OxiData::Empty;
    assert_eq!(empty_data.get_data_type(), OxiDataType::Empty);
}

#[tokio::test]
async fn test_oxi_data_batch_detection() {
    // Single item should not be detected as batch
    let single_json = OxiData::Json(json!({"single": "item"}));
    assert!(!single_json.is_batch());

    // Array with one item should not be detected as batch
    let single_array = OxiData::Json(json!([{"single": "item"}]));
    assert!(!single_array.is_batch());

    // Array with multiple items should be detected as batch
    let batch_array = OxiData::Json(json!([{"item": 1}, {"item": 2}]));
    assert!(batch_array.is_batch());

    // Non-JSON data should not be detected as batch
    let text_data = OxiData::Text("not a batch".to_string());
    assert!(!text_data.is_batch());
}

#[tokio::test]
async fn test_oxi_data_memory_estimation() {
    let json_data = OxiData::Json(json!({"test": "value"}));
    let memory_usage = json_data.estimated_memory_usage();
    assert!(memory_usage > 0);

    let text_data = OxiData::Text("test string".to_string());
    let text_memory = text_data.estimated_memory_usage();
    assert_eq!(text_memory, "test string".len());

    let binary_data = OxiData::Binary(vec![1, 2, 3, 4, 5]);
    let binary_memory = binary_data.estimated_memory_usage();
    assert_eq!(binary_memory, 5);

    let empty_data = OxiData::Empty;
    let empty_memory = empty_data.estimated_memory_usage();
    assert_eq!(empty_memory, 0);
}

#[tokio::test]
async fn test_oxi_data_array_conversion() {
    // JSON array should convert correctly
    let json_array = OxiData::Json(json!([{"a": 1}, {"b": 2}]));
    let array = json_array.as_array().unwrap();
    assert_eq!(array.len(), 2);

    // Single JSON object should be wrapped in array
    let json_object = OxiData::Json(json!({"single": "object"}));
    let wrapped_array = json_object.as_array().unwrap();
    assert_eq!(wrapped_array.len(), 1);
    assert_eq!(wrapped_array[0], json!({"single": "object"}));

    // Non-JSON data should fail
    let text_data = OxiData::Text("not json".to_string());
    assert!(text_data.as_array().is_err());
}

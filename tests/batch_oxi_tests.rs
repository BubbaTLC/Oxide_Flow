use oxide_flow::oxis::batch::oxi::Batch;
use oxide_flow::oxis::prelude::*;
use oxide_flow::Oxi;
use serde_json::json;

#[tokio::test]
async fn test_batch_size_strategy() {
    let batch_oxi = Batch;
    let mut config = OxiConfig::default();
    config.values.insert(
        "batch_size".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(3)),
    );
    config.values.insert(
        "strategy".to_string(),
        serde_yaml::Value::String("Size".to_string()),
    );

    // Test with JSON array
    let input = OxiData::from_json(json!([1, 2, 3, 4, 5, 6, 7]));
    let result = batch_oxi.process(input, &config).await.unwrap();

    if let Data::Json(serde_json::Value::Array(batches)) = result.data() {
        assert_eq!(batches.len(), 3); // Should have 3 batches

        // First batch should have 3 items
        if let serde_json::Value::Array(first_batch) = &batches[0] {
            assert_eq!(first_batch.len(), 3);
        }

        // Last batch should have 1 item
        if let serde_json::Value::Array(last_batch) = &batches[2] {
            assert_eq!(last_batch.len(), 1);
        }
    } else {
        panic!("Expected batched JSON array");
    }
}

#[tokio::test]
async fn test_batch_memory_strategy() {
    let batch_oxi = Batch;
    let mut config = OxiConfig::default();
    config.values.insert(
        "batch_size".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(100)),
    );
    config.values.insert(
        "max_memory_mb".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(1)),
    );
    config.values.insert(
        "strategy".to_string(),
        serde_yaml::Value::String("Memory".to_string()),
    );

    // Create some JSON data
    let input = OxiData::from_json(json!([
        {"data": "x".repeat(1000)},
        {"data": "y".repeat(1000)},
        {"data": "z".repeat(1000)}
    ]));

    let result = batch_oxi.process(input, &config).await.unwrap();

    // Should create batches based on memory limits
    if let Data::Json(serde_json::Value::Array(batches)) = result.data() {
        assert!(batches.len() > 0);
        // Memory strategy should create multiple batches due to size
        println!("Created {} batches with memory strategy", batches.len());
    } else {
        panic!("Expected batched JSON array");
    }
}

#[tokio::test]
async fn test_batch_single_item() {
    let batch_oxi = Batch;
    let config = OxiConfig::default();

    // Test with single JSON object (not array)
    let input = OxiData::from_json(json!({"name": "test", "value": 42}));
    let result = batch_oxi.process(input, &config).await.unwrap();

    if let Data::Json(serde_json::Value::Array(batches)) = result.data() {
        assert_eq!(batches.len(), 1); // Should have 1 batch

        // The batch should contain the single item wrapped in an array
        if let serde_json::Value::Array(first_batch) = &batches[0] {
            assert_eq!(first_batch.len(), 1);
            assert_eq!(first_batch[0], json!({"name": "test", "value": 42}));
        }
    } else {
        panic!("Expected batched JSON array");
    }
}

#[tokio::test]
async fn test_batch_text_processing() {
    let batch_oxi = Batch;
    let mut config = OxiConfig::default();
    config.values.insert(
        "batch_size".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(2)),
    );
    config.values.insert(
        "strategy".to_string(),
        serde_yaml::Value::String("Size".to_string()),
    );

    // Test with text input
    let input = OxiData::from_text("line1\nline2\nline3\nline4\nline5".to_string());
    let result = batch_oxi.process(input, &config).await.unwrap();

    if let Data::Text(batched_text) = result.data() {
        assert!(batched_text.contains("---BATCH---"));

        let batches: Vec<&str> = batched_text.split("---BATCH---").collect();
        assert_eq!(batches.len(), 3); // Should have 3 batches

        // First batch should have 2 lines
        let first_batch_lines: Vec<&str> = batches[0].lines().collect();
        assert_eq!(first_batch_lines.len(), 2);
    } else {
        panic!("Expected batched text");
    }
}

#[tokio::test]
async fn test_batch_empty_data() {
    let batch_oxi = Batch;
    let config = OxiConfig::default();

    let input = OxiData::empty();
    let result = batch_oxi.process(input, &config).await.unwrap();

    // Empty data should pass through unchanged
    assert!(matches!(result.data(), Data::Empty));
}

#[tokio::test]
async fn test_batch_binary_data() {
    let batch_oxi = Batch;
    let mut config = OxiConfig::default();
    config.values.insert(
        "batch_size".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(2)),
    ); // 2KB chunks
    config.values.insert(
        "strategy".to_string(),
        serde_yaml::Value::String("Size".to_string()),
    );

    let binary_data = vec![1u8; 5000]; // 5KB of data
    let input = OxiData::from_binary(binary_data.clone());
    let result = batch_oxi.process(input, &config).await.unwrap();

    if let Data::Binary(batched_data) = result.data() {
        assert_eq!(batched_data.len(), binary_data.len()); // Should have same total size
        println!(
            "Binary data batched successfully: {} bytes",
            batched_data.len()
        );
    } else {
        panic!("Expected batched binary data");
    }
}

#[tokio::test]
async fn test_batch_size_or_time_strategy() {
    let batch_oxi = Batch;
    let mut config = OxiConfig::default();
    config.values.insert(
        "batch_size".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(5)),
    );
    config.values.insert(
        "flush_interval_ms".to_string(),
        serde_yaml::Value::Number(serde_yaml::Number::from(100)),
    );
    config.values.insert(
        "strategy".to_string(),
        serde_yaml::Value::String("SizeOrTime".to_string()),
    );

    // Test with small array (should batch by size)
    let input = OxiData::from_json(json!([1, 2, 3, 4, 5, 6, 7]));
    let result = batch_oxi.process(input, &config).await.unwrap();

    if let Data::Json(serde_json::Value::Array(batches)) = result.data() {
        assert_eq!(batches.len(), 2); // Should have 2 batches (5 + 2)
    } else {
        panic!("Expected batched JSON array");
    }
}

#[tokio::test]
async fn test_batch_schema_passthrough() {
    let batch_oxi = Batch;
    let config = OxiConfig::default();

    // Create input with schema
    let input = OxiData::from_json(json!([{"name": "test"}]));
    let original_schema = input.schema().clone();

    let result = batch_oxi.process(input, &config).await.unwrap();

    // Schema should pass through unchanged
    assert_eq!(
        result.schema().metadata.version,
        original_schema.metadata.version
    );
    assert_eq!(result.schema().fields.len(), original_schema.fields.len());
}

#[tokio::test]
async fn test_batch_processing_limits() {
    let batch_oxi = Batch;
    let limits = batch_oxi.processing_limits();

    // Verify the batch Oxi has appropriate processing limits
    assert_eq!(limits.max_batch_size, Some(10000));
    assert_eq!(limits.max_memory_mb, Some(1024));
    assert_eq!(limits.max_processing_time_ms, Some(300000));
    assert!(limits.supported_input_types.contains(&OxiDataType::Json));
    assert!(limits.supported_input_types.contains(&OxiDataType::Text));
    assert!(limits.supported_input_types.contains(&OxiDataType::Binary));
}

#[tokio::test]
async fn test_batch_config_schema() {
    let batch_oxi = Batch;
    let schema = batch_oxi.config_schema();

    // Verify the configuration schema is valid
    assert!(schema.is_mapping());

    if let serde_yaml::Value::Mapping(map) = schema {
        assert!(map.contains_key(&serde_yaml::Value::String("properties".to_string())));
    }
}

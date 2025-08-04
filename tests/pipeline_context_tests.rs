use oxide_flow::config::PipelineContext;
use std::collections::HashMap;

#[test]
fn test_pipeline_context_step_references() {
    let mut context = PipelineContext::new();

    // Add some mock step output
    let output_data = serde_yaml::from_str(
        r#"
path: "/tmp/processed_file.csv"
rows: 150
metadata:
  timestamp: "2025-08-01T10:30:00Z"
  size_bytes: 45000
"#,
    )
    .unwrap();

    context.add_step_output("reader", output_data);

    // Add some metadata
    let mut metadata = HashMap::new();
    metadata.insert(
        "execution_time".to_string(),
        serde_yaml::Value::Number(250.into()),
    );
    metadata.insert(
        "memory_used".to_string(),
        serde_yaml::Value::String("128MB".to_string()),
    );
    context.add_step_metadata("reader", metadata);

    // Test simple property reference
    let input1 = "The file is located at ${reader.path}";
    let result1 = context.resolve_step_references(input1).unwrap();
    assert_eq!(result1, "The file is located at /tmp/processed_file.csv");

    // Test nested property reference
    let input2 = "File size: ${reader.metadata.size_bytes} bytes";
    let result2 = context.resolve_step_references(input2).unwrap();
    assert_eq!(result2, "File size: 45000 bytes");

    // Test metadata reference
    let input3 = "Execution took ${reader.metadata.execution_time}ms";
    let result3 = context.resolve_step_references(input3).unwrap();
    assert_eq!(result3, "Execution took 250ms");

    // Test multiple references in one string
    let input4 = "Processed ${reader.rows} rows from ${reader.path}";
    let result4 = context.resolve_step_references(input4).unwrap();
    assert_eq!(result4, "Processed 150 rows from /tmp/processed_file.csv");
}

#[test]
fn test_pipeline_context_missing_references() {
    let context = PipelineContext::new();

    // Test reference to non-existent step
    let input = "File: ${missing_step.path}";
    let result = context.resolve_step_references(input);
    assert!(result.is_err());

    // Test reference to non-existent property
    let mut context = PipelineContext::new();
    let output_data = serde_yaml::from_str(
        r#"
path: "/tmp/test.csv"
"#,
    )
    .unwrap();
    context.add_step_output("reader", output_data);

    let input = "Count: ${reader.missing_property}";
    let result = context.resolve_step_references(input);
    assert!(result.is_err());
}

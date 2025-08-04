use crate::oxis::prelude::*;
use ::csv::{ReaderBuilder, WriterBuilder};
use async_trait::async_trait;

/// ParseCsv parses text input as CSV data
pub struct ParseCsv;

#[async_trait]
impl Oxi for ParseCsv {
    fn name(&self) -> &str {
        "parse_csv"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
            type: object
            properties:
              delimiter:
                type: string
                description: "Field delimiter character"
                default: ","
              has_headers:
                type: boolean
                description: "Whether the first row contains headers"
                default: true
        "#,
        )
        .unwrap()
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Get text from input
        let text = input.as_text()?;

        // Get configuration
        let delimiter = config.get_string_or("delimiter", ",");
        let has_headers = config.get_bool_or("has_headers", true);

        // Parse CSV
        let delimiter_char = delimiter.chars().next().unwrap_or(',');
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter_char as u8)
            .has_headers(has_headers)
            .from_reader(text.as_bytes());

        // Get headers if available
        let headers = if has_headers {
            reader
                .headers()?
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        } else {
            // Generate numeric headers
            let first_record = reader.records().next();
            if let Some(Ok(record)) = first_record {
                (0..record.len()).map(|i| format!("column_{i}")).collect()
            } else {
                return Ok(OxiData::Json(serde_json::Value::Array(vec![])));
            }
        };

        // Process records into JSON array of objects
        let mut json_array = Vec::new();
        for result in reader.records() {
            let record = result?;
            let mut json_object = serde_json::Map::new();

            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    let value = parse_csv_field(field);
                    json_object.insert(headers[i].clone(), value);
                }
            }

            json_array.push(serde_json::Value::Object(json_object));
        }

        Ok(OxiData::Json(serde_json::Value::Array(json_array)))
    }
}

/// Parse a CSV field into the appropriate JSON value type
fn parse_csv_field(field: &str) -> serde_json::Value {
    // Try to parse as number
    if let Ok(i) = field.parse::<i64>() {
        return serde_json::Value::Number(serde_json::Number::from(i));
    }

    // Try to parse as floating point
    if let Ok(f) = field.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(f) {
            return serde_json::Value::Number(num);
        }
    }

    // Try to parse as boolean
    match field.to_lowercase().as_str() {
        "true" => return serde_json::Value::Bool(true),
        "false" => return serde_json::Value::Bool(false),
        _ => {}
    }

    // Handle empty values
    if field.is_empty() {
        return serde_json::Value::Null;
    }

    // Default to string
    serde_json::Value::String(field.to_string())
}

/// FormatCsv formats JSON array data as CSV
pub struct FormatCsv;

#[async_trait]
impl Oxi for FormatCsv {
    fn name(&self) -> &str {
        "format_csv"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
            type: object
            properties:
              delimiter:
                type: string
                description: "Field delimiter character"
                default: ","
              include_headers:
                type: boolean
                description: "Whether to include headers from structured data"
                default: true
        "#,
        )
        .unwrap()
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Get JSON array from input
        let json_array = input.as_json()?;

        let array = match json_array {
            serde_json::Value::Array(arr) => arr,
            _ => return Err(anyhow::anyhow!("FormatCsv requires a JSON array input")),
        };

        if array.is_empty() {
            return Ok(OxiData::Text(String::new()));
        }

        // Get configuration
        let delimiter = config.get_string_or("delimiter", ",");
        let delimiter_char = delimiter.chars().next().unwrap_or(',');
        let include_headers = config.get_bool_or("include_headers", true);

        // Extract headers from first object
        let headers: Vec<String> = if let Some(first_obj) = array.first() {
            if let serde_json::Value::Object(obj) = first_obj {
                obj.keys().cloned().collect()
            } else {
                return Err(anyhow::anyhow!("FormatCsv requires array of JSON objects"));
            }
        } else {
            Vec::new()
        };

        // Format as CSV
        let mut writer = WriterBuilder::new()
            .delimiter(delimiter_char as u8)
            .from_writer(Vec::new());

        // Write headers if requested
        if include_headers && !headers.is_empty() {
            writer.write_record(&headers)?;
        }

        // Write data rows
        for item in array {
            if let serde_json::Value::Object(obj) = item {
                let row: Vec<String> = headers
                    .iter()
                    .map(|header| match obj.get(header) {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(serde_json::Value::Number(n)) => n.to_string(),
                        Some(serde_json::Value::Bool(b)) => b.to_string(),
                        Some(serde_json::Value::Null) => String::new(),
                        Some(other) => other.to_string(),
                        None => String::new(),
                    })
                    .collect();
                writer.write_record(&row)?;
            }
        }

        // Get the CSV as a string
        let csv_data = String::from_utf8(writer.into_inner()?)?;

        Ok(OxiData::Text(csv_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_csv() {
        let oxi = ParseCsv;
        let input = OxiData::Text("name,value\ntest,123\nother,456".to_string());
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Json(json_value) = result {
            if let serde_json::Value::Array(array) = json_value {
                assert_eq!(array.len(), 2);

                // Check first row
                if let serde_json::Value::Object(first_row) = &array[0] {
                    assert_eq!(
                        first_row.get("name").unwrap(),
                        &serde_json::Value::String("test".to_string())
                    );
                    assert_eq!(
                        first_row.get("value").unwrap(),
                        &serde_json::Value::Number(serde_json::Number::from(123))
                    );
                }

                // Check second row
                if let serde_json::Value::Object(second_row) = &array[1] {
                    assert_eq!(
                        second_row.get("name").unwrap(),
                        &serde_json::Value::String("other".to_string())
                    );
                    assert_eq!(
                        second_row.get("value").unwrap(),
                        &serde_json::Value::Number(serde_json::Number::from(456))
                    );
                }
            } else {
                panic!("Expected JSON array");
            }
        } else {
            panic!("Expected JSON data");
        }
    }

    #[tokio::test]
    async fn test_format_csv() {
        let oxi = FormatCsv;

        // Create JSON array of objects
        let json_data = serde_json::json!([
            {"name": "test", "value": 123},
            {"name": "other", "value": 456}
        ]);

        let input = OxiData::Json(json_data);
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Text(text) = result {
            assert!(text.contains("name,value"));
            assert!(text.contains("test,123"));
            assert!(text.contains("other,456"));
        } else {
            panic!("Expected text data");
        }
    }
}

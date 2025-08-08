use crate::oxis::prelude::*;
use async_trait::async_trait;

/// FormatJson formats structured data as JSON
pub struct FormatJson;

#[async_trait]
impl Oxi for FormatJson {
    fn name(&self) -> &str {
        "format_json"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
            type: object
            properties:
              pretty:
                type: boolean
                description: "Whether to format JSON with indentation"
                default: true
              indent:
                type: integer
                description: "Number of spaces for indentation"
                default: 2
        "#,
        )
        .unwrap()
    }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Passthrough
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        // Get JSON data from input
        let json_value = input
            .data()
            .as_json()
            .map_err(|_e| OxiError::TypeMismatch {
                expected: "JSON".to_string(),
                actual: input.data().data_type().to_string(),
                step: "format_json".to_string(),
            })?;

        // Get configuration
        let pretty = config.get_bool_or("pretty", false);

        // Format as JSON string
        let json_string = if pretty {
            serde_json::to_string_pretty(json_value)
        } else {
            serde_json::to_string(json_value)
        }
        .map_err(|e| OxiError::ValidationError {
            details: format!("Failed to serialize JSON: {e}"),
        })?;

        // Return as text data with original schema (passthrough strategy)
        Ok(OxiData::with_schema(
            Data::Text(json_string),
            input.schema.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_format_json() {
        let oxi = FormatJson;
        let config = OxiConfig::default();

        // Create a JSON value
        let json_value = serde_json::json!({
            "name": "test",
            "value": 123
        });

        let input = OxiData::from_json(json_value);

        let result = oxi.process(input, &config).await.unwrap();

        if let Data::Text(text) = &result.data {
            assert!(text.contains("\"name\":\"test\""));
            assert!(text.contains("\"value\":123"));
        } else {
            panic!("Expected text data");
        }
    }
}

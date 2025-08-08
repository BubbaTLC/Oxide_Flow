use crate::oxis::prelude::*;
use async_trait::async_trait;

/// ParseJson parses text input as JSON
pub struct ParseJson;

#[async_trait]
impl Oxi for ParseJson {
    fn name(&self) -> &str {
        "parse_json"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
            type: object
            properties:
              flatten:
                type: boolean
                description: "Whether to flatten nested JSON objects"
                default: false
        "#,
        )
        .unwrap()
    }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Modify {
            description: "Converts text data to JSON, infers JSON structure schema".to_string(),
        }
    }

    async fn process(&self, input: OxiData, _config: &OxiConfig) -> Result<OxiData, OxiError> {
        match &input.data {
            Data::Text(text) => {
                // Parse JSON from text
                let json_value: serde_json::Value =
                    serde_json::from_str(text).map_err(|e| OxiError::ValidationError {
                        details: format!("Failed to parse JSON: {e}"),
                    })?;

                // Create new data with inferred schema
                Ok(OxiData::from_json(json_value))
            }
            Data::Json(json) => {
                // If it's a structured object with content field, extract just the content
                if let Some(content) = json.get("content") {
                    if let Some(content_str) = content.as_str() {
                        let json_value: serde_json::Value = serde_json::from_str(content_str)
                            .map_err(|e| OxiError::ValidationError {
                                details: format!("Failed to parse nested JSON: {e}"),
                            })?;
                        Ok(OxiData::from_json(json_value))
                    } else {
                        // Content is already JSON
                        Ok(OxiData::from_json(content.clone()))
                    }
                } else {
                    // No content field, return as-is
                    Ok(input) // Pass through with existing schema
                }
            }
            _ => Err(OxiError::TypeMismatch {
                expected: "Text or JSON".to_string(),
                actual: input.data().data_type().to_string(),
                step: "parse_json".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_json() {
        let oxi = ParseJson;
        let config = OxiConfig::default();
        let input = OxiData::from_text(r#"{"name":"test","value":123}"#.to_string());

        let result = oxi.process(input, &config).await.unwrap();

        if let Data::Json(value) = &result.data {
            assert_eq!(value["name"].as_str(), Some("test"));
            assert_eq!(value["value"].as_i64(), Some(123));
        } else {
            panic!("Expected JSON data");
        }
    }
}

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

    async fn process(&self, input: OxiData, _config: &OxiConfig) -> anyhow::Result<OxiData> {
        match input {
            OxiData::Text(text) => {
                // Parse JSON from text
                let json_value: serde_json::Value = serde_json::from_str(&text)?;
                Ok(OxiData::Json(json_value))
            }
            OxiData::Json(json) => {
                // If it's a structured object with content field, extract just the content
                if let Some(content) = json.get("content") {
                    if let Some(content_str) = content.as_str() {
                        let json_value: serde_json::Value = serde_json::from_str(content_str)?;
                        Ok(OxiData::Json(json_value))
                    } else {
                        // Content is already JSON
                        Ok(OxiData::Json(content.clone()))
                    }
                } else {
                    // No content field, return as-is
                    Ok(OxiData::Json(json))
                }
            }
            _ => {
                anyhow::bail!("ParseJson requires text or JSON input")
            }
        }
    }
}

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

    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Get JSON data from input
        let json_value = input.as_json()?;

        // Get configuration
        let pretty = config.get_bool_or("pretty", false);

        // Format as JSON string
        let json_string = if pretty {
            serde_json::to_string_pretty(json_value)?
        } else {
            serde_json::to_string(json_value)?
        };

        Ok(OxiData::Text(json_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_json() {
        let oxi = ParseJson;
        let input = OxiData::Text(r#"{"name":"test","value":123}"#.to_string());
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Json(value) = result {
            assert_eq!(value["name"].as_str(), Some("test"));
            assert_eq!(value["value"].as_i64(), Some(123));
        } else {
            panic!("Expected JSON data");
        }
    }

    #[tokio::test]
    async fn test_format_json() {
        let oxi = FormatJson;

        // Create a JSON value
        let json_value = serde_json::json!({
            "name": "test",
            "value": 123
        });

        let input = OxiData::Json(json_value);
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Text(text) = result {
            assert!(text.contains("\"name\":\"test\""));
            assert!(text.contains("\"value\":123"));
        } else {
            panic!("Expected text data");
        }
    }
}

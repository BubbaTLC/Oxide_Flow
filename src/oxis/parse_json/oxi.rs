use crate::oxis::prelude::*;
use crate::types::OxiDataWithSchema;
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

    async fn process(
        &self,
        data_with_schema: OxiDataWithSchema,
        config: &OxiConfig,
    ) -> anyhow::Result<OxiDataWithSchema> {
        // Validate input data if schema is present
        if let Some(schema) = &data_with_schema.schema {
            schema
                .validate_data(&data_with_schema.data)
                .map_err(anyhow::Error::from)?;
        }

        // Process the actual data
        let output_data = self.process_data(data_with_schema.data, config).await?;

        // Calculate output schema
        let output_schema = self.output_schema(data_with_schema.schema.as_ref(), config)?;

        Ok(OxiDataWithSchema::new(output_data, output_schema))
    }

    async fn process_data(&self, input: OxiData, _config: &OxiConfig) -> anyhow::Result<OxiData> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_json() {
        let oxi = ParseJson;
        let config = OxiConfig::default();
        let input = OxiDataWithSchema::from_data(OxiData::Text(
            r#"{"name":"test","value":123}"#.to_string(),
        ));

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Json(value) = result.data {
            assert_eq!(value["name"].as_str(), Some("test"));
            assert_eq!(value["value"].as_i64(), Some(123));
        } else {
            panic!("Expected JSON data");
        }
    }
}

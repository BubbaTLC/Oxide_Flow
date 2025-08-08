use crate::oxis::prelude::*;
use crate::types::OxiDataWithSchema;
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

    async fn process_data(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
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
    async fn test_format_json() {
        let oxi = FormatJson;
        let config = OxiConfig::default();

        // Create a JSON value
        let json_value = serde_json::json!({
            "name": "test",
            "value": 123
        });

        let input = OxiDataWithSchema::from_data(OxiData::Json(json_value));

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Text(text) = result.data {
            assert!(text.contains("\"name\":\"test\""));
            assert!(text.contains("\"value\":123"));
        } else {
            panic!("Expected text data");
        }
    }
}

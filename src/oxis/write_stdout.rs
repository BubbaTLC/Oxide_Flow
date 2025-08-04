use crate::oxis::prelude::*;
use crate::types::{OxiData, OxiConfig};
use async_trait::async_trait;

pub struct WriteStdOut;

#[async_trait]
impl Oxi for WriteStdOut {
    fn name(&self) -> &str {
        "write_stdout"
    }
    
    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
            type: object
            properties:
              format:
                type: string
                enum: [auto, text, json, yaml]
                description: "Output format"
                default: auto
        "#).unwrap()
    }
    
    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        let format = config.get_string_or("format", "auto");
        
        match format.as_str() {
            "text" => {
                let text = input.as_text()?;
                println!("{}", text);
            },
            "json" => {
                let value = input.as_json()?;
                let json = serde_json::to_string_pretty(&value)?;
                println!("{}", json);
            },
            "yaml" => {
                let value = input.as_json()?;
                let yaml = serde_yaml::to_string(&value)?;
                println!("{}", yaml);
            },
            _ => {
                // Auto-detect based on input type
                match &input {
                    OxiData::Text(text) => println!("{}", text),
                    OxiData::Json(value) => {
                        let json = serde_json::to_string_pretty(&value)?;
                        println!("{}", json);
                    },
                    OxiData::Binary(data) => {
                        println!("<Binary data: {} bytes>", data.len());
                    },
                    OxiData::Empty => {},
                }
            }
        }
        
        // Return the input data unchanged
        Ok(input)
    }
}

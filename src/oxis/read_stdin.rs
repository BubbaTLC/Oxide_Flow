use crate::oxis::prelude::*;
use crate::types::{OxiConfig, OxiData};
use async_trait::async_trait;
use tokio::io::{self, AsyncReadExt};

pub struct ReadStdIn;

#[async_trait]
impl Oxi for ReadStdIn {
    fn name(&self) -> &str {
        "read_stdin"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
            type: object
            properties:
              binary:
                type: boolean
                description: "Whether to read input as binary"
                default: false
        "#,
        )
        .unwrap()
    }

    async fn process(&self, _input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        let is_binary = config.get_bool_or("binary", false);

        if is_binary {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer).await?;
            Ok(OxiData::Binary(buffer))
        } else {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).await?;
            Ok(OxiData::Text(buffer))
        }
    }
}

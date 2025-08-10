use crate::oxis::prelude::*;
use async_trait::async_trait;
use serde_json::Value;

/// JsonSelect extracts data from JSON using path expressions
pub struct JsonSelect;

#[async_trait]
impl Oxi for JsonSelect {
    fn name(&self) -> &str {
        "json_select"
    }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Modify {
            description:
                "Selects JSON data using path expressions like '[0].users' or 'data.items'"
                    .to_string(),
        }
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
            type: object
            properties:
              path:
                type: string
                description: "JSON path selector (e.g., '[0].users', 'data.items', 'users[1].profile')"
              strict:
                type: boolean
                default: true
                description: "Fail if path is not found (true) or return default value (false)"
              default_on_missing:
                description: "Default value when path is missing and strict=false"
            required:
              - path
        "#).unwrap()
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        let json_data = input.data().as_json().map_err(|_| OxiError::TypeMismatch {
            expected: "JSON".to_string(),
            actual: input.data().data_type().to_string(),
            step: "json_select".to_string(),
        })?;

        let path = config.get_string("path").map_err(|_| {
            OxiError::ConfigError("Missing required 'path' configuration".to_string())
        })?;

        let strict = config.get_bool("strict").unwrap_or(true);

        match select_json_path(json_data, &path) {
            Ok(selected_data) => Ok(OxiData::from_json(selected_data)),
            Err(_e) if !strict => {
                if let Ok(default_value) = config.get_structured("default_on_missing") {
                    // Convert yaml value to json value
                    let json_value: serde_json::Value = serde_yaml::from_value(default_value)
                        .map_err(|_| {
                            OxiError::ConfigError("Invalid default_on_missing value".to_string())
                        })?;
                    Ok(OxiData::from_json(json_value))
                } else {
                    Ok(OxiData::empty())
                }
            }
            Err(e) => Err(OxiError::JsonOperationError {
                operation: format!("JSON path selection for path '{}'", path),
                details: e.to_string(),
            }),
        }
    }
}

/// JSON path selection implementation
fn select_json_path(data: &Value, path: &str) -> Result<Value, JsonPathError> {
    let mut current = data;
    let parts = parse_json_path(path)?;

    for part in parts {
        current = match part {
            PathPart::Index(i) => match current {
                Value::Array(arr) => arr.get(i).ok_or(JsonPathError::IndexOutOfBounds {
                    index: i,
                    array_len: arr.len(),
                })?,
                _ => {
                    return Err(JsonPathError::ExpectedArray {
                        actual: current.clone(),
                    })
                }
            },
            PathPart::Key(ref key) => match current {
                Value::Object(obj) => obj
                    .get(key)
                    .ok_or(JsonPathError::KeyNotFound { key: key.clone() })?,
                _ => {
                    return Err(JsonPathError::ExpectedObject {
                        actual: current.clone(),
                        key: key.clone(),
                    })
                }
            },
        };
    }

    Ok(current.clone())
}

#[derive(Debug)]
enum PathPart {
    Index(usize),
    Key(String),
}

fn parse_json_path(path: &str) -> Result<Vec<PathPart>, JsonPathError> {
    let mut parts = Vec::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '[' => {
                // Parse array index: [0], [123]
                let mut index_str = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ']' {
                        chars.next(); // consume ']'
                        break;
                    }
                    index_str.push(chars.next().unwrap());
                }
                let index: usize = index_str
                    .parse()
                    .map_err(|_| JsonPathError::InvalidIndex { text: index_str })?;
                parts.push(PathPart::Index(index));
            }
            '.' => {
                // Parse object key: .users, .profile
                let mut key = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '.' || next_ch == '[' {
                        break;
                    }
                    key.push(chars.next().unwrap());
                }
                if !key.is_empty() {
                    parts.push(PathPart::Key(key));
                }
            }
            _ => {
                // Parse object key without leading dot: users, profile
                let mut key = String::new();
                key.push(ch);
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '.' || next_ch == '[' {
                        break;
                    }
                    key.push(chars.next().unwrap());
                }
                parts.push(PathPart::Key(key));
            }
        }
    }

    Ok(parts)
}

#[derive(Debug, thiserror::Error)]
pub enum JsonPathError {
    #[error("Array index {index} out of bounds (array length: {array_len})")]
    IndexOutOfBounds { index: usize, array_len: usize },

    #[error("Key '{key}' not found in object")]
    KeyNotFound { key: String },

    #[error("Expected array but got {actual}")]
    ExpectedArray { actual: Value },

    #[error("Expected object but got {actual} when looking for key '{key}'")]
    ExpectedObject { actual: Value, key: String },

    #[error("Invalid array index: '{text}' is not a valid number")]
    InvalidIndex { text: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_array_index_selection() {
        let oxi = JsonSelect;
        let input_data = json!([
            {"name": "first"},
            {"name": "second"}
        ]);
        let input = OxiData::from_json(input_data);

        let mut config = OxiConfig::default();
        config.set("path", "[0]".to_string()).unwrap();

        let result = oxi.process(input, &config).await.unwrap();
        let output = result.data().as_json().unwrap();

        assert_eq!(output, &json!({"name": "first"}));
    }

    #[tokio::test]
    async fn test_object_key_selection() {
        let oxi = JsonSelect;
        let input_data = json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ],
            "metadata": {"count": 2}
        });
        let input = OxiData::from_json(input_data);

        let mut config = OxiConfig::default();
        config.set("path", "users".to_string()).unwrap();

        let result = oxi.process(input, &config).await.unwrap();
        let output = result.data().as_json().unwrap();

        assert_eq!(
            output,
            &json!([
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ])
        );
    }

    #[tokio::test]
    async fn test_complex_path_selection() {
        let oxi = JsonSelect;
        let input_data = json!([{
            "users": [
                {"id": 1, "profile": {"name": "Alice", "age": 30}},
                {"id": 2, "profile": {"name": "Bob", "age": 25}}
            ]
        }]);
        let input = OxiData::from_json(input_data);

        let mut config = OxiConfig::default();
        config.set("path", "[0].users".to_string()).unwrap();

        let result = oxi.process(input, &config).await.unwrap();
        let output = result.data().as_json().unwrap();

        assert_eq!(
            output,
            &json!([
                {"id": 1, "profile": {"name": "Alice", "age": 30}},
                {"id": 2, "profile": {"name": "Bob", "age": 25}}
            ])
        );
    }

    #[tokio::test]
    async fn test_missing_path_strict_mode() {
        let oxi = JsonSelect;
        let input_data = json!({"existing": "data"});
        let input = OxiData::from_json(input_data);

        let mut config = OxiConfig::default();
        config.set("path", "nonexistent".to_string()).unwrap();
        config.set("strict", true).unwrap();

        let result = oxi.process(input, &config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_path_non_strict_mode() {
        let oxi = JsonSelect;
        let input_data = json!({"existing": "data"});
        let input = OxiData::from_json(input_data);

        let mut config = OxiConfig::default();
        config.set("path", "nonexistent".to_string()).unwrap();
        config.set("strict", false).unwrap();

        let result = oxi.process(input, &config).await.unwrap();
        assert!(matches!(result.data(), Data::Empty));
    }

    #[tokio::test]
    async fn test_schema_strategy() {
        let oxi = JsonSelect;
        match oxi.schema_strategy() {
            SchemaStrategy::Modify { description } => {
                assert!(description.contains("path expressions"));
            }
            _ => panic!("Expected Modify schema strategy"),
        }
    }

    #[test]
    fn test_json_path_parsing() {
        let parts = parse_json_path("[0].users[1].profile").unwrap();
        assert_eq!(parts.len(), 4);

        match &parts[0] {
            PathPart::Index(i) => assert_eq!(*i, 0),
            _ => panic!("Expected index"),
        }

        match &parts[1] {
            PathPart::Key(k) => assert_eq!(k, "users"),
            _ => panic!("Expected key"),
        }
    }
}

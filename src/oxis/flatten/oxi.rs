use crate::oxis::prelude::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// Flatten transforms nested structured data into a flattened format
pub struct Flatten;

#[async_trait]
impl Oxi for Flatten {
    fn name(&self) -> &str {
        "flatten"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
            type: object
            properties:
              delimiter:
                type: string
                description: "Delimiter to use when flattening nested keys"
                default: "_"
              array_mode:
                type: string
                enum: ["index", "explode", "ignore"]
                description: "How to handle arrays (index: include indices, explode: create row per item, ignore: skip arrays)"
                default: "explode"
        "#).unwrap()
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Get configuration
        let delimiter = config.get_string_or("delimiter", "_");
        let array_mode = config.get_string_or("array_mode", "explode");

        // Get JSON data from input
        let value = input.as_json()?;

        // Flatten the structure
        let flattened_result = if let serde_json::Value::Array(array) = &value {
            // Process array of objects
            let mut flattened_objects = Vec::new();
            for item in array {
                let flattened = flatten_json_value(item, &delimiter, &array_mode)?;
                flattened_objects.push(flattened);
            }
            serde_json::Value::Array(flattened_objects)
        } else {
            // Process single object
            flatten_json_value(value, &delimiter, &array_mode)?
        };

        Ok(OxiData::Json(flattened_result))
    }
}

// Flatten a JSON value into a flat JSON object
fn flatten_json_value(
    value: &serde_json::Value,
    delimiter: &str,
    array_mode: &str,
) -> anyhow::Result<serde_json::Value> {
    let mut result = serde_json::Map::new();
    flatten_json_recursive(value, "", delimiter, array_mode, &mut result)?;
    Ok(serde_json::Value::Object(result))
}

// Recursively flatten a JSON value
fn flatten_json_recursive(
    value: &serde_json::Value,
    prefix: &str,
    delimiter: &str,
    array_mode: &str,
    result: &mut serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}{delimiter}{key}")
                };

                flatten_json_recursive(val, &new_prefix, delimiter, array_mode, result)?;
            }
        }
        serde_json::Value::Array(arr) => {
            if array_mode == "index" {
                for (i, item) in arr.iter().enumerate() {
                    let new_prefix = format!("{prefix}{delimiter}{i}");
                    flatten_json_recursive(item, &new_prefix, delimiter, array_mode, result)?;
                }
            } else if array_mode == "explode" {
                // For explode mode, we join array values as a comma-separated string
                let values: Vec<String> = arr
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Null => "null".to_string(),
                        _ => v.to_string(),
                    })
                    .collect();

                result.insert(
                    prefix.to_string(),
                    serde_json::Value::String(values.join(",")),
                );
            }
            // For "ignore" mode, we skip arrays entirely
        }
        _ => {
            // Insert primitive values directly
            result.insert(prefix.to_string(), value.clone());
        }
    }
    Ok(())
}

// Legacy YAML-based flatten functions (keeping for backward compatibility)

// Flatten a nested structure into a HashMap with flattened keys
#[allow(dead_code)]
fn flatten_structure(
    value: &serde_yaml::Value,
    delimiter: &str,
    array_mode: &str,
) -> anyhow::Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    flatten_value(value, "", delimiter, array_mode, &mut result)?;
    Ok(result)
}

// Recursively flatten a value
#[allow(dead_code)]
fn flatten_value(
    value: &serde_yaml::Value,
    prefix: &str,
    delimiter: &str,
    array_mode: &str,
    result: &mut HashMap<String, String>,
) -> anyhow::Result<()> {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let key = if let serde_yaml::Value::String(key_str) = k {
                    key_str.clone()
                } else {
                    k.as_str().unwrap_or("unknown").to_string()
                };

                let new_prefix = if prefix.is_empty() {
                    key
                } else {
                    format!("{prefix}{delimiter}{key}")
                };

                flatten_value(v, &new_prefix, delimiter, array_mode, result)?;
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            if array_mode == "index" {
                for (i, item) in seq.iter().enumerate() {
                    let new_prefix = format!("{prefix}{delimiter}{i}");
                    flatten_value(item, &new_prefix, delimiter, array_mode, result)?;
                }
            } else if array_mode == "explode" {
                // For explode mode, we join array values with commas
                let values: Vec<String> = seq
                    .iter()
                    .map(|v| match v {
                        serde_yaml::Value::String(s) => Ok(s.clone()),
                        _ => serde_yaml::to_string(v).map_err(anyhow::Error::from),
                    })
                    .collect::<anyhow::Result<Vec<String>>>()?;

                result.insert(prefix.to_string(), values.join(","));
            }
            // For "ignore" mode, we skip arrays entirely
        }
        serde_yaml::Value::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        serde_yaml::Value::Number(n) => {
            result.insert(prefix.to_string(), n.to_string());
        }
        serde_yaml::Value::Bool(b) => {
            result.insert(prefix.to_string(), b.to_string());
        }
        serde_yaml::Value::Null => {
            result.insert(prefix.to_string(), "".to_string());
        }
        _ => {
            // Skip unknown types
        }
    }

    Ok(())
}

// Collect all unique keys from flattened data
#[allow(dead_code)]
fn collect_all_keys(flattened: &HashMap<String, String>) -> Vec<String> {
    let mut keys: Vec<String> = flattened.keys().cloned().collect();
    keys.sort();
    keys
}

// Create a row for tabular output using the flattened data
#[allow(dead_code)]
fn create_row_from_flattened(
    flattened: &HashMap<String, String>,
    headers: &[String],
) -> Vec<String> {
    headers
        .iter()
        .map(|header| flattened.get(header).cloned().unwrap_or_default())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flatten_nested_object() {
        let oxi = Flatten;

        // Create test JSON data
        let json_data = serde_json::json!({
            "name": "John",
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            }
        });

        let input = OxiData::Json(json_data);
        let config = OxiConfig::default();

        let result = oxi.process(input, &config).await.unwrap();

        if let OxiData::Json(json_result) = result {
            if let serde_json::Value::Object(obj) = json_result {
                assert!(obj.contains_key("name"));
                assert!(obj.contains_key("address_street"));
                assert!(obj.contains_key("address_city"));

                assert_eq!(obj["name"], serde_json::Value::String("John".to_string()));
                assert_eq!(
                    obj["address_street"],
                    serde_json::Value::String("123 Main St".to_string())
                );
                assert_eq!(
                    obj["address_city"],
                    serde_json::Value::String("Anytown".to_string())
                );
            } else {
                panic!("Expected JSON object");
            }
        } else {
            panic!("Expected JSON data");
        }
    }
}

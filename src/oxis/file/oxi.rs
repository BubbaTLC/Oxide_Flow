use crate::oxis::prelude::*;
use async_trait::async_trait;
use std::fs;
use std::path::Path;

/// ReadFile reads content from a file
pub struct ReadFile;

#[async_trait]
impl Oxi for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }
    
    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
            type: object
            properties:
              path:
                type: string
                description: "Path to the file to read"
                required: true
              encoding:
                type: string
                description: "File encoding (utf-8, etc.)"
                default: "utf-8"
        "#).unwrap()
    }
    
    async fn process(&self, _input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Get file path from config
        let path = config.get_string("path")?;
        
        // Check if file exists
        if !Path::new(&path).exists() {
            anyhow::bail!("File not found: {}", path);
        }
        
        // Read file content
        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path, e))?;
        
        Ok(OxiData::Text(content))
    }
}

/// WriteFile writes content to a file
pub struct WriteFile;

#[async_trait]
impl Oxi for WriteFile {
    fn name(&self) -> &str {
        "write_file"
    }
    
    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
            type: object
            properties:
              path:
                type: string
                description: "Path to the output file"
                required: true
              create_dirs:
                type: boolean
                description: "Create parent directories if they don't exist"
                default: true
              append:
                type: boolean
                description: "Append to file instead of overwriting"
                default: false
        "#).unwrap()
    }
    
    async fn process(&self, input: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Get file path from config
        let path = config.get_string("path")?;
        let create_dirs = config.get_bool_or("create_dirs", true);
        let append = config.get_bool_or("append", false);
        
        // Create parent directories if needed
        if create_dirs {
            if let Some(parent) = Path::new(&path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| anyhow::anyhow!("Failed to create directories for '{}': {}", path, e))?;
            }
        }
        
        // Convert input to text
        let content = input.to_text()?;
        
        // Write to file
        if append {
            fs::write(&path, content)
                .map_err(|e| anyhow::anyhow!("Failed to append to file '{}': {}", path, e))?;
        } else {
            fs::write(&path, content)
                .map_err(|e| anyhow::anyhow!("Failed to write to file '{}': {}", path, e))?;
        }
        
        // Return the input unchanged for potential chaining
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_read_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let content = "Hello, World!";
        
        // Create test file
        fs::write(&file_path, content).unwrap();
        
        // Test reading
        let oxi = ReadFile;
        let mut config = OxiConfig::default();
        config.values.insert("path".to_string(), serde_yaml::Value::String(file_path.to_string_lossy().to_string()));
        
        let result = oxi.process(OxiData::Empty, &config).await.unwrap();
        
        assert_eq!(result.as_text().unwrap(), content);
    }
    
    #[tokio::test]
    async fn test_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("output.txt");
        let content = "Test output";
        
        // Test writing
        let oxi = WriteFile;
        let mut config = OxiConfig::default();
        config.values.insert("path".to_string(), serde_yaml::Value::String(file_path.to_string_lossy().to_string()));
        
        let input = OxiData::Text(content.to_string());
        let result = oxi.process(input, &config).await.unwrap();
        
        // Verify file was written
        let written_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written_content, content);
        
        // Verify input was passed through
        assert_eq!(result.as_text().unwrap(), content);
    }
}

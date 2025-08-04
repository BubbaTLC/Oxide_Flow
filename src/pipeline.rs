use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Pipeline configuration loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// List of pipeline steps
    pub pipeline: Vec<PipelineStep>,
    
    /// Pipeline metadata
    pub metadata: Option<PipelineMetadata>,
}

/// A single step in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Name of the Oxi to execute
    pub name: String,
    
    /// Optional ID for this step (for referencing)
    pub id: Option<String>,
    
    /// Configuration for this step
    #[serde(default)]
    pub config: HashMap<String, serde_yaml::Value>,
}

/// Pipeline metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetadata {
    /// Pipeline name
    pub name: Option<String>,
    
    /// Pipeline description
    pub description: Option<String>,
    
    /// Pipeline version
    pub version: Option<String>,
    
    /// Pipeline author
    pub author: Option<String>,
}

impl Pipeline {
    /// Load a pipeline from a YAML file
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read pipeline file '{}': {}", path, e))?;
        
        let pipeline: Pipeline = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse pipeline YAML '{}': {}", path, e))?;
        
        Ok(pipeline)
    }
    
    /// Get the number of steps in this pipeline
    pub fn step_count(&self) -> usize {
        self.pipeline.len()
    }
    
    /// Get pipeline name from metadata or default
    pub fn name(&self) -> String {
        self.metadata
            .as_ref()
            .and_then(|m| m.name.as_ref())
            .map(|s| s.clone())
            .unwrap_or_else(|| "Unnamed Pipeline".to_string())
    }
    
    /// Get pipeline description from metadata
    pub fn description(&self) -> Option<String> {
        self.metadata
            .as_ref()
            .and_then(|m| m.description.as_ref())
            .map(|s| s.clone())
    }
}

impl PipelineStep {
    /// Get the step ID, using the name as fallback
    pub fn get_id(&self) -> &str {
        self.id.as_ref().unwrap_or(&self.name)
    }
    
    /// Convert config HashMap to OxiConfig
    pub fn to_oxi_config(&self) -> crate::types::OxiConfig {
        let mut oxi_config = crate::types::OxiConfig::default();
        for (key, value) in &self.config {
            oxi_config.values.insert(key.clone(), value.clone());
        }
        oxi_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_load_pipeline() {
        let yaml_content = r#"
pipeline:
  - name: read_file
    id: reader
    config:
      path: "input.json"
  - name: parse_json
    id: parser
  - name: format_csv
    id: formatter
    config:
      headers: true
      delimiter: ","
  - name: write_file
    id: writer
    config:
      path: "output.csv"

metadata:
  name: "Test Pipeline"
  description: "A test pipeline"
  version: "1.0.0"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();
        
        let pipeline = Pipeline::load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(pipeline.step_count(), 4);
        assert_eq!(pipeline.name(), "Test Pipeline");
        assert_eq!(pipeline.pipeline[0].name, "read_file");
        assert_eq!(pipeline.pipeline[0].get_id(), "reader");
    }
}

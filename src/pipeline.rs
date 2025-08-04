use crate::config_resolver::ConfigResolver;
use crate::oxis::csv::oxi::FormatCsv;
use crate::oxis::file::oxi::{ReadFile, WriteFile};
use crate::oxis::flatten::oxi::Flatten;
use crate::oxis::json::oxi::ParseJson;
use crate::oxis::read_stdin::ReadStdIn;
use crate::oxis::write_stdout::WriteStdOut;
use crate::types::OxiData;
use crate::Oxi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tokio::time::{timeout, Duration};

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

    /// Whether to continue pipeline execution if this step fails
    #[serde(default)]
    pub continue_on_error: bool,

    /// Maximum number of retry attempts
    #[serde(default)]
    pub retry_attempts: u32,

    /// Timeout in seconds for this step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

/// Result of a pipeline step execution
#[derive(Debug)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub data: Option<OxiData>,
    pub error: Option<String>,
    pub retry_count: u32,
    pub duration_ms: u64,
}

/// Overall pipeline execution result
#[derive(Debug)]
pub struct PipelineResult {
    pub success: bool,
    pub steps_executed: u32,
    pub steps_failed: u32,
    pub steps_skipped: u32,
    pub total_duration_ms: u64,
    pub step_results: Vec<StepResult>,
    pub final_data: Option<OxiData>,
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
            .cloned()
            .unwrap_or_else(|| "Unnamed Pipeline".to_string())
    }

    /// Get pipeline description from metadata
    pub fn description(&self) -> Option<String> {
        self.metadata
            .as_ref()
            .and_then(|m| m.description.as_ref())
            .cloned()
    }

    /// Execute the entire pipeline with enhanced error handling
    pub async fn execute_with_retries(
        &self,
        initial_data: OxiData,
        resolver: &ConfigResolver,
    ) -> PipelineResult {
        let start_time = std::time::Instant::now();
        let mut current_data = initial_data;
        let mut step_results = Vec::new();
        let mut steps_executed = 0;
        let mut steps_failed = 0;
        let mut steps_skipped = 0;

        println!("🚀 Starting pipeline execution: {}", self.name());

        for (index, step) in self.pipeline.iter().enumerate() {
            println!(
                "\n📋 Step {} of {}: '{}'",
                index + 1,
                self.pipeline.len(),
                step.get_id()
            );

            let step_result = step
                .execute_with_retries(current_data.clone(), resolver)
                .await;

            if step_result.success {
                if let Some(data) = step_result.data.clone() {
                    current_data = data;
                }
                steps_executed += 1;
            } else {
                steps_failed += 1;

                if step.continue_on_error {
                    println!("⚠️  Step failed but continue_on_error is true, continuing...");
                    // Continue with the same data
                } else {
                    println!("💥 Step failed and continue_on_error is false, stopping pipeline");
                    step_results.push(step_result);

                    // Mark remaining steps as skipped
                    steps_skipped = self.pipeline.len() - index - 1;

                    let total_duration = start_time.elapsed().as_millis() as u64;
                    return PipelineResult {
                        success: false,
                        steps_executed,
                        steps_failed,
                        steps_skipped: steps_skipped as u32,
                        total_duration_ms: total_duration,
                        step_results,
                        final_data: None,
                    };
                }
            }

            step_results.push(step_result);
        }

        let total_duration = start_time.elapsed().as_millis() as u64;
        let success = steps_failed == 0;

        if success {
            println!("\n🎉 Pipeline completed successfully!");
        } else {
            println!(
                "\n⚠️  Pipeline completed with {} failed steps",
                steps_failed
            );
        }

        println!(
            "📊 Summary: {} executed, {} failed, {} skipped",
            steps_executed, steps_failed, steps_skipped
        );
        println!("⏱️  Total time: {}ms", total_duration);

        PipelineResult {
            success,
            steps_executed: steps_executed as u32,
            steps_failed: steps_failed as u32,
            steps_skipped: steps_skipped as u32,
            total_duration_ms: total_duration,
            step_results,
            final_data: if success { Some(current_data) } else { None },
        }
    }
}

impl PipelineStep {
    /// Get the step ID, using the name as fallback
    pub fn get_id(&self) -> &str {
        self.id.as_ref().unwrap_or(&self.name)
    }

    /// Convert config HashMap to OxiConfig with configuration resolution
    pub fn to_oxi_config(
        &self,
        resolver: &ConfigResolver,
    ) -> anyhow::Result<crate::types::OxiConfig> {
        let mut oxi_config = crate::types::OxiConfig::default();

        for (key, value) in &self.config {
            let resolved_value = resolver.resolve_value(value)?;
            oxi_config.values.insert(key.clone(), resolved_value);
        }

        Ok(oxi_config)
    }

    /// Execute this step with enhanced error handling and retries
    pub async fn execute_with_retries(
        &self,
        input: OxiData,
        resolver: &ConfigResolver,
    ) -> StepResult {
        let start_time = std::time::Instant::now();
        let step_id = self.get_id().to_string();

        for attempt in 0..=self.retry_attempts {
            println!(
                "🔄 Executing step '{}' (attempt {} of {})",
                step_id,
                attempt + 1,
                self.retry_attempts + 1
            );

            let result = if let Some(timeout_secs) = self.timeout_seconds {
                // Execute with timeout
                let duration = Duration::from_secs(timeout_secs);
                match timeout(duration, self.execute_once(input.clone(), resolver)).await {
                    Ok(result) => result,
                    Err(_) => Err(anyhow::anyhow!(
                        "Step timed out after {} seconds",
                        timeout_secs
                    )),
                }
            } else {
                // Execute without timeout
                self.execute_once(input.clone(), resolver).await
            };

            match result {
                Ok(data) => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    println!("✅ Step '{}' completed successfully", step_id);
                    return StepResult {
                        step_id,
                        success: true,
                        data: Some(data),
                        error: None,
                        retry_count: attempt,
                        duration_ms: duration,
                    };
                }
                Err(e) => {
                    if attempt < self.retry_attempts {
                        println!(
                            "⚠️  Step '{}' failed (attempt {}): {}. Retrying...",
                            step_id,
                            attempt + 1,
                            e
                        );
                        tokio::time::sleep(Duration::from_millis(1000 * (attempt + 1) as u64))
                            .await;
                    } else {
                        let duration = start_time.elapsed().as_millis() as u64;
                        println!(
                            "❌ Step '{}' failed after {} attempts: {}",
                            step_id,
                            attempt + 1,
                            e
                        );
                        return StepResult {
                            step_id,
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                            retry_count: attempt,
                            duration_ms: duration,
                        };
                    }
                }
            }
        }

        unreachable!()
    }

    /// Execute the step once (internal helper)
    async fn execute_once(
        &self,
        input: OxiData,
        resolver: &ConfigResolver,
    ) -> anyhow::Result<OxiData> {
        let config = self.to_oxi_config(resolver)?;

        // Import and execute the specific Oxi
        match self.name.as_str() {
            "read_file" => {
                let oxi = ReadFile;
                oxi.process(input, &config).await
            }
            "write_file" => {
                let oxi = WriteFile;
                oxi.process(input, &config).await
            }
            "parse_json" => {
                let oxi = ParseJson;
                oxi.process(input, &config).await
            }
            "format_csv" => {
                let oxi = FormatCsv;
                oxi.process(input, &config).await
            }
            "read_stdin" => {
                let oxi = ReadStdIn;
                oxi.process(input, &config).await
            }
            "write_stdout" => {
                let oxi = WriteStdOut;
                oxi.process(input, &config).await
            }
            "flatten" => {
                let oxi = Flatten;
                oxi.process(input, &config).await
            }
            _ => Err(anyhow::anyhow!("Unknown Oxi: {}", self.name)),
        }
    }

    /// Convert config HashMap to OxiConfig without resolution (for backward compatibility)
    pub fn to_oxi_config_simple(&self) -> crate::types::OxiConfig {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

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

use crate::project::ProjectConfig;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata extracted from pipeline YAML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created: Option<String>,
    pub file_path: PathBuf,
    pub step_count: usize,
}

/// Manages pipeline discovery, listing, and metadata extraction
pub struct PipelineManager {
    project_config: ProjectConfig,
}

impl PipelineManager {
    /// Create a new pipeline manager
    pub fn new() -> Result<Self> {
        let project_config = ProjectConfig::load()
            .map_err(|e| anyhow!("Failed to load project configuration: {}", e))?;

        Ok(Self { project_config })
    }

    /// Discover all pipelines in the configured pipeline directory
    pub fn discover_pipelines(&self) -> Result<Vec<PipelineMetadata>> {
        let pipeline_dir = self.project_config.get_pipeline_directory();

        if !pipeline_dir.exists() {
            return Ok(Vec::new());
        }

        let mut pipelines = Vec::new();

        // Read all YAML files in the pipeline directory
        for entry in fs::read_dir(&pipeline_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .yaml and .yml files
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    if let Ok(metadata) = self.extract_metadata(&path) {
                        pipelines.push(metadata);
                    }
                }
            }
        }

        // Sort pipelines by name for consistent output
        pipelines.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(pipelines)
    }

    /// Extract metadata from a pipeline YAML file
    fn extract_metadata(&self, file_path: &Path) -> Result<PipelineMetadata> {
        let content = fs::read_to_string(file_path)?;

        // Parse the YAML to extract metadata
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;

        // Extract name from filename if not specified in metadata
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Extract metadata from YAML document
        // First try to get from metadata section, then fall back to root level
        let metadata_section = yaml_value.get("metadata");

        let name = metadata_section
            .and_then(|m| m.get("name"))
            .and_then(|v| v.as_str())
            .or_else(|| yaml_value.get("name").and_then(|v| v.as_str()))
            .unwrap_or(&file_stem)
            .to_string();

        let description = metadata_section
            .and_then(|m| m.get("description"))
            .and_then(|v| v.as_str())
            .or_else(|| yaml_value.get("description").and_then(|v| v.as_str()))
            .map(|s| s.to_string());

        let version = metadata_section
            .and_then(|m| m.get("version"))
            .and_then(|v| v.as_str())
            .or_else(|| yaml_value.get("version").and_then(|v| v.as_str()))
            .map(|s| s.to_string());

        let author = metadata_section
            .and_then(|m| m.get("author"))
            .and_then(|v| v.as_str())
            .or_else(|| yaml_value.get("author").and_then(|v| v.as_str()))
            .map(|s| s.to_string());

        let tags = metadata_section
            .and_then(|m| m.get("tags"))
            .and_then(|v| v.as_sequence())
            .or_else(|| yaml_value.get("tags").and_then(|v| v.as_sequence()))
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let created = metadata_section
            .and_then(|m| m.get("created"))
            .and_then(|v| v.as_str())
            .or_else(|| yaml_value.get("created").and_then(|v| v.as_str()))
            .map(|s| s.to_string());

        // Count steps in the pipeline
        let step_count = yaml_value
            .get("pipeline")
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.len())
            .or_else(|| {
                // Fallback to "steps" field for compatibility
                yaml_value
                    .get("steps")
                    .and_then(|v| v.as_sequence())
                    .map(|seq| seq.len())
            })
            .unwrap_or(0);

        Ok(PipelineMetadata {
            name,
            description,
            version,
            author,
            tags,
            created,
            file_path: file_path.to_path_buf(),
            step_count,
        })
    }

    /// Filter pipelines by tags
    pub fn filter_by_tags(
        &self,
        pipelines: &[PipelineMetadata],
        filter_tags: &str,
    ) -> Vec<PipelineMetadata> {
        let filter_set: std::collections::HashSet<String> = filter_tags
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .collect();

        pipelines
            .iter()
            .filter(|pipeline| {
                if let Some(tags) = &pipeline.tags {
                    tags.iter()
                        .any(|tag| filter_set.contains(&tag.to_lowercase()))
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Filter pipelines by keyword in name or description
    pub fn filter_by_keyword(
        &self,
        pipelines: &[PipelineMetadata],
        keyword: &str,
    ) -> Vec<PipelineMetadata> {
        let keyword_lower = keyword.to_lowercase();

        pipelines
            .iter()
            .filter(|pipeline| {
                // Check if keyword matches name
                if pipeline.name.to_lowercase().contains(&keyword_lower) {
                    return true;
                }

                // Check if keyword matches description
                if let Some(description) = &pipeline.description {
                    if description.to_lowercase().contains(&keyword_lower) {
                        return true;
                    }
                }

                false
            })
            .cloned()
            .collect()
    }

    /// Format pipelines for display in table format
    pub fn format_pipeline_table(&self, pipelines: &[PipelineMetadata], verbose: bool) -> String {
        if pipelines.is_empty() {
            return "No pipelines found.".to_string();
        }

        if verbose {
            self.format_verbose_output(pipelines)
        } else {
            self.format_table_output(pipelines)
        }
    }

    /// Format pipelines in a compact table
    fn format_table_output(&self, pipelines: &[PipelineMetadata]) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "📂 Available pipelines in {} ({} total):\n\n",
            self.project_config.get_pipeline_directory().display(),
            pipelines.len()
        ));

        // Table header
        output.push_str(
            "┌─────────────────────┬──────────────────────────────┬─────────┬───────────┐\n",
        );
        output.push_str(
            "│ Name                │ Description                  │ Version │ Steps     │\n",
        );
        output.push_str(
            "├─────────────────────┼──────────────────────────────┼─────────┼───────────┤\n",
        );

        // Table rows
        for pipeline in pipelines {
            let name = truncate_string(&pipeline.name, 19);
            let description = pipeline
                .description
                .as_ref()
                .map(|d| truncate_string(d, 28))
                .unwrap_or_else(|| "No description".to_string());
            let version = pipeline
                .version
                .as_ref()
                .map(|v| truncate_string(v, 7))
                .unwrap_or_else(|| "N/A".to_string());
            let steps = format!("{} steps", pipeline.step_count);

            output.push_str(&format!(
                "│ {:<19} │ {:<28} │ {:<7} │ {:<9} │\n",
                name, description, version, steps
            ));
        }

        // Table footer
        output.push_str(
            "└─────────────────────┴──────────────────────────────┴─────────┴───────────┘\n\n",
        );

        // Help text
        output.push_str("💡 Use 'oxide_flow pipeline info <name>' for detailed information\n");
        output.push_str("🚀 Use 'oxide_flow run <name>' to execute a pipeline\n");

        output
    }

    /// Format pipelines in verbose mode
    fn format_verbose_output(&self, pipelines: &[PipelineMetadata]) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "📂 Available pipelines in {} ({} total):\n\n",
            self.project_config.get_pipeline_directory().display(),
            pipelines.len()
        ));

        for (i, pipeline) in pipelines.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }

            output.push_str(&format!("📂 Pipeline: {}\n", pipeline.name));

            if let Some(description) = &pipeline.description {
                output.push_str(&format!("   📝 Description: {}\n", description));
            }

            if let Some(author) = &pipeline.author {
                output.push_str(&format!("   👤 Author: {}\n", author));
            }

            if let Some(tags) = &pipeline.tags {
                output.push_str(&format!("   🏷️  Tags: {}\n", tags.join(", ")));
            }

            if let Some(version) = &pipeline.version {
                output.push_str(&format!("   📅 Version: {}\n", version));
            }

            output.push_str(&format!(
                "   📍 Location: {}\n",
                pipeline.file_path.display()
            ));
            output.push_str(&format!("   ⚙️  Steps: {} total\n", pipeline.step_count));

            if let Some(created) = &pipeline.created {
                output.push_str(&format!("   📅 Created: {}\n", created));
            }
        }

        output
    }

    /// Get pipeline directory path for display
    pub fn get_pipeline_directory(&self) -> PathBuf {
        self.project_config.get_pipeline_directory()
    }

    /// Get available templates
    pub fn get_available_templates(&self) -> Vec<&'static str> {
        vec!["basic", "etl", "validation", "batch", "api", "streaming"]
    }

    /// Create a new pipeline from a template
    pub fn create_pipeline(
        &self,
        name: &str,
        template: &str,
        description: Option<&str>,
        author: Option<&str>,
    ) -> Result<PathBuf> {
        // Validate pipeline name (should be snake_case)
        if !is_valid_pipeline_name(name) {
            return Err(anyhow!(
                "Invalid pipeline name '{}'. Use snake_case format (e.g., my_pipeline)",
                name
            ));
        }

        // Get template content
        let template_content = self.get_template_content(template)?;

        // Get default values from project config
        let default_author = self.project_config.project.name.clone();
        let default_description = format!("Pipeline created from {} template", template);
        let pipeline_description = description.unwrap_or(&default_description);
        let pipeline_author = author.unwrap_or(&default_author);

        // Replace template variables
        let pipeline_content = template_content
            .replace("{{pipeline_name}}", &format_display_name(name))
            .replace("{{pipeline_description}}", pipeline_description)
            .replace("{{pipeline_author}}", pipeline_author)
            .replace("{{input_file}}", "data.json")
            .replace("{{output_file}}", "output.csv")
            .replace("{{backup_file}}", &format!("{}_backup.csv", name));

        // Create pipeline file path
        let pipeline_dir = self.project_config.get_pipeline_directory();
        let pipeline_path = pipeline_dir.join(format!("{}.yaml", name));

        // Check if pipeline already exists
        if pipeline_path.exists() {
            return Err(anyhow!(
                "Pipeline '{}' already exists at {}",
                name,
                pipeline_path.display()
            ));
        }

        // Create pipeline directory if it doesn't exist
        if !pipeline_dir.exists() {
            fs::create_dir_all(&pipeline_dir)?;
        }

        // Write pipeline file
        fs::write(&pipeline_path, pipeline_content)?;

        Ok(pipeline_path)
    }

    /// Get template content by name
    fn get_template_content(&self, template: &str) -> Result<String> {
        let template_content = match template {
            "basic" => include_str!("templates/basic.yaml"),
            "etl" => include_str!("templates/etl.yaml"),
            "validation" => include_str!("templates/validation.yaml"),
            "batch" => include_str!("templates/batch.yaml"),
            "api" => include_str!("templates/api.yaml"),
            "streaming" => include_str!("templates/streaming.yaml"),
            _ => return Err(anyhow!("Unknown template: {}", template)),
        };

        Ok(template_content.to_string())
    }

    /// Interactive pipeline creation with prompts
    pub fn create_pipeline_interactive(&self, name: &str) -> Result<PathBuf> {
        use std::io::{self, Write};

        println!("📝 Creating new pipeline: {}\n", name);

        // Template selection
        println!("🎯 Select template:");
        let templates = self.get_available_templates();
        for (i, template) in templates.iter().enumerate() {
            let description = match *template {
                "basic" => "Simple read → transform → write",
                "etl" => "Extract, Transform, Load pattern",
                "validation" => "Data validation and quality checking",
                "batch" => "Batch processing with error handling",
                "api" => "API data processing",
                "streaming" => "Streaming data processing",
                _ => "Unknown template",
            };
            println!("  {}. {:<12} - {}", i + 1, template, description);
        }

        print!("\nEnter choice [1-{}] (default: 1): ", templates.len());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        let template_index = if choice.is_empty() {
            0
        } else {
            choice.parse::<usize>().unwrap_or(1).saturating_sub(1)
        };

        let selected_template = templates.get(template_index).unwrap_or(&"basic");

        // Get description
        print!("\nEnter description (optional): ");
        io::stdout().flush().unwrap();
        let mut description = String::new();
        io::stdin().read_line(&mut description)?;
        let description = description.trim();
        let description = if description.is_empty() {
            None
        } else {
            Some(description)
        };

        // Get author (default from project config)
        let default_author = &self.project_config.project.name;
        print!("\nEnter author (default: {}): ", default_author);
        io::stdout().flush().unwrap();
        let mut author = String::new();
        io::stdin().read_line(&mut author)?;
        let author = author.trim();
        let author = if author.is_empty() {
            None
        } else {
            Some(author)
        };

        println!("\n📋 Pipeline Details:");
        println!("  Name: {}", format_display_name(name));
        println!(
            "  Description: {}",
            description.unwrap_or(&format!("{} pipeline", selected_template))
        );
        println!("  Author: {}", author.unwrap_or(default_author));
        println!("  Template: {}", selected_template);

        // Create the pipeline
        let pipeline_path = self.create_pipeline(name, selected_template, description, author)?;

        println!("\n✅ Created pipeline: {}", pipeline_path.display());
        println!("\n💡 Use 'oxide_flow pipeline test {}' to validate", name);
        println!("🚀 Use 'oxide_flow run {}' to execute", name);

        Ok(pipeline_path)
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:<width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Validate pipeline name (should be snake_case)
fn is_valid_pipeline_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Check if name contains only lowercase letters, numbers, and underscores
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.starts_with('_')
        && !name.ends_with('_')
        && !name.contains("__")
}

/// Format a snake_case name into a display name
fn format_display_name(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if !chars.is_empty() {
                chars[0] = chars[0].to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short     ");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
        assert_eq!(truncate_string("exact", 5), "exact");
    }
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Project configuration from oxiflow.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMetadata,
    pub oxis: HashMap<String, OxiSource>,
    pub settings: ProjectSettings,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxiSource {
    pub version: String,
    pub source: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub output_dir: String,
    pub pipeline_dir: String,
    pub oxis_dir: String,
}

impl ProjectConfig {
    /// Load project configuration from oxiflow.yaml
    pub fn load() -> Result<Self> {
        Self::load_from_path("oxiflow.yaml")
    }

    /// Load project configuration from a specific path
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path).with_context(|| {
            format!("Failed to read config file at {}", path.as_ref().display())
        })?;

        let config: ProjectConfig = serde_yaml::from_str(&content).with_context(|| {
            format!("Failed to parse config file at {}", path.as_ref().display())
        })?;

        Ok(config)
    }

    /// Find a pipeline by name in the configured pipeline directory
    pub fn find_pipeline(&self, name: &str) -> Result<PathBuf> {
        let pipeline_dir = Path::new(&self.settings.pipeline_dir);

        // Try different extensions and exact matches
        let candidates = vec![
            format!("{}.yaml", name),
            format!("{}.yml", name),
            format!("{name}/pipeline.yaml"),
            format!("{name}/pipeline.yml"),
        ];

        for candidate in candidates {
            let path = pipeline_dir.join(&candidate);
            if path.exists() && path.is_file() {
                println!("📋 Found pipeline: {}", path.display());
                return Ok(path);
            }
        }

        // If not found, list available pipelines to help the user
        self.list_available_pipelines()?;
        anyhow::bail!(
            "Pipeline '{}' not found in {}",
            name,
            pipeline_dir.display()
        )
    }

    /// List all available pipelines in the configured directory
    pub fn list_available_pipelines(&self) -> Result<Vec<String>> {
        let pipeline_dir = Path::new(&self.settings.pipeline_dir);

        if !pipeline_dir.exists() {
            println!(
                "⚠️  Pipeline directory '{}' does not exist",
                pipeline_dir.display()
            );
            return Ok(vec![]);
        }

        let mut pipelines = Vec::new();

        for entry in fs::read_dir(pipeline_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "yaml" || extension == "yml" {
                        if let Some(stem) = path.file_stem() {
                            if let Some(name) = stem.to_str() {
                                pipelines.push(name.to_string());
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                // Check for pipeline.yaml in subdirectories
                let pipeline_file = path.join("pipeline.yaml");
                let pipeline_file_yml = path.join("pipeline.yml");

                if pipeline_file.exists() || pipeline_file_yml.exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        pipelines.push(name.to_string());
                    }
                }
            }
        }

        if pipelines.is_empty() {
            println!("📂 No pipelines found in {}", pipeline_dir.display());
        } else {
            println!("📂 Available pipelines in {}:", pipeline_dir.display());
            for pipeline in &pipelines {
                println!("  • {}", pipeline);
            }
        }

        Ok(pipelines)
    }
}

/// Initialize a new Oxide Flow project
pub fn init_project(name: Option<String>, directory: Option<String>) -> Result<()> {
    // Get project name
    let project_name = match name {
        Some(name) => name,
        None => {
            print!("Enter project name: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if project_name.is_empty() {
        anyhow::bail!("Project name cannot be empty");
    }

    // Determine target directory
    let target_dir = match directory {
        Some(dir) => PathBuf::from(dir),
        None => std::env::current_dir()?.join(&project_name),
    };

    // Create project structure
    create_project_structure(&target_dir, &project_name)?;

    println!(
        "✅ Initialized Oxide Flow project '{}' in {}",
        project_name,
        target_dir.display()
    );
    println!("📁 Created directories: output/, oxis/, pipelines/");
    println!("📄 Created files: oxiflow.yaml, pipelines/pipeline.yaml");
    println!("\nNext steps:");
    println!("  cd {}", target_dir.display());
    println!("  oxiflow run  # Run the default JSON→CSV pipeline");

    Ok(())
}

fn create_project_structure(target_dir: &Path, project_name: &str) -> Result<()> {
    // Create main project directory
    fs::create_dir_all(target_dir).with_context(|| {
        format!(
            "Failed to create project directory: {}",
            target_dir.display()
        )
    })?;

    // Create subdirectories
    let subdirs = ["output", "oxis", "pipelines"];
    for subdir in &subdirs {
        let dir_path = target_dir.join(subdir);
        fs::create_dir_all(&dir_path)
            .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
    }

    // Create oxiflow.yaml
    let oxiflow_yaml = create_oxiflow_yaml(project_name);
    let oxiflow_path = target_dir.join("oxiflow.yaml");
    fs::write(&oxiflow_path, oxiflow_yaml).with_context(|| {
        format!(
            "Failed to create oxiflow.yaml at {}",
            oxiflow_path.display()
        )
    })?;

    // Create default pipeline.yaml
    let pipeline_yaml = create_default_pipeline_yaml();
    let pipeline_path = target_dir.join("pipelines").join("pipeline.yaml");
    fs::write(&pipeline_path, pipeline_yaml).with_context(|| {
        format!(
            "Failed to create pipeline.yaml at {}",
            pipeline_path.display()
        )
    })?;

    // Create sample input file
    let sample_json = create_sample_input_json();
    let sample_path = target_dir.join("input.json");
    fs::write(&sample_path, sample_json).with_context(|| {
        format!(
            "Failed to create sample input.json at {}",
            sample_path.display()
        )
    })?;

    Ok(())
}

fn create_oxiflow_yaml(project_name: &str) -> String {
    format!(
        r#"# Oxide Flow Project Configuration
project:
  name: "{project_name}"
  version: "1.0.0"
  description: "Data transformation pipeline project"

# Registry of available Oxis and their sources
oxis:
  core:
    version: "1.0.0"
    source: "builtin"
    description: "Core Oxis for file I/O and basic transformations"

# Project settings
settings:
  output_dir: "./output"
  pipeline_dir: "./pipelines"
  oxis_dir: "./oxis"

# Default environment variables (can be overridden)
environment:
  LOG_LEVEL: "info"
  OUTPUT_FORMAT: "pretty"
"#
    )
}

fn create_default_pipeline_yaml() -> String {
    r#"# Default JSON to CSV Pipeline
# This pipeline reads a JSON file, parses it, and converts it to CSV format

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
      path: "output/data.csv"

# Pipeline metadata
metadata:
  name: "JSON to CSV Converter"
  description: "Converts JSON data to CSV format"
  version: "1.0.0"
  author: "Oxide Flow"
"#
    .to_string()
}

fn create_sample_input_json() -> String {
    r#"[
  {
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "age": 30,
    "city": "New York"
  },
  {
    "id": 2,
    "name": "Jane Smith",
    "email": "jane@example.com",
    "age": 25,
    "city": "Los Angeles"
  },
  {
    "id": 3,
    "name": "Bob Johnson",
    "email": "bob@example.com",
    "age": 35,
    "city": "Chicago"
  }
]"#
    .to_string()
}

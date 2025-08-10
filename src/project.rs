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
    #[serde(default)]
    pub state_manager: Option<StateConfig>,
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

/// State management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Backend type: "file", "memory", "redis" (future)
    #[serde(default = "default_backend")]
    pub backend: String,

    /// File backend configuration
    #[serde(default)]
    pub file: Option<FileStateConfig>,

    /// Heartbeat interval (e.g., "10s", "5m")
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval: String,

    /// Checkpoint interval (e.g., "30s", "1m")
    #[serde(default = "default_checkpoint_interval")]
    pub checkpoint_interval: String,

    /// Cleanup interval (e.g., "1h", "24h")
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: String,
}

/// File backend specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStateConfig {
    /// Base path for state files
    #[serde(default = "default_state_path")]
    pub base_path: String,

    /// Lock timeout (e.g., "30s", "1m")
    #[serde(default = "default_lock_timeout")]
    pub lock_timeout: String,

    /// Enable backup
    #[serde(default = "default_backup_enabled")]
    pub backup_enabled: bool,

    /// Backup retention period (e.g., "7d", "30d")
    #[serde(default = "default_backup_retention")]
    pub backup_retention: String,
}

// Default functions for serde
fn default_backend() -> String {
    "file".to_string()
}
fn default_heartbeat_interval() -> String {
    "10s".to_string()
}
fn default_checkpoint_interval() -> String {
    "30s".to_string()
}
fn default_cleanup_interval() -> String {
    "1h".to_string()
}
fn default_state_path() -> String {
    ".oxiflow/state".to_string()
}
fn default_lock_timeout() -> String {
    "30s".to_string()
}
fn default_backup_enabled() -> bool {
    true
}
fn default_backup_retention() -> String {
    "7d".to_string()
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

    /// Get the configured pipeline directory as a PathBuf
    pub fn get_pipeline_directory(&self) -> PathBuf {
        PathBuf::from(&self.settings.pipeline_dir)
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
                println!("  • {pipeline}");
            }
        }

        Ok(pipelines)
    }

    /// Create a StateManagerConfig from the project configuration
    pub fn create_state_manager_config(&self) -> crate::state::manager::StateManagerConfig {
        use crate::state::backend::{BackendConfig, SerializationFormat};
        use crate::state::manager::StateManagerConfig;

        let backend = match &self.state_manager {
            Some(state_config) => match state_config.backend.as_str() {
                "file" => {
                    let file_config =
                        state_config
                            .file
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| FileStateConfig {
                                base_path: default_state_path(),
                                lock_timeout: default_lock_timeout(),
                                backup_enabled: default_backup_enabled(),
                                backup_retention: default_backup_retention(),
                            });

                    BackendConfig::File {
                        base_path: PathBuf::from(&file_config.base_path),
                        format: SerializationFormat::Json,
                        atomic_writes: true,
                        lock_timeout_ms: parse_duration(&file_config.lock_timeout).unwrap_or(30000),
                    }
                }
                "memory" => BackendConfig::Memory { persistent: false },
                _ => {
                    eprintln!(
                        "⚠️  Unknown backend type '{}', falling back to file",
                        state_config.backend
                    );
                    BackendConfig::File {
                        base_path: PathBuf::from(".oxiflow/state"),
                        format: SerializationFormat::Json,
                        atomic_writes: true,
                        lock_timeout_ms: 30000,
                    }
                }
            },
            None => {
                // Default to file backend
                BackendConfig::File {
                    base_path: PathBuf::from(".oxiflow/state"),
                    format: SerializationFormat::Json,
                    atomic_writes: true,
                    lock_timeout_ms: 30000,
                }
            }
        };

        StateManagerConfig {
            backend,
            default_lock_timeout_ms: 30000,
            worker_id: format!("worker_{}", std::process::id()),
            heartbeat_interval_ms: self
                .state_manager
                .as_ref()
                .and_then(|s| parse_duration(&s.heartbeat_interval))
                .unwrap_or(10000),
            max_retries: 3,
            cleanup_interval_hours: 24,
            max_state_age_hours: 168,
        }
    }
}

/// Parse duration string (e.g., "30s", "5m", "1h") to milliseconds
fn parse_duration(duration_str: &str) -> Option<u64> {
    let duration_str = duration_str.trim();
    if duration_str.is_empty() {
        return None;
    }

    let (number_str, unit) = if let Some(stripped) = duration_str.strip_suffix("ms") {
        (stripped, "ms")
    } else if let Some(stripped) = duration_str.strip_suffix('s') {
        (stripped, "s")
    } else if let Some(stripped) = duration_str.strip_suffix('m') {
        (stripped, "m")
    } else if let Some(stripped) = duration_str.strip_suffix('h') {
        (stripped, "h")
    } else if let Some(stripped) = duration_str.strip_suffix('d') {
        (stripped, "d")
    } else {
        // Assume seconds if no unit
        (duration_str, "s")
    };

    let number: u64 = number_str.parse().ok()?;

    let milliseconds = match unit {
        "ms" => number,
        "s" => number * 1000,
        "m" => number * 60 * 1000,
        "h" => number * 60 * 60 * 1000,
        "d" => number * 24 * 60 * 60 * 1000,
        _ => return None,
    };

    Some(milliseconds)
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

# State management configuration
state_manager:
  backend: file  # Currently only file backend supported

  file:
    base_path: ".oxiflow/state"
    lock_timeout: "30s"
    backup_enabled: true
    backup_retention: "7d"

  heartbeat_interval: "10s"
  checkpoint_interval: "30s"
  cleanup_interval: "1h"

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

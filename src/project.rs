use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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

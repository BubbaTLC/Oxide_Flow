# `init` - Initialize New Project

Creates a new Oxide Flow project with default structure and configuration.

## Syntax

```bash
oxide_flow init [OPTIONS] [PROJECT_NAME]
```

## Arguments

- `[PROJECT_NAME]` - Name of the project (optional, will prompt if not provided)

## Options

- `--name` / `-n` `<NAME>` - Project name (alternative to positional argument)
- `--directory` / `-d` `<PATH>` - Target directory (default: current directory)

## Examples

```bash
# Interactive project creation
oxide_flow init

# Create project with specific name
oxide_flow init my_data_project

# Create in specific directory
oxide_flow init --directory ~/projects --name analytics_pipeline

# Using short flags
oxide_flow init -d ~/projects -n analytics_pipeline
```

## Generated Structure

```
my_project/
├── oxiflow.yaml              # Project configuration
├── pipelines/               # Pipeline definitions
│   └── pipeline.yaml        # Default pipeline
└── README.md                # Project documentation
```

## Generated Files

### `oxiflow.yaml` - Project configuration

```yaml
# Oxide Flow Project Configuration
project:
  name: "my_project"
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
```

### `pipelines/pipeline.yaml` - Default pipeline

```yaml
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

# Pipeline metadata
metadata:
  name: "JSON to CSV Converter"
  description: "Converts JSON data to CSV format"
  version: "1.0.0"
  author: "Pipeline Creator"
  tags: ["conversion", "json", "csv"]
```

### `README.md` - Project documentation

```markdown
# My Project

Data transformation pipeline project

## Getting Started

Run the default pipeline:
```bash
oxide_flow run
```

## Available Pipelines

- `pipeline` - JSON to CSV Converter

## Project Structure

- `pipelines/` - Pipeline definitions
- `output/` - Generated output files
- `oxiflow.yaml` - Project configuration
```

## Workflow Integration

### Quick Start

```bash
# Initialize and run
oxide_flow init hello_world
cd hello_world
echo '{"name": "John", "age": 30}' > input.json
oxide_flow run
```

### CI/CD Integration

```bash
# In deployment scripts
oxide_flow init production_etl --directory /opt/pipelines
cd /opt/pipelines/production_etl
# Copy configuration files
cp /config/production.yaml oxiflow.yaml
# Run pipelines
oxide_flow run etl_daily
```

## Error Handling

### Common Issues

**Project already exists:**
```bash
$ oxide_flow init existing_project
❌ Project initialization failed: Directory 'existing_project' already exists
```

**Permission errors:**
```bash
$ oxide_flow init --directory /root/restricted
❌ Project initialization failed: Permission denied creating directory '/root/restricted'
```

**Invalid project names:**
```bash
$ oxide_flow init "invalid/name"
❌ Project initialization failed: Invalid project name 'invalid/name'. Use alphanumeric characters, hyphens, and underscores only.
```

## Best Practices

1. **Use descriptive project names:** `customer_analytics` instead of `project1`
2. **Initialize in dedicated directories:** Keep projects organized
3. **Version control:** Initialize git repository after project creation
4. **Configure immediately:** Update `oxiflow.yaml` with your specific settings

## Related Commands

- [`run`](run.md) - Execute pipelines in the initialized project
- [`pipeline`](pipeline.md) - Manage pipelines after project creation

# Oxide Flow CLI Reference

This guide covers the complete command-line interface for Oxide Flow, including all commands, options, and usage patterns.

## Table of Contents

- [Overview](#overview)
- [Global Options](#global-options)
- [Commands](#commands)
- [Usage Patterns](#usage-patterns)
- [Environment Variables](#environment-variables)
- [Exit Codes](#exit-codes)
- [Examples](#examples)

## Overview

Oxide Flow provides a simple, intuitive command-line interface for data pipeline management and execution. The CLI follows modern conventions with clear subcommands and helpful error messages.

### Basic Syntax

```bash
oxide_flow [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGUMENTS]
```

### Getting Help

```bash
# General help
oxide_flow --help

# Command-specific help
oxide_flow <command> --help

# Version information
oxide_flow --version
```

## Global Options

These options are available for all commands:

### `--verbose` / `-v`
Enables detailed output for debugging and monitoring.

```bash
oxide_flow --verbose run my_pipeline
oxide_flow -v init new_project
```

**Output with verbose:**
- Detailed step execution information
- Configuration resolution details
- Performance timing information
- Debug messages from Oxis

## Commands

### `init` - Initialize New Project

Creates a new Oxide Flow project with default structure and configuration.

**Syntax:**
```bash
oxide_flow init [OPTIONS] [PROJECT_NAME]
```

**Arguments:**
- `[PROJECT_NAME]` - Name of the project (optional, will prompt if not provided)

**Options:**
- `--name` / `-n` `<NAME>` - Project name (alternative to positional argument)
- `--directory` / `-d` `<PATH>` - Target directory (default: current directory)

**Examples:**
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

**Generated Structure:**
```
my_project/
├── oxiflow.yaml              # Project configuration
├── pipelines/               # Pipeline definitions
│   └── pipeline.yaml        # Default pipeline
└── README.md                # Project documentation
```

**Generated Files:**

**`oxiflow.yaml`** - Project configuration:
```yaml
# Oxide Flow Project Configuration
project:
  name: "my_project"
  version: "1.0.0"
  description: "Data transformation pipeline project"

oxis:
  core:
    version: "1.0.0"
    source: "builtin"
    description: "Core Oxis for file I/O and basic transformations"

settings:
  output_dir: "./output"
  pipeline_dir: "./pipelines"
  oxis_dir: "./oxis"

environment:
  LOG_LEVEL: "info"
  OUTPUT_FORMAT: "pretty"
```

**`pipelines/pipeline.yaml`** - Default pipeline:
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

metadata:
  name: "JSON to CSV Converter"
  description: "Converts JSON data to CSV format"
```

---

### `run` - Execute Pipeline

Discovers and executes a pipeline by name using the project configuration.

**Syntax:**
```bash
oxide_flow run [OPTIONS] [PIPELINE_NAME]
```

**Arguments:**
- `[PIPELINE_NAME]` - Name of the pipeline to run (default: "pipeline")

**Options:**
- `--config` / `-c` `<PATH>` - Path to configuration file (optional)
- `--verbose` / `-v` - Enable detailed output (global option)

**Pipeline Discovery:**

The `run` command discovers pipelines in the following order:
1. `{name}.yaml` in pipeline directory
2. `{name}.yml` in pipeline directory
3. `{name}/pipeline.yaml` subdirectory
4. `{name}/pipeline.yml` subdirectory

**Examples:**
```bash
# Run default pipeline
oxide_flow run

# Run specific pipeline by name
oxide_flow run data_processor

# Run with verbose output
oxide_flow run --verbose etl_pipeline

# Run with custom config
oxide_flow run --config dev.yaml my_pipeline
```

**Pipeline Discovery Output:**
```bash
$ oxide_flow run data_processor
📋 Found pipeline: ./pipelines/data_processor.yaml
🔍 Running pipeline 'data_processor' from: ./pipelines/data_processor.yaml
Running pipeline: Data Processing Pipeline
Description: Processes customer data for analysis
Steps: 5
🚀 Starting pipeline execution: Data Processing Pipeline
```

**Error Handling - Pipeline Not Found:**
```bash
$ oxide_flow run nonexistent
📂 Available pipelines in ./pipelines:
  • data_processor
  • etl_pipeline
  • json_converter
  • validation_pipeline
❌ Pipeline execution failed: Pipeline 'nonexistent' not found in ./pipelines
```

**Execution Output:**

The run command provides detailed execution feedback:

```bash
🚀 Starting pipeline execution: Data Processing Pipeline

📋 Step 1 of 4: 'reader'
🔄 Executing step 'reader' (attempt 1 of 2)
✅ Step 'reader' completed successfully

📋 Step 2 of 4: 'parser'
🔄 Executing step 'parser' (attempt 1 of 1)
✅ Step 'parser' completed successfully

📋 Step 3 of 4: 'transformer'
🔄 Executing step 'transformer' (attempt 1 of 3)
⚠️  Step 'transformer' failed (attempt 1): Connection timeout. Retrying...
🔄 Executing step 'transformer' (attempt 2 of 3)
✅ Step 'transformer' completed successfully

📋 Step 4 of 4: 'writer'
🔄 Executing step 'writer' (attempt 1 of 1)
✅ Step 'writer' completed successfully

🎉 Pipeline completed successfully!
📊 Summary: 4 executed, 0 failed, 0 skipped
⏱️  Total time: 1.2s
Final Result: CSV data (245 rows, 12 columns)
✅ Pipeline execution completed successfully!
```

**Error Recovery Output:**
```bash
📋 Step 2 of 5: 'unreliable_step'
🔄 Executing step 'unreliable_step' (attempt 1 of 3)
⚠️  Step 'unreliable_step' failed (attempt 1): Network error. Retrying...
🔄 Executing step 'unreliable_step' (attempt 2 of 3)
⚠️  Step 'unreliable_step' failed (attempt 2): Network error. Retrying...
🔄 Executing step 'unreliable_step' (attempt 3 of 3)
❌ Step 'unreliable_step' failed after 3 attempts: Network error
⚠️  Step failed but continue_on_error is true, continuing...

📋 Step 3 of 5: 'fallback_step'
🔄 Executing step 'fallback_step' (attempt 1 of 1)
✅ Step 'fallback_step' completed successfully
```

## Usage Patterns

### Project Workflow

```bash
# 1. Initialize new project
oxide_flow init my_analytics

# 2. Navigate to project
cd my_analytics

# 3. Edit pipeline configuration
nano pipelines/pipeline.yaml

# 4. Run pipeline
oxide_flow run

# 5. Create additional pipelines
cp pipelines/pipeline.yaml pipelines/data_cleaner.yaml
nano pipelines/data_cleaner.yaml

# 6. Run specific pipeline
oxide_flow run data_cleaner
```

### Development Workflow

```bash
# Run with verbose output for debugging
oxide_flow run --verbose my_pipeline

# Test pipeline with different configurations
CONFIG_FILE=dev.json oxide_flow run test_pipeline
CONFIG_FILE=prod.json oxide_flow run test_pipeline

# Quick validation of pipeline structure
oxide_flow run validation_pipeline
```

### Production Workflow

```bash
# Set production environment variables
export DATA_INPUT_PATH="/data/input/daily_export.csv"
export DATA_OUTPUT_PATH="/data/output/processed_$(date +%Y%m%d).csv"
export PROCESSING_TIMEOUT="300"
export RETRY_COUNT="5"

# Run production pipeline
oxide_flow run production_etl

# Check exit code for monitoring
if [ $? -eq 0 ]; then
    echo "Pipeline completed successfully"
else
    echo "Pipeline failed - check logs"
    exit 1
fi
```

## Environment Variables

### Pipeline Configuration

Oxide Flow supports dynamic configuration through environment variables in pipeline YAML files:

```yaml
# In pipeline.yaml
config:
  input_path: "${INPUT_FILE}"                # Required variable
  output_path: "${OUTPUT_FILE:-output.csv}"  # Optional with default
  batch_size: "${BATCH_SIZE:-1000}"          # Numeric with default
```

```bash
# Set before running
export INPUT_FILE="data.csv"
export OUTPUT_FILE="results.csv"
export BATCH_SIZE="500"
oxide_flow run my_pipeline
```

### Common Environment Variables

**File Paths:**
```bash
export DATA_DIR="/path/to/data"
export OUTPUT_DIR="/path/to/output"
export CONFIG_DIR="/path/to/config"
```

**Processing Options:**
```bash
export BATCH_SIZE="1000"
export TIMEOUT="300"
export RETRY_COUNT="3"
export STRICT_MODE="true"
```

**Format Options:**
```bash
export CSV_DELIMITER=","
export JSON_PRETTY="true"
export INCLUDE_HEADERS="true"
```

### Runtime Environment

**Logging and Debug:**
```bash
export RUST_LOG="debug"          # Rust logging level
export OXIDE_FLOW_DEBUG="1"      # Enable debug mode
```

**Performance:**
```bash
export OXIDE_FLOW_WORKERS="4"    # Number of worker threads
export OXIDE_FLOW_MEMORY="2GB"   # Memory limit
```

## Exit Codes

Oxide Flow uses standard exit codes for integration with scripts and monitoring systems:

| Exit Code | Meaning |
|-----------|---------|
| `0` | Success - Command completed successfully |
| `1` | General Error - Pipeline execution failed, invalid arguments, etc. |
| `2` | Misuse - Invalid command syntax, unknown command |
| `125` | Pipeline Not Found - Specified pipeline could not be located |
| `126` | Permission Error - Cannot access files or directories |
| `127` | Command Not Found - Oxide Flow binary not found |
| `130` | Interrupted - User cancelled execution (Ctrl+C) |

**Checking Exit Codes:**
```bash
# In shell scripts
oxide_flow run my_pipeline
case $? in
    0)   echo "Success!" ;;
    1)   echo "Pipeline failed" ;;
    2)   echo "Invalid command" ;;
    125) echo "Pipeline not found" ;;
    *)   echo "Unexpected error: $?" ;;
esac

# Simple success check
if oxide_flow run my_pipeline; then
    echo "Pipeline completed successfully"
else
    echo "Pipeline failed with exit code $?"
fi
```

## Examples

### Basic Usage

**Initialize and run default pipeline:**
```bash
oxide_flow init hello_world
cd hello_world
echo '{"name": "John", "age": 30}' > input.json
oxide_flow run
```

**Output:**
```
🔍 Running pipeline 'pipeline' from: ./pipelines/pipeline.yaml
Running pipeline: JSON to CSV Converter
Steps: 4
🚀 Starting pipeline execution: JSON to CSV Converter
✅ Step 'reader' completed successfully
✅ Step 'parser' completed successfully
✅ Step 'formatter' completed successfully
✅ Step 'writer' completed successfully
🎉 Pipeline completed successfully!
```

### Environment-Driven Pipeline

**Create configurable pipeline:**
```yaml
# pipelines/configurable.yaml
pipeline:
  - name: read_file
    config:
      path: "${INPUT_FILE}"
  - name: "${PROCESSOR:-parse_json}"
  - name: "${OUTPUT_FORMAT:-format_csv}"
    config:
      headers: "${INCLUDE_HEADERS:-true}"
      delimiter: "${DELIMITER:-,}"
  - name: write_file
    config:
      path: "${OUTPUT_FILE}"
```

**Run with different configurations:**
```bash
# CSV processing
INPUT_FILE="data.csv" OUTPUT_FILE="result.json" \
PROCESSOR="parse_csv" OUTPUT_FORMAT="format_json" \
oxide_flow run configurable

# JSON processing
INPUT_FILE="data.json" OUTPUT_FILE="result.csv" \
PROCESSOR="parse_json" OUTPUT_FORMAT="format_csv" \
DELIMITER="|" \
oxide_flow run configurable
```

### Error Handling Demonstration

**Create pipeline with retry logic:**
```yaml
# pipelines/resilient.yaml
pipeline:
  - name: read_file
    id: primary_reader
    config:
      path: "primary_data.json"
    continue_on_error: true
    retry_attempts: 3
    timeout_seconds: 30

  - name: read_file
    id: backup_reader
    config:
      path: "backup_data.json"
    continue_on_error: false
    retry_attempts: 1

  - name: parse_json
  - name: write_stdout
```

**Run and observe error handling:**
```bash
# Create backup data but not primary
echo '{"status": "backup"}' > backup_data.json

oxide_flow run resilient
```

**Output shows retry behavior:**
```
📋 Step 1 of 4: 'primary_reader'
🔄 Executing step 'primary_reader' (attempt 1 of 4)
⚠️  Step 'primary_reader' failed (attempt 1): File not found. Retrying...
🔄 Executing step 'primary_reader' (attempt 2 of 4)
⚠️  Step 'primary_reader' failed (attempt 2): File not found. Retrying...
🔄 Executing step 'primary_reader' (attempt 3 of 4)
⚠️  Step 'primary_reader' failed (attempt 3): File not found. Retrying...
🔄 Executing step 'primary_reader' (attempt 4 of 4)
❌ Step 'primary_reader' failed after 4 attempts: File not found
⚠️  Step failed but continue_on_error is true, continuing...

📋 Step 2 of 4: 'backup_reader'
🔄 Executing step 'backup_reader' (attempt 1 of 2)
✅ Step 'backup_reader' completed successfully
```

### Batch Processing Script

**Create batch processing script:**
```bash
#!/bin/bash
# process_daily_data.sh

set -e  # Exit on any error

# Configuration
DATA_DIR="/data/input"
OUTPUT_DIR="/data/output"
PIPELINE="daily_etl"
DATE=$(date +%Y%m%d)

# Setup environment
export INPUT_PATH="${DATA_DIR}/export_${DATE}.json"
export OUTPUT_PATH="${OUTPUT_DIR}/processed_${DATE}.csv"
export PROCESSING_DATE="${DATE}"

# Validate input exists
if [[ ! -f "${INPUT_PATH}" ]]; then
    echo "Error: Input file ${INPUT_PATH} not found"
    exit 1
fi

# Run pipeline with verbose output
echo "Processing daily data for ${DATE}..."
if oxide_flow run --verbose "${PIPELINE}"; then
    echo "Daily processing completed successfully"

    # Validate output
    if [[ -f "${OUTPUT_PATH}" ]]; then
        ROWS=$(wc -l < "${OUTPUT_PATH}")
        echo "Generated ${ROWS} rows in ${OUTPUT_PATH}"
    fi
else
    echo "Daily processing failed!"
    exit 1
fi
```

**Run batch script:**
```bash
chmod +x process_daily_data.sh
./process_daily_data.sh
```

### Integration with CI/CD

**GitHub Actions example:**
```yaml
# .github/workflows/data-pipeline.yml
name: Data Pipeline
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
  workflow_dispatch:

jobs:
  process-data:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Oxide Flow
        run: |
          cargo install --path .

      - name: Setup data directories
        run: |
          mkdir -p data/input data/output

      - name: Download input data
        run: |
          curl -o data/input/daily_export.json "${{ secrets.DATA_URL }}"

      - name: Run pipeline
        env:
          INPUT_FILE: data/input/daily_export.json
          OUTPUT_FILE: data/output/processed_${{ github.run_number }}.csv
        run: |
          oxide_flow run daily_pipeline

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: processed-data
          path: data/output/
```

This comprehensive CLI documentation covers all aspects of using Oxide Flow from the command line, including practical examples and integration patterns!

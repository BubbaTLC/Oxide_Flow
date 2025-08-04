# Oxide Flow CLI Reference

This guide provides a comprehensive overview of the Oxide Flow command-line interface. For detailed information about specific commands, see the individual command references.

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

Oxide Flow provides three main commands for different aspects of pipeline management:

### [`init`](init.md) - Initialize New Project

Creates a new Oxide Flow project with default structure and configuration.

```bash
oxide_flow init [OPTIONS] [PROJECT_NAME]
```

**Key features:**
- Interactive project creation
- Default pipeline templates
- Project structure generation
- Configuration file setup

**Quick examples:**
```bash
# Interactive creation
oxide_flow init

# Create specific project
oxide_flow init my_data_project --directory ~/projects
```

[**→ Full `init` documentation**](init.md)

### [`pipeline`](pipeline.md) - Pipeline Management

Comprehensive pipeline management including discovery, creation, testing, and analysis.

```bash
oxide_flow pipeline <SUBCOMMAND> [OPTIONS]
```

**Subcommands:**
- `list` - Discover and list available pipelines
- `add` - Create new pipelines from templates
- `test` - Validate pipeline configuration and structure
- `info` - Show detailed pipeline information

**Key features:**
- Pipeline template system (6 built-in templates)
- Comprehensive validation and testing
- Step-by-step pipeline analysis
- Filtering and search capabilities

**Quick examples:**
```bash
# List all pipelines with step details
oxide_flow pipeline list --verbose

# Create ETL pipeline from template
oxide_flow pipeline add customer_etl --template etl

# Validate pipeline before running
oxide_flow pipeline test customer_etl --verbose
```

[**→ Full `pipeline` documentation**](pipeline.md)

### [`run`](run.md) - Execute Pipeline

Discovers and executes pipelines with comprehensive error handling and reporting.

```bash
oxide_flow run [OPTIONS] [PIPELINE_NAME]
```

**Key features:**
- Automatic pipeline discovery
- Detailed execution reporting
- Error recovery and retry logic
- Environment variable support

**Quick examples:**
```bash
# Run default pipeline
oxide_flow run

# Run specific pipeline with verbose output
oxide_flow run --verbose data_processor

# Run with environment variables
INPUT_FILE=data.csv oxide_flow run processor
```

[**→ Full `run` documentation**](run.md)
[**→ Full `run` documentation**](run.md)

## Usage Patterns

### Project Workflow

The typical workflow for working with Oxide Flow projects:

```bash
# 1. Initialize new project
oxide_flow init my_analytics

# 2. Navigate to project
cd my_analytics

# 3. List available pipelines
oxide_flow pipeline list

# 4. Create additional pipelines from templates
oxide_flow pipeline add data_cleaner --template etl

# 5. Validate pipeline before running
oxide_flow pipeline test data_cleaner

# 6. Run pipeline
oxide_flow run data_cleaner
```

### Development Workflow

For development and testing:

```bash
# Run with verbose output for debugging
oxide_flow run --verbose my_pipeline

# Validate pipeline structure
oxide_flow pipeline test my_pipeline --dry-run

# Test with different configurations
CONFIG_FILE=dev.json oxide_flow run test_pipeline
CONFIG_FILE=prod.json oxide_flow run test_pipeline
```

### Production Workflow

For production deployment:

```bash
# Set production environment variables
export DATA_INPUT_PATH="/data/input/daily_export.csv"
export DATA_OUTPUT_PATH="/data/output/processed_$(date +%Y%m%d).csv"
export PROCESSING_TIMEOUT="300"
export RETRY_COUNT="5"

# Validate before running
oxide_flow pipeline test production_etl --verbose

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

Oxide Flow supports dynamic configuration through environment variables, allowing flexible pipeline behavior across different environments.

### Pipeline Configuration

Environment variables can be used directly in pipeline YAML files:

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

**Runtime Environment:**
```bash
export RUST_LOG="debug"          # Rust logging level
export OXIDE_FLOW_DEBUG="1"      # Enable debug mode
export OXIDE_FLOW_WORKERS="4"    # Number of worker threads
```

See the [`run` command documentation](run.md) for detailed environment variable usage.## Exit Codes

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

### Complete Project Workflow

**Initialize and run a new project:**
```bash
# Create project
oxide_flow init hello_world
cd hello_world

# Check what pipelines are available
oxide_flow pipeline list

# Add input data
echo '{"name": "John", "age": 30}' > input.json

# Run default pipeline
oxide_flow run

# Check output
cat output.csv
```

### Pipeline Development Workflow

**Create and test custom pipelines:**
```bash
# Start in project directory
cd my_project

# Create ETL pipeline from template
oxide_flow pipeline add data_processor --template etl --description "Process customer data"

# Validate the pipeline
oxide_flow pipeline test data_processor --verbose

# Edit if needed
nano pipelines/data_processor.yaml

# Test again
oxide_flow pipeline test data_processor

# Run when ready
oxide_flow run data_processor
```

### Production Deployment

**Deploy and run pipelines in production:**
```bash
# Clone/copy project to production environment
cd /opt/pipelines/customer_analytics

# List available pipelines
oxide_flow pipeline list --verbose

# Set production environment
export DATA_INPUT_PATH="/data/input/customers.json"
export DATA_OUTPUT_PATH="/data/output/processed_customers.csv"
export PROCESSING_MODE="production"

# Validate before running
oxide_flow pipeline test customer_etl --verbose

# Run production pipeline
oxide_flow run customer_etl

# Check exit status for monitoring
echo "Pipeline exit code: $?"
```

### Multi-Pipeline Processing

**Process data through multiple pipelines:**
```bash
# Process raw data
oxide_flow run data_ingestion

# Validate processed data
oxide_flow run data_validation

# Generate reports
oxide_flow run report_generation

# Archive results
oxide_flow run data_archival
```

## Command Reference

For detailed information about each command, see:

- [`init`](init.md) - Project initialization and setup
- [`pipeline`](pipeline.md) - Pipeline management (list, add, test, info)
- [`run`](run.md) - Pipeline execution and monitoring

## Getting Help

Each command provides built-in help:

```bash
# General help
oxide_flow --help

# Command-specific help
oxide_flow init --help
oxide_flow pipeline --help
oxide_flow run --help

# Subcommand help
oxide_flow pipeline list --help
oxide_flow pipeline add --help
```
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

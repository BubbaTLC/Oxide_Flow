# `run` - Execute Pipeline

Discovers and executes a pipeline by name using the project configuration.

## Syntax

```bash
oxide_flow run [OPTIONS] [PIPELINE_NAME]
```

## Arguments

- `[PIPELINE_NAME]` - Name of the pipeline to run (default: "pipeline")

## Options

- `--config` / `-c` `<PATH>` - Path to configuration file (optional)
- `--verbose` / `-v` - Enable detailed output (global option)

## Pipeline Discovery

The `run` command discovers pipelines in the following order:
1. `{name}.yaml` in pipeline directory
2. `{name}.yml` in pipeline directory
3. `{name}/pipeline.yaml` subdirectory
4. `{name}/pipeline.yml` subdirectory

## Examples

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

## Output Examples

### Pipeline Discovery Output

```bash
$ oxide_flow run data_processor
📋 Found pipeline: ./pipelines/data_processor.yaml
🔍 Running pipeline 'data_processor' from: ./pipelines/data_processor.yaml
Running pipeline: Data Processing Pipeline
Description: Processes customer data for analysis
Steps: 5 (read_file → parse_json → flatten → format_csv → write_file)
🚀 Starting pipeline execution: Data Processing Pipeline
```

### Execution Output

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

### Error Recovery Output

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

## Error Handling

### Pipeline Not Found

```bash
$ oxide_flow run nonexistent
📂 Available pipelines in ./pipelines:
  • data_processor
  • etl_pipeline
  • json_converter
  • validation_pipeline
❌ Pipeline execution failed: Pipeline 'nonexistent' not found in ./pipelines
```

### Configuration Errors

```bash
$ oxide_flow run broken_pipeline
📋 Found pipeline: ./pipelines/broken_pipeline.yaml
❌ Pipeline execution failed: YAML syntax error at line 15: expected string, found number
```

### Runtime Errors

```bash
$ oxide_flow run file_pipeline
📋 Found pipeline: ./pipelines/file_pipeline.yaml
🔍 Running pipeline 'file_pipeline' from: ./pipelines/file_pipeline.yaml
🚀 Starting pipeline execution: File Processing Pipeline

📋 Step 1 of 3: 'reader'
🔄 Executing step 'reader' (attempt 1 of 2)
❌ Step 'reader' failed (attempt 1): File not found: input.csv
🔄 Executing step 'reader' (attempt 2 of 2)
❌ Step 'reader' failed (attempt 2): File not found: input.csv

❌ Pipeline execution failed: Step 'reader' failed after 2 attempts
📊 Summary: 0 executed, 1 failed, 2 skipped
⏱️  Total time: 45ms
❌ Pipeline execution failed: Step 'reader' failed after 2 attempts
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

## Usage Patterns

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

### Batch Processing

```bash
# Process multiple files in a loop
for file in /data/input/*.json; do
    export INPUT_FILE="$file"
    export OUTPUT_FILE="/data/output/$(basename "$file" .json).csv"

    if ! oxide_flow run json_to_csv; then
        echo "Failed processing $file"
        exit 1
    fi
done
```

## Performance Considerations

- **File I/O**: Use appropriate buffer sizes for large files
- **Memory usage**: Monitor memory consumption for large datasets
- **Parallel processing**: Some Oxis support parallel execution
- **Network operations**: Configure appropriate timeouts for external APIs

## Exit Codes

| Exit Code | Meaning |
|-----------|---------|
| `0` | Success - Pipeline completed successfully |
| `1` | General Error - Pipeline execution failed |
| `125` | Pipeline Not Found - Specified pipeline could not be located |
| `126` | Permission Error - Cannot access files or directories |
| `130` | Interrupted - User cancelled execution (Ctrl+C) |

## Related Commands

- [`init`](init.md) - Initialize a new project with pipelines
- [`pipeline`](pipeline.md) - Manage and validate pipelines before running

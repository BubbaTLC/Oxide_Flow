# Pipeline Configuration Guide

This guide covers the complete YAML configuration system for Oxide Flow pipelines, including advanced features like environment variables, step references, error handling, and schema validation.

## Table of Contents

- [Basic Pipeline Structure](#basic-pipeline-structure)
- [Pipeline Metadata](#pipeline-metadata)
- [Step Configuration](#step-configuration)
- [Error Handling & Retry Logic](#error-handling--retry-logic)
- [Environment Variables](#environment-variables)
- [Step References](#step-references)
- [Available Oxis](#available-oxis)
- [Schema Validation](#schema-validation)
- [Advanced Examples](#advanced-examples)

## Basic Pipeline Structure

Every pipeline YAML file follows this basic structure:

```yaml
pipeline:
  - name: oxi_name
    id: step_id
    config:
      key: value

metadata:
  name: "Pipeline Name"
  description: "Pipeline description"
  version: "1.0.0"
  author: "Your Name"
```

### Running Pipelines

```bash
# Run a pipeline by name (discovers in ./pipelines/)
oxide_flow run my_pipeline

# Run default pipeline
oxide_flow run
```

## Pipeline Metadata

Pipeline metadata provides information about the pipeline:

```yaml
metadata:
  name: "Data Processing Pipeline"           # Human-readable name
  description: "Processes customer data"    # What the pipeline does
  version: "2.1.0"                         # Semantic version
  author: "Data Team"                      # Who created it
  tags: ["data", "etl", "customers"]      # Optional tags
  created: "2024-01-15"                   # Optional creation date
```

## Step Configuration

Each pipeline step represents an Oxi (plugin) execution:

### Basic Step

```yaml
- name: read_file          # Oxi name (required)
  id: reader              # Unique step ID (optional)
  config:                 # Oxi-specific configuration
    path: "data.json"
```

### Step with Error Handling

```yaml
- name: parse_json
  id: parser
  config:
    strict: true
  continue_on_error: false    # Stop pipeline if this step fails (default: false)
  retry_attempts: 2          # Retry up to 2 times on failure (default: 0)
  timeout_seconds: 30        # Timeout after 30 seconds (optional)
```

## Error Handling & Retry Logic

Oxide Flow provides sophisticated error handling capabilities:

### Retry Configuration

```yaml
- name: read_file
  id: unreliable_reader
  config:
    path: "remote://data.json"
  retry_attempts: 3           # Try up to 4 times total (1 + 3 retries)
  timeout_seconds: 60         # Each attempt times out after 60 seconds
```

### Continue on Error

```yaml
pipeline:
  # Step 1: Try to read primary data source
  - name: read_file
    id: primary_reader
    config:
      path: "primary.json"
    continue_on_error: true   # Continue even if this fails
    retry_attempts: 1

  # Step 2: Fallback to secondary source
  - name: read_file
    id: fallback_reader
    config:
      path: "backup.json"
    continue_on_error: false  # This must succeed
```

### Error Handling Behavior

- **`continue_on_error: false`** (default): Pipeline stops on step failure
- **`continue_on_error: true`**: Pipeline continues with same data
- **`retry_attempts: N`**: Retries failed step N times with exponential backoff
- **`timeout_seconds: N`**: Each attempt times out after N seconds

## Environment Variables

Use environment variables for dynamic configuration:

### Basic Environment Variables

```yaml
- name: read_file
  id: reader
  config:
    path: "${INPUT_FILE}"           # Required environment variable
    format: "${FORMAT:-json}"       # Optional with default value
```

### Environment Variable Patterns

```yaml
config:
  # Required variable (fails if not set)
  database_url: "${DATABASE_URL}"

  # Optional with default
  timeout: "${TIMEOUT:-30}"

  # Nested in complex values
  connection_string: "host=${DB_HOST:-localhost};port=${DB_PORT:-5432}"

  # In arrays
  endpoints:
    - "${API_ENDPOINT_1}"
    - "${API_ENDPOINT_2:-https://backup.api.com}"
```

### Setting Environment Variables

```bash
# Set environment variables before running
export INPUT_FILE="data.json"
export FORMAT="csv"
oxide_flow run my_pipeline

# Or inline
INPUT_FILE="data.json" FORMAT="csv" oxide_flow run my_pipeline
```

## Step References

Reference outputs from previous steps using dot notation:

### Basic Step References

```yaml
pipeline:
  # Step 1: Read and process data
  - name: read_file
    id: reader
    config:
      path: "input.json"

  # Step 2: Use data from step 1
  - name: write_file
    id: writer
    config:
      path: "output_${steps.reader.filename}.processed"  # References reader step
```

### Advanced Step References

```yaml
pipeline:
  # Step 1: Read configuration
  - name: read_file
    id: config_reader
    config:
      path: "config.json"

  # Step 2: Process data using config from step 1
  - name: transform_data
    id: transformer
    config:
      output_format: "${steps.config_reader.format}"
      batch_size: "${steps.config_reader.batch_size:-100}"

  # Step 3: Use metadata from transformer
  - name: write_file
    id: writer
    config:
      path: "results_${steps.transformer.timestamp}.${steps.config_reader.format}"
```

### Available Step Reference Data

Step references provide access to:
- Oxi output metadata
- File information (name, size, etc.)
- Processing results
- Custom data set by individual Oxis

## Available Oxis

### File I/O Oxis

#### `read_file` - Read File Content

```yaml
- name: read_file
  config:
    path: "data.json"              # File path (required)
    encoding: "utf-8"              # Text encoding (optional)
```

#### `write_file` - Write File Content

```yaml
- name: write_file
  config:
    path: "output.csv"             # Output path (required)
    overwrite: true                # Overwrite existing (default: true)
    create_dirs: true              # Create directories (default: true)
```

### Data Format Oxis

#### `parse_json` - Parse JSON Data

```yaml
- name: parse_json
  config:
    strict: true                   # Strict JSON parsing (default: true)
    array_handling: "flatten"      # How to handle arrays: "flatten", "preserve"
```

#### `format_csv` - Format as CSV

```yaml
- name: format_csv
  config:
    headers: true                  # Include headers (default: true)
    delimiter: ","                 # Field delimiter (default: ",")
    quote_char: "\""               # Quote character (default: "\"")
    escape_char: "\\"              # Escape character (optional)
```

#### `parse_csv` - Parse CSV Data

```yaml
- name: parse_csv
  config:
    headers: true                  # First row contains headers (default: true)
    delimiter: ","                 # Field delimiter (default: ",")
    skip_rows: 0                   # Rows to skip (default: 0)
```

#### `format_json` - Format as JSON

```yaml
- name: format_json
  config:
    pretty: true                   # Pretty print (default: true)
    indent: 2                      # Indentation spaces (default: 2)
```

### Transformation Oxis

#### `flatten` - Flatten Nested Data

```yaml
- name: flatten
  config:
    separator: "."                 # Key separator (default: ".")
    max_depth: 10                  # Maximum nesting depth (default: unlimited)
```

### I/O Oxis

#### `read_stdin` - Read from Standard Input

```yaml
- name: read_stdin
  config:
    prompt: "Enter data:"          # User prompt (optional)
    timeout: 30                    # Input timeout seconds (optional)
```

#### `write_stdout` - Write to Standard Output

```yaml
- name: write_stdout
  config:
    newline: true                  # Add trailing newline (default: true)
```

## Schema Validation

Oxide Flow automatically validates step configurations against schemas:

### Built-in Validation

```yaml
- name: format_csv
  config:
    delimiter: "||"               # ❌ Invalid: must be single character
    headers: "yes"                # ❌ Invalid: must be boolean
```

### Validation Errors

```
❌ Configuration validation failed for step 'formatter' (oxi: 'format_csv'):
  • Invalid property value for 'delimiter': Must match pattern: ^.{1}$
  • Invalid property type for 'headers': Expected boolean, got string
```

### Custom Validation

You can extend validation by adding custom schemas for your Oxis.

## Advanced Examples

### Complex Data Processing Pipeline

```yaml
pipeline:
  # Read configuration
  - name: read_file
    id: config
    config:
      path: "${CONFIG_FILE:-config.json}"
    retry_attempts: 2
    timeout_seconds: 10

  # Read input data with fallback
  - name: read_file
    id: primary_data
    config:
      path: "${steps.config.primary_source}"
    continue_on_error: true
    retry_attempts: 3

  # Fallback data source
  - name: read_file
    id: fallback_data
    config:
      path: "${steps.config.fallback_source}"
    continue_on_error: false
    retry_attempts: 1

  # Parse and validate JSON
  - name: parse_json
    id: parser
    config:
      strict: true
    retry_attempts: 1

  # Transform data structure
  - name: flatten
    id: flattener
    config:
      separator: "_"
      max_depth: 5

  # Format as CSV with dynamic delimiter
  - name: format_csv
    id: csv_formatter
    config:
      headers: true
      delimiter: "${CSV_DELIMITER:-,}"

  # Write to timestamped output
  - name: write_file
    id: writer
    config:
      path: "output/data_${steps.parser.timestamp}.csv"
      create_dirs: true

metadata:
  name: "Robust Data Processing Pipeline"
  description: "Processes JSON data with multiple fallbacks and error handling"
  version: "2.0.0"
  author: "Data Engineering Team"
```

### Error Recovery Pipeline

```yaml
pipeline:
  # Try multiple data sources
  - name: read_file
    id: source1
    config:
      path: "data/primary.json"
    continue_on_error: true
    retry_attempts: 2

  - name: read_file
    id: source2
    config:
      path: "data/secondary.json"
    continue_on_error: true
    retry_attempts: 1

  - name: read_stdin
    id: manual_input
    config:
      prompt: "Please enter JSON data manually:"
    continue_on_error: false

  # Process whatever data we got
  - name: parse_json
    id: parser
    retry_attempts: 2

  - name: write_stdout
    id: output

metadata:
  name: "Fault-Tolerant Data Reader"
  description: "Tries multiple sources with manual fallback"
```

### Dynamic Pipeline with Environment Configuration

```yaml
pipeline:
  - name: read_file
    id: reader
    config:
      path: "${INPUT_PATH}"
    timeout_seconds: "${READ_TIMEOUT:-60}"
    retry_attempts: "${RETRY_COUNT:-3}"

  - name: parse_json
    id: parser
    config:
      strict: "${STRICT_MODE:-true}"

  - name: "${OUTPUT_FORMAT:-format_csv}"
    id: formatter
    config:
      headers: "${INCLUDE_HEADERS:-true}"
      delimiter: "${DELIMITER:-,}"

  - name: write_file
    id: writer
    config:
      path: "${OUTPUT_PATH:-output.${OUTPUT_FORMAT}}"
      overwrite: "${OVERWRITE:-true}"

metadata:
  name: "Configurable Processing Pipeline"
  description: "Fully configurable via environment variables"
```

## Best Practices

### 1. Use Meaningful Step IDs
```yaml
# Good
- name: read_file
  id: customer_data_reader

# Avoid
- name: read_file
  id: step1
```

### 2. Handle Errors Gracefully
```yaml
# For critical steps
continue_on_error: false
retry_attempts: 3

# For optional steps
continue_on_error: true
retry_attempts: 1
```

### 3. Use Environment Variables for Configuration
```yaml
# Good - configurable
path: "${DATA_DIR}/input.json"

# Avoid - hardcoded
path: "/home/user/data/input.json"
```

### 4. Provide Meaningful Metadata
```yaml
metadata:
  name: "Customer Data ETL"
  description: "Extracts customer data from JSON, transforms it, and loads to CSV"
  version: "1.2.0"
  author: "Data Team"
```

### 5. Use Step References for Dynamic Behavior
```yaml
# Dynamic output naming
path: "processed_${steps.reader.filename}_${steps.processor.timestamp}.csv"
```

## Troubleshooting

### Common Issues

1. **Pipeline Not Found**
   ```
   📂 Available pipelines in ./pipelines:
     • data_processing
     • customer_etl
   ```
   Solution: Check pipeline name and ensure it exists in the pipelines directory.

2. **Environment Variable Not Set**
   ```
   ❌ Environment variable 'DATABASE_URL' is required but not set
   ```
   Solution: Set the required environment variable or provide a default value.

3. **Step Reference Error**
   ```
   ❌ Step reference 'steps.nonexistent.value' not found
   ```
   Solution: Ensure the referenced step ID exists and ran successfully.

4. **Schema Validation Error**
   ```
   ❌ Invalid property value for 'delimiter': Must match pattern: ^.{1}$
   ```
   Solution: Fix the configuration to match the expected schema.

This comprehensive guide covers all YAML configuration capabilities in Oxide Flow. For more examples, check the `example_project/pipelines/` directory.

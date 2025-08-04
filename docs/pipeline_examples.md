# Pipeline Examples

This document provides practical, real-world examples of Oxide Flow pipelines for common data processing tasks.

## Table of Contents

- [Basic Examples](#basic-examples)
- [Data Transformation](#data-transformation)
- [Error Handling Patterns](#error-handling-patterns)
- [Dynamic Configuration](#dynamic-configuration)
- [Production Pipelines](#production-pipelines)

## Basic Examples

### Simple JSON to CSV Conversion

**Use Case:** Convert a JSON file to CSV format with headers.

```yaml
pipeline:
  - name: read_file
    id: reader
    config:
      path: "data.json"

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
  description: "Simple conversion from JSON to CSV"
```

**Run with:**
```bash
oxide_flow run json_to_csv
```

---

### Data Validation Pipeline

**Use Case:** Read data, validate it's proper JSON, and output results.

```yaml
pipeline:
  - name: read_stdin
    id: input
    config:
      prompt: "Enter JSON data to validate:"

  - name: parse_json
    id: validator
    config:
      strict: true
    retry_attempts: 0  # Don't retry validation failures

  - name: write_stdout
    id: output

metadata:
  name: "JSON Validator"
  description: "Validates JSON data from user input"
```

---

### File Processing with Backup

**Use Case:** Process a file with automatic fallback to backup file.

```yaml
pipeline:
  # Try primary file first
  - name: read_file
    id: primary
    config:
      path: "data/primary.json"
    continue_on_error: true
    retry_attempts: 1

  # Fallback to backup if primary fails
  - name: read_file
    id: backup
    config:
      path: "data/backup.json"
    continue_on_error: false

  - name: parse_json
    id: parser

  - name: write_stdout
    id: output

metadata:
  name: "Resilient File Reader"
  description: "Reads from primary source with backup fallback"
```

## Data Transformation

### Flatten Complex JSON

**Use Case:** Flatten nested JSON data for easier CSV export.

```yaml
pipeline:
  - name: read_file
    id: reader
    config:
      path: "${INPUT_FILE}"

  - name: parse_json
    id: parser

  - name: flatten
    id: flattener
    config:
      separator: "_"
      max_depth: 10

  - name: format_csv
    id: csv_out
    config:
      headers: true
      delimiter: ","

  - name: write_file
    id: writer
    config:
      path: "${OUTPUT_FILE:-flattened_output.csv}"

metadata:
  name: "JSON Flattener"
  description: "Flattens nested JSON and exports to CSV"
```

**Environment Setup:**
```bash
export INPUT_FILE="complex_data.json"
export OUTPUT_FILE="flat_data.csv"
oxide_flow run flatten_json
```

---

### Data Format Converter

**Use Case:** Universal data format converter supporting JSON and CSV.

```yaml
pipeline:
  - name: read_file
    id: reader
    config:
      path: "${INPUT_FILE}"

  # Parse input based on format
  - name: "${INPUT_FORMAT:-parse_json}"
    id: parser
    config:
      headers: "${CSV_HAS_HEADERS:-true}"  # Only used if parsing CSV
      delimiter: "${CSV_DELIMITER:-,}"      # Only used if parsing CSV

  # Format output based on target format
  - name: "${OUTPUT_FORMAT:-format_csv}"
    id: formatter
    config:
      headers: "${INCLUDE_HEADERS:-true}"   # Only used if formatting CSV
      delimiter: "${OUT_DELIMITER:-,}"      # Only used if formatting CSV
      pretty: "${PRETTY_JSON:-true}"        # Only used if formatting JSON

  - name: write_file
    id: writer
    config:
      path: "${OUTPUT_FILE}"

metadata:
  name: "Universal Data Converter"
  description: "Converts between JSON and CSV formats"
```

**Usage Examples:**
```bash
# JSON to CSV
INPUT_FILE="data.json" OUTPUT_FILE="data.csv" \
INPUT_FORMAT="parse_json" OUTPUT_FORMAT="format_csv" \
oxide_flow run data_converter

# CSV to JSON
INPUT_FILE="data.csv" OUTPUT_FILE="data.json" \
INPUT_FORMAT="parse_csv" OUTPUT_FORMAT="format_json" \
PRETTY_JSON="true" \
oxide_flow run data_converter
```

## Error Handling Patterns

### Robust ETL Pipeline

**Use Case:** Production ETL with comprehensive error handling.

```yaml
pipeline:
  # Extract: Try multiple data sources
  - name: read_file
    id: primary_source
    config:
      path: "${PRIMARY_DATA_URL}"
    continue_on_error: true
    retry_attempts: 3
    timeout_seconds: 30

  - name: read_file
    id: secondary_source
    config:
      path: "${SECONDARY_DATA_URL}"
    continue_on_error: true
    retry_attempts: 2
    timeout_seconds: 60

  - name: read_file
    id: local_cache
    config:
      path: "cache/last_known_good.json"
    continue_on_error: false  # Must have some data

  # Transform: Parse and validate
  - name: parse_json
    id: parser
    config:
      strict: false  # Allow some malformed data
    retry_attempts: 1

  - name: flatten
    id: transformer
    config:
      separator: "."
      max_depth: 5
    continue_on_error: true  # Continue even if flattening fails

  # Load: Save to multiple formats
  - name: format_csv
    id: csv_formatter
    config:
      headers: true
      delimiter: ","
    continue_on_error: true

  - name: write_file
    id: csv_writer
    config:
      path: "output/data_${steps.parser.timestamp}.csv"
      create_dirs: true
    continue_on_error: true

  - name: format_json
    id: json_formatter
    config:
      pretty: true

  - name: write_file
    id: json_writer
    config:
      path: "output/data_${steps.parser.timestamp}.json"
      create_dirs: true

metadata:
  name: "Production ETL Pipeline"
  description: "Robust ETL with multiple fallbacks and error recovery"
```

---

### Retry with Exponential Backoff

**Use Case:** Handle flaky network resources with intelligent retry.

```yaml
pipeline:
  # Network resource with aggressive retry
  - name: read_file
    id: api_data
    config:
      path: "${API_ENDPOINT}/data.json"
    retry_attempts: 5      # Try 6 times total
    timeout_seconds: 10    # 10 second timeout per attempt
    continue_on_error: false

  - name: parse_json
    id: parser
    retry_attempts: 1      # Retry parsing once

  - name: write_file
    id: cache_writer
    config:
      path: "cache/api_data_backup.json"
    continue_on_error: true  # Don't fail if can't cache

  - name: format_csv
    id: formatter

  - name: write_stdout
    id: output

metadata:
  name: "Network Data Processor"
  description: "Handles unreliable network data sources"
```

## Dynamic Configuration

### Environment-Driven Pipeline

**Use Case:** Single pipeline that adapts behavior based on environment.

```yaml
pipeline:
  # Input source varies by environment
  - name: "${INPUT_SOURCE:-read_file}"
    id: data_input
    config:
      path: "${DATA_PATH:-data.json}"      # For read_file
      prompt: "${INPUT_PROMPT:-Enter data:}" # For read_stdin
    retry_attempts: "${RETRY_COUNT:-2}"
    timeout_seconds: "${TIMEOUT:-30}"

  # Parser configuration from environment
  - name: parse_json
    id: parser
    config:
      strict: "${STRICT_PARSING:-true}"
      array_handling: "${ARRAY_MODE:-preserve}"

  # Conditional transformation
  - name: flatten
    id: flattener
    config:
      separator: "${FLATTEN_SEP:-.}"
      max_depth: "${MAX_DEPTH:-5}"
    continue_on_error: "${ALLOW_FLATTEN_FAIL:-true}"

  # Output format determined by environment
  - name: "${OUTPUT_FORMAT:-format_csv}"
    id: formatter
    config:
      # CSV options
      headers: "${CSV_HEADERS:-true}"
      delimiter: "${CSV_DELIM:-,}"
      # JSON options
      pretty: "${JSON_PRETTY:-true}"
      indent: "${JSON_INDENT:-2}"

  # Output destination varies
  - name: "${OUTPUT_SINK:-write_stdout}"
    id: output
    config:
      path: "${OUTPUT_PATH}"  # Only used for write_file
      newline: "${ADD_NEWLINE:-true}"  # Only used for write_stdout

metadata:
  name: "Adaptive Processing Pipeline"
  description: "Completely configurable via environment variables"
```

**Environment Configurations:**

**Development Mode:**
```bash
export INPUT_SOURCE="read_stdin"
export INPUT_PROMPT="Paste your test JSON:"
export STRICT_PARSING="false"
export OUTPUT_FORMAT="format_json"
export OUTPUT_SINK="write_stdout"
export JSON_PRETTY="true"
```

**Production Mode:**
```bash
export INPUT_SOURCE="read_file"
export DATA_PATH="/data/input/daily_export.json"
export STRICT_PARSING="true"
export OUTPUT_FORMAT="format_csv"
export OUTPUT_SINK="write_file"
export OUTPUT_PATH="/data/output/processed_$(date +%Y%m%d).csv"
export RETRY_COUNT="5"
export TIMEOUT="120"
```

---

### Pipeline with Step References

**Use Case:** Dynamic behavior based on previous step outputs.

```yaml
pipeline:
  # Read configuration file first
  - name: read_file
    id: config_reader
    config:
      path: "${CONFIG_FILE:-config.json}"

  - name: parse_json
    id: config_parser

  # Use config to determine input file
  - name: read_file
    id: data_reader
    config:
      path: "${steps.config_parser.input_file}"
    retry_attempts: "${steps.config_parser.retry_count:-2}"
    timeout_seconds: "${steps.config_parser.timeout:-30}"

  - name: parse_json
    id: data_parser
    config:
      strict: "${steps.config_parser.strict_mode:-true}"

  # Transform based on config
  - name: flatten
    id: flattener
    config:
      separator: "${steps.config_parser.flatten_separator:-.}"
      max_depth: "${steps.config_parser.max_depth:-10}"

  # Output with dynamic naming
  - name: format_csv
    id: formatter
    config:
      headers: "${steps.config_parser.include_headers:-true}"
      delimiter: "${steps.config_parser.csv_delimiter:-,}"

  - name: write_file
    id: writer
    config:
      path: "${steps.config_parser.output_dir}/processed_${steps.data_reader.filename}_${steps.data_parser.timestamp}.csv"

metadata:
  name: "Config-Driven Pipeline"
  description: "Uses configuration file to control all behavior"
```

**Sample config.json:**
```json
{
  "input_file": "data/daily_export.json",
  "output_dir": "output/processed",
  "retry_count": 3,
  "timeout": 60,
  "strict_mode": true,
  "flatten_separator": "_",
  "max_depth": 8,
  "include_headers": true,
  "csv_delimiter": "|"
}
```

## Production Pipelines

### Log Processing Pipeline

**Use Case:** Process application logs for analysis.

```yaml
pipeline:
  - name: read_file
    id: log_reader
    config:
      path: "${LOG_FILE}"
    retry_attempts: 2
    timeout_seconds: 60

  # Parse JSON logs (one per line)
  - name: parse_json
    id: log_parser
    config:
      strict: false  # Logs might have malformed entries
      array_handling: "flatten"
    continue_on_error: true  # Don't stop on bad log entries

  # Extract relevant fields
  - name: flatten
    id: log_flattener
    config:
      separator: "_"
      max_depth: 3

  # Convert to CSV for analysis tools
  - name: format_csv
    id: csv_formatter
    config:
      headers: true
      delimiter: ","
      quote_all: false

  # Save with timestamp
  - name: write_file
    id: csv_writer
    config:
      path: "processed_logs/logs_${steps.log_parser.date}.csv"
      create_dirs: true

  # Also keep JSON backup
  - name: format_json
    id: json_formatter
    config:
      pretty: false  # Compact for storage

  - name: write_file
    id: json_backup
    config:
      path: "backup/logs_${steps.log_parser.date}.json"
      create_dirs: true
    continue_on_error: true  # Backup is optional

metadata:
  name: "Log Processing Pipeline"
  description: "Converts JSON logs to CSV for analysis"
  version: "2.1.0"
  author: "DevOps Team"
```

---

### Data Quality Pipeline

**Use Case:** Validate and clean incoming data.

```yaml
pipeline:
  # Read input data
  - name: read_file
    id: raw_data
    config:
      path: "${INPUT_DATA}"
    retry_attempts: 3

  # Parse and validate structure
  - name: parse_json
    id: structure_validator
    config:
      strict: true
    # Don't retry - either valid JSON or not

  # Create validation report
  - name: flatten
    id: field_extractor
    config:
      separator: "."
      max_depth: 10

  # Check for required fields
  - name: format_json
    id: validation_report
    config:
      pretty: true

  - name: write_file
    id: report_writer
    config:
      path: "reports/validation_${steps.structure_validator.timestamp}.json"
      create_dirs: true

  # Clean and standardize data
  - name: format_csv
    id: clean_formatter
    config:
      headers: true
      delimiter: ","

  - name: write_file
    id: clean_output
    config:
      path: "clean_data/cleaned_${steps.raw_data.filename}.csv"
      create_dirs: true

metadata:
  name: "Data Quality Pipeline"
  description: "Validates, cleans, and reports on data quality"
  version: "1.0.0"
  author: "Data Quality Team"
```

---

### Batch Processing Pipeline

**Use Case:** Process multiple files in a batch operation.

```yaml
pipeline:
  # Process file 1
  - name: read_file
    id: file1_reader
    config:
      path: "${BATCH_DIR}/file1.json"
    continue_on_error: true
    retry_attempts: 2

  - name: parse_json
    id: file1_parser
    continue_on_error: true

  # Process file 2
  - name: read_file
    id: file2_reader
    config:
      path: "${BATCH_DIR}/file2.json"
    continue_on_error: true
    retry_attempts: 2

  - name: parse_json
    id: file2_parser
    continue_on_error: true

  # Merge results (simplified - would need custom Oxi for real merging)
  - name: format_json
    id: merger
    config:
      pretty: true

  # Output combined results
  - name: write_file
    id: batch_output
    config:
      path: "${OUTPUT_DIR}/batch_${steps.file1_parser.timestamp}.json"
      create_dirs: true

  # Create processing summary
  - name: format_csv
    id: summary_formatter
    config:
      headers: true

  - name: write_file
    id: summary_writer
    config:
      path: "${OUTPUT_DIR}/batch_summary_${steps.file1_parser.timestamp}.csv"
    continue_on_error: true

metadata:
  name: "Batch File Processor"
  description: "Processes multiple files and creates combined output"
```

## Tips for Writing Effective Pipelines

### 1. Use Descriptive IDs and Names
```yaml
# Good
- name: read_file
  id: customer_data_loader

# Better
- name: read_file
  id: daily_customer_export_loader
```

### 2. Handle Errors Appropriately
```yaml
# Critical step - must succeed
- name: parse_json
  id: schema_validator
  continue_on_error: false
  retry_attempts: 0

# Optional step - can fail
- name: write_file
  id: backup_writer
  continue_on_error: true
  retry_attempts: 1
```

### 3. Use Environment Variables for Flexibility
```yaml
# Production-ready configuration
config:
  input_path: "${DATA_INPUT_PATH}"
  output_path: "${DATA_OUTPUT_PATH}"
  batch_size: "${PROCESSING_BATCH_SIZE:-1000}"
  timeout: "${PROCESSING_TIMEOUT:-300}"
```

### 4. Document Your Pipelines
```yaml
metadata:
  name: "Customer Data ETL"
  description: "Daily processing of customer data from CRM to data warehouse"
  version: "2.3.1"
  author: "Data Engineering Team"
  tags: ["etl", "customers", "daily"]
  documentation: "https://wiki.company.com/data/customer-etl"
```

### 5. Plan for Monitoring and Debugging
```yaml
# Add intermediate outputs for debugging
- name: write_file
  id: debug_checkpoint
  config:
    path: "debug/checkpoint_${steps.processor.timestamp}.json"
  continue_on_error: true  # Don't fail pipeline if debug write fails
```

These examples should give you a solid foundation for building robust, production-ready Oxide Flow pipelines!

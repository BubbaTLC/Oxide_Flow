# Oxi Reference Guide

Quick reference for all available Oxis (plugins) and their configuration options.

## File I/O Oxis

### `read_file` - Read File Content

Reads content from a file and outputs it as data. Supports text, JSON, and binary files with automatic content type detection.

**Configuration:**
```yaml
- name: read_file
  config:
    path: string              # File path (required)
    encoding: string          # Text encoding (default: "utf-8")
    binary: boolean           # Force binary mode (default: false)
```

**Output:** File content as Text, JSON, or Binary data
**Schema Strategy:** Infer (detects JSON vs text automatically)
**Metadata:** `path`, `size`, `content_type`

**Example:**
```yaml
- name: read_file
  id: data_reader
  config:
    path: "${DATA_FILE:-input.json}"
    encoding: "utf-8"
```

---

### `write_file` - Write Data to File

Writes input data to a specified file with automatic directory creation and backup options.

**Configuration:**
```yaml
- name: write_file
  config:
    path: string              # Output file path (required)
    append: boolean           # Append to existing file (default: false)
    create_dirs: boolean      # Create parent directories (default: true)
    encoding: string          # Text encoding (default: "utf-8")
    backup: boolean           # Create backup of existing file (default: false)
```

**Input:** Any data type
**Output:** Empty data
**Schema Strategy:** Passthrough
**Metadata:** `path`, `size`, `backup_path` (if backup created)

**Example:**
```yaml
- name: write_file
  id: output_writer
  config:
    path: "output/result_${steps.processor.timestamp}.csv"
    create_dirs: true
    backup: true
```

---

## Standard I/O Oxis

### `read_stdin` - Read from Standard Input

Reads data from standard input with configurable prompts and timeouts.

**Configuration:**
```yaml
- name: read_stdin
  config:
    prompt: string            # User prompt message (optional)
    timeout_seconds: number   # Read timeout in seconds (optional)
    echo: boolean             # Echo input to stderr (default: true)
```

**Output:** Text data from stdin
**Schema Strategy:** Infer (attempts JSON parsing, falls back to text)
**Metadata:** `bytes_read`, `read_time`

**Example:**
```yaml
- name: read_stdin
  id: user_input
  config:
    prompt: "Enter JSON data:"
    timeout_seconds: 30
    echo: false
```

---

### `write_stdout` - Write to Standard Output

Writes data to standard output with formatting options.

**Configuration:**
```yaml
- name: write_stdout
  config:
    newline: boolean          # Add trailing newline (default: true)
    prefix: string            # Prefix for each line (optional)
    json_pretty: boolean      # Pretty-print JSON data (default: false)
```

**Input:** Any data type
**Output:** Same data (passthrough)
**Schema Strategy:** Passthrough
**Metadata:** `bytes_written`, `lines_written`

**Example:**
```yaml
- name: write_stdout
  id: console_output
  config:
    newline: true
    prefix: "📄 "
    json_pretty: true
```

---

## JSON Processing Oxis

### `parse_json` - Parse JSON Data

Parses JSON text into structured data with comprehensive error handling and validation.

**Configuration:**
```yaml
- name: parse_json
  config:
    strict: boolean           # Strict JSON parsing (default: true)
    allow_comments: boolean   # Allow JSON with comments (default: false)
    allow_trailing_commas: boolean # Allow trailing commas (default: false)
```

**Input:** Text data containing JSON
**Output:** Structured JSON data
**Schema Strategy:** Modify (converts text schema to JSON schema)
**Metadata:** `parsed_at`, `json_type`, `size_bytes`

**Error Handling:**
- Provides detailed parsing error messages with line/column information
- Validates JSON structure before processing
- Supports both strict and lenient parsing modes

**Example:**
```yaml
- name: parse_json
  id: json_parser
  config:
    strict: false
    allow_comments: true
```

---

### `format_json` - Format Data as JSON

Formats structured data as JSON text with customizable formatting options.

**Configuration:**
```yaml
- name: format_json
  config:
    pretty: boolean           # Pretty print with indentation (default: true)
    indent: number            # Indentation spaces (default: 2)
    sort_keys: boolean        # Sort object keys alphabetically (default: false)
    compact: boolean          # Minimize whitespace (default: false)
```

**Input:** Structured data (JSON objects, arrays, primitives)
**Output:** JSON text
**Schema Strategy:** Modify (converts data schema to text schema)
**Metadata:** `formatted_at`, `output_size`, `line_count`

**Example:**
```yaml
- name: format_json
  id: json_formatter
  config:
    pretty: true
    indent: 4
    sort_keys: true
```

---

### `json_select` - JSON Path Selection

Extracts specific data from JSON structures using JSONPath-style selectors. Essential for working with complex nested JSON where you need to extract specific arrays or objects.

**Configuration:**
```yaml
- name: json_select
  config:
    path: string              # JSON path selector (required)
    strict: boolean           # Fail if path not found (default: true)
    default_on_missing: any   # Default value when path missing and strict=false (optional)
```

**Input:** Structured JSON data (objects, arrays, primitives)
**Output:** Selected JSON data portion
**Schema Strategy:** Modify (extracts subset of input schema)
**Metadata:** `selected_path`, `extraction_time`, `result_type`

**Path Syntax:**
- **Array indices**: `[0]`, `[1]`, `[99]` - Select array elements by index
- **Object keys**: `users`, `.profile`, `data.items` - Select object properties
- **Complex paths**: `[0].users[1].profile` - Chain selectors for deep extraction
- **Mixed access**: `data.users[0]`, `items[2].metadata` - Combine object and array access

**Error Handling:**
- **Index out of bounds**: Clear error with array length information
- **Missing keys**: Specific error indicating which key wasn't found
- **Type mismatches**: Detailed error when expecting array but finding object (or vice versa)
- **Invalid syntax**: Parse errors for malformed path expressions

**Example:**
```yaml
- name: json_select
  id: extract_users
  config:
    path: "[0].users"
    strict: true
```

**Path Selection Examples:**
```json
// Input data
[
  {
    "metadata": {"count": 2},
    "users": [
      {"id": 1, "name": "Alice", "profile": {"age": 30}},
      {"id": 2, "name": "Bob", "profile": {"age": 25}}
    ]
  }
]

// path: "[0].users" → extracts the users array
[
  {"id": 1, "name": "Alice", "profile": {"age": 30}},
  {"id": 2, "name": "Bob", "profile": {"age": 25}}
]

// path: "[0].users[0].profile" → extracts Alice's profile
{"age": 30}

// path: "[0].metadata.count" → extracts the count value
2
```

**Use Cases:**
- Extract arrays from complex API responses
- Navigate nested configuration structures
- Select specific objects from multi-level JSON
- Filter data before transformation steps
- Handle API responses with wrapper objects

**Error Examples:**
```yaml
# Missing path (strict mode)
path: "[0].nonexistent"  # → Error: Key 'nonexistent' not found

# Index out of bounds
path: "[99].data"        # → Error: Array index 99 out of bounds (array length: 1)

# Type mismatch
path: "users[0]"         # → Error: Expected array but got object when looking for key 'users'
```

**Non-strict Mode:**
```yaml
- name: json_select
  config:
    path: "[0].maybe_missing"
    strict: false
    default_on_missing: []  # Return empty array if path not found
```

---

## CSV Processing Oxis

### `parse_csv` - Parse CSV Data

Parses CSV text into structured data with flexible configuration for various CSV formats.

**Configuration:**
```yaml
- name: parse_csv
  config:
    headers: boolean          # First row contains headers (default: true)
    delimiter: string         # Field delimiter (default: ",")
    quote_char: string        # Quote character (default: "\"")
    escape_char: string       # Escape character (optional)
    skip_rows: number         # Number of rows to skip (default: 0)
    trim_whitespace: boolean  # Trim leading/trailing whitespace (default: true)
```

**Input:** Text data containing CSV
**Output:** Structured JSON data (array of objects if headers=true, array of arrays if headers=false)
**Schema Strategy:** Modify (converts text to structured data schema)
**Metadata:** `rows_parsed`, `columns_detected`, `headers_used`

**Example:**
```yaml
- name: parse_csv
  id: csv_parser
  config:
    headers: true
    delimiter: "${CSV_DELIMITER:-,}"
    skip_rows: 1
    trim_whitespace: true
```

---

### `format_csv` - Format Data as CSV

Formats structured data as CSV text with comprehensive formatting control.

**Configuration:**
```yaml
- name: format_csv
  config:
    headers: boolean          # Include column headers (default: true)
    delimiter: string         # Field delimiter (default: ",", must be single char)
    quote_char: string        # Quote character (default: "\"", must be single char)
    escape_char: string       # Escape character (optional, single char)
    quote_style: string       # "minimal", "all", "non_numeric", "none" (default: "minimal")
    line_ending: string       # "unix" (\n), "windows" (\r\n), "mac" (\r) (default: "unix")
```

**Input:** Structured data (JSON objects/arrays)
**Output:** CSV text
**Schema Strategy:** Modify (converts structured data to text schema)
**Metadata:** `rows_formatted`, `columns_count`, `output_size`

**Schema Validation:**
- `delimiter` must be exactly one character
- `quote_char` must be exactly one character
- `escape_char` must be exactly one character (if specified)
- `quote_style` must be one of the allowed values

**Example:**
```yaml
- name: format_csv
  id: csv_formatter
  config:
    headers: true
    delimiter: "|"
    quote_style: "all"
    line_ending: "windows"
```

---

## Data Transformation Oxis

### `flatten` - Flatten Nested Data Structures

Flattens nested JSON objects into flat key-value pairs with configurable depth and array handling.

**Configuration:**
```yaml
- name: flatten
  config:
    separator: string         # Key separator (default: ".")
    max_depth: number         # Maximum nesting depth (optional, no limit if not set)
    preserve_arrays: boolean  # Keep arrays intact (default: false)
    array_index_format: string # Format for array indices (default: "[{index}]")
```

**Input:** Structured JSON data (objects, arrays, primitives)
**Output:** Flattened structured data
**Schema Strategy:** Modify (transforms nested schema to flat schema)
**Metadata:** `flattened_keys`, `original_depth`, `final_depth`

**Flattening Behavior:**
- Objects: Keys joined with separator (e.g., `user.name`)
- Arrays (preserve_arrays=false): Indexed with format (e.g., `items[0].name`)
- Arrays (preserve_arrays=true): Kept as arrays in the output
- Primitive values: Passed through unchanged

**Example:**
```yaml
- name: flatten
  id: flattener
  config:
    separator: "_"
    max_depth: 5
    preserve_arrays: true
    array_index_format: ".{index}"
```

**Flattening Example:**
```json
// Input
{
  "user": {
    "name": "John",
    "address": {
      "city": "NYC",
      "zip": "10001"
    },
    "tags": ["admin", "user"]
  }
}

// Output (separator: ".", preserve_arrays: false)
{
  "user.name": "John",
  "user.address.city": "NYC",
  "user.address.zip": "10001",
  "user.tags[0]": "admin",
  "user.tags[1]": "user"
}

// Output (separator: ".", preserve_arrays: true)
{
  "user.name": "John",
  "user.address.city": "NYC",
  "user.address.zip": "10001",
  "user.tags": ["admin", "user"]
}
```

---

## Batch Processing Oxis

### `batch` - Batch Data Processing

Processes large datasets in configurable batches with memory management and progress tracking. Ideal for handling datasets that are too large to process in memory all at once.

**Configuration:**
```yaml
- name: batch
  config:
    batch_size: number        # Number of items per batch (default: 1000)
    memory_limit_mb: number   # Memory limit in MB (default: 100)
    input_path: string        # Path to input file/directory (optional)
    output_path: string       # Path for batch outputs (optional)
    parallel_batches: number  # Number of concurrent batches (default: 1)
    preserve_order: boolean   # Maintain item order across batches (default: true)
    checkpoint_frequency: number # Save state every N batches (default: 10)
```

**Input:** Large structured data (JSON arrays, CSV data, or file references)
**Output:** Processed data in batches or aggregated results
**Schema Strategy:** Modify (adds batch metadata to schema)
**Metadata:** `total_batches`, `items_processed`, `memory_used`, `processing_time`

**Features:**
- **Memory Management**: Automatically manages memory usage and garbage collection
- **Progress Tracking**: Reports progress and estimates completion time
- **Checkpointing**: Saves processing state for recovery from failures
- **Parallel Processing**: Supports concurrent batch processing
- **Large File Support**: Streams data from files without loading everything into memory

**Error Handling:**
- Failed batches can be retried individually
- Checkpoint system allows resuming from last successful batch
- Memory pressure detection with automatic batch size adjustment

**Example:**
```yaml
- name: batch
  id: batch_processor
  config:
    batch_size: 500
    memory_limit_mb: 200
    parallel_batches: 2
    checkpoint_frequency: 5
    preserve_order: true
```

**Use Cases:**
- Processing large CSV files (millions of rows)
- Transforming large JSON datasets
- ETL operations on big data
- Memory-constrained environments
- Long-running data processing jobs

**Batch Progress Output:**
```
📊 Batch Progress: 45/100 batches complete (45%)
⏱️  Estimated time remaining: 2m 15s
💾 Memory usage: 145/200 MB (72%)
✅ Items processed: 22,500 / 50,000
```

---

## Pipeline Configuration & Error Handling

### Universal Step Configuration

Every Oxi step supports these universal configuration options:

```yaml
- name: any_oxi
  id: step_identifier          # Unique step identifier (optional)
  config:
    # ... oxi-specific configuration
  # Universal error handling options:
  continue_on_error: boolean    # Continue pipeline on failure (default: false)
  retry_attempts: number        # Number of retry attempts (default: 0)
  retry_delay_seconds: number   # Initial retry delay in seconds (default: 1)
  timeout_seconds: number       # Timeout per attempt in seconds (optional)
  max_memory_mb: number         # Memory limit for this step (optional)
```

### Error Handling Behavior

- **`retry_attempts: N`**: Retries the step N times with exponential backoff (1s, 2s, 4s, 8s, ...)
- **`retry_delay_seconds: N`**: Sets the initial delay before first retry (affects backoff sequence)
- **`timeout_seconds: N`**: Each attempt times out after N seconds, then retries if attempts remain
- **`continue_on_error: true`**: Pipeline continues with the original input data if step fails completely
- **`continue_on_error: false`**: Pipeline stops immediately on step failure (default behavior)
- **`max_memory_mb: N`**: Enforces memory limit for the step, useful for large data processing

### Memory Management

```yaml
- name: large_data_processor
  config:
    # oxi config
  max_memory_mb: 512           # Limit this step to 512MB
  timeout_seconds: 300         # 5 minute timeout
  retry_attempts: 2            # Retry twice on failure
```

---

## Configuration Patterns

### Environment Variable Substitution

Oxide Flow supports flexible environment variable substitution with defaults and type conversion:

```yaml
config:
  # Required environment variable
  path: "${INPUT_FILE}"

  # Optional with string default
  format: "${FORMAT:-json}"

  # Optional with numeric default
  timeout: "${TIMEOUT:-30}"

  # Optional with boolean default
  strict: "${STRICT_MODE:-true}"

  # Complex path construction
  output: "${OUTPUT_DIR:-./output}/result_${TIMESTAMP}.csv"
```

### Step Reference Usage

Reference data and metadata from previous steps:

```yaml
config:
  # Use output from previous step
  path: "output_${steps.reader.metadata.filename}.processed"

  # Use metadata for validation
  max_size: "${steps.analyzer.metadata.max_size}"

  # Conditional processing based on previous results
  format: "${steps.detector.metadata.detected_format}"

  # Complex reference patterns
  filename: "${steps.reader.metadata.path}_processed_${steps.validator.metadata.timestamp}"
```

### Conditional Configuration

Use environment variables to control pipeline behavior:

```yaml
# Dynamic Oxi selection
- name: "${PROCESSOR_TYPE:-parse_json}"
  config:
    strict: "${STRICT_MODE:-true}"

# Conditional steps
- name: format_csv
  config:
    headers: "${INCLUDE_HEADERS:-true}"
    delimiter: "${CSV_DELIMITER:-,}"
  # Only run if processing CSV data
  condition: "${steps.detector.metadata.format} == 'csv'"
```

### Advanced Configuration Examples

**Multi-environment setup:**
```yaml
- name: write_file
  config:
    path: "${OUTPUT_PATH:-${ENV:-dev}/output/result.json}"
    backup: "${ENABLE_BACKUP:-false}"
    create_dirs: true
```

**Resource management:**
```yaml
- name: batch
  config:
    batch_size: "${BATCH_SIZE:-1000}"
    memory_limit_mb: "${MEMORY_LIMIT:-100}"
    parallel_batches: "${PARALLEL_JOBS:-1}"
  max_memory_mb: "${MAX_STEP_MEMORY:-500}"
  timeout_seconds: "${STEP_TIMEOUT:-3600}"
```

**Error handling strategy:**
```yaml
- name: external_api_call
  config:
    url: "${API_URL}"
    timeout: "${API_TIMEOUT:-30}"
  retry_attempts: "${API_RETRIES:-3}"
  retry_delay_seconds: 2
  continue_on_error: "${IGNORE_API_ERRORS:-false}"
```

---

## Schema Validation

All Oxis implement comprehensive schema validation that checks:

### Configuration Validation
- **Required properties**: Ensures all mandatory configuration is provided
- **Property types**: Validates strings, numbers, booleans match expected types
- **String patterns**: Validates format requirements (e.g., single character constraints)
- **Numeric ranges**: Ensures values are within acceptable bounds
- **Enum validation**: Checks values are from allowed sets
- **Cross-property validation**: Validates relationships between configuration properties

### Data Schema Validation
- **Input schema compliance**: Ensures input data matches expected schema
- **Schema evolution**: Tracks how data schemas change through the pipeline
- **Type safety**: Prevents type mismatches between pipeline steps
- **Schema documentation**: Automatic schema documentation generation

### Validation Error Examples

**Configuration errors:**
```
❌ Configuration validation failed for step 'formatter' (oxi: 'format_csv'):
  • Missing required property: 'headers'
  • Invalid property type for 'delimiter': Expected string, got number
  • Invalid property value for 'quote_char': Must be exactly one character
  • Invalid enum value for 'quote_style': Got 'invalid', expected one of: minimal, all, non_numeric, none
```

**Data schema errors:**
```
❌ Schema validation failed for step 'csv_parser':
  • Expected input type: Text, got: Binary
  • Input data does not match expected schema
  • Missing required fields in JSON object: ['name', 'email']
```

### Schema Strategy Types

Each Oxi declares how it handles schema evolution:

- **`Passthrough`**: Data and schema pass through unchanged (e.g., `write_stdout`)
- **`Infer`**: Automatically detects and creates appropriate schema (e.g., `read_file`)
- **`Modify`**: Transforms data and explicitly defines output schema (e.g., `parse_json`, `format_csv`)

---

## Performance & Optimization

### Memory Management
- **Streaming processing**: Large files processed in chunks to minimize memory usage
- **Garbage collection**: Automatic cleanup of intermediate data
- **Memory limits**: Per-step and global memory limits prevent system overload
- **Memory monitoring**: Real-time memory usage tracking and reporting

### Performance Characteristics by Oxi

**File I/O Oxis:**
- `read_file`: Streams large files, memory usage scales with file size
- `write_file`: Buffered writes for performance, atomic operations for safety

**Data Processing Oxis:**
- `parse_json`: Memory usage scales with JSON complexity, uses streaming parser for large files
- `format_json`: Memory efficient for large data structures
- `parse_csv`: Streams CSV data, minimal memory footprint
- `format_csv`: Buffered output generation, efficient for large datasets
- `flatten`: Memory usage scales with nesting depth and object size
- `batch`: Designed for large data, configurable memory limits and parallel processing

### Optimization Tips

**For large datasets:**
```yaml
- name: batch
  config:
    batch_size: 100          # Smaller batches for memory-constrained environments
    memory_limit_mb: 50      # Conservative memory limit
    parallel_batches: 1      # Single batch processing for stability
```

**For performance:**
```yaml
- name: batch
  config:
    batch_size: 5000         # Larger batches for better throughput
    memory_limit_mb: 500     # More memory for faster processing
    parallel_batches: 4      # Parallel processing on multi-core systems
```

**For reliability:**
```yaml
- name: any_oxi
  retry_attempts: 3          # Retry failed operations
  timeout_seconds: 300       # Reasonable timeout
  continue_on_error: false   # Fail fast on errors
```

---

## Custom Oxis Development

### Creating Custom Oxis

To create custom Oxis, implement the `Oxi` trait from the Oxide Flow SDK:

```rust
use crate::oxis::prelude::*;  // Standard imports for Oxi development

pub struct MyCustomOxi;

#[async_trait]
impl Oxi for MyCustomOxi {
    fn name(&self) -> &str {
        "my_custom_oxi"
    }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Modify {
            description: "Transforms input data with custom logic".to_string()
        }
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        // Access input data
        let data = match input.data() {
            Data::Json(json_value) => {
                // Process JSON data
                let processed = your_custom_logic(json_value)?;
                OxiData::from_json(processed)
            },
            Data::Text(text) => {
                // Process text data
                let processed = your_text_processing(&text)?;
                OxiData::from_text(processed)
            },
            Data::Binary(bytes) => {
                // Process binary data
                let processed = your_binary_processing(&bytes)?;
                OxiData::from_binary(processed)
            },
            Data::Empty => OxiData::empty(),
        };

        Ok(data)
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
          type: object
          properties:
            my_setting:
              type: string
              description: "Custom configuration setting"
            numeric_option:
              type: number
              default: 42
              minimum: 0
          required:
            - my_setting
        "#).unwrap()
    }
}
```

### Development Best Practices

**1. Use the Standard Prelude:**
```rust
use crate::oxis::prelude::*;  // Includes all necessary imports
```

**2. Implement Proper Error Handling:**
```rust
// Use OxiError for consistent error reporting
return Err(OxiError::ProcessingError {
    message: "Failed to process data".to_string(),
    source: Some(Box::new(underlying_error)),
});
```

**3. Choose Appropriate Schema Strategy:**
```rust
// For data transformation
SchemaStrategy::Modify { description: "...".to_string() }

// For passthrough operations
SchemaStrategy::Passthrough

// For format detection/inference
SchemaStrategy::Infer
```

**4. Implement Comprehensive Configuration Schema:**
```yaml
# Define clear configuration schema with defaults
type: object
properties:
  required_setting:
    type: string
    description: "This setting is required"
  optional_setting:
    type: boolean
    default: false
    description: "This setting is optional with default"
  enum_setting:
    type: string
    enum: ["option1", "option2", "option3"]
    default: "option1"
required:
  - required_setting
```

### Oxi Development Patterns

**Data Type Handling:**
```rust
// Safe data access with proper error handling
let json_data = input.data().as_json()
    .map_err(|e| OxiError::InvalidInput {
        message: "Expected JSON input".to_string()
    })?;

// Type conversion with validation
let text_data = input.data().as_text()
    .map_err(|e| OxiError::InvalidInput {
        message: "Expected text input".to_string()
    })?;
```

**Configuration Access:**
```rust
// Type-safe configuration access
let my_setting: String = config.get("my_setting")
    .ok_or_else(|| OxiError::ConfigurationError {
        message: "Missing required setting: my_setting".to_string()
    })?;

let numeric_option: f64 = config.get("numeric_option").unwrap_or(42.0);
```

**Async Processing:**
```rust
// Use async/await for I/O operations
async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
    let url = config.get::<String>("url")?;

    // Async HTTP request example
    let response = reqwest::get(&url).await
        .map_err(|e| OxiError::ProcessingError {
            message: format!("HTTP request failed: {}", e),
            source: Some(Box::new(e)),
        })?;

    let data = response.text().await?;
    Ok(OxiData::from_text(data))
}
```

### Testing Custom Oxis

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_custom_oxi() {
        let oxi = MyCustomOxi;
        let input = OxiData::from_json(json!({"test": "data"}));
        let config = OxiConfig::new();

        let result = oxi.process(input, &config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        // Add your assertions here
    }
}
```

### Integration with Oxide Flow

**1. Register in `src/oxis/mod.rs`:**
```rust
pub mod my_custom_oxi;
pub use my_custom_oxi::MyCustomOxi;
```

**2. Add to Oxi Registry:**
```rust
// In your pipeline execution code
let oxi_registry = vec![
    Box::new(ReadFileOxi) as Box<dyn Oxi>,
    Box::new(WriteFileOxi) as Box<dyn Oxi>,
    Box::new(MyCustomOxi) as Box<dyn Oxi>,  // Add your custom Oxi
];
```

**3. Use in Pipeline YAML:**
```yaml
steps:
  - name: my_custom_oxi
    id: custom_processor
    config:
      my_setting: "custom_value"
      numeric_option: 100
```

### Resources

- **Examples**: See `src/oxis/` directory for implementation examples
- **Prelude**: Use `src/oxis/prelude.rs` for standard imports
- **Testing**: Follow patterns in `tests/oxi_sdk_tests.rs`
- **Documentation**: Add documentation following existing Oxi patterns

---

## Summary

Oxide Flow provides a rich set of built-in Oxis for common data transformation tasks:

| Category | Oxis | Purpose |
|----------|------|---------|
| **File I/O** | `read_file`, `write_file` | File system operations |
| **Standard I/O** | `read_stdin`, `write_stdout` | Console input/output |
| **JSON** | `parse_json`, `format_json` | JSON processing |
| **CSV** | `parse_csv`, `format_csv` | CSV processing |
| **Transformation** | `flatten` | Data structure transformation |
| **Batch Processing** | `batch` | Large dataset processing |

Each Oxi supports:
- ✅ **Schema validation** with detailed error messages
- ✅ **Environment variable substitution** with defaults
- ✅ **Error handling** with retries and timeouts
- ✅ **Memory management** with configurable limits
- ✅ **Step references** for cross-step data access
- ✅ **Comprehensive configuration** with type safety

For the most up-to-date information and implementation details, refer to the source code in `src/oxis/` and the test examples in `tests/`.

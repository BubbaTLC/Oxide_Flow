# Oxi Reference Guide

Quick reference for all available Oxis (plugins) and their configuration options.

## File I/O Oxis

### `read_file` - Read File Content

Reads content from a file and outputs it as data.

**Configuration:**
```yaml
- name: read_file
  config:
    path: string              # File path (required)
    encoding: string          # Text encoding (default: "utf-8")
```

**Output:** File content as Text, JSON, or Binary data
**Metadata:** `filename`, `size`, `modified_time`

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

Writes input data to a specified file.

**Configuration:**
```yaml
- name: write_file
  config:
    path: string              # Output file path (required)
    overwrite: boolean        # Overwrite existing file (default: true)
    create_dirs: boolean      # Create parent directories (default: true)
    encoding: string          # Text encoding (default: "utf-8")
```

**Input:** Any data type
**Output:** Empty data
**Metadata:** `path`, `size`, `created_time`

**Example:**
```yaml
- name: write_file
  id: output_writer
  config:
    path: "output/result_${steps.processor.timestamp}.csv"
    create_dirs: true
    overwrite: false
```

---

## Standard I/O Oxis

### `read_stdin` - Read from Standard Input

Reads data from standard input with optional user prompt.

**Configuration:**
```yaml
- name: read_stdin
  config:
    prompt: string            # User prompt message (optional)
    timeout: number           # Timeout in seconds (optional)
```

**Output:** Text data from stdin
**Metadata:** `bytes_read`, `timestamp`

**Example:**
```yaml
- name: read_stdin
  id: user_input
  config:
    prompt: "Enter JSON data:"
    timeout: 30
```

---

### `write_stdout` - Write to Standard Output

Writes data to standard output (console).

**Configuration:**
```yaml
- name: write_stdout
  config:
    newline: boolean          # Add trailing newline (default: true)
    prefix: string            # Prefix for output (optional)
```

**Input:** Any data type
**Output:** Same data (passthrough)
**Metadata:** `bytes_written`, `timestamp`

**Example:**
```yaml
- name: write_stdout
  id: console_output
  config:
    newline: true
    prefix: "Result: "
```

---

## JSON Processing Oxis

### `parse_json` - Parse JSON Data

Parses JSON text into structured data.

**Configuration:**
```yaml
- name: parse_json
  config:
    strict: boolean           # Strict JSON parsing (default: true)
    array_handling: string    # "flatten" or "preserve" (default: "preserve")
```

**Input:** Text data containing JSON
**Output:** Structured JSON data
**Metadata:** `parsed_at`, `object_count`, `array_count`

**Example:**
```yaml
- name: parse_json
  id: json_parser
  config:
    strict: true
    array_handling: "flatten"
```

---

### `format_json` - Format Data as JSON

Formats structured data as JSON text.

**Configuration:**
```yaml
- name: format_json
  config:
    pretty: boolean           # Pretty print with indentation (default: true)
    indent: number            # Indentation spaces (default: 2)
    sort_keys: boolean        # Sort object keys (default: false)
```

**Input:** Structured data
**Output:** JSON text
**Metadata:** `formatted_at`, `size`, `lines`

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

## CSV Processing Oxis

### `parse_csv` - Parse CSV Data

Parses CSV text into structured data.

**Configuration:**
```yaml
- name: parse_csv
  config:
    headers: boolean          # First row contains headers (default: true)
    delimiter: string         # Field delimiter (default: ",")
    quote_char: string        # Quote character (default: "\"")
    escape_char: string       # Escape character (optional)
    skip_rows: number         # Number of rows to skip (default: 0)
```

**Input:** Text data containing CSV
**Output:** Structured data
**Metadata:** `rows_parsed`, `columns`, `headers`

**Example:**
```yaml
- name: parse_csv
  id: csv_parser
  config:
    headers: true
    delimiter: "${CSV_DELIMITER:-,}"
    skip_rows: 1
```

---

### `format_csv` - Format Data as CSV

Formats structured data as CSV text.

**Configuration:**
```yaml
- name: format_csv
  config:
    headers: boolean          # Include column headers (default: true)
    delimiter: string         # Field delimiter (default: ",", must be single char)
    quote_char: string        # Quote character (default: "\"")
    escape_char: string       # Escape character (optional)
    quote_all: boolean        # Quote all fields (default: false)
```

**Input:** Structured data (JSON objects/arrays)
**Output:** CSV text
**Metadata:** `rows_formatted`, `columns`, `size`

**Schema Validation:**
- `delimiter` must be exactly one character
- `headers` must be boolean
- `quote_char` must be single character if specified

**Example:**
```yaml
- name: format_csv
  id: csv_formatter
  config:
    headers: true
    delimiter: "|"
    quote_all: false
```

---

## Data Transformation Oxis

### `flatten` - Flatten Nested Data Structures

Flattens nested JSON objects into flat key-value pairs.

**Configuration:**
```yaml
- name: flatten
  config:
    separator: string         # Key separator (default: ".")
    max_depth: number         # Maximum nesting depth (optional)
    preserve_arrays: boolean  # Keep arrays intact (default: false)
```

**Input:** Structured JSON data
**Output:** Flattened structured data
**Metadata:** `flattened_keys`, `original_depth`, `final_depth`

**Example:**
```yaml
- name: flatten
  id: flattener
  config:
    separator: "_"
    max_depth: 5
    preserve_arrays: true
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
    }
  }
}

// Output (separator: ".")
{
  "user.name": "John",
  "user.address.city": "NYC",
  "user.address.zip": "10001"
}
```

---

## Error Handling for All Oxis

Every Oxi supports these error handling configurations:

```yaml
- name: any_oxi
  config:
    # ... oxi-specific config
  continue_on_error: boolean    # Continue pipeline on failure (default: false)
  retry_attempts: number        # Number of retry attempts (default: 0)
  timeout_seconds: number       # Timeout per attempt in seconds (optional)
```

### Error Handling Behavior

- **`retry_attempts: N`**: Retries the step N times with exponential backoff (1s, 2s, 4s, ...)
- **`timeout_seconds: N`**: Each attempt times out after N seconds
- **`continue_on_error: true`**: Pipeline continues with the same input data if step fails
- **`continue_on_error: false`**: Pipeline stops immediately on step failure

---

## Common Configuration Patterns

### Environment Variable Substitution

```yaml
config:
  path: "${INPUT_FILE}"                    # Required variable
  format: "${FORMAT:-json}"                # Optional with default
  timeout: "${TIMEOUT:-30}"                # Numeric with default
```

### Step Reference Usage

```yaml
config:
  path: "output_${steps.reader.filename}.processed"
  size_limit: "${steps.analyzer.max_size}"
```

### Conditional Configuration

```yaml
# Use environment to control behavior
- name: "${PROCESSOR_TYPE:-parse_json}"
  config:
    strict: "${STRICT_MODE:-true}"
```

---

## Schema Validation

All Oxis have built-in schema validation that checks:

- **Required properties** are present
- **Property types** match expected types (string, boolean, number)
- **String patterns** match format requirements (e.g., single character for CSV delimiter)
- **Numeric ranges** are within acceptable bounds
- **Enum values** are from allowed sets

### Validation Error Format

```
❌ Configuration validation failed for step 'formatter' (oxi: 'format_csv'):
  • Missing required property: 'headers'
  • Invalid property type for 'delimiter': Expected string, got number
  • Invalid property value for 'quote_char': Must be exactly one character
```

---

## Performance Notes

- **File I/O**: Large files are processed in streaming mode where possible
- **JSON Parsing**: Very large JSON files may require additional memory
- **CSV Processing**: Headers are auto-detected if not explicitly configured
- **Retries**: Use exponential backoff to avoid overwhelming failing services
- **Timeouts**: Set reasonable timeouts for network or slow file operations

---

## Custom Oxis

To create custom Oxis, implement the `Oxi` trait:

```rust
use oxide_flow::Oxi;
use async_trait::async_trait;

pub struct MyCustomOxi;

#[async_trait]
impl Oxi for MyCustomOxi {
    fn name(&self) -> &str { "my_custom_oxi" }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData> {
        // Your custom processing logic
    }

    fn config_schema(&self) -> serde_yaml::Value {
        // Define configuration schema
    }
}
```

See the `src/oxis/` directory for implementation examples.

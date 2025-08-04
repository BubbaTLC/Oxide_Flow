# Oxide Flow

Oxide Flow is a command-line data transformation and integration tool built in Rust with a modular, plugin-based architecture. It follows Unix philosophy - simple, composable tools that work together through data pipes.

## Features

- 🔌 **Plugin Architecture**: Extensible "Oxis" (plugins) for data transformation
- 🔄 **Enhanced Error Handling**: Retry logic, timeouts, and graceful failure recovery
- 🌍 **Environment Variable Support**: Dynamic configuration with `${VAR:-default}` syntax
- 🔗 **Step References**: Reference outputs from previous steps with `${steps.step_id.property}`
- ✅ **Schema Validation**: Built-in validation for Oxi configurations
- 📂 **Smart Pipeline Discovery**: Run pipelines by name, auto-discovers from project config
- 🎯 **YAML-First Configuration**: Clean, readable pipeline definitions

## Quick Start

### Initialize a New Project

```bash
# Create a new Oxide Flow project
oxide_flow init my_project
cd my_project

# Run the default pipeline
oxide_flow run

# Run a specific pipeline by name
oxide_flow run my_pipeline
```

### Basic Pipeline Example

```yaml
# pipelines/json_to_csv.yaml
pipeline:
  - name: read_file
    id: reader
    config:
      path: "data.json"
    retry_attempts: 2

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

```bash
# Run the pipeline
oxide_flow run json_to_csv
```

## Documentation

- **[Pipeline Configuration Guide](docs/pipeline.md)** - Complete YAML configuration reference
- **[Oxi Reference](docs/oxi_reference.md)** - All available Oxis and their configurations
- **[Pipeline Examples](docs/pipeline_examples.md)** - Real-world pipeline examples
- **[Oxi Development Guide](docs/oxi.md)** - How to create custom Oxis

## Available Oxis (Plugins)

### File I/O
- `read_file` / `write_file` - File operations
- `read_stdin` / `write_stdout` - Standard I/O

### Data Formats
- `parse_json` / `format_json` - JSON processing
- `parse_csv` / `format_csv` - CSV processing

### Transformations
- `flatten` - Flatten nested data structures

## Advanced Features

### Environment Variables
```yaml
config:
  path: "${INPUT_FILE}"                    # Required variable
  format: "${FORMAT:-json}"                # Optional with default
```

### Step References
```yaml
config:
  path: "output_${steps.reader.filename}.processed"
```

### Error Handling
```yaml
- name: read_file
  config:
    path: "might_not_exist.json"
  continue_on_error: true     # Continue pipeline on failure
  retry_attempts: 3          # Retry up to 3 times
  timeout_seconds: 30        # Timeout after 30 seconds
```

## Project Structure

```
my_project/
├── oxiflow.yaml              # Project configuration
├── pipelines/               # Pipeline definitions
│   ├── pipeline.yaml        # Default pipeline
│   └── my_pipeline.yaml     # Custom pipelines
└── output/                  # Generated outputs
```

## Installation

```bash
# Clone and build
git clone https://github.com/BubbaTLC/Oxide_Flow.git
cd Oxide_Flow
cargo build --release

# Install (optional)
cargo install --path .
```

## License

This project is licensed under the MIT License.

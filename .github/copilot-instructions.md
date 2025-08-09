# Oxide Flow - AI Agent Instructions

Always use a TODO list (`chat.todoListTool.enabled`)

## Project Overview
Oxide Flow is a mature command-line data transformation tool built in Rust with a modular, plugin-based architecture. It follows Unix philosophy - simple, composable "Oxis" (plugins) that work together through data pipes with schema-aware data flow.

## Core Architecture

### Plugin System - "Oxis"
- **Oxis** extend functionality as async plugins implementing the `Oxi` trait
- All Oxis process `OxiData` (schema-aware data wrapper around `Data` enum)
- Data flows: `Data::Json | Data::Text | Data::Binary | Data::Empty`
- Use `src/oxis/prelude.rs` for standard imports when creating Oxis
- Each Oxi declares `SchemaStrategy` (Passthrough/Modify/Infer) for schema evolution

### Schema-Aware Data Flow
```rust
// Current OxiData API patterns:
let data = OxiData::from_json(json!({"key": "value"}));  // NOT OxiData::Json()
let text = OxiData::from_text("content".to_string());    // NOT OxiData::Text()
let empty = OxiData::empty();                            // NOT OxiData::Empty

// Access data: data.data().as_json()?, data.schema(), data.validate()
```

### Pipeline Architecture
- YAML-first pipeline definitions in `pipelines/` directory
- Steps reference by `id`, support retries, timeouts, environment variables
- Built-in step reference system: `${steps.reader.metadata.path}`
- Config resolution with `${VAR:-default}` environment variable syntax

## Development Workflow

### Project Structure
```
src/
├── lib.rs              # Oxi trait definition + module exports
├── main.rs             # CLI entry point with clap commands
├── cli.rs              # CLI command structure
├── pipeline.rs         # Pipeline execution engine
├── pipeline_manager.rs # Pipeline management (list, test, info)
├── types.rs            # Core data types (OxiData, Data, schemas)
├── oxis/               # Built-in Oxis
│   ├── prelude.rs      # Standard imports for Oxi development
│   ├── parse_json/     # Text → JSON transformation
│   ├── format_json/    # JSON → formatted JSON
│   ├── file/           # File I/O operations
│   └── ...
docs/                   # Oxi concepts and examples
tests/                  # Integration tests using TestOxi pattern
```

### Essential Commands
```bash
# Development workflow
cargo check --tests    # Fast compilation check including tests
cargo test             # Run full test suite
cargo clippy           # Rust linting (required by pre-commit)
pre-commit run --all-files  # Quality checks (format, lint, audit)
cargo add <dependency>  # Add new dependencies

# CLI usage
cargo run -- init project_name     # Initialize new project
cargo run -- run pipeline_name     # Execute pipeline by name
cargo run -- pipeline list         # Show available pipelines
```

### Oxi Implementation Pattern
```rust
use crate::oxis::prelude::*;  // Standard imports

pub struct MyOxi;

#[async_trait]
impl Oxi for MyOxi {
    fn name(&self) -> &str { "my_oxi" }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Modify { description: "...".to_string() }
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        // Access input: input.data().as_json()?, input.data().as_text()?
        // Return: Ok(OxiData::from_json(result))
    }
}
```

## Testing & Quality

### Test Patterns
- Integration tests in `tests/` using `TestOxi` struct pattern
- Use `ProcessingLimits` for resource constraints testing
- Test data construction: `OxiData::from_json(json!({...}))`
- Error testing: validate `OxiError::ValidationError` patterns

### Pre-commit Quality Gates
- **Rust formatting**: `cargo fmt` (enforced)
- **Clippy linting**: Full strictness with `clippy::all`
- **Security audit**: `cargo audit` for dependency vulnerabilities
- **File hygiene**: Trailing whitespace, line endings, YAML/TOML validation

## Configuration Philosophy
- **YAML-first**: All configs in YAML, avoid hardcoded behaviors
- **Environment-aware**: Support `${VAR:-default}` syntax everywhere
- **Schema validation**: Built-in validation via `OxiConfigSchema`
- **CLI-optimized**: Command-line UX prioritized over GUI

## Key Integration Points
- **Pipeline discovery**: Auto-finds pipelines in `pipelines/` directory
- **Step references**: Cross-step data access via `${steps.id.property}`
- **Error handling**: Retry logic with exponential backoff, timeout support
- **Config resolution**: Dynamic environment variable substitution

## Critical Files for Understanding
- `src/types.rs` - Core data types, schema system (1193 lines)
- `src/pipeline.rs` - Pipeline execution engine with retry/timeout logic
- `src/oxis/prelude.rs` - Essential imports for any Oxi development
- `tests/oxi_sdk_tests.rs` - Reference implementation patterns

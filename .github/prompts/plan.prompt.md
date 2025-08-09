---
mode: agent
---

# Oxide Flow Project Planning Assistant

## 🎯 Purpose
This prompt helps create comprehensive project plans for Oxide Flow features, Oxis (plugins), and system improvements. It produces detailed implementation plans with clear phases, deliverables, and documentation requirements.

Update the `docs/current_plan.md` file with your project plan.

## 📋 Planning Process

### 1. **Initial Assessment**
Determine the project type and scope:
- **New Oxi Plugin**: Data transformation following Unix pipe philosophy
- **Core Feature**: CLI commands, API endpoints, configuration enhancements
- **Architecture Change**: Structural improvements, refactoring, or optimization
- **Integration**: External system connections, data sources, or API integrations
- **Documentation**: Comprehensive docs, guides, or reference materials


### 2. **Requirement Analysis**
Gather comprehensive requirements by asking:

**Functional Requirements:**
- What specific problem does this solve?
- What are the expected inputs and outputs?
- What configuration options are needed?
- How should errors be handled?
- What performance expectations exist?

**Technical Requirements:**
- What existing Oxis or modules does this depend on?
- What new dependencies might be needed?
- How does this integrate with the CLI structure?
- What YAML configuration schema is required?
- What API endpoints need to be created/modified?

**User Experience Requirements:**
- What CLI commands and options are needed?
- What help text and examples should be provided?
- What documentation needs to be created or updated?
- How should progress and status be communicated?

**Current CLI Architecture:**
```rust
// Main CLI structure (in src/cli.rs)
#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init { /* project initialization */ },
    Run { pipeline: String, config: Option<String> },
    Pipeline { #[command(subcommand)] action: PipelineAction },
}

// Available commands:
// oxide_flow init [--name NAME] [--directory DIR]
// oxide_flow run [PIPELINE] [--config CONFIG]
// oxide_flow pipeline list [--tags TAGS] [--filter FILTER]
// oxide_flow pipeline add NAME [--template TEMPLATE]
// oxide_flow pipeline test NAME [--dry-run] [--fix]
// oxide_flow pipeline info NAME
```

### 3. **Architecture Planning**
Design the solution structure based on Oxide Flow's current architecture:

**Current Data Flow Architecture:**
```rust
// Schema-aware data pipeline
OxiData::from_json(input)
  -> Oxi1::process(data, config)
  -> OxiData::with_updated_schema(new_schema)
  -> Oxi2::process(data, config)
  -> output

// Data types and conversion patterns
Data::Json(value)    // Primary structured data format
Data::Text(string)   // Plain text, logs, unstructured content
Data::Binary(bytes)  // Files, images, binary content
Data::Empty          // Pipeline initialization

// Schema evolution strategies
SchemaStrategy::Passthrough          // Schema unchanged (filters)
SchemaStrategy::Modify { description } // Schema modified (transformations)
SchemaStrategy::Infer                // Schema inferred from data
```

**Pipeline Execution Flow:**
1. **Pipeline Loading**: YAML parsed with environment variable resolution
2. **Step Execution**: Sequential async processing with retry/timeout logic
3. **Data Passing**: `OxiData` flows between steps with schema evolution
4. **Error Handling**: Configurable `continue_on_error` and retry strategies
5. **Step References**: Dynamic resolution of `${steps.id.property}` values

**Current Project Structure:**
```
src/
├── lib.rs                   # Oxi trait + core module exports
├── main.rs                  # CLI entry point with clap commands
├── cli.rs                   # CLI command structure (Commands, PipelineAction)
├── pipeline.rs              # Pipeline execution engine with retry/timeout
├── pipeline_manager.rs      # Pipeline management (list, test, info)
├── types.rs                 # Core data types (OxiData, Data, SchemaStrategy)
├── error.rs                 # Error types (OxiError variants)
├── config.rs                # Configuration structures
├── config_resolver.rs       # Environment variable resolution
├── schema.rs                # Schema validation and inference
├── project.rs               # Project initialization and management
├── oxis/                    # Built-in Oxis
│   ├── mod.rs              # Oxi registry and module exports
│   ├── prelude.rs          # Standard imports for Oxi development
│   ├── [oxi_name]/         # Individual Oxi implementations
│   │   ├── mod.rs          # Module declaration
│   │   └── oxi.rs          # Oxi implementation
│   └── read_stdin.rs       # Special I/O Oxis
└── templates/              # Pipeline templates for 'init' command

tests/
├── [feature]_tests.rs      # Unit tests per feature
├── oxi_sdk_tests.rs        # Oxi development patterns
└── integration_tests.rs    # End-to-end pipeline tests

docs/
├── current_plan.md         # Project planning document
├── oxi_reference.md        # All available Oxis and configurations
├── pipeline.md             # Pipeline YAML reference
└── cli/                    # CLI command documentation
```

**For New Oxis:**
```
src/oxis/new_oxi/
├── mod.rs                  # pub use oxi::NewOxi;
└── oxi.rs                  # Implementation with prelude imports
```

**For Core Features:**
```
src/
├── feature_name.rs         # Core implementation
├── cli.rs                  # Add Commands enum variants
└── main.rs                 # Integrate new commands
```

**Core Design Principles:**
- **Schema-Aware Data Flow**: All data carries schema information via `OxiData`
- **Async Processing**: All Oxis use async/await for non-blocking operations
- **Resource Management**: Oxis define `ProcessingLimits` for memory, batch size, and time constraints
- **Environment Variable Support**: Full `${VAR:-default}` syntax in all configurations
- **Step References**: Cross-step data access via `${steps.step_id.property}` syntax
- **Comprehensive Error Handling**: Rich error types with retry logic and timeouts
- **YAML-First Configuration**: All configuration in YAML with schema validation
- **CLI-Centric UX**: Command-line interface prioritized over GUI
- **Unix Pipe Philosophy**: Simple, composable data transformations
- **Modular Plugin Architecture**: Extensible via the `Oxi` trait system

### 4. **Implementation Strategy**
Break down the project into manageable phases:

**Phase Structure Template:**
- **Phase 1: Foundation** - Core structures, types, and module setup
- **Phase 2: Core Logic** - Main functionality implementation
- **Phase 3: Integration** - CLI/API integration and configuration
- **Phase 4: Testing & Validation** - Comprehensive testing and error handling
- **Phase 5: Documentation** - User guides, examples, and API docs

### 5. **Technical Specifications**
Provide concrete implementation guidance:

**Rust Implementation Patterns:**
```rust
// Current Oxi trait implementation (async with schema-aware data)
use crate::oxis::prelude::*;

pub struct NewOxi;

#[async_trait]
impl Oxi for NewOxi {
    fn name(&self) -> &str {
        "new_oxi"
    }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Modify {
            description: "Description of how this Oxi transforms data".to_string()
        }
    }

    // Optional: Define processing limits for resource management
    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: Some(10_000),
            max_memory_mb: Some(256),
            max_processing_time_ms: Some(15_000),
            supported_input_types: vec![OxiDataType::Json, OxiDataType::Text],
        }
    }

    // Optional: Configuration schema for validation
    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(r#"
            type: object
            properties:
              setting:
                type: string
                description: "Configuration setting"
                default: "default_value"
        "#).unwrap()
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        // Access input data: input.data().as_json()?, input.data().as_text()?
        // Get config values: config.get_string("setting")?

        match input.data() {
            Data::Json(json) => {
                // Process JSON data
                let result = process_json_data(json, config)?;
                Ok(OxiData::from_json(result))
            }
            Data::Text(text) => {
                // Process text data
                let result = process_text_data(text, config)?;
                Ok(OxiData::from_text(result))
            }
            _ => Err(OxiError::TypeMismatch {
                expected: "JSON or Text".to_string(),
                actual: input.data().data_type().to_string(),
                step: "new_oxi".to_string(),
            }),
        }
    }
}

// CLI command structure (using clap)
#[derive(Subcommand, Debug)]
pub enum FeatureAction {
    Command {
        /// Command description
        #[arg(short, long)]
        option: Option<String>,

        /// Boolean flag
        #[arg(long)]
        enable_feature: bool,
    },
}
```

**Pipeline Configuration Schema:**
```yaml
# Modern pipeline configuration with enhanced error handling
pipeline:
  - name: read_file
    id: reader
    config:
      path: "${INPUT_FILE:-input/data.json}"  # Environment variable support
    retry_attempts: 3          # Retry logic
    timeout_seconds: 30        # Timeout handling
    continue_on_error: false   # Error propagation control

  - name: parse_json
    id: parser
    config:
      flatten: false
    retry_attempts: 2

  - name: write_file
    id: writer
    config:
      path: "output/${steps.reader.filename}.processed"  # Step references
      create_directories: true
      backup: true

# Pipeline metadata (required)
metadata:
  name: "Pipeline Name"
  description: "Pipeline description"
  version: "1.0.0"
  author: "Author Name"
  tags: ["processing", "example"]

# Optional: Environment variables definition
environment:
  INPUT_FILE: "input/default.json"
  OUTPUT_DIR: "output"
```

**Oxi Configuration Schema:**
```yaml
# Individual Oxi configuration options
oxi_name:
  # Core settings
  setting1: "value1"
  setting2: 42
  enabled: true

  # Advanced configuration
  processing:
    batch_size: 1000
    max_memory_mb: 256
    timeout_ms: 15000

  # Validation rules
  validation:
    required_fields: ["field1", "field2"]
    allowed_types: ["string", "number"]
```

## 📝 Plan Document Structure

Create a comprehensive plan document with the following sections:

### 1. **Project Overview**
- **Title**: Clear, descriptive project name
- **Purpose**: Problem statement and solution summary
- **Scope**: What's included and excluded
- **Success Criteria**: Measurable objectives

### 2. **Requirements**
- **Functional Requirements**: What the system must do
- **Technical Requirements**: Implementation constraints
- **Performance Requirements**: Speed, memory, scalability needs
- **Integration Requirements**: How it fits with existing systems

### 3. **Architecture & Design**
- **System Design**: High-level architecture diagrams
- **Data Flow**: How data moves through the system
- **Configuration Design**: YAML schema and validation
- **API Design**: CLI commands and REST endpoints

### 4. **Implementation Plan**
- **Phase Breakdown**: Detailed phase descriptions with deliverables
- **Dependencies**: Prerequisites and integration points
- **File Structure**: Exact directory and file organization
- **Timeline Estimates**: Rough time estimates for each phase

### 5. **Technical Details**
- **Code Examples**: Key implementation patterns and pseudo-code
- **Error Handling**: Comprehensive error scenarios and responses
- **Testing Strategy**: Unit tests, integration tests, and validation
- **Performance Considerations**: Optimization and scaling strategies

### 6. **Documentation Plan**
- **User Documentation**: CLI help, usage guides, tutorials
- **Technical Documentation**: API references, architecture docs
- **Examples**: Real-world usage scenarios and sample configurations
- **Migration Guides**: If applicable, upgrade/migration instructions

### 7. **Quality Assurance**
- **Testing Checklist**: Comprehensive validation requirements
- **Code Review Guidelines**: Standards and best practices
- **Documentation Review**: Accuracy and completeness verification
- **Integration Testing**: End-to-end validation scenarios

**Current Testing Patterns:**
```rust
// Unit testing pattern for Oxis
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oxi_functionality() {
        let oxi = MyOxi;
        let config = OxiConfig::default();
        let input = OxiData::from_json(json!({"test": "data"}));

        let result = oxi.process(input, &config).await.unwrap();

        // Validate output data
        assert!(matches!(result.data(), Data::Json(_)));

        // Validate schema evolution
        assert!(result.schema().fields.contains_key("expected_field"));
    }

    #[tokio::test]
    async fn test_processing_limits() {
        let oxi = MyOxi;
        let limits = oxi.processing_limits();

        assert!(limits.max_batch_size.unwrap() > 0);
        assert!(limits.supported_input_types.contains(&OxiDataType::Json));
    }
}

// Integration testing with TestOxi pattern
struct TestOxi {
    expected_calls: Vec<OxiData>,
    responses: Vec<Result<OxiData, OxiError>>,
}
```

## 🎯 Success Criteria

A complete project plan should include:
- ✅ Clear problem statement and solution approach
- ✅ Detailed phase-by-phase implementation strategy
- ✅ Schema-aware data flow design with `OxiData` patterns
- ✅ Async Oxi implementation with proper error handling
- ✅ Processing limits and resource management considerations
- ✅ Complete file structure following current project organization
- ✅ CLI integration with clap command structure
- ✅ YAML configuration with environment variable support
- ✅ Comprehensive testing strategy including async unit tests
- ✅ Documentation plan with current CLI command patterns
- ✅ Integration points with pipeline execution engine
- ✅ Schema evolution strategy (Passthrough/Modify/Infer)
- ✅ Error handling with retry logic and timeout considerations
- ✅ Performance and scalability requirements with processing limits
- ✅ Clear success metrics and acceptance criteria

## 🚀 Output Format

The final plan should be saved to `docs/[project_name]_plan.md` with:
- Executive summary for quick reference
- Detailed implementation phases with clear deliverables
- Technical specifications with code examples
- Documentation requirements and content plan
- Testing and validation checklist
- Timeline and dependency information

This planning process ensures systematic, well-architected implementations that follow Oxide Flow's design principles and maintain high quality standards.

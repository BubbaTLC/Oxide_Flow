---
mode: ask
---

# Oxide Flow Project Planning Assistant

## 🎯 Purpose
This prompt helps create comprehensive project plans for Oxide Flow features, Oxis (plugins), and system improvements. It produces detailed implementation plans with clear phases, deliverables, and documentation requirements.

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

### 3. **Architecture Planning**
Design the solution structure:

**File Organization:**
```
src/
├── oxis/[feature_name]/     # For new Oxis
│   ├── mod.rs               # Module declaration
│   ├── oxi.rs              # Main implementation
│   └── config.rs           # Configuration structures
├── [feature]/              # For core features
│   ├── mod.rs
│   ├── implementation.rs
│   └── types.rs
├── cli.rs                  # CLI command integration
└── main.rs                 # Main application updates

tests/
├── [feature]_tests.rs      # Unit tests
└── integration_tests.rs    # Integration tests

docs/
├── [feature].md            # Feature documentation
├── cli.md                  # CLI command reference
└── examples/               # Usage examples
```

**Core Design Principles:**
- Follow Oxide Flow's modular "Oxi" architecture
- Use YAML-first configuration approach
- Implement comprehensive error handling with Result<T, E>
- Follow Unix pipe philosophy for data flow
- Minimize external dependencies
- Prioritize debuggability and verbose logging

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
// Oxi trait implementation
pub struct NewOxi {
    config: OxiConfig,
}

impl Oxi for NewOxi {
    fn process(&self, input: OxiData) -> Result<OxiData, OxiError> {
        // Implementation following error handling patterns
    }

    fn validate_config(&self) -> Result<(), OxiError> {
        // Configuration validation
    }
}

// CLI command structure
#[derive(Subcommand, Debug)]
pub enum FeatureAction {
    Command {
        #[arg(short, long)]
        option: Option<String>,
    },
}
```

**Configuration Schema:**
```yaml
# Example YAML configuration
feature_name:
  enabled: true
  options:
    setting1: value1
    setting2: value2
  advanced:
    timeout_seconds: 30
    retry_attempts: 3
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

## 🎯 Success Criteria

A complete project plan should include:
- ✅ Clear problem statement and solution approach
- ✅ Detailed phase-by-phase implementation strategy
- ✅ Comprehensive technical specifications and code examples
- ✅ Complete file structure and organization plan
- ✅ Thorough testing and validation strategy
- ✅ Documentation and user experience plan
- ✅ Integration points with existing Oxide Flow systems
- ✅ Error handling and edge case considerations
- ✅ Performance and scalability requirements
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

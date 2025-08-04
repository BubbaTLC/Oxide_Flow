# Oxide Flow - AI Agent Instructions

## Project Overview
Oxide Flow is a command-line data transformation and integration tool built in Rust with a modular, plugin-based architecture. The project follows Unix philosophy - simple, composable tools that work together through data pipes.

## Core Architecture

### Plugin System - "Oxis"
- **Oxis** are plugins that extend functionality, designed like Unix pipes (`|`)
- Each Oxi handles both data input and output for flexible chaining
- Modular design is the "core of oxides" - all features should be implemented as Oxis
- Document new Oxis in `docs/` following the pattern in `docs/oxi.md`

### Configuration Philosophy
- YAML-first configuration approach - avoid hard-coded behaviors
- Command-line first optimization - prioritize CLI UX over GUI
- REST API driven - all functionality should be accessible via API

## Development Workflow

### Project Structure
```
src/main.rs          # Entry point (currently minimal)
docs/               # Documentation for concepts like Oxis
instructions.md     # Core design principles (read this first)
Cargo.toml         # Standard Rust project manifest
```

### Build Commands
```bash
cargo build          # Build the project
cargo run            # Run with default
cargo test           # Run tests (when added)
```

### Design Principles (from `instructions.md`)
1. **Command-line First** - Optimize CLI experience
2. **API Driven** - REST API backbone for integrations
3. **Modular Design** - Everything as Oxis (plugins)
4. **Minimal Dependencies** - Keep dependency tree lightweight
5. **Debuggable** - Verbose logs, clear error messages
6. **YAML Configurable** - Avoid hard-coded configurations

## Implementation Patterns

### Adding New Oxis
- Create new modules in `src/` for Oxi implementations
- Follow Unix pipe philosophy - clear input/output contracts
- Add documentation in `docs/` explaining the Oxi's purpose
- Ensure chainability with other Oxis

### Error Handling
- Prioritize "simple error messages" for user debugging
- Include verbose logging capabilities for troubleshooting
- Follow Rust's Result<T, E> pattern for error propagation

### Configuration
- Use YAML for all configuration files
- Place configs in logical locations (root for global, per-Oxi as needed)
- Validate configurations early with clear error messages

## Current State
- **Early Development**: Main entry point is placeholder "Hello, world!"
- **No Dependencies**: Clean slate for adding minimal, necessary crates
- **No Tests**: Test framework needs to be established
- **No CI/CD**: GitHub Actions or similar should be added

## Key Files to Understand
- `instructions.md` - Core design philosophy and principles
- `docs/oxi.md` - Plugin system concept explanation
- `Cargo.toml` - Note the Rust 2025 edition usage

## Development Priorities
1. Establish core Oxi plugin architecture in `src/`
2. Implement CLI argument parsing and command structure
3. Add YAML configuration parsing capabilities
4. Create foundational Oxis for common data transformations
5. Build REST API layer for programmatic access

## AI Personality
1. Ask questions when you need clarification and or more context
1. Work in steps and do not try to do everything at once
1. Provide clear, actionable feedback on progress
1. Focus on building a solid foundation before adding complexity

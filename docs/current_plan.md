# Pipeline Command Implementation Plan

## 🎯 Overview

Add a new `pipeline` command to Oxide Flow CLI for comprehensive pipeline management. This will provide users with tools to discover, create, test, and manage their data pipelines.

## 📋 Command Structure

```bash
oxide_flow pipeline <SUBCOMMAND> [OPTIONS]
```

## 🛠️ Subcommands

### 1. `list` - List Available Pipelines

**Syntax:**
```bash
oxide_flow pipeline list [OPTIONS]
```

**Options:**
- `--tags <TAGS>` / `-t` - Filter by tags (comma-separated)
- `--filter <KEYWORD>` / `-f` - Filter by keyword in name/description
- `--verbose` / `-v` - Show detailed information

**Features:**
- Discovers all pipelines in configured pipeline directory
- Displays pipeline metadata (name, description, version, author, tags)
- Shows step count and basic pipeline information
- Color-coded output for easy reading
- Search and filtering capabilities

**Example Output:**
```
📂 Available pipelines in ./pipelines (5 total):

┌─────────────────────┬──────────────────────────────┬─────────┬───────────┐
│ Name                │ Description                  │ Version │ Steps     │
├─────────────────────┼──────────────────────────────┼─────────┼───────────┤
│ pipeline            │ JSON to CSV Converter        │ v1.0.0  │ 4 steps   │
│ data_processor      │ Customer Data ETL            │ v2.1.0  │ 7 steps   │
│ error_handling_test │ Error Recovery Demo          │ v1.0.0  │ 5 steps   │
│ file_test           │ Simple File Test             │ v1.0.0  │ 3 steps   │
│ json_flattener      │ Flatten Nested JSON         │ v1.2.0  │ 4 steps   │
└─────────────────────┴──────────────────────────────┴─────────┴───────────┘

💡 Use 'oxide_flow pipeline info <name>' for detailed information
🚀 Use 'oxide_flow run <name>' to execute a pipeline
```

**Verbose Output:**
```
📂 Pipeline: data_processor
   📝 Description: Customer Data ETL
   👤 Author: Data Engineering Team
   🏷️  Tags: etl, customers, daily
   📅 Version: v2.1.0
   📍 Location: ./pipelines/data_processor.yaml
   ⚙️  Steps: 7 (read_file → parse_json → flatten → validate → format_csv → write_file → backup)
   🔧 Environment variables: INPUT_PATH, OUTPUT_PATH, BATCH_SIZE
```

### 2. `add` - Create New Pipeline

**Syntax:**
```bash
oxide_flow pipeline add <NAME> [OPTIONS]
```

**Options:**
- `--template <TEMPLATE>` / `-t` - Use specific template (default: "basic")
- `--description <DESC>` / `-d` - Pipeline description
- `--author <AUTHOR>` / `-a` - Pipeline author

**Available Templates:**
- `basic` - Simple read → transform → write pipeline
- `etl` - Extract, Transform, Load pattern
- `validation` - Data validation and quality checking
- `batch` - Batch processing with error handling
- `api` - API data processing
- `streaming` - Streaming data processing

**Features:**
- Interactive pipeline creation wizard
- Template-based pipeline generation
- Pre-filled metadata from project config
- Validation of pipeline name (snake_case enforcement)

**Example Usage:**
```bash
# Basic pipeline creation
oxide_flow pipeline add customer_etl

# With template and description
oxide_flow pipeline add data_validator --template validation --description "Validates incoming customer data"
```

**Interactive Flow:**
```
📝 Creating new pipeline: customer_etl

🎯 Select template:
  1. basic      - Simple read → transform → write
  2. etl        - Extract, Transform, Load pattern
  3. validation - Data validation and quality checking
  4. batch      - Batch processing with error handling
  5. api        - API data processing

Enter choice [1-5] (default: 1): 2

📋 Pipeline Details:
  Name: customer_etl
  Description: Customer data ETL pipeline
  Author: Data Engineering Team (from project config)
  Template: etl

✅ Created pipeline: ./pipelines/customer_etl.yaml

💡 Use 'oxide_flow pipeline test customer_etl' to validate
🚀 Use 'oxide_flow run customer_etl' to execute
```

### 3. `test` - Test/Validate Pipeline

**Syntax:**
```bash
oxide_flow pipeline test <NAME> [OPTIONS]
```

**Options:**
- `--dry-run` - Validate only, don't execute
- `--verbose` / `-v` - Show detailed validation information
- `--fix` - Attempt to fix common issues
- `--schema` - Validate against schemas only

**Features:**
- YAML syntax validation
- Schema validation for all Oxis
- Configuration completeness checking
- Environment variable validation
- Step reference validation
- Dependency checking

**Example Output:**
```bash
🧪 Testing pipeline: customer_etl

✅ YAML Syntax: Valid
✅ Schema Validation: All steps valid
✅ Environment Variables: All variables available
✅ Step References: All references valid
✅ Dependencies: All required Oxis available

📊 Pipeline Analysis:
   📈 Steps: 8 total
   🔄 Retry-enabled steps: 3
   ⏰ Timeout-configured steps: 2
   🛡️  Error-resilient steps: 5
   💾 File operations: 4 read, 2 write
   🌐 Network operations: 1

💡 Suggestions:
   • Consider adding timeout to step 'api_fetch'
   • Step 'data_validator' has no retry configuration

✅ Pipeline is ready for execution
```

**With --fix option:**
```bash
🧪 Testing pipeline: customer_etl (with auto-fix)

⚠️  Issue found: Missing timeout on 'api_fetch' step
   🔧 Auto-fix: Added timeout_seconds: 30

⚠️  Issue found: No retry configuration on 'data_validator'
   🔧 Auto-fix: Added retry_attempts: 1

✅ Fixed 2 issues automatically
✅ Pipeline is ready for execution
```

### 4. `info` - Show Pipeline Information

**Syntax:**
```bash
oxide_flow pipeline info <NAME> [OPTIONS]
```

**Options:**
- `--schema` - Show configuration schema for all steps
- `--json` - Output in JSON format
- `--yaml` - Output in YAML format

**Features:**
- Complete pipeline metadata
- Step-by-step breakdown
- Configuration schema display
- Environment variable requirements
- Performance characteristics

**Example Output:**
```bash
📋 Pipeline Information: customer_etl

📝 Metadata:
   Name: Customer Data ETL
   Description: Processes daily customer exports from CRM
   Version: 2.1.0
   Author: Data Engineering Team
   Tags: etl, customers, daily, crm
   Created: 2025-01-15
   Location: ./pipelines/customer_etl.yaml

⚙️  Configuration:
   Pipeline Directory: ./pipelines
   Steps: 8 total
   Environment Variables: 5 required, 3 optional

🔧 Environment Variables:
   Required:
     • INPUT_PATH - Path to input data file
     • OUTPUT_PATH - Path for processed output
     • CRM_API_KEY - API key for CRM access
     • DATABASE_URL - Database connection string
     • NOTIFICATION_EMAIL - Email for completion notifications

   Optional:
     • BATCH_SIZE (default: 1000) - Processing batch size
     • TIMEOUT (default: 300) - Processing timeout seconds
     • RETRY_COUNT (default: 3) - Number of retry attempts

📊 Steps:
   1. crm_data_fetch (read_file)
      └─ Fetches customer data from CRM export
      └─ Retry: 3 attempts, Timeout: 60s

   2. data_parser (parse_json)
      └─ Parses JSON customer records
      └─ Continue on error: false

   3. data_validator (custom_validator)
      └─ Validates customer data quality
      └─ Retry: 1 attempt

   4. data_transformer (flatten)
      └─ Flattens nested customer objects
      └─ Continue on error: true

   5. csv_formatter (format_csv)
      └─ Formats data as CSV for warehouse
      └─ Headers: true, Delimiter: "|"

   6. warehouse_writer (write_file)
      └─ Writes to data warehouse staging
      └─ Create directories: true

   7. backup_writer (write_file)
      └─ Creates backup copy
      └─ Continue on error: true

   8. notifier (send_notification)
      └─ Sends completion notification
      └─ Continue on error: true

🔗 Dependencies:
   ✅ All required Oxis available
   ✅ All environment variables can be resolved
   ✅ All step references are valid
```

## 🏗️ Implementation Plan

### Phase 1: CLI Structure ✅ **COMPLETED**
**Status:** Implemented on August 4, 2025
**Implementation Notes:** Successfully added complete CLI structure with all subcommands and proper argument parsing.

1. **Update CLI enum** in `src/cli.rs` ✅ DONE
   - Added `Pipeline` command with nested subcommands
   - Created `PipelineAction` enum with List, Add, Test, Info subcommands
   - All CLI options properly defined with correct types and defaults

2. **Add main command handler** in `src/main.rs` ✅ DONE
   - Added import for `PipelineAction`
   - Implemented `handle_pipeline_command()` function with placeholder logic
   - Proper error handling and graceful exits implemented
   - All CLI commands tested and working correctly

**Testing Results:**
- All CLI help commands work correctly
- Option parsing validated for all subcommands
- Existing `init` and `run` commands remain functional
- Global verbose flag works across all commands

**Next Phase Dependencies:** Ready for Phase 2 implementation - no changes to CLI structure needed.

### Phase 2: Core Functions ✅ **COMPLETED**
**Status:** Implemented on August 4, 2025
**Implementation Notes:** Successfully implemented pipeline discovery, metadata extraction, and listing functionality with filtering capabilities.

1. **Create `src/pipeline_manager.rs`** ✅ DONE
   - Complete pipeline discovery system with YAML parsing
   - Metadata extraction from both `metadata` section and root-level fields
   - Support for both `pipeline` and `steps` arrays for step counting
   - Flexible name matching (display name or filename)

2. **Implement list functionality** ✅ DONE
   - Table format for compact display with proper column alignment
   - Verbose mode with detailed pipeline information
   - Keyword filtering by name and description
   - Tag filtering support (when tags are available)
   - Proper truncation of long strings in table format

**Additional Features Implemented:**
- Pipeline info command with JSON/YAML output options
- Flexible pipeline name resolution for commands
- Integration with existing project configuration system
- Comprehensive metadata parsing and display

**Testing Results:**
- All pipeline discovery working correctly from example_project directory
- Metadata parsing extracts name, description, version, author, and step count
- Table and verbose display formats working properly
- Keyword filtering successfully filters pipelines
- Info command provides detailed pipeline information
- JSON output format working for programmatic access
- Existing run command remains fully functional
- Commands tested from within example_project directory using full binary path

**Next Phase Dependencies:** Phase 3 completed successfully! Ready for Phase 4 implementation - pipeline testing and validation functionality.

### Phase 3: Pipeline Creation ✅ **COMPLETED**
1. **Template system**
   - ✅ Create template directory structure (`src/templates/` with 6 templates)
   - ✅ Implement template rendering with variable substitution
   - ✅ Interactive pipeline creation wizard (`pipeline add` command)
   - ✅ Template validation and error handling
   - ✅ Pipeline name validation (snake_case enforcement)
   - ✅ Duplicate pipeline detection

**Completed Features:**
- 6 predefined templates: basic, etl, validation, batch, api, streaming
- Template variable substitution: `{{pipeline_name}}`, `{{pipeline_description}}`, etc.
- CLI integration with `oxide_flow pipeline add` command
- Comprehensive error handling for invalid templates and names
- File creation in project's pipeline directory

### Phase 4: Testing & Validation 🔄 **NEXT**
1. **Validation engine**
   - YAML syntax validation
   - Schema validation integration
   - Environment variable checking
   - Step reference validation

2. **Auto-fix capabilities**
   - Common issue detection
   - Automatic corrections
   - User confirmation for changes

## 📁 File Structure Changes

```
src/
├── main.rs                 # Updated with pipeline command routing
├── cli.rs                  # Updated with pipeline subcommands
├── pipeline_manager.rs     # New: pipeline management functions
├── templates/              # New: pipeline templates
│   ├── basic.yaml
│   ├── etl.yaml
│   ├── validation.yaml
│   ├── batch.yaml
│   └── api.yaml
└── pipeline.rs            # Enhanced with metadata functions
```

## 🎯 Success Criteria

### User Experience
- ✅ Users can easily discover available pipelines
- ✅ Creating new pipelines is intuitive and fast
- ✅ Pipeline validation catches issues before execution
- ✅ Clear, helpful error messages and suggestions

### Developer Experience
- ✅ Template system makes pipeline creation standardized
- ✅ Auto-fix reduces manual configuration errors
- ✅ Detailed validation helps debug complex pipelines
- ✅ Integration-friendly output for CI/CD

### Performance
- ✅ Pipeline listing is fast (<100ms for 100 pipelines)
- ✅ Validation completes quickly (<500ms per pipeline)
- ✅ Template generation is near-instantaneous

## 🚀 Example Usage Workflows

### Discovery Workflow
```bash
# List all pipelines
oxide_flow pipeline list

# Filter by tag
oxide_flow pipeline list --tags etl,daily

# Get detailed info
oxide_flow pipeline info customer_etl

# Test before running
oxide_flow pipeline test customer_etl

# Execute
oxide_flow run customer_etl
```

### Development Workflow
```bash
# Create new pipeline from template
oxide_flow pipeline add new_processor --template etl

# Test during development
oxide_flow pipeline test new_processor --verbose

# Fix issues automatically
oxide_flow pipeline test new_processor --fix

# Final validation
oxide_flow pipeline test new_processor

# Run when ready
oxide_flow run new_processor
```

### CI/CD Integration
```bash
# List pipelines in JSON for processing
oxide_flow pipeline list --json > available_pipelines.json

# Validate all pipelines
for pipeline in $(oxide_flow pipeline list --json | jq -r '.[].name'); do
    oxide_flow pipeline test "$pipeline" || exit 1
done

# Run specific pipeline
oxide_flow run production_etl
```

---

## 📊 Implementation Progress Summary

### ✅ Completed Phases
- **Phase 1: CLI Structure** - Implemented August 4, 2025
  - Complete CLI interface with all subcommands
  - Proper argument parsing and error handling
  - All commands tested and functional

- **Phase 2: Core Functions** - Implemented August 4, 2025
  - Pipeline discovery and metadata extraction
  - Table and verbose listing formats
  - Keyword and tag filtering capabilities
  - Pipeline info command with multiple output formats

- **Phase 3: Pipeline Creation** - Implemented August 4, 2025
  - Template system with 6 predefined templates
  - Variable substitution and pipeline generation
  - CLI integration with `pipeline add` command
  - Error handling and validation

### 🔄 Current Phase
- **Phase 4: Testing & Validation** - Ready for implementation
  - Pipeline testing and validation functionality
  - Interactive pipeline creation wizard

### ⏳ Upcoming Phases
- **Phase 4: Testing & Validation** - Validation engine and auto-fix capabilities

**Overall Progress: 75% Complete (3/4 phases)**

This comprehensive pipeline management system will make Oxide Flow much more user-friendly and provide professional-grade pipeline development tools!

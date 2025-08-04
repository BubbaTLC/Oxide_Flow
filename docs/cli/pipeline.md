# `pipeline` - Pipeline Management

Comprehensive pipeline management commands for discovering, creating, testing, and analyzing data pipelines.

## Syntax

```bash
oxide_flow pipeline <SUBCOMMAND> [OPTIONS]
```

## Subcommands

### `list` - List Available Pipelines

Discover and display all pipelines in the project.

**Syntax:**
```bash
oxide_flow pipeline list [OPTIONS]
```

**Options:**
- `--tags` / `-t` `<TAGS>` - Filter by tags (comma-separated)
- `--filter` / `-f` `<KEYWORD>` - Filter by keyword in name/description
- `--verbose` / `-v` - Show detailed information including step names

**Examples:**
```bash
# List all pipelines
oxide_flow pipeline list

# List with verbose details (shows step names)
oxide_flow pipeline list --verbose

# Filter by tags
oxide_flow pipeline list --tags etl,production

# Filter by keyword
oxide_flow pipeline list --filter json
```

**Standard Output:**
```bash
📂 Available pipelines in ./pipelines (6 total):

┌─────────────────────┬──────────────────────────────┬─────────┬───────────┐
│ Name                │ Description                  │ Version │ Steps     │
├─────────────────────┼──────────────────────────────┼─────────┼───────────┤
│ Template Api        │ API template test pipeline   │ 1.0.0   │ 5 steps   │
│ Template Basic      │ Basic template test pipeline │ 1.0.0   │ 3 steps   │
│ Template Etl        │ ETL template test pipeline   │ 1.0.0   │ 5 steps   │
└─────────────────────┴──────────────────────────────┴─────────┴───────────┘

💡 Use 'oxide_flow pipeline info <name>' for detailed information
🚀 Use 'oxide_flow run <name>' to execute a pipeline
```

**Verbose Output:**
```bash
📂 Available pipelines in ./pipelines (3 total):

📂 Pipeline: Template Basic
   📝 Description: Basic template test pipeline
   👤 Author: Template Testing
   🏷️  Tags: basic, simple
   📅 Version: 1.0.0
   📍 Location: ./pipelines/template_basic.yaml
   ⚙️  Steps: 3 (read_file → parse_json → write_file)

📂 Pipeline: Template Etl
   📝 Description: ETL template test pipeline
   👤 Author: Template Testing
   🏷️  Tags: etl, data-processing, production
   📅 Version: 1.0.0
   📍 Location: ./pipelines/template_etl.yaml
   ⚙️  Steps: 5 (read_file → parse_json → flatten → format_csv → write_file)
```

### `add` - Create New Pipeline

Create a new pipeline from a predefined template.

**Syntax:**
```bash
oxide_flow pipeline add <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Name of the new pipeline (must be snake_case)

**Options:**
- `--template` / `-t` `<TEMPLATE>` - Template to use (default: "basic")
- `--description` / `-d` `<DESC>` - Pipeline description
- `--author` / `-a` `<AUTHOR>` - Pipeline author

**Available Templates:**
- `basic` - Simple read → transform → write pattern
- `etl` - Extract, Transform, Load pattern for data processing
- `validation` - Data validation and quality checking
- `batch` - Batch processing with error handling
- `api` - API data processing pipeline
- `streaming` - Real-time data streaming

**Examples:**
```bash
# Create basic pipeline
oxide_flow pipeline add my_pipeline

# Create ETL pipeline with description
oxide_flow pipeline add customer_etl --template etl --description "Customer data processing"

# Create with full metadata
oxide_flow pipeline add api_processor --template api --description "API data processor" --author "Data Team"
```

**Output:**
```bash
📝 Creating new pipeline: customer_etl
  Template: etl
✅ Pipeline 'customer_etl' created successfully!
```

### `test` - Test/Validate Pipeline

Validate pipeline configuration and structure.

**Syntax:**
```bash
oxide_flow pipeline test <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Name of the pipeline to test

**Options:**
- `--dry-run` - Validate only, don't execute
- `--verbose` / `-v` - Show detailed validation information
- `--fix` - Attempt to fix common issues (future feature)
- `--schema` - Validate against schemas only

**Examples:**
```bash
# Basic validation
oxide_flow pipeline test my_pipeline

# Detailed validation
oxide_flow pipeline test my_pipeline --verbose

# Schema-only validation
oxide_flow pipeline test my_pipeline --schema

# Dry-run validation
oxide_flow pipeline test my_pipeline --dry-run
```

**Output:**
```bash
🧪 Testing pipeline: template_etl

✅ YAML Syntax: Valid
✅ Schema Validation: All steps valid
✅ Environment Variables: All variables available
✅ Step References: All references valid

📊 Pipeline Analysis:
   📈 Steps: 5 total
   🔄 Retry-enabled steps: 3
   ⏰ Timeout-configured steps: 2
   🛡️  Error-resilient steps: 1
   💾 File operations: 1 read, 1 write
   🌐 Network operations: 0

💡 Suggestions:
   • Consider adding timeout to step 'api_fetch'
   • Step 'data_validator' has no retry configuration

✅ Pipeline is ready for execution
```

### `info` - Show Pipeline Information

Display detailed information about a specific pipeline.

**Syntax:**
```bash
oxide_flow pipeline info <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Name of the pipeline

**Options:**
- `--schema` - Show configuration schema for all steps
- `--json` - Output in JSON format
- `--yaml` - Output in YAML format

**Examples:**
```bash
# Standard information
oxide_flow pipeline info template_basic

# JSON output for scripting
oxide_flow pipeline info template_basic --json

# YAML output
oxide_flow pipeline info template_basic --yaml
```

**Output:**
```bash
📋 Pipeline Information: Template Basic

📝 Metadata:
   Description: Basic template test pipeline
   Version: 1.0.0
   Author: Template Testing
   Tags: basic, simple
   Location: ./pipelines/template_basic.yaml

⚙️  Configuration:
   Steps: 3 (read_file → parse_json → write_file)
```

## Error Handling

### Pipeline Not Found

```bash
$ oxide_flow pipeline test nonexistent
❌ Pipeline testing failed: Pipeline 'nonexistent' not found
```

### Invalid Pipeline Name

```bash
$ oxide_flow pipeline add "Invalid Name"
❌ Pipeline command failed: Invalid pipeline name 'Invalid Name'. Use snake_case format (e.g., my_pipeline)
```

### Invalid Template

```bash
$ oxide_flow pipeline add test_pipeline --template nonexistent
❌ Pipeline command failed: Unknown template: nonexistent
```

### Validation Errors

```bash
$ oxide_flow pipeline test broken_pipeline
🧪 Testing pipeline: broken_pipeline

❌ YAML Syntax: Invalid
❌ Schema Validation: Issues found
✅ Environment Variables: All variables available
✅ Step References: All references valid

❌ Issues Found:
   • YAML Syntax: YAML syntax error: expected string at line 10
   • Structure: Missing required 'name' field in step 2

❌ Pipeline has 2 issues that need to be fixed
```

## Filtering and Search

### Tag Filtering

```bash
# Find all ETL pipelines
oxide_flow pipeline list --tags etl

# Find production-ready pipelines
oxide_flow pipeline list --tags production

# Multiple tags (OR logic)
oxide_flow pipeline list --tags etl,batch
```

### Keyword Filtering

```bash
# Find pipelines with "json" in name or description
oxide_flow pipeline list --filter json

# Find API-related pipelines
oxide_flow pipeline list --filter api

# Case-insensitive search
oxide_flow pipeline list --filter CSV
```

### Combined Filtering

```bash
# ETL pipelines containing "customer"
oxide_flow pipeline list --tags etl --filter customer
```

## Pipeline Templates

### Template Structure

Each template includes:
- **Pipeline steps** - Pre-configured Oxi sequence
- **Metadata** - Name, description, tags, version
- **Configuration** - Common settings and patterns
- **Documentation** - Inline comments and examples

### Customizing Templates

After creation, you can modify generated pipelines:

```bash
# Create from template
oxide_flow pipeline add my_etl --template etl

# Edit the generated pipeline
nano pipelines/my_etl.yaml

# Test your changes
oxide_flow pipeline test my_etl

# Run when ready
oxide_flow run my_etl
```

## Best Practices

### Pipeline Naming

- Use **snake_case**: `customer_data_etl`
- Be **descriptive**: `daily_sales_report` vs `pipeline1`
- Include **purpose**: `validate_user_data` vs `validator`

### Template Selection

- **basic**: Simple data transformations
- **etl**: Complex data processing workflows
- **validation**: Data quality and schema checking
- **batch**: Large dataset processing
- **api**: External service integration
- **streaming**: Real-time data processing

### Validation Workflow

```bash
# 1. Create pipeline
oxide_flow pipeline add new_pipeline --template etl

# 2. Validate structure
oxide_flow pipeline test new_pipeline --dry-run

# 3. Fix any issues
nano pipelines/new_pipeline.yaml

# 4. Full validation
oxide_flow pipeline test new_pipeline --verbose

# 5. Test execution
oxide_flow run new_pipeline
```

## Integration with Other Commands

### Project Workflow

```bash
# 1. Initialize project
oxide_flow init my_project
cd my_project

# 2. List available pipelines
oxide_flow pipeline list

# 3. Create new pipelines
oxide_flow pipeline add data_processor --template etl

# 4. Validate pipelines
oxide_flow pipeline test data_processor

# 5. Run pipelines
oxide_flow run data_processor
```

## Related Commands

- [`init`](init.md) - Initialize projects that contain pipelines
- [`run`](run.md) - Execute pipelines managed by these commands

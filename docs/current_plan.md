# Pipeline Template Fixes & Oxi SDK Foundation Plan

## 🎯 Project Overview

**Title**: Pipeline Template Failures, Core System Fixes & Oxi SDK Foundation
**Purpose**: Fix failing pipeline templates, implement missing Oxis, establish robust Oxi SDK, and add batch processing capabilities
**Scope**: Implement `json` & `batch` Oxis, enhance type system, create Oxi SDK foundation, and fix type conversions
**Success Criteria**: All 6 pipeline templates execute successfully + robust Oxi SDK for future development + flexible batch processing

## 📋 Current Issues Analysis

### ❌ **Template Failures:**

1. **Validation Pipeline** - Missing `json` Oxi for schema validation operations
2. **Streaming Pipeline** - Missing `json` Oxi for filtering/query operations
3. **API Pipeline** - Missing `json` Oxi + type mismatch between JSON and CSV formatter

### ⚠️ **Core System Issues:**

1. **Missing JSON Oxi** - Templates reference non-existent `json` Oxi type
2. **Type System Gaps** - Limited conversion between OxiData variants
3. **Error Handling Gaps** - Basic error types without pipeline context
4. **Format Mismatch** - CSV formatter expects specific input structure
5. **No Oxi SDK Foundation** - Inconsistent Oxi implementations without standard patterns
6. **No Batch Processing** - Missing modular batch Oxi for flexible batch processing

## 🏗️ Implementation Strategy

### **Phase 1: Oxi SDK Foundation (Priority 1) - ✅ COMPLETED**

✅ **COMPLETED** - The Oxi SDK foundation has been fully implemented with:
- Enhanced Oxi trait with processing limits and validation
- ProcessingLimits system with resource constraints
- Enhanced OxiData with batch awareness and memory estimation
- Context-aware error handling
- Comprehensive test suite (9/9 tests passing)
- Complete documentation and templates

### **Phase 1.5: Schema System Enhancement (NEW PRIORITY)**

**NEWLY IDENTIFIED NEED**: Each Oxi needs to understand and communicate schema information for the data it processes. This enables:
- Type checking and validation between Oxis
- Automatic data conversion when possible
- Better error messages with field-level context
- Schema evolution tracking through pipelines

**Key Design Decision**: Schema travels as **metadata alongside data**, not embedded within it. This keeps data clean while enabling powerful schema-aware processing.

**Core Implementation**:

1. **OxiSchema System**:
```rust
// Schema travels with data as metadata
#[derive(Debug, Clone)]
pub struct OxiDataWithSchema {
    pub data: OxiData,
    pub schema: Option<OxiSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OxiSchema {
    pub fields: HashMap<String, FieldSchema>,
    pub metadata: SchemaMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldSchema {
    pub field_type: FieldType,      // String, Integer, Float, Boolean, Array, Object, etc.
    pub nullable: bool,             // Can this field be null/empty?
    pub max_size: Option<usize>,    // Size limits for strings/arrays
    pub constraints: Vec<FieldConstraint>, // Validation rules
    pub description: Option<String>,
    pub examples: Vec<serde_json::Value>,
}
```

2. **Enhanced Oxi Trait with Schema Awareness**:
```rust
#[async_trait]
pub trait Oxi {
    // Existing methods...

    /// Calculate output schema from input schema and config
    fn output_schema(
        &self,
        input_schema: Option<&OxiSchema>,
        config: &OxiConfig
    ) -> anyhow::Result<OxiSchema>;

    /// Schema-aware processing (optional with default implementation)
    async fn process_with_schema(
        &self,
        input: OxiDataWithSchema,
        config: &OxiConfig,
    ) -> anyhow::Result<OxiDataWithSchema> {
        // Default: call process() and calculate output schema
        let output_data = self.process(input.data, config).await?;
        let output_schema = self.output_schema(input.schema.as_ref(), config)?;
        Ok(OxiDataWithSchema::new(output_data, output_schema))
    }
}
```

3. **Automatic Schema Inference**:
```rust
// JSON data automatically infers schema
let data = OxiData::Json(json!({"name": "John", "age": 30}));
let schema = data.infer_schema()?;
// Produces: name: String, age: Integer

// Array data merges schemas from sample elements
let array_data = OxiData::Json(json!([
    {"name": "John", "age": 30},
    {"name": "Jane", "age": 25, "email": "jane@example.com"}
]));
let schema = array_data.infer_schema()?;
// Produces: name: String, age: Integer, email: String (nullable)
```

4. **Schema Evolution Through Pipeline**:
```yaml
# Input: JSON with nested users array
# Schema: { users: Array<Object> }

pipeline:
  - name: parse_json
    # Output schema: { users: Array<Object> }
  - name: json
    config: { operation: query, query: ".users[]" }
    # Output schema: Object (individual user)
  - name: csv
    # Output schema: { csv_data: String }
```

**Benefits**:
- **Type Safety**: Automatic validation between stages
- **Better Errors**: "Field 'age' expected Integer, got String '30'" instead of generic type errors
- **Documentation**: Schemas document data structure through pipeline
- **Auto-conversion**: Schema mismatches can trigger automatic type conversion when safe
- **Backward Compatible**: Existing Oxis work without schemas, but gain benefits when schemas are present

**Implementation Sub-phases**:
- **1.5a**: Core schema types and inference logic
- **1.5b**: Enhanced OxiData with schema support
- **1.5c**: Oxi trait enhancement and existing Oxi updates
- **1.5d**: Pipeline schema validation and documentation

```rust
// src/oxis/prelude.rs
use crate::types::OxiData;
use crate::error::OxiError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Core trait that all Oxis must implement
#[async_trait]
pub trait Oxi: Send + Sync {
    /// Unique name for this Oxi type
    fn name(&self) -> &str;

    /// Configuration schema for validation
    fn config_schema(&self) -> serde_yaml::Value;

    /// Process data through this Oxi
    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError>;

    /// Optional: Set processing limits (memory, batch size, etc.)
    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits::default()
    }

    /// Optional: Validate input data before processing
    fn validate_input(&self, input: &OxiData) -> Result<(), OxiError> {
        Ok(()) // Default: accept all inputs
    }
}

/// Processing limits that each Oxi can define
#[derive(Debug, Clone)]
pub struct ProcessingLimits {
    pub max_batch_size: Option<usize>,
    pub max_memory_mb: Option<usize>,
    pub max_processing_time_ms: Option<u64>,
    pub supported_input_types: Vec<OxiDataType>,
}

impl Default for ProcessingLimits {
    fn default() -> Self {
        Self {
            max_batch_size: Some(10_000), // Default 10K records
            max_memory_mb: Some(256),     // Default 256MB
            max_processing_time_ms: Some(30_000), // Default 30s
            supported_input_types: vec![
                OxiDataType::Json,
                OxiDataType::Text,
                OxiDataType::Binary,
                OxiDataType::Empty,
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub enum OxiDataType {
    Json,
    Text,
    Binary,
    Empty,
}
```

### **Phase 2: Batch Oxi Implementation (Priority 2)**

The key insight: batch processing as a composable Oxi that can be inserted anywhere:

```rust
// src/oxis/batch/mod.rs
pub struct BatchOxi {
    config: BatchConfig,
}

#[derive(Debug, Deserialize)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub flush_interval_ms: Option<u64>,
    pub max_memory_mb: Option<usize>,
    pub strategy: BatchStrategy,
}

#[derive(Debug, Deserialize)]
pub enum BatchStrategy {
    Size,        // Flush when batch reaches size
    Time,        // Flush on time interval
    SizeOrTime,  // Flush on either condition
    Memory,      // Flush when memory limit approached
}

impl BatchOxi {
    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        match input {
            OxiData::Json(serde_json::Value::Array(items)) => {
                self.process_array_in_batches(items).await
            }
            OxiData::Json(single_item) => {
                // Single items go into a batch of 1
                let batch = vec![single_item];
                Ok(OxiData::Json(serde_json::Value::Array(batch)))
            }
            _ => Err(OxiError::TypeMismatch {
                expected: "JSON Array or Object".to_string(),
                actual: format!("{:?}", input),
                step: "batch".to_string(),
            })
        }
    }

    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: Some(self.config.batch_size),
            max_memory_mb: self.config.max_memory_mb,
            max_processing_time_ms: self.config.flush_interval_ms,
            supported_input_types: vec![OxiDataType::Json],
        }
    }
}
```

**Batch Processing Usage Examples:**
```yaml
# Streaming processing (no batch Oxi)
pipeline:
  - name: read_file
  - name: parse_json
  - name: transform
  - name: write_file

# Batch processing (insert batch Oxi)
pipeline:
  - name: read_file
  - name: parse_json
  - name: batch              # ← Batch Oxi inserted here
    config:
      batch_size: 1000
      strategy: size_or_time
      flush_interval_ms: 5000
  - name: transform           # Now processes batches of 1000
  - name: write_file
```

### **Phase 3: JSON Oxi Implementation (Priority 3)**

### **Phase 3: JSON Oxi Implementation (Priority 3)**

The core missing piece that breaks 3 templates:

```rust
// src/oxis/json/mod.rs
pub struct JsonOxi {
    config: JsonConfig,
}

#[derive(Debug, Deserialize)]
pub struct JsonConfig {
    pub operation: JsonOperation,
    // For validation operations
    pub schema: Option<String>,
    // For query/filter operations
    pub query: Option<String>,
    // For transformation operations
    pub transform: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub enum JsonOperation {
    Validate,    // Schema validation
    Query,       // JQ-style filtering
    Transform,   // Data transformation
    Extract,     // Extract specific fields
}

impl Oxi for JsonOxi {
    fn name(&self) -> &str { "json" }

    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: Some(50_000), // JSON can handle larger batches
            max_memory_mb: Some(512),
            supported_input_types: vec![OxiDataType::Json, OxiDataType::Text],
            ..Default::default()
        }
    }

    fn validate_input(&self, input: &OxiData) -> Result<(), OxiError> {
        match input {
            OxiData::Json(_) | OxiData::Text(_) => Ok(()),
            _ => Err(OxiError::TypeMismatch {
                expected: "JSON or Text".to_string(),
                actual: format!("{:?}", input),
                step: "json".to_string(),
            })
        }
    }
}
```

**Configuration Support:**
```yaml
# Validation use case (validation.yaml)
json:
  operation: validate
  schema: |
    {
      "type": "array",
      "items": {...}
    }

# Query use case (streaming.yaml, api.yaml)
json:
  operation: query
  query: "select(.timestamp > (now - 3600))"

json:
  operation: query
  query: ".data[]"
```

### **Phase 4: Type System Enhancement (Priority 4)**

Enhanced `OxiData` with better conversion support and batch awareness:

```rust
// src/types.rs enhancements
impl OxiData {
    /// Enhanced JSON conversion with validation
    pub fn as_json(&self) -> anyhow::Result<&serde_json::Value> {
        match self {
            OxiData::Json(data) => Ok(data),
            OxiData::Text(text) => {
                Err(anyhow::anyhow!("Cannot convert text to JSON without parsing. Use to_json() instead."))
            }
            _ => Err(anyhow::anyhow!("Cannot convert {:?} to JSON", self))
        }
    }

    /// Convert to JSON with fallback parsing
    pub fn to_json(&self) -> anyhow::Result<serde_json::Value> {
        match self {
            OxiData::Json(data) => Ok(data.clone()),
            OxiData::Text(text) => {
                serde_json::from_str(text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse text as JSON: {}", e))
            }
            OxiData::Binary(_) => Err(anyhow::anyhow!("Cannot convert binary data to JSON")),
            OxiData::Empty => Ok(serde_json::Value::Null),
        }
    }

    /// Enhanced array handling for CSV formatting and batch processing
    pub fn as_array(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        match self {
            OxiData::Json(serde_json::Value::Array(arr)) => Ok(arr.clone()),
            OxiData::Json(single_obj) => Ok(vec![single_obj.clone()]),
            _ => Err(anyhow::anyhow!("Cannot convert to array"))
        }
    }

    /// Check if data represents a batch (array with multiple items)
    pub fn is_batch(&self) -> bool {
        match self {
            OxiData::Json(serde_json::Value::Array(arr)) => arr.len() > 1,
            _ => false,
        }
    }

    /// Get estimated memory usage for processing limits
    pub fn estimated_memory_usage(&self) -> usize {
        match self {
            OxiData::Json(value) => {
                // Rough estimate: JSON string length * 2 for overhead
                value.to_string().len() * 2
            }
            OxiData::Text(text) => text.len(),
            OxiData::Binary(bytes) => bytes.len(),
            OxiData::Empty => 0,
        }
    }
}
```

### **Phase 5: Enhanced Error Handling (Priority 5)**

Context-aware errors with processing limit violations:

```rust
// src/error.rs enhancements
#[derive(Error, Debug)]
pub enum OxiError {
    // Existing errors...

    #[error("JSON operation failed: {operation} - {details}")]
    JsonOperationError { operation: String, details: String },

    #[error("Type mismatch in pipeline: expected {expected}, got {actual} at step '{step}'")]
    TypeMismatch { expected: String, actual: String, step: String },

    #[error("Schema validation failed: {details}")]
    ValidationError { details: String },

    #[error("Query operation failed: {query} - {error}")]
    QueryError { query: String, error: String },

    #[error("Data format incompatible: {source_format} cannot be converted to {target_format}")]
    FormatIncompatible { source_format: String, target_format: String },

    // New batch processing errors
    #[error("Batch size limit exceeded: {actual_size} > {max_size} in '{oxi_name}'")]
    BatchSizeExceeded { actual_size: usize, max_size: usize, oxi_name: String },

    #[error("Memory limit exceeded: {actual_mb}MB > {max_mb}MB in '{oxi_name}'")]
    MemoryLimitExceeded { actual_mb: usize, max_mb: usize, oxi_name: String },

    #[error("Processing timeout: {actual_ms}ms > {max_ms}ms in '{oxi_name}'")]
    ProcessingTimeout { actual_ms: u64, max_ms: u64, oxi_name: String },

    #[error("Unsupported input type: {oxi_name} does not support {input_type}")]
    UnsupportedInputType { oxi_name: String, input_type: String },
}
```

### **Phase 6: CSV Format Oxi Enhancement (Priority 6)**

Fix the type mismatch in API pipeline and add processing limits:

```rust
// src/oxis/csv/oxi.rs enhancements
impl Oxi for CsvOxi {
    fn name(&self) -> &str { "csv" }

    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: Some(100_000), // CSV can handle large batches efficiently
            max_memory_mb: Some(1024),     // 1GB for large CSV operations
            supported_input_types: vec![OxiDataType::Json],
            ..Default::default()
        }
    }

    async fn process(&self, data: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        // Check memory limits before processing
        let memory_usage = data.estimated_memory_usage();
        let limits = self.processing_limits();

        if let Some(max_mb) = limits.max_memory_mb {
            let max_bytes = max_mb * 1024 * 1024;
            if memory_usage > max_bytes {
                return Err(OxiError::MemoryLimitExceeded {
                    actual_mb: memory_usage / 1024 / 1024,
                    max_mb,
                    oxi_name: self.name().to_string(),
                });
            }
        }

        // Enhanced input handling
        let json_array = match data {
            OxiData::Json(serde_json::Value::Array(arr)) => {
                // Check batch size limits
                if let Some(max_size) = limits.max_batch_size {
                    if arr.len() > max_size {
                        return Err(OxiError::BatchSizeExceeded {
                            actual_size: arr.len(),
                            max_size,
                            oxi_name: self.name().to_string(),
                        });
                    }
                }
                arr
            }
            OxiData::Json(single_obj) => vec![single_obj], // Wrap single objects
            _ => {
                return Err(OxiError::FormatIncompatible {
                    source_format: format!("{:?}", data),
                    target_format: "CSV".to_string(),
                });
            }
        };

        // Convert to CSV
        let csv_output = self.json_array_to_csv(json_array)?;
        Ok(OxiData::Text(csv_output))
    }
}
```

## 📈 Implementation Steps

### **Step 1: Oxi SDK Foundation**
1. Create robust `src/oxis/prelude.rs` with core `Oxi` trait
2. Add `ProcessingLimits` and validation mechanisms
3. Create Oxi template and documentation
4. Refactor existing Oxis to use new SDK

### **Step 2: Batch Oxi Implementation**
1. Create `src/oxis/batch/mod.rs` and `src/oxis/batch/oxi.rs`
2. Implement size, time, and memory-based batching strategies
3. Add batch size and memory limit checking
4. Test with various batch sizes and data types

### **Step 3: JSON Oxi Foundation**
1. Create `src/oxis/json/mod.rs` and `src/oxis/json/oxi.rs`
2. Implement basic structure with configuration parsing
3. Add to `src/oxis/mod.rs` registry
4. Implement schema validation operation first (validation.yaml fix)

### **Step 4: Query Operations**
1. Add JQ-style query support (or simpler JSON path)
2. Implement filtering for streaming.yaml fix
3. Implement extraction for api.yaml fix

### **Step 5: Type System & Limits**
1. Enhance `OxiData` conversion methods with batch awareness
2. Add memory estimation and processing limit checks
3. Test conversion edge cases

### **Step 6: CSV Format Enhancement**
1. Modify CSV Oxi to use new SDK with processing limits
2. Add auto-wrapping for non-array JSON
3. Implement memory and batch size checking
4. Validate api.yaml template works end-to-end

### **Step 7: Error Context & Limits**
1. Add processing limit violations to error types
2. Improve error messages with Oxi context
3. Add configuration validation for processing limits

## 🧪 Testing Strategy

### **Oxi SDK Tests:**
```rust
// tests/oxi_sdk_tests.rs
#[test]
fn test_processing_limits_validation() {
    // Test that Oxis properly validate processing limits
}

#[test]
fn test_oxi_trait_implementation() {
    // Test core Oxi trait functionality
}
```

### **Batch Processing Tests:**
```rust
// tests/batch_oxi_tests.rs
#[test]
fn test_batch_size_limits() {
    // Test batch size limit enforcement
}

#[test]
fn test_memory_limit_enforcement() {
    // Test memory limit checking
}

#[test]
fn test_batch_strategies() {
    // Test size, time, and memory-based batching
}
```

### **JSON Oxi Tests:**
```rust
// tests/json_oxi_tests.rs
#[test]
fn test_json_validation() {
    // Test schema validation
}

#[test]
fn test_json_query() {
    // Test query operations
}

#[test]
fn test_processing_limits() {
    // Test JSON Oxi respects processing limits
}
```

### **Type Conversion Tests:**
```rust
// tests/type_conversion_tests.rs
#[test]
fn test_json_array_conversion() {
    // Test OxiData::as_array()
}

#[test]
fn test_memory_estimation() {
    // Test OxiData::estimated_memory_usage()
}

#[test]
fn test_batch_detection() {
    // Test OxiData::is_batch()
}
```

### **Pipeline Integration Tests:**
```rust
// tests/pipeline_integration_tests.rs
#[test]
fn test_streaming_vs_batch_processing() {
    // Test same pipeline with and without batch Oxi
}

#[test]
fn test_template_execution() {
    // Test all 6 templates execute successfully
}

#[test]
fn test_processing_limit_violations() {
    // Test proper error handling when limits are exceeded
}
```

## 🎯 Success Criteria

### **Oxi SDK Foundation:**
- ✅ All Oxis implement consistent `Oxi` trait with processing limits
- ✅ Processing limits are enforced with clear error messages
- ✅ Oxi template and documentation enable easy new Oxi development
- ✅ Memory usage estimation works accurately

### **Batch Processing:**
- ✅ Batch Oxi can be inserted anywhere in pipeline for flexible batching
- ✅ Size, time, and memory-based batching strategies work correctly
- ✅ Processing limits prevent memory exhaustion and timeouts
- ✅ Same pipeline works both streaming and batch with just Oxi insertion

### **Template Execution:**
- ✅ validation.yaml executes without "Unknown Oxi: json" error
- ✅ streaming.yaml processes JSON with query filtering
- ✅ api.yaml converts JSON to CSV without type mismatch
- ✅ All templates handle errors gracefully with clear messages

### **Error Handling:**
- ✅ Processing limit violations show clear context (which Oxi, what limit)
- ✅ Type mismatches show clear context (which pipeline step)
- ✅ JSON parsing errors include original data snippet
- ✅ Configuration errors point to specific YAML issues

### **Type Conversions:**
- ✅ Single JSON objects can be processed by array-expecting Oxis
- ✅ Text data can be converted to JSON when valid
- ✅ Format mismatches are caught early with helpful errors
- ✅ Memory usage can be estimated for processing limit checks

## 📅 Implementation Timeline

**Week 1 (Days 1-2)**: Oxi SDK foundation with ProcessingLimits trait
**Week 1 (Days 3-4)**: Batch Oxi implementation with multiple strategies
**Week 1 (Day 5)**: JSON Oxi implementation and validation operation

**Week 2 (Days 1-2)**: Query operations and streaming template fix
**Week 2 (Days 3-4)**: Type system enhancements and CSV format fixes with limits
**Week 2 (Day 5)**: Enhanced error handling and processing limit violations

**Week 3 (Days 1-2)**: Integration testing and template validation
**Week 3 (Days 3-4)**: Performance testing with large datasets and batch processing
**Week 3 (Day 5)**: Documentation and Oxi development guides

**Deliverables:**
- Robust Oxi SDK with processing limits and consistent patterns
- Working `batch` Oxi for flexible batch processing anywhere in pipelines
- Working `json` Oxi with validation, query, and extraction operations
- Enhanced type conversion system with memory estimation and batch awareness
- All 6 pipeline templates executing successfully
- Comprehensive test suite covering streaming vs batch processing
- Complete Oxi development documentation and templates

## 🔄 Batch Processing Flexibility Examples

### **Basic Streaming Pipeline:**
```yaml
pipeline:
  - name: read_file
    config: { path: "large_dataset.json" }
  - name: parse_json
  - name: json
    config: { operation: query, query: ".users[]" }
  - name: transform
  - name: write_file
    config: { path: "output.json" }
```

### **Same Pipeline with Batch Processing:**
```yaml
pipeline:
  - name: read_file
    config: { path: "large_dataset.json" }
  - name: parse_json
  - name: json
    config: { operation: query, query: ".users[]" }
  - name: batch                    # ← Insert batch Oxi anywhere!
    config:
      batch_size: 5000
      strategy: size_or_time
      flush_interval_ms: 10000
      max_memory_mb: 512
  - name: transform                # Now processes 5000 users at a time
  - name: write_file
    config: { path: "output.json" }
```

### **Multiple Batch Points:**
```yaml
pipeline:
  - name: read_file
  - name: parse_json
  - name: batch                    # Small batches for validation
    config: { batch_size: 100 }
  - name: json
    config: { operation: validate }
  - name: batch                    # Larger batches for transformation
    config: { batch_size: 10000 }
  - name: transform
  - name: write_file
```

This approach makes batch processing a first-class citizen that can be composed anywhere in the pipeline, following the Unix pipe philosophy while providing enterprise-scale processing capabilities.

# Pipeline Fix and Enhancement Plan

## 🎯 Project Overview

**Title**: Pipeline Template Fixes and Batch Processing Enhancement
**Purpose**: Fix failing pipeline templates and implement robust batch processing capabilities
**Scope**: Implement missing Oxis, fix data flow issues, and enhance batch processing architecture
**Success Criteria**: All 6 pipeline templates execute successfully with proper error handling and enhanced batch capabilities

## 📋 Current Issues Analysis

### ❌ **Template Failures:**

1. **Validation Pipeline** - Missing `json` Oxi for filtering operations
2. **Streaming Pipeline** - Missing `json` Oxi for data transformation
3. **API Pipeline** - Missing `json` Oxi + CSV formatter input type mismatch

### ⚠️ **Architecture Gaps:**

1. **Limited Oxi Ecosystem** - Only basic file I/O, missing core data processing
2. **Weak Batch Processing** - Current batch template lacks true batch capabilities
3. **Type System Issues** - Data type mismatches between Oxis
4. **Error Handling Gaps** - Insufficient error recovery in complex pipelines

## 🏗️ Architecture & Design

### **Enhanced Oxi Ecosystem:**
```
Data Processing Oxis:
├── json/              # JSON manipulation and filtering
├── transform/         # Data transformation utilities
├── validate/          # Data validation and schema checking
├── batch/             # Batch processing coordination
├── stream/            # Streaming data processing
└── format/            # Enhanced formatters with type flexibility
```

### **Batch Processing Architecture:**
```
Batch System Components:
├── BatchCoordinator   # Manages batch job lifecycle
├── ChunkProcessor     # Processes data in configurable chunks
├── ProgressTracker    # Monitors and reports batch progress
├── ErrorRecovery      # Handles partial failures and retries
└── ResourceManager    # Manages memory and processing resources
```

## 📈 Implementation Plan

### **Phase 1: Core Missing Oxis (Week 1)**

#### 1.1 JSON Oxi Implementation
```rust
// src/oxis/json/mod.rs
pub struct JsonOxi {
    config: JsonConfig,
}

#[derive(Debug, Deserialize)]
pub struct JsonConfig {
    pub operation: JsonOperation,
    pub filter_path: Option<String>,
    pub transform_rules: Option<Vec<TransformRule>>,
}

#[derive(Debug, Deserialize)]
pub enum JsonOperation {
    Filter,
    Transform,
    Validate,
    Extract,
}
```

**Configuration Schema:**
```yaml
json:
  operation: filter
  filter_path: "$.users[*].active"
  criteria:
    equals: true
```

#### 1.2 Enhanced Format Oxi
```rust
// src/oxis/format/mod.rs
pub struct FormatOxi {
    config: FormatConfig,
}

#[derive(Debug, Deserialize)]
pub struct FormatConfig {
    pub output_format: OutputFormat,
    pub input_handling: InputHandling,
}

#[derive(Debug, Deserialize)]
pub enum InputHandling {
    AutoDetect,
    ForceArray,
    ForceObject,
    WrapSingle,
}
```

### **Phase 2: Advanced Batch Processing (Week 2)**

#### 2.1 Batch Coordinator
```rust
// src/oxis/batch/coordinator.rs
pub struct BatchCoordinator {
    chunk_size: usize,
    max_concurrent: usize,
    retry_strategy: RetryStrategy,
    progress_tracker: ProgressTracker,
}

impl BatchCoordinator {
    pub async fn process_batch<T>(&self, items: Vec<T>) -> Result<BatchResult<T>, BatchError> {
        // Implement chunked processing with concurrency control
    }
}
```

#### 2.2 Enhanced Batch Template
```yaml
# templates/batch_advanced.yaml
name: "Advanced Batch Processing"
description: "Process large datasets with chunking, progress tracking, and error recovery"

pipeline:
  - type: batch_coordinator
    config:
      chunk_size: 1000
      max_concurrent: 4
      retry_attempts: 3
      progress_reporting: true

  - type: read
    config:
      input_pattern: "data/batch/*.json"

  - type: json
    config:
      operation: validate
      schema_path: "schemas/user.json"

  - type: transform
    config:
      operations:
        - flatten
        - normalize_dates

  - type: batch_writer
    config:
      output_pattern: "output/batch_{chunk_id}.csv"
      format: csv
      write_strategy: streaming
```

### **Phase 3: Data Processing Enhancements (Week 3)**

#### 3.1 Transform Oxi
```rust
// src/oxis/transform/mod.rs
pub struct TransformOxi {
    config: TransformConfig,
}

#[derive(Debug, Deserialize)]
pub struct TransformConfig {
    pub operations: Vec<TransformOperation>,
}

#[derive(Debug, Deserialize)]
pub enum TransformOperation {
    Flatten { separator: String },
    NormalizeDates { format: String },
    FilterFields { include: Vec<String> },
    RenameFields { mapping: HashMap<String, String> },
}
```

#### 3.2 Validation Oxi
```rust
// src/oxis/validate/mod.rs
pub struct ValidateOxi {
    config: ValidateConfig,
}

#[derive(Debug, Deserialize)]
pub struct ValidateConfig {
    pub schema_path: Option<String>,
    pub rules: Vec<ValidationRule>,
    pub on_error: ErrorAction,
}
```

### **Phase 4: Pipeline Integration & Testing (Week 4)**

#### 4.1 Updated Pipeline Templates

**Validation Pipeline Fix:**
```yaml
# templates/validation.yaml
pipeline:
  - type: read
    config:
      input: "data/input.json"

  - type: json
    config:
      operation: validate
      schema_path: "schemas/data.json"

  - type: validate
    config:
      rules:
        - field: "email"
          type: email
        - field: "age"
          type: integer
          min: 0
          max: 150

  - type: write
    config:
      output: "output/validated.json"
```

**Streaming Pipeline Fix:**
```yaml
# templates/streaming.yaml
pipeline:
  - type: stream_reader
    config:
      source: stdin
      buffer_size: 8192

  - type: json
    config:
      operation: transform
      streaming: true

  - type: transform
    config:
      operations:
        - flatten

  - type: stream_writer
    config:
      destination: stdout
      flush_interval: 100ms
```

**API Pipeline Fix:**
```yaml
# templates/api.yaml
pipeline:
  - type: read
    config:
      input: "data/users.json"

  - type: json
    config:
      operation: filter
      filter_path: "$.users[*]"

  - type: format
    config:
      output_format: csv
      input_handling: auto_detect  # Handle both arrays and objects

  - type: write
    config:
      output: "output/api_results.csv"
```

## 🧪 Technical Specifications

### **Data Type System Enhancement:**
```rust
// src/core/data_types.rs
#[derive(Debug, Clone)]
pub enum OxiData {
    Json(serde_json::Value),
    Array(Vec<serde_json::Value>),
    Stream(Box<dyn Iterator<Item = serde_json::Value>>),
    Batch(BatchData),
}

pub struct BatchData {
    pub chunks: Vec<Vec<serde_json::Value>>,
    pub metadata: BatchMetadata,
}
```

### **Error Handling Enhancement:**
```rust
// src/core/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum OxiError {
    #[error("JSON processing error: {0}")]
    JsonError(String),

    #[error("Batch processing error: {0}")]
    BatchError(String),

    #[error("Data type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Validation failed: {0}")]
    ValidationError(String),
}
```

## 📊 Enhanced Batch Processing Features

### **1. Chunked Processing:**
- Configurable chunk sizes for memory management
- Parallel processing of chunks with concurrency limits
- Progress tracking and ETA calculations

### **2. Advanced Error Recovery:**
```yaml
batch:
  error_handling:
    strategy: continue_on_error
    failed_chunk_retry: 3
    isolate_bad_records: true
    bad_record_output: "errors/failed_records.json"
```

### **3. Resource Management:**
```yaml
batch:
  resources:
    max_memory_mb: 512
    max_concurrent_chunks: 4
    temp_directory: "/tmp/oxide_batch"
    cleanup_on_completion: true
```

### **4. Progress and Monitoring:**
```yaml
batch:
  monitoring:
    progress_interval: 1000  # Report every 1000 records
    metrics_output: "metrics/batch_stats.json"
    log_level: info
```

## ✅ Testing Strategy

### **Unit Tests:**
- Individual Oxi functionality validation
- Data type conversion testing
- Error handling verification

### **Integration Tests:**
- Complete pipeline execution testing
- Template validation with sample data
- Performance testing with large datasets

### **Batch Processing Tests:**
- Large dataset processing (10K+ records)
- Memory usage validation
- Concurrent processing verification
- Error recovery testing

## 📚 Documentation Plan

### **User Documentation:**
- Updated CLI command reference
- Pipeline template usage guides
- Batch processing best practices
- Troubleshooting guide

### **Technical Documentation:**
- New Oxi implementation guides
- Data type system documentation
- Batch processing architecture
- Performance tuning guidelines

## 🎯 Success Metrics

### **Pipeline Success:**
- ✅ All 6 templates execute without errors
- ✅ Complex nested JSON processing works correctly
- ✅ Data type mismatches are handled gracefully
- ✅ Error messages are clear and actionable

### **Batch Processing:**
- ✅ Process 100K+ records efficiently
- ✅ Memory usage stays within configured limits
- ✅ Progress reporting provides accurate ETAs
- ✅ Error recovery handles partial failures

### **Performance Targets:**
- ✅ JSON processing: 10K records/second
- ✅ Batch chunking: Configurable 100-10K records/chunk
- ✅ Memory usage: <512MB for 100K record batches
- ✅ Error recovery: <5% performance impact

## 📅 Implementation Timeline

**Week 1**: Core missing Oxis (json, enhanced format)
**Week 2**: Batch processing architecture and coordinator
**Week 3**: Transform, validate, and stream processing Oxis
**Week 4**: Integration testing and documentation

**Deliverables:**
- Working implementations of all missing Oxis
- Enhanced batch processing capabilities
- Fixed pipeline templates with comprehensive testing
- Complete documentation and usage examples

This plan addresses all current pipeline failures while significantly enhancing the batch processing capabilities to handle enterprise-scale data processing workflows.

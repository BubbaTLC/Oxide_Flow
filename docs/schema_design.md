# Schema System Design for Oxide Flow

## 🎯 Overview

This document outlines the design for a comprehensive schema system that allows Oxis to understand and communicate data structure information throughout the pipeline.

## 🎨 Design Principles

1. **Schema as Metadata**: Schema travels alongside data, not embedded within it
2. **Optional but Powerful**: Schemas are optional - Oxis work without them, but better with them
3. **Automatic Inference**: Schemas can be automatically inferred from data when not provided
4. **Type Evolution**: Schemas can evolve through the pipeline (JSON → CSV changes structure)
5. **Validation Ready**: Schemas enable automatic validation between pipeline stages

## 🏗️ Core Schema Types

### OxiSchema
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OxiSchema {
    /// Field definitions keyed by field name
    pub fields: HashMap<String, FieldSchema>,
    /// Schema metadata and hints
    pub metadata: SchemaMetadata,
}

impl OxiSchema {
    /// Create a new empty schema
    pub fn empty() -> Self;

    /// Add a field to the schema
    pub fn add_field(&mut self, name: String, field: FieldSchema);

    /// Check if this schema is compatible with another
    pub fn is_compatible_with(&self, other: &OxiSchema) -> Result<(), SchemaError>;

    /// Merge two schemas (for batch processing)
    pub fn merge(&self, other: &OxiSchema) -> Result<OxiSchema, SchemaError>;

    /// Validate data against this schema
    pub fn validate_data(&self, data: &OxiData) -> Result<(), SchemaError>;
}
```

### FieldSchema
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldSchema {
    /// The data type of this field
    pub field_type: FieldType,
    /// Whether this field can be null/empty
    pub nullable: bool,
    /// Maximum size (for strings, arrays, etc.)
    pub max_size: Option<usize>,
    /// Field constraints and validation rules
    pub constraints: Vec<FieldConstraint>,
    /// Human-readable description
    pub description: Option<String>,
    /// Examples of valid values
    pub examples: Vec<serde_json::Value>,
}
```

### FieldType
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    // Primitive types
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Binary,

    // Complex types
    Array(Box<FieldType>),
    Object(HashMap<String, FieldSchema>),

    // Special types
    Unknown,    // For fields we can't determine the type
    Mixed,      // For fields that contain multiple types
}

impl FieldType {
    /// Check if a JSON value matches this field type
    pub fn matches_value(&self, value: &serde_json::Value) -> bool;

    /// Try to convert a value to match this type
    pub fn convert_value(&self, value: serde_json::Value) -> Result<serde_json::Value, ConversionError>;
}
```

### FieldConstraint
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldConstraint {
    // Numeric constraints
    MinValue(f64),
    MaxValue(f64),

    // String constraints
    MinLength(usize),
    MaxLength(usize),
    Pattern(String), // Regex pattern

    // Enum constraints
    OneOf(Vec<serde_json::Value>),

    // Custom validation
    Custom { name: String, rule: String },
}
```

## 🔄 Data Flow with Schema

### OxiDataWithSchema
```rust
#[derive(Debug, Clone)]
pub struct OxiDataWithSchema {
    /// The actual data
    pub data: OxiData,
    /// Optional schema describing the data structure
    pub schema: Option<OxiSchema>,
}

impl OxiDataWithSchema {
    /// Create data with schema
    pub fn new(data: OxiData, schema: OxiSchema) -> Self;

    /// Create data and infer schema automatically
    pub fn with_inferred_schema(data: OxiData) -> anyhow::Result<Self>;

    /// Validate data against its schema
    pub fn validate(&self) -> Result<(), SchemaError>;

    /// Get schema, inferring if not present
    pub fn get_or_infer_schema(&mut self) -> anyhow::Result<&OxiSchema>;
}
```

### Enhanced Oxi Trait (Clean Single Process Method)
```rust
#[async_trait]
pub trait Oxi {
    // Existing core methods...
    fn name(&self) -> &str;
    fn config_schema(&self) -> serde_yaml::Value;
    fn processing_limits(&self) -> ProcessingLimits;
    fn validate_input(&self, input: &OxiData) -> Result<(), OxiError>;

    /// Single process method that handles both data and optional schema
    async fn process(
        &self,
        input: OxiDataWithSchema,  // Always takes schema-aware data container
        config: &OxiConfig,
    ) -> anyhow::Result<OxiDataWithSchema>;

    /// Determine output schema given input schema and configuration
    /// This is used internally by the default process implementation
    fn output_schema(
        &self,
        input_schema: Option<&OxiSchema>,
        config: &OxiConfig
    ) -> anyhow::Result<OxiSchema> {
        // Default: pass through input schema or infer from data
        match input_schema {
            Some(schema) => Ok(schema.clone()),
            None => Ok(OxiSchema::empty())
        }
    }

    /// Core processing logic - implement this in your Oxi
    async fn process_data(
        &self,
        data: OxiData,
        config: &OxiConfig,
    ) -> anyhow::Result<OxiData>;
}

// Default implementation that handles schema automatically
#[async_trait]
impl<T: Oxi + ?Sized> Oxi for T {
    async fn process(
        &self,
        input: OxiDataWithSchema,
        config: &OxiConfig,
    ) -> anyhow::Result<OxiDataWithSchema> {
        // Validate input data if schema is present
        if let Some(schema) = &input.schema {
            schema.validate_data(&input.data)?;
        }

        // Process the actual data
        let output_data = self.process_data(input.data, config).await?;

        // Calculate output schema
        let output_schema = self.output_schema(input.schema.as_ref(), config)?;

        Ok(OxiDataWithSchema::new(output_data, output_schema))
    }
}
```

## 📋 Schema Communication Strategies

### 1. Automatic Inference (Primary)
```rust
// Data without schema - automatically inferred
let data = OxiData::Json(json!({
    "name": "John Doe",
    "age": 30,
    "active": true
}));

let schema = data.infer_schema()?;
// Results in:
// - name: String, nullable: false
// - age: Integer, nullable: false
// - active: Boolean, nullable: false
```

### 2. Explicit Schema Definition
```rust
// Manually defined schema for validation
let mut schema = OxiSchema::empty();
schema.add_field("name".to_string(), FieldSchema {
    field_type: FieldType::String,
    nullable: false,
    max_size: Some(100),
    constraints: vec![
        FieldConstraint::MinLength(1),
        FieldConstraint::Pattern(r"^[A-Za-z\s]+$".to_string())
    ],
    description: Some("Full name of the person".to_string()),
    examples: vec![json!("John Doe"), json!("Jane Smith")],
});
```

### 3. Schema Evolution Through Pipeline
```rust
// Input: JSON with complex structure
let input_schema = OxiSchema {
    fields: {
        "users": FieldSchema {
            field_type: FieldType::Array(Box::new(FieldType::Object(user_fields))),
            ...
        }
    }
};

// After JSON query operation: .users[]
let flattened_schema = json_oxi.output_schema(Some(&input_schema), &config)?;
// Results in individual user object schema

// After CSV conversion
let csv_schema = csv_oxi.output_schema(Some(&flattened_schema), &config)?;
// Results in text schema with CSV format metadata
```

## 🧪 Schema Inference Logic

### JSON Schema Inference
```rust
impl OxiData {
    pub fn infer_schema(&self) -> anyhow::Result<OxiSchema> {
        match self {
            OxiData::Json(value) => Self::infer_json_schema(value),
            OxiData::Text(text) => Self::infer_text_schema(text),
            OxiData::Binary(_) => Self::infer_binary_schema(),
            OxiData::Empty => Ok(OxiSchema::empty()),
        }
    }

    fn infer_json_schema(value: &serde_json::Value) -> anyhow::Result<OxiSchema> {
        match value {
            serde_json::Value::Object(obj) => {
                let mut fields = HashMap::new();
                for (key, val) in obj {
                    fields.insert(key.clone(), Self::infer_field_schema(val)?);
                }
                Ok(OxiSchema {
                    fields,
                    metadata: SchemaMetadata::default(),
                })
            }
            serde_json::Value::Array(arr) => {
                // For arrays, infer schema from first element or merge all
                if arr.is_empty() {
                    return Ok(OxiSchema::empty());
                }

                // Sample first few elements and merge schemas
                let mut merged_schema = Self::infer_json_schema(&arr[0])?;
                for item in arr.iter().take(10).skip(1) {
                    let item_schema = Self::infer_json_schema(item)?;
                    merged_schema = merged_schema.merge(&item_schema)?;
                }
                Ok(merged_schema)
            }
            _ => {
                // Single value - create schema with single field
                Ok(OxiSchema {
                    fields: {
                        let mut fields = HashMap::new();
                        fields.insert("value".to_string(), Self::infer_field_schema(value)?);
                        fields
                    },
                    metadata: SchemaMetadata::default(),
                })
            }
        }
    }
}
```

## 🔧 Oxi Implementation Examples (Clean Single Process)

### JSON Oxi with Schema (Simple Implementation)
```rust
impl Oxi for JsonOxi {
    async fn process_data(&self, data: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Focus only on data processing logic
        let json = data.to_json()?;
        let operation = self.get_operation(config)?;

        match operation {
            JsonOperation::Validate => {
                let schema_def = self.get_schema_definition(config)?;
                self.validate_against_schema(&json, &schema_def)?;
                Ok(OxiData::Json(json))
            }
            JsonOperation::Query => {
                let query = self.get_query(config)?;
                let result = self.apply_query(&json, &query)?;
                Ok(OxiData::Json(result))
            }
        }
    }

    fn output_schema(&self, input_schema: Option<&OxiSchema>, config: &OxiConfig) -> anyhow::Result<OxiSchema> {
        let operation = self.get_operation(config)?;

        match operation {
            JsonOperation::Validate => {
                // Validation doesn't change schema
                input_schema.cloned().unwrap_or_else(OxiSchema::empty)
            }
            JsonOperation::Query => {
                // Query operations transform schema
                let query = self.get_query(config)?;
                self.apply_query_to_schema(input_schema, &query)
            }
        }
    }
}
```

### CSV Oxi with Schema (Simple Implementation)
```rust
impl Oxi for CsvOxi {
    async fn process_data(&self, data: OxiData, config: &OxiConfig) -> anyhow::Result<OxiData> {
        // Focus only on CSV conversion logic
        let json_array = data.as_array()?;
        let csv_output = self.json_array_to_csv(&json_array)?;
        Ok(OxiData::Text(csv_output))
    }

    fn output_schema(&self, input_schema: Option<&OxiSchema>, _config: &OxiConfig) -> anyhow::Result<OxiSchema> {
        // CSV always produces text output
        let mut schema = OxiSchema::empty();
        schema.add_field("csv_data".to_string(), FieldSchema {
            field_type: FieldType::String,
            nullable: false,
            description: Some("CSV formatted data".to_string()),
            ..FieldSchema::default()
        });

        // Preserve input schema info in metadata
        if let Some(input) = input_schema {
            schema.metadata.add_note(format!("Converted from schema: {:?}", input));
        }

        Ok(schema)
    }
}
```

### Backward Compatibility Helper
```rust
// For existing code that doesn't use schemas yet
impl OxiDataWithSchema {
    /// Create from plain OxiData (schema will be inferred when needed)
    pub fn from_data(data: OxiData) -> Self {
        Self { data, schema: None }
    }

    /// Extract just the data
    pub fn into_data(self) -> OxiData {
        self.data
    }
}

// Pipeline automatically handles schema-aware data
let data = OxiData::Json(json!({"test": "data"}));
let schema_aware = OxiDataWithSchema::from_data(data);
let result = my_oxi.process(schema_aware, &config).await?;
let output_data = result.into_data();
```

## 🚀 Benefits

### 1. **Type Safety**
- Automatic validation between pipeline stages
- Early detection of type mismatches
- Clear error messages with field-level context

### 2. **Better Error Messages**
```rust
// Before
"Type mismatch: expected JSON, got Text"

// After
"Schema validation failed at field 'users[0].age': expected Integer, got String '30' (hint: use json operation to parse)"
```

### 3. **Automatic Conversions**
```rust
// Schema-aware conversion
if input_schema.fields["age"].field_type == FieldType::String
   && output_schema.fields["age"].field_type == FieldType::Integer {
    // Automatically convert "30" -> 30
}
```

### 4. **Documentation**
- Schemas serve as living documentation
- Examples and descriptions help users understand data structure
- Pipeline validation shows data flow transformations

## 📋 Implementation Plan

### Phase 1.5a: Core Schema Types
1. Add schema types to `src/types.rs`
2. Implement schema inference for JSON data
3. Add schema validation methods
4. Create schema serialization/deserialization

### Phase 1.5b: Enhanced OxiData
1. Create `OxiDataWithSchema` wrapper
2. Add schema inference methods to `OxiData`
3. Update existing Oxis to support optional schema
4. Add schema validation to pipeline execution

### Phase 1.5c: Oxi Trait Enhancement
1. Add `output_schema()` method to Oxi trait
2. Add optional `process_with_schema()` method
3. Update existing Oxis with schema awareness
4. Add schema compatibility validation

### Phase 1.5d: Testing & Documentation
1. Comprehensive test suite for schema inference
2. Schema validation tests
3. Pipeline schema evolution tests
4. Update documentation with schema examples

This design provides a robust foundation for schema awareness while maintaining backward compatibility with the existing SDK.

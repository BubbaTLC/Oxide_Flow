use crate::oxis::prelude::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Batch processing Oxi that can be inserted anywhere in a pipeline
/// Provides flexible batching strategies: size, time, memory, or combinations
pub struct Batch;

/// Configuration for batch processing
#[derive(Debug, Deserialize)]
pub struct BatchConfig {
    /// Maximum number of items in a batch before flushing
    pub batch_size: Option<usize>,
    /// Flush interval in milliseconds (for time-based batching)
    pub flush_interval_ms: Option<u64>,
    /// Maximum memory usage in MB before flushing
    pub max_memory_mb: Option<usize>,
    /// Batching strategy to use
    pub strategy: Option<BatchStrategy>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: Some(100),
            flush_interval_ms: None,
            max_memory_mb: None,
            strategy: Some(BatchStrategy::Size),
        }
    }
}

/// Different batching strategies
#[derive(Debug, Deserialize, Clone, Default)]
pub enum BatchStrategy {
    /// Flush when batch reaches size limit
    #[default]
    Size,
    /// Flush on time interval
    Time,
    /// Flush when either size or time condition is met
    SizeOrTime,
    /// Flush when memory limit is approached
    Memory,
    /// Flush when either size or memory condition is met
    SizeOrMemory,
    /// Flush when any condition is met (size, time, or memory)
    Any,
}

#[async_trait]
impl Oxi for Batch {
    fn name(&self) -> &str {
        "batch"
    }

    fn config_schema(&self) -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
            type: object
            properties:
              batch_size:
                type: integer
                description: "Maximum number of items in a batch before flushing"
                default: 100
                minimum: 1
              flush_interval_ms:
                type: integer
                description: "Flush interval in milliseconds (for time-based batching)"
                minimum: 1
              max_memory_mb:
                type: integer
                description: "Maximum memory usage in MB before flushing"
                minimum: 1
              strategy:
                type: string
                enum: ["Size", "Time", "SizeOrTime", "Memory", "SizeOrMemory", "Any"]
                description: "Batching strategy to use"
                default: "Size"
        "#,
        )
        .unwrap()
    }

    fn schema_strategy(&self) -> SchemaStrategy {
        SchemaStrategy::Passthrough
    }

    fn processing_limits(&self) -> ProcessingLimits {
        ProcessingLimits {
            max_batch_size: Some(10000), // Allow large batches but with reasonable limit
            max_memory_mb: Some(1024),   // 1GB default memory limit
            max_processing_time_ms: Some(300000), // 5 minute timeout
            supported_input_types: vec![
                OxiDataType::Json,
                OxiDataType::Text,
                OxiDataType::Binary,
                OxiDataType::Empty,
            ],
        }
    }

    async fn process(&self, input: OxiData, config: &OxiConfig) -> Result<OxiData, OxiError> {
        // Parse configuration using helper methods
        let batch_size = config.get_i64_or("batch_size", 100) as usize;
        let flush_interval_ms = config.get_i64_or("flush_interval_ms", 0);
        let flush_interval = if flush_interval_ms > 0 {
            Some(Duration::from_millis(flush_interval_ms as u64))
        } else {
            None
        };
        let max_memory_mb = config.get_i64_or("max_memory_mb", 256) as usize;
        let strategy_str = config.get_string_or("strategy", "Size");
        let strategy = match strategy_str.as_str() {
            "Time" => BatchStrategy::Time,
            "SizeOrTime" => BatchStrategy::SizeOrTime,
            "Memory" => BatchStrategy::Memory,
            "SizeOrMemory" => BatchStrategy::SizeOrMemory,
            "Any" => BatchStrategy::Any,
            _ => BatchStrategy::Size,
        };

        // Process the input data based on its type
        match input.data() {
            Data::Json(value) => {
                self.process_json(
                    value,
                    &strategy,
                    batch_size,
                    flush_interval,
                    max_memory_mb,
                    &input,
                )
                .await
            }
            Data::Text(text) => {
                self.process_text(
                    text,
                    &strategy,
                    batch_size,
                    flush_interval,
                    max_memory_mb,
                    &input,
                )
                .await
            }
            Data::Binary(data) => {
                self.process_binary(
                    data,
                    &strategy,
                    batch_size,
                    flush_interval,
                    max_memory_mb,
                    &input,
                )
                .await
            }
            Data::Empty => {
                // Empty data passes through unchanged
                Ok(input)
            }
        }
    }
}

impl Batch {
    /// Process JSON data with batching
    async fn process_json(
        &self,
        value: &serde_json::Value,
        strategy: &BatchStrategy,
        batch_size: usize,
        flush_interval: Option<Duration>,
        max_memory_mb: usize,
        input: &OxiData,
    ) -> Result<OxiData, OxiError> {
        match value {
            serde_json::Value::Array(items) => {
                // Process array in batches
                let batches = self
                    .create_batches(items, strategy, batch_size, flush_interval, max_memory_mb)
                    .await?;

                // Return batched array
                let batched_json = serde_json::Value::Array(
                    batches.into_iter().map(serde_json::Value::Array).collect(),
                );

                Ok(OxiData::with_schema(
                    Data::Json(batched_json),
                    input.schema().clone(),
                ))
            }
            _ => {
                // Single items are wrapped in a batch of size 1
                let batch = vec![value.clone()];
                let batched = serde_json::Value::Array(vec![serde_json::Value::Array(batch)]);

                Ok(OxiData::with_schema(
                    Data::Json(batched),
                    input.schema().clone(),
                ))
            }
        }
    }

    /// Process text data with batching
    async fn process_text(
        &self,
        text: &str,
        strategy: &BatchStrategy,
        batch_size: usize,
        flush_interval: Option<Duration>,
        max_memory_mb: usize,
        input: &OxiData,
    ) -> Result<OxiData, OxiError> {
        // Split text into lines and batch them
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();

        if lines.is_empty() {
            return Ok(input.clone());
        }

        let batches = self
            .create_text_batches(&lines, strategy, batch_size, flush_interval, max_memory_mb)
            .await?;

        // Join batches back into text with batch separators
        let batched_text = batches
            .into_iter()
            .map(|batch| batch.join("\n"))
            .collect::<Vec<String>>()
            .join("\n---BATCH---\n");

        Ok(OxiData::with_schema(
            Data::Text(batched_text),
            input.schema().clone(),
        ))
    }

    /// Process binary data with batching
    async fn process_binary(
        &self,
        data: &[u8],
        strategy: &BatchStrategy,
        batch_size: usize,
        flush_interval: Option<Duration>,
        max_memory_mb: usize,
        input: &OxiData,
    ) -> Result<OxiData, OxiError> {
        // For binary data, we batch by byte chunks
        let chunk_size = batch_size * 1024; // Convert batch_size to KB
        let max_memory_bytes = max_memory_mb * 1024 * 1024;

        let batches = self
            .create_binary_batches(data, strategy, chunk_size, flush_interval, max_memory_bytes)
            .await?;

        // Concatenate all batches back together
        let batched_data: Vec<u8> = batches.into_iter().flatten().collect();

        Ok(OxiData::with_schema(
            Data::Binary(batched_data),
            input.schema().clone(),
        ))
    }

    /// Create batches from JSON array items
    async fn create_batches(
        &self,
        items: &[serde_json::Value],
        strategy: &BatchStrategy,
        batch_size: usize,
        flush_interval: Option<Duration>,
        max_memory_mb: usize,
    ) -> Result<Vec<Vec<serde_json::Value>>, OxiError> {
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();
        let mut current_memory_estimate = 0usize;
        let max_memory_bytes = max_memory_mb * 1024 * 1024;
        let mut last_flush = Instant::now();

        for item in items {
            // Estimate memory usage of this item
            let item_memory = self.estimate_json_memory(item);

            // Check if we should flush based on strategy
            let should_flush = self.should_flush(
                strategy,
                current_batch.len(),
                batch_size,
                current_memory_estimate + item_memory,
                max_memory_bytes,
                last_flush,
                flush_interval,
            );

            if should_flush && !current_batch.is_empty() {
                batches.push(current_batch.clone());
                current_batch.clear();
                current_memory_estimate = 0;
                last_flush = Instant::now();

                // Add small delay for time-based batching
                if let Some(_interval) = flush_interval {
                    if matches!(
                        strategy,
                        BatchStrategy::Time | BatchStrategy::SizeOrTime | BatchStrategy::Any
                    ) {
                        sleep(Duration::from_millis(1)).await;
                    }
                }
            }

            current_batch.push(item.clone());
            current_memory_estimate += item_memory;
        }

        // Add remaining items as final batch
        if !current_batch.is_empty() {
            batches.push(current_batch);
        }

        Ok(batches)
    }

    /// Create batches from text lines
    async fn create_text_batches(
        &self,
        lines: &[String],
        strategy: &BatchStrategy,
        batch_size: usize,
        flush_interval: Option<Duration>,
        max_memory_mb: usize,
    ) -> Result<Vec<Vec<String>>, OxiError> {
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();
        let mut current_memory_estimate = 0usize;
        let max_memory_bytes = max_memory_mb * 1024 * 1024;
        let mut last_flush = Instant::now();

        for line in lines {
            let line_memory = line.len();

            let should_flush = self.should_flush(
                strategy,
                current_batch.len(),
                batch_size,
                current_memory_estimate + line_memory,
                max_memory_bytes,
                last_flush,
                flush_interval,
            );

            if should_flush && !current_batch.is_empty() {
                batches.push(current_batch.clone());
                current_batch.clear();
                current_memory_estimate = 0;
                last_flush = Instant::now();

                if let Some(_interval) = flush_interval {
                    if matches!(
                        strategy,
                        BatchStrategy::Time | BatchStrategy::SizeOrTime | BatchStrategy::Any
                    ) {
                        sleep(Duration::from_millis(1)).await;
                    }
                }
            }

            current_batch.push(line.clone());
            current_memory_estimate += line_memory;
        }

        if !current_batch.is_empty() {
            batches.push(current_batch);
        }

        Ok(batches)
    }

    /// Create batches from binary data
    async fn create_binary_batches(
        &self,
        data: &[u8],
        strategy: &BatchStrategy,
        chunk_size: usize,
        flush_interval: Option<Duration>,
        max_memory_bytes: usize,
    ) -> Result<Vec<Vec<u8>>, OxiError> {
        let mut batches = Vec::new();
        let mut current_pos = 0;
        let mut last_flush = Instant::now();

        while current_pos < data.len() {
            let remaining = data.len() - current_pos;
            let mut chunk_end = current_pos + chunk_size.min(remaining);

            // For memory-based strategies, adjust chunk size
            if matches!(
                strategy,
                BatchStrategy::Memory | BatchStrategy::SizeOrMemory | BatchStrategy::Any
            ) {
                chunk_end = current_pos + max_memory_bytes.min(remaining);
            }

            // For time-based strategies, check if we should flush
            if let Some(interval) = flush_interval {
                if matches!(
                    strategy,
                    BatchStrategy::Time | BatchStrategy::SizeOrTime | BatchStrategy::Any
                ) && last_flush.elapsed() >= interval
                {
                    // Force smaller chunk for time-based flush
                    chunk_end = current_pos + (chunk_size / 2).max(1).min(remaining);
                    last_flush = Instant::now();
                }
            }

            let chunk = data[current_pos..chunk_end].to_vec();
            batches.push(chunk);
            current_pos = chunk_end;

            // Add delay for time-based batching
            if let Some(_interval) = flush_interval {
                if matches!(
                    strategy,
                    BatchStrategy::Time | BatchStrategy::SizeOrTime | BatchStrategy::Any
                ) {
                    sleep(Duration::from_millis(1)).await;
                }
            }
        }

        Ok(batches)
    }

    /// Determine if we should flush the current batch
    #[allow(clippy::too_many_arguments)]
    fn should_flush(
        &self,
        strategy: &BatchStrategy,
        current_count: usize,
        max_count: usize,
        current_memory: usize,
        max_memory: usize,
        last_flush: Instant,
        flush_interval: Option<Duration>,
    ) -> bool {
        match strategy {
            BatchStrategy::Size => current_count >= max_count,
            BatchStrategy::Time => {
                if let Some(interval) = flush_interval {
                    last_flush.elapsed() >= interval
                } else {
                    false
                }
            }
            BatchStrategy::SizeOrTime => {
                let size_condition = current_count >= max_count;
                let time_condition = if let Some(interval) = flush_interval {
                    last_flush.elapsed() >= interval
                } else {
                    false
                };
                size_condition || time_condition
            }
            BatchStrategy::Memory => current_memory >= max_memory,
            BatchStrategy::SizeOrMemory => {
                current_count >= max_count || current_memory >= max_memory
            }
            BatchStrategy::Any => {
                let size_condition = current_count >= max_count;
                let memory_condition = current_memory >= max_memory;
                let time_condition = if let Some(interval) = flush_interval {
                    last_flush.elapsed() >= interval
                } else {
                    false
                };
                size_condition || memory_condition || time_condition
            }
        }
    }

    /// Estimate memory usage of a JSON value
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_json_memory(&self, value: &serde_json::Value) -> usize {
        match value {
            serde_json::Value::Null => 4,
            serde_json::Value::Bool(_) => 1,
            serde_json::Value::Number(_) => 8,
            serde_json::Value::String(s) => s.len(),
            serde_json::Value::Array(arr) => {
                arr.iter()
                    .map(|v| self.estimate_json_memory(v))
                    .sum::<usize>()
                    + arr.len() * 8
            }
            serde_json::Value::Object(obj) => {
                obj.iter()
                    .map(|(k, v)| k.len() + self.estimate_json_memory(v))
                    .sum::<usize>()
                    + obj.len() * 16
            }
        }
    }
}

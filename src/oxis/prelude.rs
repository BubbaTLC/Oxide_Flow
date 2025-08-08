// Core Oxi SDK prelude - everything an Oxi needs to implement
pub use super::*;
pub use crate::error::OxiError;
pub use crate::types::{Data, OxiConfig, OxiData, SchemaStrategy};
pub use crate::Oxi;
pub use anyhow::Result;
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};

// Export the new processing limits system
pub use crate::types::{OxiDataType, ProcessingLimits};

// Common imports that most Oxis will need
pub use std::collections::HashMap;
pub use std::time::{Duration, Instant};

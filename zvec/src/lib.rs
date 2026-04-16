//! Safe Rust bindings for the zvec vector database.
//!
//! Zvec is an open-source, in-process vector database — lightweight, lightning-fast,
//! and designed to embed directly into applications. This Rust SDK wraps the zvec
//! C-API to provide safe, idiomatic Rust access to all zvec functionality.
//!
//! ## Quick Start
//!
//! ```no_run
//! use zvec::*;
//!
//! fn main() -> zvec::Result<()> {
//!     initialize(None)?;
//!
//!     let schema = CollectionSchema::builder("example")
//!         .add_field(FieldSchema::new("id", DataType::String, false, 0))
//!         .add_vector_field("embedding", DataType::VectorFp32, 4,
//!             IndexParams::hnsw(MetricType::Cosine, 16, 200))
//!         .build()?;
//!
//!     let collection = Collection::create_and_open("./data", &schema, None)?;
//!
//!     let mut doc = Doc::new()?;
//!     doc.set_pk("doc1");
//!     doc.add_string("id", "doc1")?;
//!     doc.add_vector_f32("embedding", &[0.1, 0.2, 0.3, 0.4])?;
//!     collection.insert(&[&doc])?;
//!
//!     let query = VectorQuery::new("embedding", &[0.4, 0.3, 0.3, 0.1], 10)?;
//!     let results = collection.query(&query)?;
//!     for result in &results {
//!         println!("PK={} Score={:.4}", result.get_pk().unwrap_or(""), result.get_score());
//!     }
//!
//!     shutdown()?;
//!     Ok(())
//! }
//! ```

pub mod collection;
pub mod config;
pub mod doc;
pub mod error;
pub mod query;
pub mod schema;
pub mod types;

pub use collection::{Collection, CollectionOptions, CollectionStats, WriteResult};
pub use config::{initialize, is_initialized, shutdown, version, ConfigData, ConfigDataBuilder};
pub use doc::Doc;
pub use error::{Error, ErrorCode, Result};
pub use query::{
    FlatQueryParams, GroupByVectorQuery, HnswQueryParams, IvfQueryParams, VectorQuery,
};
pub use schema::{CollectionSchema, FieldSchema, IndexParams};
pub use types::{DataType, DocOperator, IndexType, LogLevel, MetricType, QuantizeType};

/// Re-export the raw FFI bindings for advanced users.
pub use zvec_sys as sys;

/// Convenience prelude that re-exports the most commonly used types.
///
/// ```ignore
/// use zvec::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        initialize, is_initialized, shutdown, version, Collection, CollectionOptions,
        CollectionSchema, CollectionStats, ConfigData, ConfigDataBuilder, DataType, Doc, Error,
        ErrorCode, FieldSchema, IndexParams, MetricType, QuantizeType, Result, VectorQuery,
        WriteResult,
    };
}

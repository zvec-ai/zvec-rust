use std::ffi::CStr;

use crate::error::{check_error, to_cstring, Error, ErrorCode, Result};
use crate::types::{DataType, IndexType, MetricType, QuantizeType};

/// Parameters for configuring an index on a field.
pub struct IndexParams {
    pub(crate) handle: *mut zvec_sys::zvec_index_params_t,
    owned: bool,
}

impl IndexParams {
    /// Returns the raw FFI handle.
    ///
    /// # Safety
    /// The caller must not use the handle after the `IndexParams` is dropped.
    pub unsafe fn as_raw(&self) -> *mut zvec_sys::zvec_index_params_t {
        self.handle
    }

    /// Creates HNSW index parameters.
    pub fn hnsw(metric: MetricType, m: i32, ef_construction: i32) -> Self {
        unsafe {
            let handle = zvec_sys::zvec_index_params_create(IndexType::Hnsw as u32);
            zvec_sys::zvec_index_params_set_metric_type(handle, metric as u32);
            zvec_sys::zvec_index_params_set_hnsw_params(handle, m, ef_construction);
            IndexParams {
                handle,
                owned: true,
            }
        }
    }

    /// Creates HNSW index parameters with quantization.
    pub fn hnsw_with_quantize(
        metric: MetricType,
        m: i32,
        ef_construction: i32,
        quantize: QuantizeType,
    ) -> Self {
        unsafe {
            let handle = zvec_sys::zvec_index_params_create(IndexType::Hnsw as u32);
            zvec_sys::zvec_index_params_set_metric_type(handle, metric as u32);
            zvec_sys::zvec_index_params_set_hnsw_params(handle, m, ef_construction);
            zvec_sys::zvec_index_params_set_quantize_type(handle, quantize as u32);
            IndexParams {
                handle,
                owned: true,
            }
        }
    }

    /// Creates IVF index parameters.
    pub fn ivf(metric: MetricType, n_list: i32, n_iters: i32, use_soar: bool) -> Self {
        unsafe {
            let handle = zvec_sys::zvec_index_params_create(IndexType::Ivf as u32);
            zvec_sys::zvec_index_params_set_metric_type(handle, metric as u32);
            zvec_sys::zvec_index_params_set_ivf_params(handle, n_list, n_iters, use_soar);
            IndexParams {
                handle,
                owned: true,
            }
        }
    }

    /// Creates Flat index parameters.
    pub fn flat(metric: MetricType) -> Self {
        unsafe {
            let handle = zvec_sys::zvec_index_params_create(IndexType::Flat as u32);
            zvec_sys::zvec_index_params_set_metric_type(handle, metric as u32);
            IndexParams {
                handle,
                owned: true,
            }
        }
    }

    /// Creates inverted index parameters for scalar fields.
    pub fn invert(enable_range_opt: bool, enable_wildcard: bool) -> Self {
        unsafe {
            let handle = zvec_sys::zvec_index_params_create(IndexType::Invert as u32);
            zvec_sys::zvec_index_params_set_invert_params(
                handle,
                enable_range_opt,
                enable_wildcard,
            );
            IndexParams {
                handle,
                owned: true,
            }
        }
    }

    /// Returns the index type.
    pub fn index_type(&self) -> IndexType {
        IndexType::from(unsafe { zvec_sys::zvec_index_params_get_type(self.handle) })
    }

    /// Returns the metric type.
    pub fn metric_type(&self) -> MetricType {
        MetricType::from(unsafe { zvec_sys::zvec_index_params_get_metric_type(self.handle) })
    }

    /// Returns the quantize type.
    pub fn quantize_type(&self) -> QuantizeType {
        QuantizeType::from(unsafe { zvec_sys::zvec_index_params_get_quantize_type(self.handle) })
    }

    /// Sets the metric type.
    pub fn set_metric_type(&mut self, metric: MetricType) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_index_params_set_metric_type(self.handle, metric as u32)
        })
    }

    /// Sets the quantize type.
    pub fn set_quantize_type(&mut self, quantize: QuantizeType) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_index_params_set_quantize_type(self.handle, quantize as u32)
        })
    }
}

impl Drop for IndexParams {
    fn drop(&mut self) {
        if self.owned && !self.handle.is_null() {
            unsafe { zvec_sys::zvec_index_params_destroy(self.handle) };
        }
    }
}

/// Schema definition for a single field in a collection.
pub struct FieldSchema {
    pub(crate) handle: *mut zvec_sys::zvec_field_schema_t,
    owned: bool,
}

impl FieldSchema {
    /// Creates a new field schema.
    ///
    /// - `name`: Field name
    /// - `data_type`: Data type of the field
    /// - `nullable`: Whether the field can be null
    /// - `dimension`: Vector dimension (0 for non-vector fields)
    pub fn new(name: &str, data_type: DataType, nullable: bool, dimension: u32) -> Self {
        let c_name = to_cstring(name).expect("field name must not contain null bytes");
        let handle = unsafe {
            zvec_sys::zvec_field_schema_create(
                c_name.as_ptr(),
                data_type as u32,
                nullable,
                dimension,
            )
        };
        FieldSchema {
            handle,
            owned: true,
        }
    }

    /// Creates a new field schema, returning an error if the name contains null bytes.
    ///
    /// This is the fallible alternative to [`FieldSchema::new`].
    pub fn try_new(
        name: &str,
        data_type: DataType,
        nullable: bool,
        dimension: u32,
    ) -> Result<Self> {
        let c_name = to_cstring(name)?;
        let handle = unsafe {
            zvec_sys::zvec_field_schema_create(
                c_name.as_ptr(),
                data_type as u32,
                nullable,
                dimension,
            )
        };
        if handle.is_null() {
            return Err(Error {
                code: ErrorCode::InternalError,
                message: "failed to create field schema".into(),
            });
        }
        Ok(FieldSchema {
            handle,
            owned: true,
        })
    }

    /// Creates a non-owning wrapper around an existing handle.
    #[allow(dead_code)]
    pub(crate) fn from_borrowed(handle: *mut zvec_sys::zvec_field_schema_t) -> Self {
        FieldSchema {
            handle,
            owned: false,
        }
    }

    /// Sets the index parameters for this field.
    pub fn set_index_params(&mut self, params: &IndexParams) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_field_schema_set_index_params(self.handle, params.handle)
        })
    }

    /// Returns the field name.
    pub fn name(&self) -> &str {
        unsafe {
            let ptr = zvec_sys::zvec_field_schema_get_name(self.handle);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Returns the data type.
    pub fn data_type(&self) -> DataType {
        DataType::from(unsafe { zvec_sys::zvec_field_schema_get_data_type(self.handle) })
    }

    /// Returns the dimension (for vector fields).
    pub fn dimension(&self) -> u32 {
        unsafe { zvec_sys::zvec_field_schema_get_dimension(self.handle) }
    }

    /// Returns whether the field is nullable.
    pub fn is_nullable(&self) -> bool {
        unsafe { zvec_sys::zvec_field_schema_is_nullable(self.handle) }
    }

    /// Returns whether this is a vector field.
    pub fn is_vector_field(&self) -> bool {
        unsafe { zvec_sys::zvec_field_schema_is_vector_field(self.handle) }
    }

    /// Returns whether this is a dense vector field.
    pub fn is_dense_vector(&self) -> bool {
        unsafe { zvec_sys::zvec_field_schema_is_dense_vector(self.handle) }
    }

    /// Returns whether this is a sparse vector field.
    pub fn is_sparse_vector(&self) -> bool {
        unsafe { zvec_sys::zvec_field_schema_is_sparse_vector(self.handle) }
    }

    /// Returns whether this field has an index.
    pub fn has_index(&self) -> bool {
        unsafe { zvec_sys::zvec_field_schema_has_index(self.handle) }
    }

    /// Returns the index type.
    pub fn index_type(&self) -> IndexType {
        IndexType::from(unsafe { zvec_sys::zvec_field_schema_get_index_type(self.handle) })
    }

    /// Returns whether this is an array type.
    pub fn is_array_type(&self) -> bool {
        unsafe { zvec_sys::zvec_field_schema_is_array_type(self.handle) }
    }
}

impl Drop for FieldSchema {
    fn drop(&mut self) {
        if self.owned && !self.handle.is_null() {
            unsafe { zvec_sys::zvec_field_schema_destroy(self.handle) };
        }
    }
}

/// Schema definition for a collection, containing field definitions.
pub struct CollectionSchema {
    pub(crate) handle: *mut zvec_sys::zvec_collection_schema_t,
    owned: bool,
}

impl CollectionSchema {
    /// Creates a new collection schema with the given name.
    pub fn new(name: &str) -> Result<Self> {
        let c_name = to_cstring(name)?;
        let handle = unsafe { zvec_sys::zvec_collection_schema_create(c_name.as_ptr()) };
        if handle.is_null() {
            return Err(Error {
                code: ErrorCode::InternalError,
                message: "failed to create collection schema".into(),
            });
        }
        Ok(CollectionSchema {
            handle,
            owned: true,
        })
    }

    /// Returns a builder for constructing a collection schema.
    pub fn builder(name: &str) -> CollectionSchemaBuilder {
        CollectionSchemaBuilder::new(name)
    }

    /// Creates a non-owning wrapper around an existing handle.
    pub(crate) fn from_owned(handle: *mut zvec_sys::zvec_collection_schema_t) -> Self {
        CollectionSchema {
            handle,
            owned: true,
        }
    }

    /// Adds a field to the schema.
    pub fn add_field(&mut self, field: &FieldSchema) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_collection_schema_add_field(self.handle, field.handle)
        })
    }

    /// Returns the collection name.
    pub fn name(&self) -> &str {
        unsafe {
            let ptr = zvec_sys::zvec_collection_schema_get_name(self.handle);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Checks if a field exists in the schema.
    pub fn has_field(&self, name: &str) -> bool {
        let c_name = match to_cstring(name) {
            Ok(s) => s,
            Err(_) => return false,
        };
        unsafe { zvec_sys::zvec_collection_schema_has_field(self.handle, c_name.as_ptr()) }
    }

    /// Checks if a field has an index.
    pub fn has_index(&self, field_name: &str) -> bool {
        let c_name = match to_cstring(field_name) {
            Ok(s) => s,
            Err(_) => return false,
        };
        unsafe { zvec_sys::zvec_collection_schema_has_index(self.handle, c_name.as_ptr()) }
    }

    /// Drops a field from the schema.
    pub fn drop_field(&mut self, name: &str) -> Result<()> {
        let c_name = to_cstring(name)?;
        check_error(unsafe {
            zvec_sys::zvec_collection_schema_drop_field(self.handle, c_name.as_ptr())
        })
    }

    /// Adds an index to a field.
    pub fn add_index(&mut self, field_name: &str, params: &IndexParams) -> Result<()> {
        let c_name = to_cstring(field_name)?;
        check_error(unsafe {
            zvec_sys::zvec_collection_schema_add_index(self.handle, c_name.as_ptr(), params.handle)
        })
    }

    /// Drops an index from a field.
    pub fn drop_index(&mut self, field_name: &str) -> Result<()> {
        let c_name = to_cstring(field_name)?;
        check_error(unsafe {
            zvec_sys::zvec_collection_schema_drop_index(self.handle, c_name.as_ptr())
        })
    }

    /// Sets the maximum document count per segment.
    pub fn set_max_doc_count_per_segment(&mut self, count: u64) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_collection_schema_set_max_doc_count_per_segment(self.handle, count)
        })
    }

    /// Returns the maximum document count per segment.
    pub fn max_doc_count_per_segment(&self) -> u64 {
        unsafe { zvec_sys::zvec_collection_schema_get_max_doc_count_per_segment(self.handle) }
    }
}

impl Drop for CollectionSchema {
    fn drop(&mut self) {
        if self.owned && !self.handle.is_null() {
            unsafe { zvec_sys::zvec_collection_schema_destroy(self.handle) };
        }
    }
}

/// Builder for constructing a [`CollectionSchema`] with a fluent API.
pub struct CollectionSchemaBuilder {
    name: String,
    fields: Vec<(FieldSchema, Option<IndexParams>)>,
    max_doc_count_per_segment: Option<u64>,
}

impl CollectionSchemaBuilder {
    /// Creates a new builder with the given collection name.
    pub fn new(name: &str) -> Self {
        CollectionSchemaBuilder {
            name: name.to_string(),
            fields: Vec::new(),
            max_doc_count_per_segment: None,
        }
    }

    /// Adds a field to the schema.
    pub fn add_field(mut self, field: FieldSchema) -> Self {
        self.fields.push((field, None));
        self
    }

    /// Adds a vector field with index parameters.
    pub fn add_vector_field(
        mut self,
        name: &str,
        data_type: DataType,
        dimension: u32,
        index_params: IndexParams,
    ) -> Self {
        let field = FieldSchema::new(name, data_type, false, dimension);
        self.fields.push((field, Some(index_params)));
        self
    }

    /// Adds a scalar field with an inverted index.
    pub fn add_indexed_field(
        mut self,
        name: &str,
        data_type: DataType,
        index_params: IndexParams,
    ) -> Self {
        let field = FieldSchema::new(name, data_type, false, 0);
        self.fields.push((field, Some(index_params)));
        self
    }

    /// Sets the maximum document count per segment.
    pub fn max_doc_count_per_segment(mut self, count: u64) -> Self {
        self.max_doc_count_per_segment = Some(count);
        self
    }

    /// Builds the collection schema.
    pub fn build(self) -> Result<CollectionSchema> {
        let mut schema = CollectionSchema::new(&self.name)?;

        for (mut field, index_params) in self.fields {
            if let Some(params) = &index_params {
                field.set_index_params(params)?;
            }
            schema.add_field(&field)?;
        }

        if let Some(count) = self.max_doc_count_per_segment {
            schema.set_max_doc_count_per_segment(count)?;
        }

        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MetricType;

    #[test]
    fn test_collection_schema_builder_default_values() {
        let builder = CollectionSchemaBuilder::new("test_collection");
        assert_eq!(builder.name, "test_collection");
        assert!(builder.fields.is_empty());
        assert!(builder.max_doc_count_per_segment.is_none());
    }

    #[test]
    fn test_collection_schema_builder_add_field() {
        let field = FieldSchema::new("test_field", DataType::Int32, false, 0);
        let builder = CollectionSchemaBuilder::new("test_collection").add_field(field);

        assert_eq!(builder.fields.len(), 1);
        assert_eq!(builder.fields[0].0.name(), "test_field");
        assert!(builder.fields[0].1.is_none());
    }

    #[test]
    fn test_collection_schema_builder_add_vector_field() {
        let index_params = IndexParams::hnsw(MetricType::L2, 16, 200);
        let builder = CollectionSchemaBuilder::new("test_collection").add_vector_field(
            "vector",
            DataType::VectorFp32,
            128,
            index_params,
        );

        assert_eq!(builder.fields.len(), 1);
        assert_eq!(builder.fields[0].0.name(), "vector");
        assert!(builder.fields[0].1.is_some());
    }

    #[test]
    fn test_collection_schema_builder_add_indexed_field() {
        let index_params = IndexParams::invert(true, false);
        let builder = CollectionSchemaBuilder::new("test_collection").add_indexed_field(
            "age",
            DataType::Int32,
            index_params,
        );

        assert_eq!(builder.fields.len(), 1);
        assert_eq!(builder.fields[0].0.name(), "age");
        assert!(builder.fields[0].1.is_some());
    }

    #[test]
    fn test_collection_schema_builder_max_doc_count_per_segment() {
        let builder =
            CollectionSchemaBuilder::new("test_collection").max_doc_count_per_segment(1000);

        assert_eq!(builder.max_doc_count_per_segment, Some(1000));
    }

    #[test]
    fn test_collection_schema_builder_multiple_fields() {
        let field1 = FieldSchema::new("id", DataType::Int64, false, 0);
        let field2 = FieldSchema::new("name", DataType::String, false, 0);
        let index_params = IndexParams::hnsw(MetricType::L2, 16, 200);

        let builder = CollectionSchemaBuilder::new("test_collection")
            .add_field(field1)
            .add_field(field2)
            .add_vector_field("vector", DataType::VectorFp32, 128, index_params);

        assert_eq!(builder.fields.len(), 3);
        assert_eq!(builder.fields[0].0.name(), "id");
        assert_eq!(builder.fields[1].0.name(), "name");
        assert_eq!(builder.fields[2].0.name(), "vector");
    }

    #[test]
    fn test_index_params_hnsw() {
        let params = IndexParams::hnsw(MetricType::L2, 16, 200);
        assert_eq!(params.index_type(), IndexType::Hnsw);
        assert_eq!(params.metric_type(), MetricType::L2);
    }

    #[test]
    fn test_index_params_hnsw_with_quantize() {
        let params = IndexParams::hnsw_with_quantize(MetricType::L2, 16, 200, QuantizeType::Int8);
        assert_eq!(params.index_type(), IndexType::Hnsw);
        assert_eq!(params.metric_type(), MetricType::L2);
        assert_eq!(params.quantize_type(), QuantizeType::Int8);
    }

    #[test]
    fn test_index_params_ivf() {
        let params = IndexParams::ivf(MetricType::Ip, 100, 10, true);
        assert_eq!(params.index_type(), IndexType::Ivf);
        assert_eq!(params.metric_type(), MetricType::Ip);
    }

    #[test]
    fn test_index_params_flat() {
        let params = IndexParams::flat(MetricType::Cosine);
        assert_eq!(params.index_type(), IndexType::Flat);
        assert_eq!(params.metric_type(), MetricType::Cosine);
    }

    #[test]
    fn test_index_params_invert() {
        let params = IndexParams::invert(true, true);
        assert_eq!(params.index_type(), IndexType::Invert);
    }

    #[test]
    fn test_field_schema_properties() {
        let field = FieldSchema::new("test_field", DataType::VectorFp32, true, 128);
        assert_eq!(field.name(), "test_field");
        assert_eq!(field.data_type(), DataType::VectorFp32);
        assert_eq!(field.dimension(), 128);
        assert!(field.is_nullable());
    }

    #[test]
    fn test_collection_schema_builder_method() {
        let builder = CollectionSchema::builder("test_collection");
        assert_eq!(builder.name, "test_collection");
        assert!(builder.fields.is_empty());
    }
}

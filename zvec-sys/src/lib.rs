//! Raw FFI bindings to the zvec C-API.
//!
//! This crate provides low-level Rust bindings to the zvec vector database C library.
//! It exposes opaque pointer types, constants, and function declarations that mirror
//! the C API exactly. For a safer, more idiomatic Rust interface, consider using the
//! higher-level `zvec` crate which wraps these raw bindings.
//!
//! # Safety
//!
//! All functions in this crate are `unsafe` by nature as they interact with C code.
//! Proper memory management and lifetime handling is the responsibility of the caller.

#![allow(non_camel_case_types, non_upper_case_globals)]

use std::os::raw::{c_char, c_int, c_void};

// =============================================================================
// Opaque Pointer Types
// =============================================================================

/// Opaque pointer to a vector collection instance.
/// Represents an opened collection that can be used for read/write operations.
#[repr(C)]
pub struct zvec_collection_t {
    _private: [u8; 0],
}

/// Opaque pointer to a collection schema definition.
/// Defines the structure of fields and indexes in a collection.
#[repr(C)]
pub struct zvec_collection_schema_t {
    _private: [u8; 0],
}

/// Opaque pointer to a field schema definition.
/// Defines the properties of a single field including type, index, etc.
#[repr(C)]
pub struct zvec_field_schema_t {
    _private: [u8; 0],
}

/// Opaque pointer to index configuration parameters.
/// Contains settings for HNSW, IVF, or other index types.
#[repr(C)]
pub struct zvec_index_params_t {
    _private: [u8; 0],
}

/// Opaque pointer to a document instance.
/// Represents a single record with fields and values in a collection.
#[repr(C)]
pub struct zvec_doc_t {
    _private: [u8; 0],
}

/// Opaque pointer to a vector query object.
/// Configures search parameters for vector similarity queries.
#[repr(C)]
pub struct zvec_vector_query_t {
    _private: [u8; 0],
}

/// Opaque pointer to a group-by vector query object.
/// Configures grouped vector search with aggregation parameters.
#[repr(C)]
pub struct zvec_group_by_vector_query_t {
    _private: [u8; 0],
}

/// Opaque pointer to collection opening options.
/// Controls behavior like MMAP, buffer size, and read-only mode.
#[repr(C)]
pub struct zvec_collection_options_t {
    _private: [u8; 0],
}

/// Opaque pointer to collection statistics data.
/// Contains metrics like document count and index completeness.
#[repr(C)]
pub struct zvec_collection_stats_t {
    _private: [u8; 0],
}

/// Opaque pointer to global configuration data.
/// Manages memory limits, thread counts, and other system-wide settings.
#[repr(C)]
pub struct zvec_config_data_t {
    _private: [u8; 0],
}

/// Opaque pointer to logging configuration.
/// Controls log level, output type (console/file), and file rotation settings.
#[repr(C)]
pub struct zvec_log_config_t {
    _private: [u8; 0],
}

/// Opaque pointer to HNSW index query parameters.
/// Configures ef, radius, and refiner settings for HNSW searches.
#[repr(C)]
pub struct zvec_hnsw_query_params_t {
    _private: [u8; 0],
}

/// Opaque pointer to IVF index query parameters.
/// Configures nprobe, scale factor, and refiner settings for IVF searches.
#[repr(C)]
pub struct zvec_ivf_query_params_t {
    _private: [u8; 0],
}

/// Opaque pointer to flat index query parameters.
/// Configures scale factor and refiner settings for brute-force searches.
#[repr(C)]
pub struct zvec_flat_query_params_t {
    _private: [u8; 0],
}

// =============================================================================
// Type Aliases
// =============================================================================

/// Error code returned by zvec functions.
pub type zvec_error_code_t = u32;
/// Data type identifier for field values.
pub type zvec_data_type_t = u32;
/// Index algorithm type identifier.
pub type zvec_index_type_t = u32;
/// Distance metric type for vector similarity.
pub type zvec_metric_type_t = u32;
/// Quantization type for vector compression.
pub type zvec_quantize_type_t = u32;
/// Log severity level.
pub type zvec_log_level_t = u32;
/// Log output destination type.
pub type zvec_log_type_t = u32;
/// Document operation type (insert/update/delete).
pub type zvec_doc_operator_t = u32;

// =============================================================================
// Error Code Constants
// =============================================================================

/// Operation completed successfully.
pub const ZVEC_OK: zvec_error_code_t = 0;
/// Requested resource was not found.
pub const ZVEC_ERROR_NOT_FOUND: zvec_error_code_t = 1;
/// Resource already exists.
pub const ZVEC_ERROR_ALREADY_EXISTS: zvec_error_code_t = 2;
/// Invalid argument provided.
pub const ZVEC_ERROR_INVALID_ARGUMENT: zvec_error_code_t = 3;
/// Permission denied for the operation.
pub const ZVEC_ERROR_PERMISSION_DENIED: zvec_error_code_t = 4;
/// Operation failed due to unmet precondition.
pub const ZVEC_ERROR_FAILED_PRECONDITION: zvec_error_code_t = 5;
/// System resources exhausted.
pub const ZVEC_ERROR_RESOURCE_EXHAUSTED: zvec_error_code_t = 6;
/// Service is currently unavailable.
pub const ZVEC_ERROR_UNAVAILABLE: zvec_error_code_t = 7;
/// Internal system error occurred.
pub const ZVEC_ERROR_INTERNAL_ERROR: zvec_error_code_t = 8;
/// Operation is not supported.
pub const ZVEC_ERROR_NOT_SUPPORTED: zvec_error_code_t = 9;
/// Unknown error occurred.
pub const ZVEC_ERROR_UNKNOWN: zvec_error_code_t = 10;

// =============================================================================
// Data Type Constants
// =============================================================================

/// Undefined data type.
pub const ZVEC_DATA_TYPE_UNDEFINED: zvec_data_type_t = 0;
/// Binary data type.
pub const ZVEC_DATA_TYPE_BINARY: zvec_data_type_t = 1;
/// String data type.
pub const ZVEC_DATA_TYPE_STRING: zvec_data_type_t = 2;
/// Boolean data type.
pub const ZVEC_DATA_TYPE_BOOL: zvec_data_type_t = 3;
/// 32-bit signed integer.
pub const ZVEC_DATA_TYPE_INT32: zvec_data_type_t = 4;
/// 64-bit signed integer.
pub const ZVEC_DATA_TYPE_INT64: zvec_data_type_t = 5;
/// 32-bit unsigned integer.
pub const ZVEC_DATA_TYPE_UINT32: zvec_data_type_t = 6;
/// 64-bit unsigned integer.
pub const ZVEC_DATA_TYPE_UINT64: zvec_data_type_t = 7;
/// 32-bit floating point.
pub const ZVEC_DATA_TYPE_FLOAT: zvec_data_type_t = 8;
/// 64-bit floating point.
pub const ZVEC_DATA_TYPE_DOUBLE: zvec_data_type_t = 9;
/// Dense vector with binary32 (float) elements.
pub const ZVEC_DATA_TYPE_VECTOR_BINARY32: zvec_data_type_t = 20;
/// Dense vector with binary64 (double) elements.
pub const ZVEC_DATA_TYPE_VECTOR_BINARY64: zvec_data_type_t = 21;
/// Dense vector with FP16 (half precision) elements.
pub const ZVEC_DATA_TYPE_VECTOR_FP16: zvec_data_type_t = 22;
/// Dense vector with FP32 (single precision) elements.
pub const ZVEC_DATA_TYPE_VECTOR_FP32: zvec_data_type_t = 23;
/// Dense vector with FP64 (double precision) elements.
pub const ZVEC_DATA_TYPE_VECTOR_FP64: zvec_data_type_t = 24;
/// Dense vector with INT4 quantized elements.
pub const ZVEC_DATA_TYPE_VECTOR_INT4: zvec_data_type_t = 25;
/// Dense vector with INT8 quantized elements.
pub const ZVEC_DATA_TYPE_VECTOR_INT8: zvec_data_type_t = 26;
/// Dense vector with INT16 quantized elements.
pub const ZVEC_DATA_TYPE_VECTOR_INT16: zvec_data_type_t = 27;
/// Sparse vector with FP16 elements.
pub const ZVEC_DATA_TYPE_SPARSE_VECTOR_FP16: zvec_data_type_t = 30;
/// Sparse vector with FP32 elements.
pub const ZVEC_DATA_TYPE_SPARSE_VECTOR_FP32: zvec_data_type_t = 31;
/// Array of binary values.
pub const ZVEC_DATA_TYPE_ARRAY_BINARY: zvec_data_type_t = 40;
/// Array of strings.
pub const ZVEC_DATA_TYPE_ARRAY_STRING: zvec_data_type_t = 41;
/// Array of booleans.
pub const ZVEC_DATA_TYPE_ARRAY_BOOL: zvec_data_type_t = 42;
/// Array of 32-bit signed integers.
pub const ZVEC_DATA_TYPE_ARRAY_INT32: zvec_data_type_t = 43;
/// Array of 64-bit signed integers.
pub const ZVEC_DATA_TYPE_ARRAY_INT64: zvec_data_type_t = 44;
/// Array of 32-bit unsigned integers.
pub const ZVEC_DATA_TYPE_ARRAY_UINT32: zvec_data_type_t = 45;
/// Array of 64-bit unsigned integers.
pub const ZVEC_DATA_TYPE_ARRAY_UINT64: zvec_data_type_t = 46;
/// Array of 32-bit floats.
pub const ZVEC_DATA_TYPE_ARRAY_FLOAT: zvec_data_type_t = 47;
/// Array of 64-bit doubles.
pub const ZVEC_DATA_TYPE_ARRAY_DOUBLE: zvec_data_type_t = 48;

// =============================================================================
// Index Type Constants
// =============================================================================

/// Undefined index type.
pub const ZVEC_INDEX_TYPE_UNDEFINED: zvec_index_type_t = 0;
/// HNSW (Hierarchical Navigable Small World) graph index.
pub const ZVEC_INDEX_TYPE_HNSW: zvec_index_type_t = 1;
/// IVF (Inverted File Index) with clustering.
pub const ZVEC_INDEX_TYPE_IVF: zvec_index_type_t = 2;
/// Flat/brute-force index (exact search).
pub const ZVEC_INDEX_TYPE_FLAT: zvec_index_type_t = 3;
/// Inverted index for scalar field filtering.
pub const ZVEC_INDEX_TYPE_INVERT: zvec_index_type_t = 10;

// =============================================================================
// Metric Type Constants
// =============================================================================

/// Undefined metric type.
pub const ZVEC_METRIC_TYPE_UNDEFINED: zvec_metric_type_t = 0;
/// L2 (Euclidean) distance.
pub const ZVEC_METRIC_TYPE_L2: zvec_metric_type_t = 1;
/// Inner product (dot product).
pub const ZVEC_METRIC_TYPE_IP: zvec_metric_type_t = 2;
/// Cosine similarity.
pub const ZVEC_METRIC_TYPE_COSINE: zvec_metric_type_t = 3;
/// MIPS converted to L2 distance.
pub const ZVEC_METRIC_TYPE_MIPSL2: zvec_metric_type_t = 4;

// =============================================================================
// Quantize Type Constants
// =============================================================================

/// Undefined quantization type.
pub const ZVEC_QUANTIZE_TYPE_UNDEFINED: zvec_quantize_type_t = 0;
/// FP16 (half precision) quantization.
pub const ZVEC_QUANTIZE_TYPE_FP16: zvec_quantize_type_t = 1;
/// INT8 (8-bit integer) quantization.
pub const ZVEC_QUANTIZE_TYPE_INT8: zvec_quantize_type_t = 2;
/// INT4 (4-bit integer) quantization.
pub const ZVEC_QUANTIZE_TYPE_INT4: zvec_quantize_type_t = 3;

// =============================================================================
// Log Level Constants
// =============================================================================

/// Debug level logging (most verbose).
pub const ZVEC_LOG_LEVEL_DEBUG: zvec_log_level_t = 0;
/// Info level logging.
pub const ZVEC_LOG_LEVEL_INFO: zvec_log_level_t = 1;
/// Warning level logging.
pub const ZVEC_LOG_LEVEL_WARN: zvec_log_level_t = 2;
/// Error level logging.
pub const ZVEC_LOG_LEVEL_ERROR: zvec_log_level_t = 3;
/// Fatal level logging (least verbose, critical errors only).
pub const ZVEC_LOG_LEVEL_FATAL: zvec_log_level_t = 4;

// =============================================================================
// Log Type Constants
// =============================================================================

/// Console output logging.
pub const ZVEC_LOG_TYPE_CONSOLE: zvec_log_type_t = 0;
/// File output logging.
pub const ZVEC_LOG_TYPE_FILE: zvec_log_type_t = 1;

// =============================================================================
// Doc Operator Constants
// =============================================================================

/// Insert operation (fails if document exists).
pub const ZVEC_DOC_OP_INSERT: zvec_doc_operator_t = 0;
/// Update operation (fails if document doesn't exist).
pub const ZVEC_DOC_OP_UPDATE: zvec_doc_operator_t = 1;
/// Upsert operation (insert or update).
pub const ZVEC_DOC_OP_UPSERT: zvec_doc_operator_t = 2;
/// Delete operation.
pub const ZVEC_DOC_OP_DELETE: zvec_doc_operator_t = 3;

// =============================================================================
// Non-Opaque Structures
// =============================================================================

/// A non-owning view of a string (pointer + length).
#[repr(C)]
pub struct zvec_string_view_t {
    pub data: *const c_char,
    pub length: usize,
}

/// An owning, mutable string with capacity tracking.
#[repr(C)]
pub struct zvec_string_t {
    pub data: *mut c_char,
    pub length: usize,
    pub capacity: usize,
}

/// An array of owned strings.
#[repr(C)]
pub struct zvec_string_array_t {
    pub strings: *mut zvec_string_t,
    pub count: usize,
}

/// A view of a float array (non-owning).
#[repr(C)]
pub struct zvec_float_array_t {
    pub data: *const f32,
    pub length: usize,
}

/// A view of an int64 array (non-owning).
#[repr(C)]
pub struct zvec_int64_array_t {
    pub data: *const i64,
    pub length: usize,
}

/// A view of a byte array (non-owning).
#[repr(C)]
pub struct zvec_byte_array_t {
    pub data: *const u8,
    pub length: usize,
}

/// A mutable, owning byte array with capacity tracking.
#[repr(C)]
pub struct zvec_mutable_byte_array_t {
    pub data: *mut u8,
    pub length: usize,
    pub capacity: usize,
}

/// Result of a write operation (insert/update/delete).
#[repr(C)]
pub struct zvec_write_result_t {
    pub code: zvec_error_code_t,
    pub message: *const c_char,
}

/// Detailed error information including source location.
#[repr(C)]
pub struct zvec_error_details_t {
    pub code: zvec_error_code_t,
    pub message: *const c_char,
    pub file: *const c_char,
    pub line: c_int,
    pub function: *const c_char,
}

// =============================================================================
// FFI Function Declarations
// =============================================================================

extern "C" {
    // -------------------------------------------------------------------------
    // Version
    // Functions for retrieving library version information.
    // -------------------------------------------------------------------------
    pub fn zvec_get_version() -> *const c_char;
    pub fn zvec_check_version(major: c_int, minor: c_int, patch: c_int) -> bool;
    pub fn zvec_get_version_major() -> c_int;
    pub fn zvec_get_version_minor() -> c_int;
    pub fn zvec_get_version_patch() -> c_int;

    // -------------------------------------------------------------------------
    // Error
    // Functions for error handling and retrieval.
    // -------------------------------------------------------------------------
    pub fn zvec_get_last_error_details(
        error_details: *mut zvec_error_details_t,
    ) -> zvec_error_code_t;
    pub fn zvec_get_last_error(error_msg: *mut *mut c_char) -> zvec_error_code_t;
    pub fn zvec_clear_error();
    pub fn zvec_error_code_to_string(error_code: zvec_error_code_t) -> *const c_char;

    // -------------------------------------------------------------------------
    // String Management
    // Functions for creating, manipulating, and freeing string objects.
    // -------------------------------------------------------------------------
    pub fn zvec_string_create(str: *const c_char) -> *mut zvec_string_t;
    pub fn zvec_string_create_from_view(view: *const zvec_string_view_t) -> *mut zvec_string_t;
    pub fn zvec_bin_create(data: *const u8, length: usize) -> *mut zvec_string_t;
    pub fn zvec_string_copy(str: *const zvec_string_t) -> *mut zvec_string_t;
    pub fn zvec_string_c_str(str: *const zvec_string_t) -> *const c_char;
    pub fn zvec_string_length(str: *const zvec_string_t) -> usize;
    pub fn zvec_string_compare(str1: *const zvec_string_t, str2: *const zvec_string_t) -> c_int;
    pub fn zvec_free_string(str: *mut zvec_string_t);

    // -------------------------------------------------------------------------
    // Array Management
    // Functions for creating and managing arrays of various types.
    // -------------------------------------------------------------------------
    pub fn zvec_string_array_create(count: usize) -> *mut zvec_string_array_t;
    pub fn zvec_string_array_add(array: *mut zvec_string_array_t, idx: usize, str: *const c_char);
    pub fn zvec_string_array_destroy(array: *mut zvec_string_array_t);
    pub fn zvec_byte_array_create(capacity: usize) -> *mut zvec_mutable_byte_array_t;
    pub fn zvec_byte_array_destroy(array: *mut zvec_mutable_byte_array_t);
    pub fn zvec_float_array_create(count: usize) -> *mut zvec_float_array_t;
    pub fn zvec_float_array_destroy(array: *mut zvec_float_array_t);
    pub fn zvec_int64_array_create(count: usize) -> *mut zvec_int64_array_t;
    pub fn zvec_int64_array_destroy(array: *mut zvec_int64_array_t);
    pub fn zvec_free_uint8_array(array: *mut u8);

    // -------------------------------------------------------------------------
    // Memory
    // Functions for memory allocation and deallocation.
    // -------------------------------------------------------------------------
    pub fn zvec_malloc(size: usize) -> *mut c_void;
    pub fn zvec_free(ptr: *mut c_void);

    // -------------------------------------------------------------------------
    // Log Configuration
    // Functions for configuring logging behavior.
    // -------------------------------------------------------------------------
    pub fn zvec_config_log_create_console(level: zvec_log_level_t) -> *mut zvec_log_config_t;
    pub fn zvec_config_log_create_file(
        level: zvec_log_level_t,
        dir: *const c_char,
        basename: *const c_char,
        file_size: u32,
        overdue_days: u32,
    ) -> *mut zvec_log_config_t;
    pub fn zvec_config_log_destroy(config: *mut zvec_log_config_t);
    pub fn zvec_config_log_get_level(config: *const zvec_log_config_t) -> zvec_log_level_t;
    pub fn zvec_config_log_set_level(
        config: *mut zvec_log_config_t,
        level: zvec_log_level_t,
    ) -> zvec_error_code_t;
    pub fn zvec_config_log_is_file_type(config: *const zvec_log_config_t) -> bool;
    pub fn zvec_config_log_get_dir(config: *const zvec_log_config_t) -> *const c_char;
    pub fn zvec_config_log_set_dir(
        config: *mut zvec_log_config_t,
        dir: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_config_log_get_basename(config: *const zvec_log_config_t) -> *const c_char;
    pub fn zvec_config_log_set_basename(
        config: *mut zvec_log_config_t,
        basename: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_config_log_get_file_size(config: *const zvec_log_config_t) -> u32;
    pub fn zvec_config_log_set_file_size(
        config: *mut zvec_log_config_t,
        file_size: u32,
    ) -> zvec_error_code_t;
    pub fn zvec_config_log_get_overdue_days(config: *const zvec_log_config_t) -> u32;
    pub fn zvec_config_log_set_overdue_days(
        config: *mut zvec_log_config_t,
        days: u32,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // Global Configuration
    // Functions for managing global system configuration.
    // -------------------------------------------------------------------------
    pub fn zvec_config_data_create() -> *mut zvec_config_data_t;
    pub fn zvec_config_data_destroy(config: *mut zvec_config_data_t);
    pub fn zvec_config_data_set_memory_limit(
        config: *mut zvec_config_data_t,
        memory_limit_bytes: u64,
    ) -> zvec_error_code_t;
    pub fn zvec_config_data_get_memory_limit(config: *const zvec_config_data_t) -> u64;
    pub fn zvec_config_data_set_log_config(
        config: *mut zvec_config_data_t,
        log_config: *mut zvec_log_config_t,
    ) -> zvec_error_code_t;
    pub fn zvec_config_data_get_log_type(config: *const zvec_config_data_t) -> zvec_log_type_t;
    pub fn zvec_config_data_set_query_thread_count(
        config: *mut zvec_config_data_t,
        thread_count: u32,
    ) -> zvec_error_code_t;
    pub fn zvec_config_data_get_query_thread_count(config: *const zvec_config_data_t) -> u32;
    pub fn zvec_config_data_set_invert_to_forward_scan_ratio(
        config: *mut zvec_config_data_t,
        ratio: f32,
    ) -> zvec_error_code_t;
    pub fn zvec_config_data_get_invert_to_forward_scan_ratio(
        config: *const zvec_config_data_t,
    ) -> f32;
    pub fn zvec_config_data_set_brute_force_by_keys_ratio(
        config: *mut zvec_config_data_t,
        ratio: f32,
    ) -> zvec_error_code_t;
    pub fn zvec_config_data_get_brute_force_by_keys_ratio(config: *const zvec_config_data_t)
        -> f32;
    pub fn zvec_config_data_set_optimize_thread_count(
        config: *mut zvec_config_data_t,
        thread_count: u32,
    ) -> zvec_error_code_t;
    pub fn zvec_config_data_get_optimize_thread_count(config: *const zvec_config_data_t) -> u32;

    // -------------------------------------------------------------------------
    // Initialization
    // Functions for initializing and shutting down the library.
    // -------------------------------------------------------------------------
    pub fn zvec_initialize(config: *const zvec_config_data_t) -> zvec_error_code_t;
    pub fn zvec_shutdown() -> zvec_error_code_t;
    pub fn zvec_is_initialized() -> bool;

    // -------------------------------------------------------------------------
    // Index Parameters
    // Functions for creating and configuring index parameters.
    // -------------------------------------------------------------------------
    pub fn zvec_index_params_create(index_type: zvec_index_type_t) -> *mut zvec_index_params_t;
    pub fn zvec_index_params_destroy(params: *mut zvec_index_params_t);
    pub fn zvec_index_params_get_type(params: *const zvec_index_params_t) -> zvec_index_type_t;
    pub fn zvec_index_params_set_metric_type(
        params: *mut zvec_index_params_t,
        metric_type: zvec_metric_type_t,
    ) -> zvec_error_code_t;
    pub fn zvec_index_params_get_metric_type(
        params: *const zvec_index_params_t,
    ) -> zvec_metric_type_t;
    pub fn zvec_index_params_set_quantize_type(
        params: *mut zvec_index_params_t,
        quantize_type: zvec_quantize_type_t,
    ) -> zvec_error_code_t;
    pub fn zvec_index_params_get_quantize_type(
        params: *const zvec_index_params_t,
    ) -> zvec_quantize_type_t;
    pub fn zvec_index_params_set_hnsw_params(
        params: *mut zvec_index_params_t,
        m: c_int,
        ef_construction: c_int,
    ) -> zvec_error_code_t;
    pub fn zvec_index_params_get_hnsw_m(params: *const zvec_index_params_t) -> c_int;
    pub fn zvec_index_params_get_hnsw_ef_construction(params: *const zvec_index_params_t) -> c_int;
    pub fn zvec_index_params_set_ivf_params(
        params: *mut zvec_index_params_t,
        n_list: c_int,
        n_iters: c_int,
        use_soar: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_index_params_get_ivf_params(
        params: *const zvec_index_params_t,
        out_n_list: *mut c_int,
        out_n_iters: *mut c_int,
        out_use_soar: *mut bool,
    ) -> zvec_error_code_t;
    pub fn zvec_index_params_set_invert_params(
        params: *mut zvec_index_params_t,
        enable_range_opt: bool,
        enable_wildcard: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_index_params_get_invert_params(
        params: *const zvec_index_params_t,
        out_enable_range_opt: *mut bool,
        out_enable_wildcard: *mut bool,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // Field Schema
    // Functions for creating and managing field schema definitions.
    // -------------------------------------------------------------------------
    pub fn zvec_field_schema_create(
        name: *const c_char,
        data_type: zvec_data_type_t,
        nullable: bool,
        dimension: u32,
    ) -> *mut zvec_field_schema_t;
    pub fn zvec_field_schema_destroy(schema: *mut zvec_field_schema_t);
    pub fn zvec_field_schema_set_name(
        schema: *mut zvec_field_schema_t,
        name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_field_schema_get_name(schema: *const zvec_field_schema_t) -> *const c_char;
    pub fn zvec_field_schema_get_data_type(schema: *const zvec_field_schema_t) -> zvec_data_type_t;
    pub fn zvec_field_schema_set_data_type(
        schema: *mut zvec_field_schema_t,
        data_type: zvec_data_type_t,
    ) -> zvec_error_code_t;
    pub fn zvec_field_schema_get_element_data_type(
        schema: *const zvec_field_schema_t,
    ) -> zvec_data_type_t;
    pub fn zvec_field_schema_get_element_data_size(schema: *const zvec_field_schema_t) -> usize;
    pub fn zvec_field_schema_is_vector_field(schema: *const zvec_field_schema_t) -> bool;
    pub fn zvec_field_schema_is_dense_vector(schema: *const zvec_field_schema_t) -> bool;
    pub fn zvec_field_schema_is_sparse_vector(schema: *const zvec_field_schema_t) -> bool;
    pub fn zvec_field_schema_is_nullable(schema: *const zvec_field_schema_t) -> bool;
    pub fn zvec_field_schema_set_nullable(
        schema: *mut zvec_field_schema_t,
        nullable: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_field_schema_has_invert_index(schema: *const zvec_field_schema_t) -> bool;
    pub fn zvec_field_schema_is_array_type(schema: *const zvec_field_schema_t) -> bool;
    pub fn zvec_field_schema_get_dimension(schema: *const zvec_field_schema_t) -> u32;
    pub fn zvec_field_schema_set_dimension(
        schema: *mut zvec_field_schema_t,
        dimension: u32,
    ) -> zvec_error_code_t;
    pub fn zvec_field_schema_get_index_type(
        schema: *const zvec_field_schema_t,
    ) -> zvec_index_type_t;
    pub fn zvec_field_schema_has_index(schema: *const zvec_field_schema_t) -> bool;
    pub fn zvec_field_schema_get_index_params(
        schema: *const zvec_field_schema_t,
    ) -> *const zvec_index_params_t;
    pub fn zvec_field_schema_set_index_params(
        schema: *mut zvec_field_schema_t,
        index_params: *const zvec_index_params_t,
    ) -> zvec_error_code_t;
    pub fn zvec_field_schema_validate(
        schema: *const zvec_field_schema_t,
        error_msg: *mut *mut zvec_string_t,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // Collection Schema
    // Functions for creating and managing collection schema definitions.
    // -------------------------------------------------------------------------
    pub fn zvec_collection_schema_create(name: *const c_char) -> *mut zvec_collection_schema_t;
    pub fn zvec_collection_schema_destroy(schema: *mut zvec_collection_schema_t);
    pub fn zvec_collection_schema_get_name(
        schema: *const zvec_collection_schema_t,
    ) -> *const c_char;
    pub fn zvec_collection_schema_set_name(
        schema: *mut zvec_collection_schema_t,
        name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_add_field(
        schema: *mut zvec_collection_schema_t,
        field: *const zvec_field_schema_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_alter_field(
        schema: *mut zvec_collection_schema_t,
        field_name: *const c_char,
        new_field: *const zvec_field_schema_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_drop_field(
        schema: *mut zvec_collection_schema_t,
        field_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_has_field(
        schema: *const zvec_collection_schema_t,
        field_name: *const c_char,
    ) -> bool;
    pub fn zvec_collection_schema_get_field(
        schema: *const zvec_collection_schema_t,
        field_name: *const c_char,
    ) -> *mut zvec_field_schema_t;
    pub fn zvec_collection_schema_get_forward_field(
        schema: *const zvec_collection_schema_t,
        field_name: *const c_char,
    ) -> *mut zvec_field_schema_t;
    pub fn zvec_collection_schema_get_vector_field(
        schema: *const zvec_collection_schema_t,
        field_name: *const c_char,
    ) -> *mut zvec_field_schema_t;
    pub fn zvec_collection_schema_get_forward_fields(
        schema: *const zvec_collection_schema_t,
        fields: *mut *mut *mut zvec_field_schema_t,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_get_forward_fields_with_index(
        schema: *const zvec_collection_schema_t,
        fields: *mut *mut *mut zvec_field_schema_t,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_get_forward_field_names(
        schema: *const zvec_collection_schema_t,
        names: *mut *mut *const c_char,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_get_forward_field_names_with_index(
        schema: *const zvec_collection_schema_t,
        names: *mut *mut *const c_char,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_get_all_field_names(
        schema: *const zvec_collection_schema_t,
        names: *mut *mut *const c_char,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_get_vector_fields(
        schema: *const zvec_collection_schema_t,
        fields: *mut *mut *mut zvec_field_schema_t,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_get_max_doc_count_per_segment(
        schema: *const zvec_collection_schema_t,
    ) -> u64;
    pub fn zvec_collection_schema_set_max_doc_count_per_segment(
        schema: *mut zvec_collection_schema_t,
        max_doc_count: u64,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_validate(
        schema: *const zvec_collection_schema_t,
        error_msg: *mut *mut zvec_string_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_add_index(
        schema: *mut zvec_collection_schema_t,
        field_name: *const c_char,
        index_params: *const zvec_index_params_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_drop_index(
        schema: *mut zvec_collection_schema_t,
        field_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_schema_has_index(
        schema: *const zvec_collection_schema_t,
        field_name: *const c_char,
    ) -> bool;

    // -------------------------------------------------------------------------
    // Collection Options
    // Functions for configuring collection opening options.
    // -------------------------------------------------------------------------
    pub fn zvec_collection_options_create() -> *mut zvec_collection_options_t;
    pub fn zvec_collection_options_destroy(options: *mut zvec_collection_options_t);
    pub fn zvec_collection_options_set_enable_mmap(
        options: *mut zvec_collection_options_t,
        enable: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_options_get_enable_mmap(
        options: *const zvec_collection_options_t,
    ) -> bool;
    pub fn zvec_collection_options_set_max_buffer_size(
        options: *mut zvec_collection_options_t,
        size: usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_options_get_max_buffer_size(
        options: *const zvec_collection_options_t,
    ) -> usize;
    pub fn zvec_collection_options_set_read_only(
        options: *mut zvec_collection_options_t,
        read_only: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_options_get_read_only(options: *const zvec_collection_options_t)
        -> bool;

    // -------------------------------------------------------------------------
    // Collection Statistics
    // Functions for retrieving collection statistics.
    // -------------------------------------------------------------------------
    pub fn zvec_collection_stats_get_doc_count(stats: *const zvec_collection_stats_t) -> u64;
    pub fn zvec_collection_stats_get_index_count(stats: *const zvec_collection_stats_t) -> usize;
    pub fn zvec_collection_stats_get_index_name(
        stats: *const zvec_collection_stats_t,
        index: usize,
    ) -> *const c_char;
    pub fn zvec_collection_stats_get_index_completeness(
        stats: *const zvec_collection_stats_t,
        index: usize,
    ) -> f32;
    pub fn zvec_collection_stats_destroy(stats: *mut zvec_collection_stats_t);

    // -------------------------------------------------------------------------
    // Collection Management
    // Functions for creating, opening, closing, and destroying collections.
    // -------------------------------------------------------------------------
    pub fn zvec_collection_create_and_open(
        path: *const c_char,
        schema: *const zvec_collection_schema_t,
        options: *const zvec_collection_options_t,
        collection: *mut *mut zvec_collection_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_open(
        path: *const c_char,
        options: *const zvec_collection_options_t,
        collection: *mut *mut zvec_collection_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_close(collection: *mut zvec_collection_t) -> zvec_error_code_t;
    pub fn zvec_collection_destroy(collection: *mut zvec_collection_t) -> zvec_error_code_t;
    pub fn zvec_collection_flush(collection: *mut zvec_collection_t) -> zvec_error_code_t;
    pub fn zvec_collection_get_schema(
        collection: *const zvec_collection_t,
        schema: *mut *mut zvec_collection_schema_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_get_options(
        collection: *const zvec_collection_t,
        options: *mut *mut zvec_collection_options_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_get_stats(
        collection: *const zvec_collection_t,
        stats: *mut *mut zvec_collection_stats_t,
    ) -> zvec_error_code_t;
    pub fn zvec_free_field_schema(field_schema: *mut zvec_field_schema_t);

    // -------------------------------------------------------------------------
    // Index Management
    // Functions for creating, dropping, and optimizing indexes.
    // -------------------------------------------------------------------------
    pub fn zvec_collection_create_index(
        collection: *mut zvec_collection_t,
        field_name: *const c_char,
        index_params: *const zvec_index_params_t,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_drop_index(
        collection: *mut zvec_collection_t,
        field_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_optimize(collection: *mut zvec_collection_t) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // Column Management (DDL)
    // Functions for adding, dropping, and altering columns (schema changes).
    // -------------------------------------------------------------------------
    pub fn zvec_collection_add_column(
        collection: *mut zvec_collection_t,
        field_schema: *const zvec_field_schema_t,
        expression: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_drop_column(
        collection: *mut zvec_collection_t,
        column_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_alter_column(
        collection: *mut zvec_collection_t,
        column_name: *const c_char,
        new_name: *const c_char,
        new_schema: *const zvec_field_schema_t,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // DML
    // Functions for data manipulation: insert, update, upsert, and delete operations.
    // -------------------------------------------------------------------------
    pub fn zvec_collection_insert(
        collection: *mut zvec_collection_t,
        docs: *const *const zvec_doc_t,
        doc_count: usize,
        success_count: *mut usize,
        error_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_insert_with_results(
        collection: *mut zvec_collection_t,
        docs: *const *const zvec_doc_t,
        doc_count: usize,
        results: *mut *mut zvec_write_result_t,
        result_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_update(
        collection: *mut zvec_collection_t,
        docs: *const *const zvec_doc_t,
        doc_count: usize,
        success_count: *mut usize,
        error_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_update_with_results(
        collection: *mut zvec_collection_t,
        docs: *const *const zvec_doc_t,
        doc_count: usize,
        results: *mut *mut zvec_write_result_t,
        result_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_upsert(
        collection: *mut zvec_collection_t,
        docs: *const *const zvec_doc_t,
        doc_count: usize,
        success_count: *mut usize,
        error_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_upsert_with_results(
        collection: *mut zvec_collection_t,
        docs: *const *const zvec_doc_t,
        doc_count: usize,
        results: *mut *mut zvec_write_result_t,
        result_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_delete(
        collection: *mut zvec_collection_t,
        pks: *const *const c_char,
        pk_count: usize,
        success_count: *mut usize,
        error_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_delete_with_results(
        collection: *mut zvec_collection_t,
        pks: *const *const c_char,
        pk_count: usize,
        results: *mut *mut zvec_write_result_t,
        result_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_write_results_free(results: *mut zvec_write_result_t, result_count: usize);
    pub fn zvec_collection_delete_by_filter(
        collection: *mut zvec_collection_t,
        filter: *const c_char,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // DQL
    // Functions for data querying: vector search and document fetch operations.
    // -------------------------------------------------------------------------
    pub fn zvec_collection_query(
        collection: *const zvec_collection_t,
        query: *const zvec_vector_query_t,
        results: *mut *mut *mut zvec_doc_t,
        result_count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_collection_fetch(
        collection: *mut zvec_collection_t,
        primary_keys: *const *const c_char,
        count: usize,
        documents: *mut *mut *mut zvec_doc_t,
        found_count: *mut usize,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // HNSW Query Parameters
    // Functions for creating and configuring HNSW index query parameters.
    // -------------------------------------------------------------------------
    pub fn zvec_query_params_hnsw_create(
        ef: c_int,
        radius: f32,
        is_linear: bool,
        is_using_refiner: bool,
    ) -> *mut zvec_hnsw_query_params_t;
    pub fn zvec_query_params_hnsw_destroy(params: *mut zvec_hnsw_query_params_t);
    pub fn zvec_query_params_hnsw_set_ef(
        params: *mut zvec_hnsw_query_params_t,
        ef: c_int,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_hnsw_get_ef(params: *const zvec_hnsw_query_params_t) -> c_int;
    pub fn zvec_query_params_hnsw_set_radius(
        params: *mut zvec_hnsw_query_params_t,
        radius: f32,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_hnsw_get_radius(params: *const zvec_hnsw_query_params_t) -> f32;
    pub fn zvec_query_params_hnsw_set_is_linear(
        params: *mut zvec_hnsw_query_params_t,
        is_linear: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_hnsw_get_is_linear(params: *const zvec_hnsw_query_params_t) -> bool;
    pub fn zvec_query_params_hnsw_set_is_using_refiner(
        params: *mut zvec_hnsw_query_params_t,
        is_using_refiner: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_hnsw_get_is_using_refiner(
        params: *const zvec_hnsw_query_params_t,
    ) -> bool;

    // -------------------------------------------------------------------------
    // IVF Query Parameters
    // Functions for creating and configuring IVF index query parameters.
    // -------------------------------------------------------------------------
    pub fn zvec_query_params_ivf_create(
        nprobe: c_int,
        is_using_refiner: bool,
        scale_factor: f32,
    ) -> *mut zvec_ivf_query_params_t;
    pub fn zvec_query_params_ivf_destroy(params: *mut zvec_ivf_query_params_t);
    pub fn zvec_query_params_ivf_set_nprobe(
        params: *mut zvec_ivf_query_params_t,
        nprobe: c_int,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_ivf_get_nprobe(params: *const zvec_ivf_query_params_t) -> c_int;
    pub fn zvec_query_params_ivf_set_scale_factor(
        params: *mut zvec_ivf_query_params_t,
        scale_factor: f32,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_ivf_get_scale_factor(params: *const zvec_ivf_query_params_t) -> f32;
    pub fn zvec_query_params_ivf_set_radius(
        params: *mut zvec_ivf_query_params_t,
        radius: f32,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_ivf_get_radius(params: *const zvec_ivf_query_params_t) -> f32;
    pub fn zvec_query_params_ivf_set_is_linear(
        params: *mut zvec_ivf_query_params_t,
        is_linear: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_ivf_get_is_linear(params: *const zvec_ivf_query_params_t) -> bool;
    pub fn zvec_query_params_ivf_set_is_using_refiner(
        params: *mut zvec_ivf_query_params_t,
        is_using_refiner: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_ivf_get_is_using_refiner(
        params: *const zvec_ivf_query_params_t,
    ) -> bool;

    // -------------------------------------------------------------------------
    // Flat Query Parameters
    // Functions for creating and configuring flat index query parameters.
    // -------------------------------------------------------------------------
    pub fn zvec_query_params_flat_create(
        is_using_refiner: bool,
        scale_factor: f32,
    ) -> *mut zvec_flat_query_params_t;
    pub fn zvec_query_params_flat_destroy(params: *mut zvec_flat_query_params_t);
    pub fn zvec_query_params_flat_set_scale_factor(
        params: *mut zvec_flat_query_params_t,
        scale_factor: f32,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_flat_get_scale_factor(params: *const zvec_flat_query_params_t) -> f32;
    pub fn zvec_query_params_flat_set_radius(
        params: *mut zvec_flat_query_params_t,
        radius: f32,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_flat_get_radius(params: *const zvec_flat_query_params_t) -> f32;
    pub fn zvec_query_params_flat_set_is_linear(
        params: *mut zvec_flat_query_params_t,
        is_linear: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_flat_get_is_linear(params: *const zvec_flat_query_params_t) -> bool;
    pub fn zvec_query_params_flat_set_is_using_refiner(
        params: *mut zvec_flat_query_params_t,
        is_using_refiner: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_query_params_flat_get_is_using_refiner(
        params: *const zvec_flat_query_params_t,
    ) -> bool;

    // -------------------------------------------------------------------------
    // Vector Query
    // Functions for creating and configuring vector similarity queries.
    // -------------------------------------------------------------------------
    pub fn zvec_vector_query_create() -> *mut zvec_vector_query_t;
    pub fn zvec_vector_query_destroy(query: *mut zvec_vector_query_t);
    pub fn zvec_vector_query_set_topk(
        query: *mut zvec_vector_query_t,
        topk: c_int,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_get_topk(query: *const zvec_vector_query_t) -> c_int;
    pub fn zvec_vector_query_set_field_name(
        query: *mut zvec_vector_query_t,
        field_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_get_field_name(query: *const zvec_vector_query_t) -> *const c_char;
    pub fn zvec_vector_query_set_query_vector(
        query: *mut zvec_vector_query_t,
        data: *const c_void,
        size: usize,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_set_filter(
        query: *mut zvec_vector_query_t,
        filter: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_get_filter(query: *const zvec_vector_query_t) -> *const c_char;
    pub fn zvec_vector_query_set_include_vector(
        query: *mut zvec_vector_query_t,
        include: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_get_include_vector(query: *const zvec_vector_query_t) -> bool;
    pub fn zvec_vector_query_set_include_doc_id(
        query: *mut zvec_vector_query_t,
        include: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_get_include_doc_id(query: *const zvec_vector_query_t) -> bool;
    pub fn zvec_vector_query_set_output_fields(
        query: *mut zvec_vector_query_t,
        fields: *const *const c_char,
        count: usize,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_get_output_fields(
        query: *const zvec_vector_query_t,
        fields: *mut *mut *const c_char,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_set_query_params(
        query: *mut zvec_vector_query_t,
        params: *mut c_void,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_set_hnsw_params(
        query: *mut zvec_vector_query_t,
        hnsw_params: *mut zvec_hnsw_query_params_t,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_set_ivf_params(
        query: *mut zvec_vector_query_t,
        ivf_params: *mut zvec_ivf_query_params_t,
    ) -> zvec_error_code_t;
    pub fn zvec_vector_query_set_flat_params(
        query: *mut zvec_vector_query_t,
        flat_params: *mut zvec_flat_query_params_t,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // Group By Vector Query
    // Functions for creating and configuring grouped vector queries.
    // -------------------------------------------------------------------------
    pub fn zvec_group_by_vector_query_create() -> *mut zvec_group_by_vector_query_t;
    pub fn zvec_group_by_vector_query_destroy(query: *mut zvec_group_by_vector_query_t);
    pub fn zvec_group_by_vector_query_set_field_name(
        query: *mut zvec_group_by_vector_query_t,
        field_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_get_field_name(
        query: *const zvec_group_by_vector_query_t,
    ) -> *const c_char;
    pub fn zvec_group_by_vector_query_set_group_by_field_name(
        query: *mut zvec_group_by_vector_query_t,
        field_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_get_group_by_field_name(
        query: *const zvec_group_by_vector_query_t,
    ) -> *const c_char;
    pub fn zvec_group_by_vector_query_set_group_count(
        query: *mut zvec_group_by_vector_query_t,
        count: u32,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_get_group_count(
        query: *const zvec_group_by_vector_query_t,
    ) -> u32;
    pub fn zvec_group_by_vector_query_set_group_topk(
        query: *mut zvec_group_by_vector_query_t,
        topk: u32,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_get_group_topk(
        query: *const zvec_group_by_vector_query_t,
    ) -> u32;
    pub fn zvec_group_by_vector_query_set_query_vector(
        query: *mut zvec_group_by_vector_query_t,
        data: *const c_void,
        size: usize,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_set_filter(
        query: *mut zvec_group_by_vector_query_t,
        filter: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_get_filter(
        query: *const zvec_group_by_vector_query_t,
    ) -> *const c_char;
    pub fn zvec_group_by_vector_query_set_include_vector(
        query: *mut zvec_group_by_vector_query_t,
        include: bool,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_get_include_vector(
        query: *const zvec_group_by_vector_query_t,
    ) -> bool;
    pub fn zvec_group_by_vector_query_set_output_fields(
        query: *mut zvec_group_by_vector_query_t,
        fields: *const *const c_char,
        count: usize,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_get_output_fields(
        query: *mut zvec_group_by_vector_query_t,
        fields: *mut *mut *const c_char,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_set_query_params(
        query: *mut zvec_group_by_vector_query_t,
        params: *mut c_void,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_set_hnsw_params(
        query: *mut zvec_group_by_vector_query_t,
        hnsw_params: *mut zvec_hnsw_query_params_t,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_set_ivf_params(
        query: *mut zvec_group_by_vector_query_t,
        ivf_params: *mut zvec_ivf_query_params_t,
    ) -> zvec_error_code_t;
    pub fn zvec_group_by_vector_query_set_flat_params(
        query: *mut zvec_group_by_vector_query_t,
        flat_params: *mut zvec_flat_query_params_t,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // Document Operations
    // Functions for creating, manipulating, and serializing documents.
    // -------------------------------------------------------------------------
    pub fn zvec_doc_create() -> *mut zvec_doc_t;
    pub fn zvec_doc_destroy(doc: *mut zvec_doc_t);
    pub fn zvec_doc_clear(doc: *mut zvec_doc_t);
    pub fn zvec_doc_add_field_by_value(
        doc: *mut zvec_doc_t,
        field_name: *const c_char,
        data_type: zvec_data_type_t,
        value: *const c_void,
        value_size: usize,
    ) -> zvec_error_code_t;
    pub fn zvec_doc_remove_field(
        doc: *mut zvec_doc_t,
        field_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_docs_free(documents: *mut *mut zvec_doc_t, count: usize);
    pub fn zvec_doc_set_pk(doc: *mut zvec_doc_t, pk: *const c_char);
    pub fn zvec_doc_set_doc_id(doc: *mut zvec_doc_t, doc_id: u64);
    pub fn zvec_doc_set_score(doc: *mut zvec_doc_t, score: f32);
    pub fn zvec_doc_set_operator(doc: *mut zvec_doc_t, op: zvec_doc_operator_t);
    pub fn zvec_doc_set_field_null(
        doc: *mut zvec_doc_t,
        field_name: *const c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_doc_get_doc_id(doc: *const zvec_doc_t) -> u64;
    pub fn zvec_doc_get_score(doc: *const zvec_doc_t) -> f32;
    pub fn zvec_doc_get_operator(doc: *const zvec_doc_t) -> zvec_doc_operator_t;
    pub fn zvec_doc_get_field_count(doc: *const zvec_doc_t) -> usize;
    pub fn zvec_doc_get_pk_pointer(doc: *const zvec_doc_t) -> *const c_char;
    pub fn zvec_doc_get_pk_copy(doc: *const zvec_doc_t) -> *const c_char;
    pub fn zvec_doc_get_field_value_basic(
        doc: *const zvec_doc_t,
        field_name: *const c_char,
        field_type: zvec_data_type_t,
        value_buffer: *mut c_void,
        buffer_size: usize,
    ) -> zvec_error_code_t;
    pub fn zvec_doc_get_field_value_copy(
        doc: *const zvec_doc_t,
        field_name: *const c_char,
        field_type: zvec_data_type_t,
        value: *mut *mut c_void,
        value_size: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_doc_get_field_value_pointer(
        doc: *const zvec_doc_t,
        field_name: *const c_char,
        field_type: zvec_data_type_t,
        value: *mut *const c_void,
        value_size: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_doc_is_empty(doc: *const zvec_doc_t) -> bool;
    pub fn zvec_doc_has_field(doc: *const zvec_doc_t, field_name: *const c_char) -> bool;
    pub fn zvec_doc_has_field_value(doc: *const zvec_doc_t, field_name: *const c_char) -> bool;
    pub fn zvec_doc_is_field_null(doc: *const zvec_doc_t, field_name: *const c_char) -> bool;
    pub fn zvec_doc_get_field_names(
        doc: *const zvec_doc_t,
        field_names: *mut *mut *mut c_char,
        count: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_free_str_array(array: *mut *mut c_char, count: usize);
    pub fn zvec_doc_serialize(
        doc: *const zvec_doc_t,
        data: *mut *mut u8,
        size: *mut usize,
    ) -> zvec_error_code_t;
    pub fn zvec_doc_deserialize(
        data: *const u8,
        size: usize,
        doc: *mut *mut zvec_doc_t,
    ) -> zvec_error_code_t;
    pub fn zvec_doc_merge(doc: *mut zvec_doc_t, other: *const zvec_doc_t);
    pub fn zvec_doc_memory_usage(doc: *const zvec_doc_t) -> usize;
    pub fn zvec_doc_validate(
        doc: *const zvec_doc_t,
        schema: *const zvec_collection_schema_t,
        is_update: bool,
        error_msg: *mut *mut c_char,
    ) -> zvec_error_code_t;
    pub fn zvec_doc_to_detail_string(
        doc: *const zvec_doc_t,
        detail_str: *mut *mut c_char,
    ) -> zvec_error_code_t;

    // -------------------------------------------------------------------------
    // Utility Functions
    // Helper functions for converting enums to string representations.
    // -------------------------------------------------------------------------
    pub fn zvec_data_type_to_string(data_type: zvec_data_type_t) -> *const c_char;
    pub fn zvec_index_type_to_string(index_type: zvec_index_type_t) -> *const c_char;
    pub fn zvec_metric_type_to_string(metric_type: zvec_metric_type_t) -> *const c_char;
}

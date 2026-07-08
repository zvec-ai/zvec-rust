//! Regression test for exact string filters.
//!
//! A string field written via `Doc::add_string` must be stored as its exact
//! bytes so that an exact filter (`name = 'alice'`) matches it. This guards
//! against the trailing-NUL regression where the stored value was `"alice\0"`,
//! causing exact string filters to return zero rows.
//!
//! Requires the zvec C library (`libzvec_c_api`). Set `ZVEC_LIB_DIR` before
//! running. Run with: `cargo test --test string_filter_test`

use std::sync::Once;
use zvec_rust::*;

static INIT: Once = Once::new();

fn ensure_initialized() {
    INIT.call_once(|| {
        initialize(None).expect("failed to initialize zvec");
    });
}

#[test]
fn test_exact_string_filter_returns_matching_rows() {
    ensure_initialized();

    let tmp_dir = tempfile::tempdir().unwrap();
    let dir = tmp_dir.path().join("zvec_data");

    // Collection with an indexed String field and a vector field to search over.
    let schema = CollectionSchema::builder("string_filter_test")
        .add_field(FieldSchema::new("id", DataType::String, false, 0).unwrap())
        .add_indexed_field(
            "name",
            DataType::String,
            IndexParams::invert(false, false).unwrap(),
        )
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            2,
            IndexParams::hnsw(MetricType::Cosine, 16, 200).unwrap(),
        )
        .build()
        .expect("failed to build schema");

    let collection = Collection::create_and_open(dir.to_str().unwrap(), &schema, None)
        .expect("failed to create collection");

    // Insert docs with two "alice" rows and two non-matching rows.
    let names = ["alice", "bob", "alice", "carol"];
    let mut docs = Vec::new();
    for (i, name) in names.iter().enumerate() {
        let mut doc = Doc::new().unwrap();
        doc.set_pk(&format!("pk_{}", i));
        doc.add_string("id", &format!("pk_{}", i)).unwrap();
        doc.add_string("name", name).unwrap();
        doc.add_vector_f32("embedding", &[0.1, 0.2]).unwrap();
        docs.push(doc);
    }
    let doc_refs: Vec<&Doc> = docs.iter().collect();
    let result = collection.insert(&doc_refs).unwrap();
    assert_eq!(result.success_count, names.len() as u64);
    collection.flush().expect("flush before filter query");

    // Search over all docs (topk >= doc count) with an exact string filter.
    let mut query = SearchQuery::new("embedding", &[0.1, 0.2], names.len() as i32).unwrap();
    query.set_filter("name = 'alice'").unwrap();

    let results = collection.query(&query).unwrap();

    // Exactly the two "alice" rows must come back. Pre-fix (trailing NUL) this
    // returns 0 rows because the stored value is "alice\0" != "alice".
    assert_eq!(
        results.len(),
        2,
        "exact string filter should return exactly the matching rows"
    );
    for doc in &results {
        assert_eq!(
            doc.get_string("name").unwrap().as_deref(),
            Some("alice"),
            "every returned row must match the filter"
        );
    }
}

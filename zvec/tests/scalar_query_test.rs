//! Regression coverage for querying a scalar-only collection through the safe API.

use zvec_rust::{
    initialize, Collection, CollectionSchema, DataType, Doc, ErrorCode, FieldSchema, IndexParams,
    SearchQuery,
};

#[test]
fn scalar_queries_support_exact_lookup_projection_and_limits() {
    initialize(None).expect("initialize zvec");
    let temporary = tempfile::tempdir().expect("temporary directory");
    let path = temporary.path().join("scalar_paths");
    let schema = CollectionSchema::builder("scalar_paths")
        .add_field(FieldSchema::new("file_id", DataType::Uint32, false, 0).expect("file id field"))
        .add_indexed_field(
            "file_name",
            DataType::String,
            IndexParams::invert(false, true).expect("file name index"),
        )
        .add_indexed_field(
            "relative_path",
            DataType::String,
            IndexParams::invert(false, true).expect("path index"),
        )
        .add_field(FieldSchema::new("payload", DataType::String, false, 0).expect("payload field"))
        .build()
        .expect("scalar schema");
    let collection =
        Collection::create_and_open(path.to_str().expect("database path"), &schema, None)
            .expect("create scalar collection");
    let mut docs = Vec::new();
    for (key, path, file_name, file_id) in [
        ("1", "src/main.rs", "main.rs", 1),
        ("2", "src/lib.rs", "lib.rs", 2),
        ("3", "tests/main.rs", "", 3),
    ] {
        let mut doc = Doc::new().expect("document");
        doc.set_pk(key);
        doc.add_u32("file_id", file_id).expect("file id");
        doc.add_string("file_name", file_name).expect("file name");
        doc.add_string("relative_path", path).expect("path");
        doc.add_string("payload", "unselected payload")
            .expect("payload");
        docs.push(doc);
    }
    let refs: Vec<_> = docs.iter().collect();
    assert_eq!(
        collection
            .insert(&refs)
            .expect("insert documents")
            .success_count,
        3
    );
    collection.flush().expect("persist scalar documents");

    let mut query = SearchQuery::scalar(2).expect("scalar query without a vector or field");
    query
        .set_filter("relative_path = 'src/main.rs'")
        .expect("exact path filter");
    query
        .set_output_fields(&["relative_path"])
        .expect("project path");
    let results = collection.query(&query).expect("query scalar index");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].get_pk(), Some("1"));
    assert_eq!(
        results[0]
            .get_string("relative_path")
            .expect("path value")
            .as_deref(),
        Some("src/main.rs")
    );
    assert!(!results[0].has_field("payload"));

    query
        .set_filter("relative_path = 'absent.rs'")
        .expect("missing path filter");
    assert!(collection
        .query(&query)
        .expect("missing path query")
        .is_empty());

    query
        .set_filter("relative_path LIKE 'src/%'")
        .expect("prefix filter");
    let mut paths: Vec<_> = collection
        .query(&query)
        .expect("prefix candidates")
        .iter()
        .map(|doc| {
            doc.get_string("relative_path")
                .expect("path value")
                .expect("path exists")
        })
        .collect();
    paths.sort();
    assert_eq!(paths, ["src/lib.rs", "src/main.rs"]);

    // Match the existence query used for non-Unicode file name projections.
    let mut exists = SearchQuery::scalar(1).expect("existence query");
    exists
        .set_filter("file_name = ''")
        .expect("empty name filter");
    exists
        .set_output_fields(&["file_id"])
        .expect("project file id");
    exists.set_include_vector(false).expect("omit vectors");
    let results = collection.query(&exists).expect("query empty file name");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].get_u32("file_id").expect("file id"), Some(3));
    assert!(!results[0].has_field("payload"));
    assert!(!results[0].has_field("relative_path"));

    let limited = SearchQuery::scalar(1).expect("limited scalar query");
    assert_eq!(
        collection.query(&limited).expect("limited results").len(),
        1
    );

    // The builder reaches the same scalar path when neither a vector nor an FTS
    // clause is set.
    let built = SearchQuery::builder()
        .topk(2)
        .filter("relative_path = 'src/main.rs'")
        .output_fields(&["relative_path"])
        .build()
        .expect("scalar query from the builder");
    let results = collection.query(&built).expect("query through the builder");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].get_pk(), Some("1"));
    assert!(!results[0].has_field("payload"));

    // A field name without a vector or FTS clause is ambiguous, not a scalar query.
    let err = match SearchQuery::builder().field_name("relative_path").build() {
        Ok(_) => panic!("field_name alone must not build a scalar query"),
        Err(err) => err,
    };
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}

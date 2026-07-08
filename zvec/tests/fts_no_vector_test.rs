//! End-to-end test for pure full-text (keyword-only) search with no vector.
//!
//! `SearchQuery::new` and `SearchQueryBuilder::build` both require a query
//! vector, so a keyword-only query can only be built via `SearchQuery::fts`.
//! This mirrors the C `test_fts_end_to_end` case: a schema with an FTS-indexed
//! string field and no vector field, queried with a match string alone.
//!
//! Requires the zvec C library (`libzvec_c_api`). Set `ZVEC_LIB_DIR` before
//! running. Run with: `cargo test --test fts_no_vector_test`

use std::sync::Once;
use zvec_rust::*;

static INIT: Once = Once::new();

fn ensure_initialized() {
    INIT.call_once(|| {
        initialize(None).expect("failed to initialize zvec");
    });
}

#[test]
fn test_fts_only_query_returns_keyword_matches() {
    ensure_initialized();

    let tmp_dir = tempfile::tempdir().unwrap();
    let dir = tmp_dir.path().join("zvec_data");

    // Schema with an FTS-indexed string field and NO vector field.
    let schema = CollectionSchema::builder("fts_no_vector_test")
        .add_field(FieldSchema::new("id", DataType::String, false, 0).unwrap())
        .add_indexed_field(
            "content",
            DataType::String,
            IndexParams::fts(None, None, None).unwrap(),
        )
        .build()
        .expect("failed to build schema");

    let collection = Collection::create_and_open(dir.to_str().unwrap(), &schema, None)
        .expect("failed to create collection");

    // Two docs mention "learning"; one does not.
    let texts = [
        "machine learning is fun",
        "deep learning uses neural networks",
        "vector databases store embeddings",
    ];
    let mut docs = Vec::new();
    for (i, text) in texts.iter().enumerate() {
        let mut doc = Doc::new().unwrap();
        doc.set_pk(&format!("pk_{}", i));
        doc.add_string("id", &format!("pk_{}", i)).unwrap();
        doc.add_string("content", text).unwrap();
        docs.push(doc);
    }
    let doc_refs: Vec<&Doc> = docs.iter().collect();
    let result = collection.insert(&doc_refs).unwrap();
    assert_eq!(result.success_count, texts.len() as u64);
    collection.flush().expect("flush before query");

    // Keyword-only query — no query vector. Builds only via `SearchQuery::fts`.
    let mut fts = Fts::new().unwrap();
    fts.set_match_string("learning").unwrap();
    let query = SearchQuery::fts("content", &fts, 10).expect("build vector-less FTS query");

    let results = collection
        .query(&query)
        .expect("vector-less FTS query must run");

    // Exactly the two "learning" docs must come back, and not the third. A
    // regression where the C query path still demands a dense vector would
    // error out above instead of returning these rows.
    let mut got: Vec<String> = results
        .iter()
        .map(|d| d.get_string("content").unwrap().unwrap())
        .collect();
    got.sort();
    assert_eq!(
        got,
        vec![
            "deep learning uses neural networks".to_string(),
            "machine learning is fun".to_string(),
        ],
        "keyword query must return exactly the two matching rows"
    );
}

//! Coverage for grouped vector search, which the C API exposes as of zvec
//! v0.8.1 through `zvec_collection_group_by_query`.

use std::path::Path;
use std::sync::Once;

use zvec_rust::{
    initialize, Collection, CollectionSchema, DataType, Doc, ErrorCode, FieldSchema,
    FlatQueryParams, GroupBySearchQuery, IndexParams, MetricType, SearchGroup,
};

static INIT: Once = Once::new();

fn ensure_initialized() {
    INIT.call_once(|| {
        initialize(None).expect("failed to initialize zvec");
    });
}

/// Opens a collection shaped for grouped search: two scalar fields to group and
/// filter on, plus a brute-force vector index so the expected order is exact.
fn open_grouped_collection(path: &Path) -> Collection {
    let schema = CollectionSchema::builder("group_by_collection")
        .add_field(FieldSchema::new("id", DataType::Int64, false, 0).expect("id field"))
        .add_field(FieldSchema::new("group_id", DataType::Int64, false, 0).expect("group_id field"))
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            4,
            IndexParams::flat(MetricType::L2).expect("flat index params"),
        )
        .build()
        .expect("grouped search schema");

    Collection::create_and_open(path.to_str().expect("database path"), &schema, None)
        .expect("create grouped search collection")
}

/// Inserts nine documents in three groups of three, placed on the x axis so
/// that the L2 distance of `doc_<id>` to the origin is `id * id`.
fn insert_grouped_docs(collection: &Collection) {
    let mut docs = Vec::new();
    for id in 0..9i64 {
        let mut doc = Doc::new().expect("document");
        doc.set_pk(&format!("doc_{}", id));
        doc.add_i64("id", id).expect("id value");
        doc.add_i64("group_id", id / 3).expect("group_id value");
        doc.add_vector_f32("embedding", &[id as f32, 0.0, 0.0, 0.0])
            .expect("vector value");
        docs.push(doc);
    }

    let refs: Vec<&Doc> = docs.iter().collect();
    let result = collection.insert(&refs).expect("insert documents");
    assert_eq!(result.success_count, 9);
    assert_eq!(result.error_count, 0);
    collection.flush().expect("persist documents");
}

/// Asserts that group `index` holds the `topk` closest documents of
/// `group_id`, in similarity order, carrying exactly the requested projection.
fn assert_group(groups: &[SearchGroup], index: usize, group_id: i64, topk: usize, vectors: bool) {
    let group = groups
        .get(index)
        .unwrap_or_else(|| panic!("expected a group at index {} of {}", index, groups.len()));

    assert_eq!(group.group_by_value, group_id.to_string());
    assert_eq!(group.docs.len(), topk);

    for (offset, doc) in group.docs.iter().enumerate() {
        // Documents are ordered by similarity, so with this data set the offsets
        // walk the group from its closest to its farthest member.
        let id = group_id * 3 + offset as i64;
        assert_eq!(doc.get_pk(), Some(format!("doc_{}", id).as_str()));
        assert_eq!(doc.get_i64("id").expect("id lookup"), Some(id));
        assert_eq!(doc.has_field("embedding"), vectors);
        // The projection asked for "id" only, so the grouping field itself is
        // not part of the returned document.
        assert!(!doc.has_field("group_id"));
        assert!(
            (doc.get_score() - (id * id) as f32).abs() < 1e-3,
            "doc_{} scored {}, expected the L2 distance {}",
            id,
            doc.get_score(),
            id * id
        );
    }
}

#[test]
fn group_by_query_returns_the_best_documents_of_each_group() {
    ensure_initialized();
    let temporary = tempfile::tempdir().expect("temporary directory");
    let collection = open_grouped_collection(&temporary.path().join("grouped"));
    insert_grouped_docs(&collection);

    let mut query =
        GroupBySearchQuery::new("embedding", "group_id", &[0.0; 4], 2, 2).expect("grouped query");
    query.set_output_fields(&["id"]).expect("output fields");
    // The query takes ownership of the index specific search parameters.
    query
        .set_flat_params(FlatQueryParams::new(false, 10.0))
        .expect("flat query params");

    let groups = collection.group_by_query(&query).expect("grouped search");
    assert_eq!(groups.len(), 2, "group_count caps the number of groups");
    assert_group(&groups, 0, 0, 2, false);
    assert_group(&groups, 1, 1, 2, false);
}

#[test]
fn group_by_query_honours_filter_group_count_and_include_vector() {
    ensure_initialized();
    let temporary = tempfile::tempdir().expect("temporary directory");
    let collection = open_grouped_collection(&temporary.path().join("filtered"));
    insert_grouped_docs(&collection);

    let mut query =
        GroupBySearchQuery::new("embedding", "group_id", &[0.0; 4], 3, 1).expect("grouped query");
    query.set_output_fields(&["id"]).expect("output fields");
    query.set_include_vector(true).expect("include vectors");
    query.set_filter("id >= 3").expect("filter");

    let groups = collection.group_by_query(&query).expect("grouped search");
    // Group 0 holds ids 0..3, so the filter drops the whole group rather than
    // returning it empty.
    assert_eq!(groups.len(), 2);
    assert_group(&groups, 0, 1, 1, true);
    assert_group(&groups, 1, 2, 1, true);

    // A filter that matches nothing is an empty result, not an error.
    query.set_filter("id < 0").expect("filter");
    let groups = collection.group_by_query(&query).expect("grouped search");
    assert!(groups.is_empty());
}

#[test]
fn group_by_query_on_an_empty_collection_returns_no_groups() {
    ensure_initialized();
    let temporary = tempfile::tempdir().expect("temporary directory");
    let collection = open_grouped_collection(&temporary.path().join("empty"));

    let query =
        GroupBySearchQuery::new("embedding", "group_id", &[0.0; 4], 2, 3).expect("grouped query");
    let groups = collection.group_by_query(&query).expect("grouped search");
    assert!(groups.is_empty());
}

#[test]
fn group_by_query_rejects_invalid_parameters() {
    ensure_initialized();
    let temporary = tempfile::tempdir().expect("temporary directory");
    let collection = open_grouped_collection(&temporary.path().join("invalid"));
    insert_grouped_docs(&collection);

    let zero_groups =
        GroupBySearchQuery::new("embedding", "group_id", &[0.0; 4], 0, 2).expect("grouped query");
    let err = collection
        .group_by_query(&zero_groups)
        .expect_err("group_count = 0 must be rejected");
    assert_eq!(err.code, ErrorCode::InvalidArgument);

    let zero_topk =
        GroupBySearchQuery::new("embedding", "group_id", &[0.0; 4], 2, 0).expect("grouped query");
    let err = collection
        .group_by_query(&zero_topk)
        .expect_err("group_topk = 0 must be rejected");
    assert_eq!(err.code, ErrorCode::InvalidArgument);

    let missing_field =
        GroupBySearchQuery::new("embedding", "missing", &[0.0; 4], 2, 2).expect("grouped query");
    let err = collection
        .group_by_query(&missing_field)
        .expect_err("an unknown grouping field must be rejected");
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}

#[test]
fn grouped_results_outlive_the_query_and_the_collection() {
    ensure_initialized();
    let temporary = tempfile::tempdir().expect("temporary directory");

    // The engine documents grouped results as valid after the query and the
    // collection are gone: the safe API moves every document into an owning
    // `Doc`, so dropping both here must not invalidate the groups.
    let groups = {
        let collection = open_grouped_collection(&temporary.path().join("outlives"));
        insert_grouped_docs(&collection);

        let mut query = GroupBySearchQuery::new("embedding", "group_id", &[0.0; 4], 2, 2)
            .expect("grouped query");
        query.set_output_fields(&["id"]).expect("output fields");

        let groups = collection.group_by_query(&query).expect("grouped search");
        assert_eq!(groups.len(), 2);
        groups
    };

    assert_group(&groups, 0, 0, 2, false);
    assert_group(&groups, 1, 1, 2, false);
}

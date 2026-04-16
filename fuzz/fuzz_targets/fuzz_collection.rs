#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Once;
use zvec::*;

static INIT: Once = Once::new();

#[derive(arbitrary::Arbitrary, Debug)]
enum Operation {
    Insert,
    Query,
    Fetch,
    Delete,
}

#[derive(arbitrary::Arbitrary, Debug)]
struct FuzzInput {
    operation: Operation,
    pk: String,
    string_value: String,
    i32_value: i32,
    i64_value: i64,
    f32_value: f32,
    vector_data: Vec<f32>,
    topk: i32,
}

fuzz_target!(|input: FuzzInput| {
    INIT.call_once(|| {
        zvec::initialize(None).expect("Failed to initialize zvec");
    });

    let FuzzInput {
        operation,
        pk,
        string_value,
        i32_value,
        i64_value,
        f32_value,
        vector_data,
        topk,
    } = input;

    // Create a temporary directory for the collection
    let tmp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(_) => return,
    };

    // Create schema
    let schema = match CollectionSchema::builder("fuzz_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_field(FieldSchema::new("category", DataType::String, true, 0))
        .add_field(FieldSchema::new("score", DataType::Float, true, 0))
        .add_field(FieldSchema::new("count", DataType::Int64, true, 0))
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            4,
            IndexParams::hnsw(MetricType::Cosine, 16, 200),
        )
        .build()
    {
        Ok(s) => s,
        Err(_) => return,
    };

    // Create collection
    let collection = match Collection::create_and_open(
        tmp_dir.path().to_str().unwrap(),
        &schema,
        None,
    ) {
        Ok(c) => c,
        Err(_) => return,
    };

    match operation {
        Operation::Insert => {
            if vector_data.len() != 4 {
                return;
            }

            let mut doc = match Doc::new() {
                Ok(d) => d,
                Err(_) => return,
            };

            doc.set_pk(&pk);
            let _ = doc.add_string("id", &pk);
            let _ = doc.add_string("category", &string_value);
            let _ = doc.add_f32("score", f32_value);
            let _ = doc.add_i64("count", i64_value);
            let _ = doc.add_vector_f32("embedding", &vector_data);

            let _ = collection.insert(&[&doc]);
        }
        Operation::Query => {
            if vector_data.len() != 4 || topk <= 0 {
                return;
            }

            // First insert some data
            let mut doc = match Doc::new() {
                Ok(d) => d,
                Err(_) => return,
            };

            doc.set_pk(&pk);
            let _ = doc.add_string("id", &pk);
            let _ = doc.add_string("category", &string_value);
            let _ = doc.add_f32("score", f32_value);
            let _ = doc.add_i64("count", i64_value);
            let _ = doc.add_vector_f32("embedding", &vector_data);

            let _ = collection.insert(&[&doc]);

            // Then query
            let query = match VectorQuery::new("embedding", &vector_data, topk) {
                Ok(q) => q,
                Err(_) => return,
            };

            let _ = collection.query(&query);
        }
        Operation::Fetch => {
            // First insert some data
            let mut doc = match Doc::new() {
                Ok(d) => d,
                Err(_) => return,
            };

            doc.set_pk(&pk);
            let _ = doc.add_string("id", &pk);
            let _ = doc.add_string("category", &string_value);
            let _ = doc.add_f32("score", f32_value);
            let _ = doc.add_i64("count", i64_value);

            let _ = collection.insert(&[&doc]);

            // Then fetch
            let _ = collection.fetch(&[pk.as_str()]);
        }
        Operation::Delete => {
            // First insert some data
            let mut doc = match Doc::new() {
                Ok(d) => d,
                Err(_) => return,
            };

            doc.set_pk(&pk);
            let _ = doc.add_string("id", &pk);
            let _ = doc.add_string("category", &string_value);
            let _ = doc.add_f32("score", f32_value);
            let _ = doc.add_i64("count", i64_value);

            let _ = collection.insert(&[&doc]);

            // Then delete
            let _ = collection.delete(&[pk.as_str()]);
        }
    }

    // Clean up
    let _ = collection.close();
});

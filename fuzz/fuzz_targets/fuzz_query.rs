#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Once;
use zvec::{FlatQueryParams, HnswQueryParams, IvfQueryParams, VectorQuery};

static INIT: Once = Once::new();

#[derive(arbitrary::Arbitrary, Debug)]
struct FuzzInput {
    field_name: String,
    topk: i32,
    vector_data: Vec<f32>,
    filter: String,
    include_vector: bool,
    include_doc_id: bool,
    output_fields: Vec<String>,
    ef: i32,
    radius: f32,
    is_linear: bool,
    is_using_refiner: bool,
    nprobe: i32,
    scale_factor: f32,
}

fuzz_target!(|input: FuzzInput| {
    INIT.call_once(|| {
        zvec::initialize(None).expect("Failed to initialize zvec");
    });

    let FuzzInput {
        field_name,
        topk,
        vector_data,
        filter,
        include_vector,
        include_doc_id,
        output_fields,
        ef,
        radius,
        is_linear,
        is_using_refiner,
        nprobe,
        scale_factor,
    } = input;

    if vector_data.is_empty() {
        return;
    }

    // Sanitize field_name to remove null bytes
    let safe_field_name = field_name.replace('\0', "_");

    // Limit topk to prevent memory issues (reasonable upper bound)
    let safe_topk = topk.clamp(1, 10000);

    let _ = VectorQuery::new(&safe_field_name, &vector_data, safe_topk);

    let mut builder = VectorQuery::builder()
        .field_name(&safe_field_name)
        .vector(&vector_data)
        .topk(safe_topk);

    if !filter.is_empty() {
        builder = builder.filter(&filter);
    }

    builder = builder.include_vector(include_vector);
    builder = builder.include_doc_id(include_doc_id);

    if !output_fields.is_empty() {
        let field_refs: Vec<&str> = output_fields.iter().map(|s| s.as_str()).collect();
        builder = builder.output_fields(&field_refs);
    }

    let _ = builder.build();

    let _ = HnswQueryParams::new(ef, radius, is_linear, is_using_refiner);

    let _ = IvfQueryParams::new(nprobe, is_using_refiner, scale_factor);

    let _ = FlatQueryParams::new(is_using_refiner, scale_factor);
});

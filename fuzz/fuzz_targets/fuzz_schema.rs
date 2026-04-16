#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Once;
use zvec::{DataType, FieldSchema, IndexParams, IndexType, MetricType};

static INIT: Once = Once::new();

#[derive(arbitrary::Arbitrary, Debug)]
struct FuzzInput {
    collection_name: String,
    field_name: String,
    dimension: u32,
    metric_type: u32,
    m: i32,
    ef_construction: i32,
    n_list: i32,
    n_iters: i32,
    use_soar: bool,
    enable_range_opt: bool,
    enable_wildcard: bool,
}

fuzz_target!(|input: FuzzInput| {
    INIT.call_once(|| {
        zvec::initialize(None).expect("Failed to initialize zvec");
    });

    let FuzzInput {
        collection_name,
        field_name,
        dimension,
        metric_type,
        m,
        ef_construction,
        n_list,
        n_iters,
        use_soar,
        enable_range_opt,
        enable_wildcard,
    } = input;

    let metric = MetricType::from(metric_type);

    let _ = FieldSchema::new(&field_name, DataType::String, false, 0);
    let _ = FieldSchema::new(&field_name, DataType::Int64, false, 0);
    let _ = FieldSchema::new(&field_name, DataType::Float, false, 0);
    let _ = FieldSchema::new(&field_name, DataType::VectorFp32, false, dimension);

    let _ = IndexParams::hnsw(metric, m, ef_construction);
    let _ = IndexParams::hnsw_with_quantize(metric, m, ef_construction, zvec::QuantizeType::Int8);
    let _ = IndexParams::ivf(metric, n_list, n_iters, use_soar);
    let _ = IndexParams::flat(metric);
    let _ = IndexParams::invert(enable_range_opt, enable_wildcard);

    let _ = zvec::CollectionSchema::builder(&collection_name)
        .add_field(FieldSchema::new(&field_name, DataType::String, false, 0))
        .add_vector_field(
            &format!("{}_vec", field_name),
            DataType::VectorFp32,
            dimension,
            IndexParams::hnsw(metric, m, ef_construction),
        );
});

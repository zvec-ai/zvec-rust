#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Once;
use zvec::Doc;

static INIT: Once = Once::new();

#[derive(arbitrary::Arbitrary, Debug)]
struct FuzzInput {
    field_name: String,
    pk: String,
    string_value: String,
    bool_value: bool,
    i32_value: i32,
    i64_value: i64,
    u32_value: u32,
    u64_value: u64,
    f32_value: f32,
    f64_value: f64,
    vector_data: Vec<f32>,
}

fuzz_target!(|input: FuzzInput| {
    INIT.call_once(|| {
        zvec::initialize(None).expect("Failed to initialize zvec");
    });

    let FuzzInput {
        field_name,
        pk,
        string_value,
        bool_value,
        i32_value,
        i64_value,
        u32_value,
        u64_value,
        f32_value,
        f64_value,
        vector_data,
    } = input;

    let mut doc = match Doc::new() {
        Ok(d) => d,
        Err(_) => return,
    };

    // Sanitize pk and field_name to remove null bytes
    let safe_pk = pk.replace('\0', "_");
    let safe_field_name = field_name.replace('\0', "_");

    doc.set_pk(&safe_pk);
    let _ = doc.get_pk();

    let _ = doc.add_string(&safe_field_name, &string_value);
    let _ = doc.get_string(&safe_field_name);

    let _ = doc.add_bool(&safe_field_name, bool_value);
    let _ = doc.get_bool(&safe_field_name);

    let _ = doc.add_i32(&safe_field_name, i32_value);
    let _ = doc.get_i32(&safe_field_name);

    let _ = doc.add_i64(&safe_field_name, i64_value);
    let _ = doc.get_i64(&safe_field_name);

    let _ = doc.add_u32(&safe_field_name, u32_value);

    let _ = doc.add_u64(&safe_field_name, u64_value);

    let _ = doc.add_f32(&safe_field_name, f32_value);
    let _ = doc.get_f32(&safe_field_name);

    let _ = doc.add_f64(&safe_field_name, f64_value);
    let _ = doc.get_f64(&safe_field_name);

    if !vector_data.is_empty() {
        let _ = doc.add_vector_f32(&safe_field_name, &vector_data);
        let _ = doc.get_vector_f32(&safe_field_name);
    }

    let _ = doc.has_field(&safe_field_name);
    let _ = doc.is_empty();
    let _ = doc.field_count();
    let _ = doc.is_field_null(&safe_field_name);

    let _ = doc.set_field_null(&safe_field_name);
    let _ = doc.remove_field(&safe_field_name);

    doc.clear();
});

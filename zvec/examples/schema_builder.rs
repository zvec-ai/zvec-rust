use zvec::*;

fn main() -> zvec::Result<()> {
    println!("zvec version: {}", version());
    initialize(None)?;

    let data_path = "./zvec_schema_builder_example_data";

    // Example 1: Basic schema with multiple field types
    println!("\n=== Example 1: Basic Schema ===");
    let _basic_schema = CollectionSchema::builder("basic_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_field(FieldSchema::new("name", DataType::String, true, 0))
        .add_field(FieldSchema::new("age", DataType::Int32, true, 0))
        .add_field(FieldSchema::new("price", DataType::Float, true, 0))
        .add_field(FieldSchema::new("is_active", DataType::Bool, true, 0))
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            128,
            IndexParams::flat(MetricType::L2),
        )
        .build()?;

    println!("Basic schema created successfully");

    // Example 2: Schema with HNSW index
    println!("\n=== Example 2: HNSW Index ===");
    let _hnsw_schema = CollectionSchema::builder("hnsw_collection")
        .add_field(FieldSchema::new("pk", DataType::String, false, 0))
        .add_vector_field(
            "vector",
            DataType::VectorFp32,
            256,
            IndexParams::hnsw(MetricType::Cosine, 32, 100),
        )
        .build()?;

    println!("HNSW index created with M=32, ef_construction=100");

    // Example 3: Schema with IVF index
    println!("\n=== Example 3: IVF Index ===");
    let _ivf_schema = CollectionSchema::builder("ivf_collection")
        .add_field(FieldSchema::new("doc_id", DataType::String, false, 0))
        .add_vector_field(
            "feature",
            DataType::VectorFp32,
            512,
            IndexParams::ivf(MetricType::L2, 100, 10, true),
        )
        .build()?;

    println!("IVF index created with nlist=100");

    // Example 4: Schema with quantized HNSW
    println!("\n=== Example 4: Quantized HNSW ===");
    let _quantized_schema = CollectionSchema::builder("quantized_collection")
        .add_field(FieldSchema::new("item_id", DataType::String, false, 0))
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            128,
            IndexParams::hnsw_with_quantize(MetricType::Cosine, 16, 200, QuantizeType::Int8),
        )
        .build()?;

    println!("Quantized HNSW created with Int8 quantization");

    // Example 5: Schema with Invert index
    println!("\n=== Example 5: Invert Index ===");
    let _invert_schema = CollectionSchema::builder("invert_collection")
        .add_field(FieldSchema::new("key", DataType::String, false, 0))
        .add_field(FieldSchema::new("text", DataType::String, true, 0))
        .add_field(FieldSchema::new("timestamp", DataType::Int64, true, 0))
        .add_vector_field(
            "dense_vector",
            DataType::VectorFp32,
            64,
            IndexParams::hnsw(MetricType::Cosine, 16, 100),
        )
        .build()?;

    println!("Schema with vector field created for hybrid search");

    // Example 6: Schema with nullable fields
    println!("\n=== Example 6: Nullable Fields ===");
    let _nullable_schema = CollectionSchema::builder("nullable_collection")
        .add_field(FieldSchema::new("user_id", DataType::String, false, 0))
        .add_field(FieldSchema::new("email", DataType::String, true, 0))
        .add_field(FieldSchema::new("phone", DataType::String, true, 0))
        .add_field(FieldSchema::new("address", DataType::String, true, 0))
        .add_field(FieldSchema::new("rating", DataType::Float, true, 0))
        .add_vector_field(
            "profile_vector",
            DataType::VectorFp32,
            32,
            IndexParams::hnsw(MetricType::Cosine, 8, 50),
        )
        .build()?;

    println!("Nullable schema created with email, phone, address, rating as nullable fields");

    // Example 7: Complex schema with multiple vector fields
    println!("\n=== Example 7: Multiple Vector Fields ===");
    let _multi_vector_schema = CollectionSchema::builder("multi_vector_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_field(FieldSchema::new("title", DataType::String, true, 0))
        .add_field(FieldSchema::new("category", DataType::String, true, 0))
        .add_vector_field(
            "text_embedding",
            DataType::VectorFp32,
            768,
            IndexParams::hnsw(MetricType::Cosine, 16, 100),
        )
        .add_vector_field(
            "image_embedding",
            DataType::VectorFp32,
            512,
            IndexParams::hnsw(MetricType::L2, 16, 100),
        )
        .build()?;

    println!("Multi-vector schema created with text and image embedding fields");

    // Example 8: Schema with different vector types
    println!("\n=== Example 8: Different Vector Types ===");
    let _vector_types_schema = CollectionSchema::builder("vector_types_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_vector_field(
            "fp32_vector",
            DataType::VectorFp32,
            128,
            IndexParams::flat(MetricType::L2),
        )
        .add_vector_field(
            "fp16_vector",
            DataType::VectorFp16,
            128,
            IndexParams::flat(MetricType::L2),
        )
        .add_vector_field(
            "int8_vector",
            DataType::VectorInt8,
            128,
            IndexParams::flat(MetricType::L2),
        )
        .build()?;

    println!("Schema created with FP32, FP16, and INT8 vector types");

    // Clean up
    shutdown()?;

    let _ = std::fs::remove_dir_all(data_path);
    println!("\nDone! All schema examples demonstrated successfully.");

    Ok(())
}

use zvec::*;

fn main() -> zvec::Result<()> {
    println!("zvec version: {}", version());
    initialize(None)?;

    let data_path = "./zvec_vector_search_example_data";

    // Create a simple schema for demonstration
    let schema = CollectionSchema::builder("search_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_field(FieldSchema::new("title", DataType::String, true, 0))
        .add_field(FieldSchema::new("category", DataType::String, true, 0))
        .add_field(FieldSchema::new("price", DataType::Float, true, 0))
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            4,
            IndexParams::hnsw(MetricType::Cosine, 16, 100),
        )
        .build()?;

    let collection = Collection::create_and_open(data_path, &schema, None)?;
    println!("Collection created successfully!");

    // Insert sample documents
    let vectors: Vec<[f32; 4]> = vec![
        [0.1, 0.2, 0.3, 0.4],
        [0.5, 0.6, 0.7, 0.8],
        [0.9, 0.1, 0.2, 0.3],
        [0.4, 0.5, 0.6, 0.7],
        [0.8, 0.9, 0.1, 0.2],
        [0.2, 0.3, 0.4, 0.5],
        [0.6, 0.7, 0.8, 0.9],
        [0.3, 0.4, 0.5, 0.6],
    ];

    let titles = [
        "Product A",
        "Product B",
        "Product C",
        "Product D",
        "Product E",
        "Product F",
        "Product G",
        "Product H",
    ];

    let categories = [
        "electronics",
        "books",
        "electronics",
        "books",
        "clothing",
        "electronics",
        "books",
        "clothing",
    ];
    let prices: [f32; 8] = [99.99, 29.99, 149.99, 19.99, 59.99, 89.99, 24.99, 79.99];

    let mut docs = Vec::new();
    for i in 0..8 {
        let mut doc = Doc::new()?;
        let pk = format!("doc_{}", i);
        doc.set_pk(&pk);
        doc.add_string("id", &pk)?;
        doc.add_string("title", titles[i])?;
        doc.add_string("category", categories[i])?;
        doc.add_f32("price", prices[i])?;
        doc.add_vector_f32("embedding", &vectors[i])?;
        docs.push(doc);
    }

    let doc_refs: Vec<&Doc> = docs.iter().collect();
    collection.insert(&doc_refs)?;
    println!("Inserted {} sample documents", docs.len());

    // Example 1: Simple vector query
    println!("\n=== Example 1: Simple Vector Query ===");
    let query_vector = [0.5, 0.6, 0.7, 0.8];
    let query = VectorQuery::new("embedding", &query_vector, 3)?;
    let results = collection.query(&query)?;

    println!("Top 3 results:");
    for (i, result) in results.iter().enumerate() {
        let pk = result.get_pk().unwrap_or("<unknown>");
        let score = result.get_score();
        println!("  #{}: pk={}, similarity={:.4}", i + 1, pk, score);
    }

    // Example 2: Builder pattern query with custom parameters
    println!("\n=== Example 2: Builder Pattern Query ===");
    let query = VectorQuery::builder()
        .field_name("embedding")
        .vector(&query_vector)
        .topk(5)
        .build()?;

    let results = collection.query(&query)?;

    println!("Top 5 results:");
    for (i, result) in results.iter().enumerate() {
        let pk = result.get_pk().unwrap_or("<unknown>");
        let score = result.get_score();
        println!("  #{}: pk={}, score={:.4}", i + 1, pk, score);
    }

    // Example 3: Query with output fields
    println!("\n=== Example 3: Query with Output Fields ===");
    let query = VectorQuery::builder()
        .field_name("embedding")
        .vector(&query_vector)
        .topk(3)
        .output_fields(&["title", "category", "price"])
        .build()?;

    let results = collection.query(&query)?;

    println!("Top 3 results with output fields:");
    for (i, result) in results.iter().enumerate() {
        let pk = result.get_pk().unwrap_or("<unknown>");
        let score = result.get_score();

        // Get output field values
        let title = result
            .get_string("title")
            .unwrap_or_else(|_| "<none>".to_string());
        let category = result
            .get_string("category")
            .unwrap_or_else(|_| "<none>".to_string());
        let price = result.get_f32("price").unwrap_or(0.0);

        println!(
            "  #{}: pk={}, similarity={:.4}, title='{}', category='{}', price={:.2}",
            i + 1,
            pk,
            score,
            title,
            category,
            price
        );
    }

    // Example 4: Different top_k values
    println!("\n=== Example 4: Different Top K Values ===");
    for topk in [1, 3, 5] {
        let query = VectorQuery::new("embedding", &query_vector, topk)?;
        let results = collection.query(&query)?;
        println!("Top {} results:", topk);
        for (i, result) in results.iter().enumerate() {
            let pk = result.get_pk().unwrap_or("<unknown>");
            let score = result.get_score();
            println!("    #{}: pk={}, similarity={:.4}", i + 1, pk, score);
        }
    }

    // Example 5: Multiple queries with different vectors
    println!("\n=== Example 5: Multiple Queries ===");
    let query_vectors: Vec<[f32; 4]> = vec![[0.1, 0.2, 0.3, 0.4], [0.9, 0.1, 0.2, 0.3]];

    for (idx, qvec) in query_vectors.iter().enumerate() {
        let query = VectorQuery::new("embedding", qvec, 2)?;
        let results = collection.query(&query)?;
        println!("Query #{} results:", idx + 1);
        for (i, result) in results.iter().enumerate() {
            let pk = result.get_pk().unwrap_or("<unknown>");
            let score = result.get_score();
            println!("    #{}: pk={}, similarity={:.4}", i + 1, pk, score);
        }
    }

    // Example 6: Query with all available fields
    println!("\n=== Example 6: Query with All Fields ===");
    let query = VectorQuery::builder()
        .field_name("embedding")
        .vector(&query_vector)
        .topk(2)
        .output_fields(&["id", "title", "category", "price"])
        .build()?;

    let results = collection.query(&query)?;

    println!("Detailed results:");
    for (i, result) in results.iter().enumerate() {
        let pk = result.get_pk().unwrap_or("<unknown>");
        let score = result.get_score();

        let id = result
            .get_string("id")
            .unwrap_or_else(|_| "<none>".to_string());
        let title = result
            .get_string("title")
            .unwrap_or_else(|_| "<none>".to_string());
        let category = result
            .get_string("category")
            .unwrap_or_else(|_| "<none>".to_string());
        let price = result.get_f32("price").unwrap_or(0.0);

        println!("  Result #{}:", i + 1);
        println!("    Primary Key: {}", pk);
        println!("    ID: {}", id);
        println!("    Title: {}", title);
        println!("    Category: {}", category);
        println!("    Price: ${:.2}", price);
        println!("    Similarity: {:.4}", score);
    }

    // Clean up
    collection.close()?;
    shutdown()?;

    let _ = std::fs::remove_dir_all(data_path);
    println!("\nDone! All vector search examples demonstrated successfully.");

    Ok(())
}

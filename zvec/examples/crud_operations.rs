use zvec::*;

fn main() -> zvec::Result<()> {
    println!("zvec version: {}", version());
    initialize(None)?;

    let data_path = "./zvec_crud_example_data";

    // Create schema
    let schema = CollectionSchema::builder("crud_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_field(FieldSchema::new("name", DataType::String, true, 0))
        .add_field(FieldSchema::new("age", DataType::Int32, true, 0))
        .add_field(FieldSchema::new("email", DataType::String, true, 0))
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            4,
            IndexParams::hnsw(MetricType::Cosine, 16, 100),
        )
        .build()?;

    let collection = Collection::create_and_open(data_path, &schema, None)?;
    println!("Collection created successfully!");

    // === INSERT ===
    println!("\n=== INSERT Operation ===");
    let mut docs = Vec::new();

    // Create documents
    for i in 0..5 {
        let mut doc = Doc::new()?;
        let pk = format!("user_{}", i);
        doc.set_pk(&pk);
        doc.add_string("id", &pk)?;
        doc.add_string("name", &format!("User {}", i))?;
        doc.add_i32("age", 20 + i)?;
        doc.add_string("email", &format!("user{}@example.com", i))?;
        doc.add_vector_f32(
            "embedding",
            &[
                i as f32 * 0.1,
                (i + 1) as f32 * 0.1,
                (i + 2) as f32 * 0.1,
                (i + 3) as f32 * 0.1,
            ],
        )?;
        docs.push(doc);
    }

    let doc_refs: Vec<&Doc> = docs.iter().collect();
    let insert_result = collection.insert(&doc_refs)?;
    println!(
        "Inserted {} documents successfully (errors: {})",
        insert_result.success_count, insert_result.error_count
    );

    // === FETCH ===
    println!("\n=== FETCH Operation ===");
    let primary_keys = &["user_0", "user_2", "user_4"];
    let fetched_docs = collection.fetch(primary_keys)?;
    println!("Fetched {} documents by primary keys:", fetched_docs.len());
    for doc in &fetched_docs {
        let pk = doc.get_pk().unwrap_or("<unknown>");
        let name = doc
            .get_string("name")
            .unwrap_or_else(|_| "<none>".to_string());
        let age = doc.get_i32("age").unwrap_or(0);
        println!("  pk={}, name={}, age={}", pk, name, age);
    }

    // === UPDATE ===
    println!("\n=== UPDATE Operation ===");
    let mut update_doc = Doc::new()?;
    update_doc.set_pk("user_1");
    update_doc.add_string("name", "Updated User 1")?;
    update_doc.add_i32("age", 30)?;
    update_doc.add_string("email", "updated1@example.com")?;
    update_doc.add_vector_f32("embedding", &[0.5, 0.6, 0.7, 0.8])?;

    let update_doc_refs: Vec<&Doc> = vec![&update_doc];
    let update_result = collection.update(&update_doc_refs)?;
    println!(
        "Updated {} documents successfully (errors: {})",
        update_result.success_count, update_result.error_count
    );

    // Verify update
    let updated_docs = collection.fetch(&["user_1"])?;
    if let Some(doc) = updated_docs.first() {
        let name = doc
            .get_string("name")
            .unwrap_or_else(|_| "<none>".to_string());
        let age = doc.get_i32("age").unwrap_or(0);
        let email = doc
            .get_string("email")
            .unwrap_or_else(|_| "<none>".to_string());
        println!(
            "  Updated document: name='{}', age={}, email='{}'",
            name, age, email
        );
    }

    // === UPSERT ===
    println!("\n=== UPSERT Operation ===");
    let mut upsert_docs = Vec::new();

    // Document with existing PK (will update)
    let mut upsert_doc1 = Doc::new()?;
    upsert_doc1.set_pk("user_2");
    upsert_doc1.add_string("name", "Upserted User 2")?;
    upsert_doc1.add_i32("age", 35)?;
    upsert_doc1.add_string("email", "upserted2@example.com")?;
    upsert_doc1.add_vector_f32("embedding", &[0.9, 0.8, 0.7, 0.6])?;
    upsert_docs.push(upsert_doc1);

    // Document with new PK (will insert)
    let mut upsert_doc2 = Doc::new()?;
    upsert_doc2.set_pk("user_10");
    upsert_doc2.add_string("name", "New User 10")?;
    upsert_doc2.add_i32("age", 40)?;
    upsert_doc2.add_string("email", "newuser10@example.com")?;
    upsert_doc2.add_vector_f32("embedding", &[1.0, 0.9, 0.8, 0.7])?;
    upsert_docs.push(upsert_doc2);

    let upsert_doc_refs: Vec<&Doc> = upsert_docs.iter().collect();
    let upsert_result = collection.upsert(&upsert_doc_refs)?;
    println!(
        "Upserted {} documents successfully (errors: {})",
        upsert_result.success_count, upsert_result.error_count
    );

    // Verify upsert
    let upsert_verify = collection.fetch(&["user_2", "user_10"])?;
    println!("  Verified upserted documents:");
    for doc in &upsert_verify {
        let pk = doc.get_pk().unwrap_or("<unknown>");
        let name = doc
            .get_string("name")
            .unwrap_or_else(|_| "<none>".to_string());
        println!("    pk={}, name='{}'", pk, name);
    }

    // === STATS ===
    println!("\n=== STATS Operation ===");
    let stats = collection.stats()?;
    println!("Collection statistics:");
    println!("  Total documents: {}", stats.doc_count);
    println!("  Indexes:");
    for (name, completeness) in stats.index_names.iter().zip(&stats.index_completeness) {
        println!("    '{}': {:.1}% complete", name, completeness * 100.0);
    }

    // === QUERY (Vector Search) ===
    println!("\n=== QUERY Operation (Vector Search) ===");
    let query_vector = [0.5, 0.6, 0.7, 0.8];
    let query = VectorQuery::builder()
        .field_name("embedding")
        .vector(&query_vector)
        .topk(3)
        .output_fields(&["name", "age", "email"])
        .build()?;

    let results = collection.query(&query)?;
    println!("Top 3 similar documents:");
    for (i, result) in results.iter().enumerate() {
        let pk = result.get_pk().unwrap_or("<unknown>");
        let score = result.get_score();
        let name = result
            .get_string("name")
            .unwrap_or_else(|_| "<none>".to_string());
        println!(
            "  #{}: pk={}, similarity={:.4}, name='{}'",
            i + 1,
            pk,
            score,
            name
        );
    }

    // === DELETE ===
    println!("\n=== DELETE Operation ===");
    let delete_keys = &["user_0", "user_3"];
    let delete_result = collection.delete(delete_keys)?;
    println!(
        "Deleted {} documents successfully (errors: {})",
        delete_result.success_count, delete_result.error_count
    );

    // Verify deletion
    let remaining_docs = collection.fetch(&["user_0", "user_1", "user_3"])?;
    println!("  Remaining documents after deletion:");
    for doc in &remaining_docs {
        let pk = doc.get_pk().unwrap_or("<unknown>");
        println!("    pk={} (still exists)", pk);
    }

    // === FLUSH ===
    println!("\n=== FLUSH Operation ===");
    collection.flush()?;
    println!("Data flushed to disk successfully");

    // Final stats after all operations
    let final_stats = collection.stats()?;
    println!("\nFinal collection statistics:");
    println!("  Total documents: {}", final_stats.doc_count);

    // Clean up
    collection.close()?;
    shutdown()?;

    let _ = std::fs::remove_dir_all(data_path);
    println!("\nDone! All CRUD operations demonstrated successfully.");

    Ok(())
}

//! Grouped vector search: the best matches per distinct value of a scalar
//! field, instead of a single flat top-k list.
//!
//! A plain top-k search can be swept by one dominant category. Grouping on a
//! scalar field guarantees every group is represented, which is what a
//! "related items" shelf or a diversified result page needs.

use zvec_rust::*;

fn main() -> zvec_rust::Result<()> {
    println!("zvec version: {}", version());
    initialize(None)?;

    let data_path = "./zvec_group_by_search_example_data";

    let schema = CollectionSchema::builder("group_by_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0)?)
        .add_field(FieldSchema::new("title", DataType::String, true, 0)?)
        .add_field(FieldSchema::new("category", DataType::String, false, 0)?)
        .add_field(FieldSchema::new("price", DataType::Float, true, 0)?)
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            4,
            IndexParams::flat(MetricType::L2)?,
        )
        .build()?;

    let collection = Collection::create_and_open(data_path, &schema, None)?;
    println!("Collection created successfully!");

    let categories = [
        "electronics",
        "books",
        "clothing",
        "electronics",
        "books",
        "clothing",
        "electronics",
        "books",
        "clothing",
    ];

    let mut docs = Vec::new();
    for (i, category) in categories.iter().enumerate() {
        let mut doc = Doc::new()?;
        let pk = format!("doc_{}", i);
        doc.set_pk(&pk);
        doc.add_string("id", &pk)?;
        doc.add_string("title", &format!("Product {}", i))?;
        doc.add_string("category", category)?;
        doc.add_f32("price", (i as f32 + 1.0) * 10.0)?;
        // Every document sits on a line, so similarity order follows `i`.
        doc.add_vector_f32("embedding", &[(i as f32) * 0.1, 0.2, 0.3, 0.4])?;
        docs.push(doc);
    }

    let doc_refs: Vec<&Doc> = docs.iter().collect();
    collection.insert(&doc_refs)?;
    collection.flush()?;
    println!("Inserted {} sample documents", docs.len());

    let query_vector = [0.0, 0.2, 0.3, 0.4];

    // Example 1: the two best matches of each of the three categories
    println!("\n=== Example 1: Grouped Search ===");
    let mut query = GroupBySearchQuery::new("embedding", "category", &query_vector, 3, 2)?;
    query.set_output_fields(&["title", "category", "price"])?;

    let groups = collection.group_by_query(&query)?;
    println!("{} group(s) returned:", groups.len());
    for group in &groups {
        println!("  category '{}':", group.group_by_value);
        for doc in &group.docs {
            println!(
                "    pk={}, title='{}', price={:.2}, distance={:.4}",
                doc.get_pk().unwrap_or("<unknown>"),
                doc.get_string("title")?.unwrap_or_default(),
                doc.get_f32("price")?.unwrap_or(0.0),
                doc.get_score()
            );
        }
    }

    // Example 2: filter before grouping, and ask for the vectors back
    println!("\n=== Example 2: Grouped Search with a Filter ===");
    let mut filtered = GroupBySearchQuery::new("embedding", "category", &query_vector, 3, 1)?;
    filtered.set_output_fields(&["title"])?;
    filtered.set_filter("price <= 30.0")?;
    filtered.set_include_vector(true)?;

    for group in collection.group_by_query(&filtered)? {
        let doc = &group.docs[0];
        println!(
            "  category '{}': pk={}, vector={:?}",
            group.group_by_value,
            doc.get_pk().unwrap_or("<unknown>"),
            doc.get_vector_f32("embedding")?
        );
    }

    // Example 3: what a flat top-3 would have returned instead
    println!("\n=== Example 3: Flat Search for Comparison ===");
    let flat = SearchQuery::new("embedding", &query_vector, 3)?;
    for doc in collection.query(&flat)? {
        println!(
            "  pk={}, category='{}'",
            doc.get_pk().unwrap_or("<unknown>"),
            doc.get_string("category")?.unwrap_or_default()
        );
    }
    println!("  -> the flat top-3 misses whole categories the grouped search keeps");

    collection.close()?;
    shutdown()?;

    let _ = std::fs::remove_dir_all(data_path);
    println!("\nDone! All grouped search examples demonstrated successfully.");

    Ok(())
}

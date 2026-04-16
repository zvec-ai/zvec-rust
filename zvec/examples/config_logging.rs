use zvec::*;

fn main() -> zvec::Result<()> {
    println!("zvec version: {}", version());

    // === Example 1: Default Initialization ===
    println!("\n=== Example 1: Default Initialization ===");
    initialize(None)?;
    println!("Initialized with default configuration");

    // Get and display version information
    let version_info = version();
    println!("Library version: {}", version_info);

    // Shutdown to allow re-initialization with custom config
    shutdown()?;

    // === Example 2: Custom Configuration with ConfigDataBuilder ===
    println!("\n=== Example 2: Custom Configuration ===");

    // Create custom configuration using the builder
    let config = ConfigDataBuilder::new()
        .memory_limit(1024 * 1024 * 1024) // 1GB memory limit
        .num_threads(4) // Use 4 threads
        .build();

    println!("Custom configuration created:");
    println!("  Memory limit: {} MB", config.memory_limit / (1024 * 1024));
    println!("  Thread count: {}", config.num_threads);

    // Initialize with custom config
    initialize(Some(&config))?;
    println!("Initialized with custom configuration");

    shutdown()?;

    // === Example 3: Configuration with Console Logging ===
    println!("\n=== Example 3: Configuration with Console Logging ===");

    // Create configuration with console logging enabled
    let config_with_logging = ConfigDataBuilder::new()
        .memory_limit(512 * 1024 * 1024) // 512MB memory limit
        .num_threads(2) // Use 2 threads
        .enable_console_log(true) // Enable console logging
        .build();

    println!("Configuration with logging:");
    println!(
        "  Memory limit: {} MB",
        config_with_logging.memory_limit / (1024 * 1024)
    );
    println!("  Thread count: {}", config_with_logging.num_threads);
    println!(
        "  Console logging: {}",
        config_with_logging.enable_console_log
    );

    // Initialize with logging enabled
    initialize(Some(&config_with_logging))?;
    println!("Initialized with console logging enabled");

    // === Example 4: Create a simple collection to demonstrate logging ===
    println!("\n=== Example 4: Demonstrate Logging with Collection Operations ===");

    let data_path = "./zvec_config_logging_example_data";

    // Create a simple schema
    let schema = CollectionSchema::builder("logging_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_field(FieldSchema::new("name", DataType::String, true, 0))
        .add_vector_field(
            "embedding",
            DataType::VectorFp32,
            4,
            IndexParams::flat(MetricType::L2),
        )
        .build()?;

    println!("Creating collection (you should see log output if console logging is enabled)...");
    let collection = Collection::create_and_open(data_path, &schema, None)?;
    println!("Collection created successfully!");

    // Insert some data
    let mut doc = Doc::new()?;
    doc.set_pk("doc_0");
    doc.add_string("id", "doc_0")?;
    doc.add_string("name", "Test Document")?;
    doc.add_vector_f32("embedding", &[0.1, 0.2, 0.3, 0.4])?;

    let doc_refs: Vec<&Doc> = vec![&doc];
    collection.insert(&doc_refs)?;
    println!("Document inserted (check logs for details)");

    // Query the collection
    let query_vector = [0.1, 0.2, 0.3, 0.4];
    let query = VectorQuery::new("embedding", &query_vector, 1)?;
    let results = collection.query(&query)?;
    println!("Query executed, found {} results", results.len());

    // Get statistics
    let stats = collection.stats()?;
    println!("Collection stats: {} documents", stats.doc_count);

    // === Example 5: Different Configuration Scenarios ===
    println!("\n=== Example 5: Configuration Scenarios ===");

    // Scenario 1: Low memory configuration
    println!("\nScenario 1: Low Memory Configuration");
    let low_mem_config = ConfigDataBuilder::new()
        .memory_limit(256 * 1024 * 1024) // 256MB
        .num_threads(1)
        .enable_console_log(false)
        .build();
    println!(
        "  Memory: {} MB, Threads: {}, Logging: {}",
        low_mem_config.memory_limit / (1024 * 1024),
        low_mem_config.num_threads,
        low_mem_config.enable_console_log
    );

    // Scenario 2: High performance configuration
    println!("\nScenario 2: High Performance Configuration");
    let high_perf_config = ConfigDataBuilder::new()
        .memory_limit(2 * 1024 * 1024 * 1024) // 2GB
        .num_threads(8)
        .enable_console_log(true)
        .build();
    println!(
        "  Memory: {} MB, Threads: {}, Logging: {}",
        high_perf_config.memory_limit / (1024 * 1024),
        high_perf_config.num_threads,
        high_perf_config.enable_console_log
    );

    // Scenario 3: Balanced configuration
    println!("\nScenario 3: Balanced Configuration");
    let balanced_config = ConfigDataBuilder::new()
        .memory_limit(512 * 1024 * 1024) // 512MB
        .num_threads(4)
        .enable_console_log(false)
        .build();
    println!(
        "  Memory: {} MB, Threads: {}, Logging: {}",
        balanced_config.memory_limit / (1024 * 1024),
        balanced_config.num_threads,
        balanced_config.enable_console_log
    );

    // === Example 6: Configuration Best Practices ===
    println!("\n=== Example 6: Configuration Best Practices ===");
    println!("Recommended configurations based on use case:");
    println!("\n1. Development/Testing:");
    println!("   - Memory: 256-512 MB");
    println!("   - Threads: 1-2");
    println!("   - Console Log: true (for debugging)");
    println!("\n2. Production (Small Scale):");
    println!("   - Memory: 1-2 GB");
    println!("   - Threads: 4");
    println!("   - Console Log: false");
    println!("\n3. Production (Large Scale):");
    println!("   - Memory: 4+ GB");
    println!("   - Threads: 8+");
    println!("   - Console Log: false");

    // Clean up
    collection.close()?;
    shutdown()?;

    let _ = std::fs::remove_dir_all(data_path);
    println!("\nDone! Configuration and logging examples demonstrated successfully.");

    Ok(())
}

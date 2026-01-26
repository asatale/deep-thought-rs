// Example demonstrating the Intermediate API (Semantic Bulk Operations)
//
// Run with: cargo run --example intermediate_api

use roaring_bitmap::RoaringBitmap;

fn main() {
    println!("=== Roaring Bitmap: Intermediate API Examples ===\n");

    // Example 1: Consecutive values (database auto-increment IDs)
    example_consecutive();

    // Example 2: Sparse values (user IDs, session IDs)
    example_sparse();

    // Example 3: Dense values (moderate fill rate)
    example_dense();

    // Example 4: Mixed workload
    example_mixed();

    // Example 5: Performance comparison
    example_performance_comparison();

    // Example 6: optimize() is data-driven
    example_optimize_behavior();
}

fn example_consecutive() {
    println!("Example 1: Consecutive Values");
    println!("-------------------------------");

    let mut bm = RoaringBitmap::new();

    // Scenario: Database with 1 million sequential user IDs
    bm.extend_consecutive(0..1_000_000);

    println!("Inserted 1,000,000 consecutive values");
    println!("  Cardinality: {}", bm.len());
    println!("  Memory: {} bytes", bm.memory_usage());
    println!("  Container type: {:?}", bm.container_type(0));

    // Show memory efficiency
    let memory_per_value = bm.memory_usage() as f64 / bm.len() as f64;
    println!(
        "  Memory per value: {:.4} bytes (Run container is very compact!)",
        memory_per_value
    );
    println!();
}

fn example_sparse() {
    println!("Example 2: Sparse Values");
    println!("-------------------------");

    let mut bm = RoaringBitmap::new();

    // Scenario: Tracking which of 1 million possible user IDs are active
    // Only 1000 users are active (0.1% density)
    let active_users: Vec<u32> = (0..1000).map(|i| i * 1000).collect();
    bm.extend_sparse(active_users);

    println!("Inserted 1,000 sparse values (gaps of 1000)");
    println!("  Cardinality: {}", bm.len());
    println!("  Memory: {} bytes", bm.memory_usage());

    // Verify some values
    assert!(bm.contains(0));
    assert!(bm.contains(500_000));
    assert!(bm.contains(999_000));
    assert!(!bm.contains(1));
    assert!(!bm.contains(500));

    println!("  Values verified: ✓");
    println!();
}

fn example_dense() {
    println!("Example 3: Dense Values");
    println!("------------------------");

    let mut bm = RoaringBitmap::new();

    // Scenario: Tracking even-numbered orders (50% density)
    bm.extend_dense((0..100_000).filter(|x| x % 2 == 0));

    println!("Inserted 50,000 values (every even number in 0..100,000)");
    println!("  Cardinality: {}", bm.len());
    println!("  Memory: {} bytes", bm.memory_usage());

    let stats = bm.container_stats();
    println!("  Number of containers: {}", stats.len());
    for (key, container_type, cardinality) in &stats {
        println!(
            "    Container {}: {} ({} values)",
            key, container_type, cardinality
        );
    }
    println!();
}

fn example_mixed() {
    println!("Example 4: Mixed Workload");
    println!("--------------------------");

    let mut bm = RoaringBitmap::new();

    // Combine different patterns in one bitmap
    bm.extend_consecutive(0..10_000); // Recent sequential IDs
    bm.extend_sparse([50_000, 100_000, 150_000]); // Sparse admin users
    bm.extend_dense((200_000..210_000).filter(|x| x % 2 == 0)); // Dense batch

    println!("Combined three different patterns:");
    println!("  - Consecutive: 10,000 values");
    println!("  - Sparse: 3 values");
    println!("  - Dense: 5,000 values");
    println!("  Total cardinality: {}", bm.len());
    println!("  Total memory: {} bytes", bm.memory_usage());

    let stats = bm.container_stats();
    println!("  Containers created: {}", stats.len());
    println!();
}

fn example_performance_comparison() {
    println!("Example 5: Performance Comparison");
    println!("-----------------------------------");

    use std::time::Instant;

    // Method 1: Regular insert
    let start = Instant::now();
    let mut bm1 = RoaringBitmap::new();
    for i in 0..100_000 {
        bm1.insert(i);
    }
    let time1 = start.elapsed();

    // Method 2: Semantic method
    let start = Instant::now();
    let mut bm2 = RoaringBitmap::new();
    bm2.extend_consecutive(0..100_000);
    let time2 = start.elapsed();

    println!("Inserting 100,000 consecutive values:");
    println!("  Regular insert():        {:?}", time1);
    println!("  extend_consecutive():    {:?}", time2);
    println!(
        "  Speedup: {:.2}x faster",
        time1.as_secs_f64() / time2.as_secs_f64()
    );

    // Both should have identical contents
    assert_eq!(bm1.len(), bm2.len());
    println!("  Results verified: ✓");
    println!();
}

fn example_optimize_behavior() {
    println!("Example 6: optimize() is Data-Driven");
    println!("--------------------------------------");

    let mut bm = RoaringBitmap::new();

    // Create Run container via semantic method
    bm.extend_consecutive(0..1000);
    let before_type = bm.container_type(0);
    println!("After extend_consecutive(): {:?}", before_type);

    // Fragment the data heavily
    for i in (0..1000).step_by(2) {
        bm.remove(i);
    }
    println!("After removing every other value (500 removals):");
    println!("  Cardinality: {}", bm.len());

    // optimize() analyzes actual data (now fragmented)
    bm.optimize();
    let after_type = bm.container_type(0);
    println!("After optimize(): {:?}", after_type);

    println!("\nKey point: optimize() is data-driven, not hint-driven.");
    println!("It converts containers based on actual data patterns,");
    println!("not on how data was originally inserted.");
    println!();
}

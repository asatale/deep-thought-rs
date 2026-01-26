// Performance benchmarks
//
// These tests are marked with #[ignore] to prevent them from running
// during normal test execution (cargo test), as they:
// - Process large datasets (up to 1,000,000 elements)
// - Measure timing rather than verifying correctness
// - Can be slow and flaky on CI machines
//
// To run these benchmarks:
//   cargo test --test performance -- --ignored --nocapture
//
// To run a specific benchmark:
//   cargo test --test performance perf_memory_usage_overhead -- --ignored --nocapture

use crate::benchmarks::format_duration;
use roaring_bitmap::RoaringBitmap;
use std::time::Instant;

#[test]
#[ignore]
fn perf_memory_usage_overhead() {
    println!("\n=== MEMORY USAGE API OVERHEAD ===");

    let mut bm = RoaringBitmap::new();

    // Add various containers
    for i in 0..1000 {
        bm.insert(i);
    }
    for i in 0..8192 {
        if i % 2 == 0 {
            bm.insert(65536 + i);
        }
    }
    for i in 0..10000 {
        bm.insert(131072 + i);
    }
    bm.optimize();

    // Measure overhead of memory_usage() call
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = bm.memory_usage();
    }
    let duration = start.elapsed();

    println!(
        "  10,000 calls to memory_usage(): {} total, {} per call",
        format_duration(duration.as_nanos()),
        format_duration(duration.as_nanos() / 10000)
    );

    // Measure detailed variant
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = bm.memory_usage_detailed();
    }
    let duration = start.elapsed();

    println!(
        "  10,000 calls to memory_usage_detailed(): {} total, {} per call",
        format_duration(duration.as_nanos()),
        format_duration(duration.as_nanos() / 10000)
    );
}

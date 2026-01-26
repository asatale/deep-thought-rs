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
//   cargo test --test performance perf_mixed_workload -- --ignored --nocapture

use crate::benchmarks::{format_duration, format_ops_per_sec};
use roaring_bitmap::RoaringBitmap;
use std::time::Instant;

#[test]
#[ignore]
fn perf_mixed_workload() {
    println!("\n=== MIXED WORKLOAD PERFORMANCE ===");
    println!("  Pattern: 50% insert, 30% lookup, 20% remove");

    let operations = 100_000;
    let mut bm = RoaringBitmap::new();

    let start = Instant::now();

    for i in 0..operations {
        let op = i % 10;
        if op < 5 {
            // 50% insert
            bm.insert(i);
        } else if op < 8 {
            // 30% lookup
            bm.contains(i / 2);
        } else {
            // 20% remove
            bm.remove(i / 2);
        }
    }

    let duration = start.elapsed();

    println!(
        "  {} operations: {} total, {} per op, {}",
        operations,
        format_duration(duration.as_nanos()),
        format_duration(duration.as_nanos() / operations as u128),
        format_ops_per_sec(operations as usize, duration.as_nanos())
    );
    println!("  Final size: {} values", bm.len());
}

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
//   cargo test --test performance perf_iterator -- --ignored --nocapture

use crate::benchmarks::{format_duration, format_ops_per_sec};
use roaring_bitmap::RoaringBitmap;
use std::time::Instant;

#[test]
#[ignore]
fn perf_iterator() {
    println!("\n=== ITERATOR PERFORMANCE ===");

    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        // Pre-populate
        for i in 0..size {
            bm.insert(i);
        }

        // Measure iteration
        let start = Instant::now();
        let count = bm.iter().count();
        let duration = start.elapsed();

        println!(
            "  {} values: {} total, {} per value, {}",
            size,
            format_duration(duration.as_nanos()),
            format_duration(duration.as_nanos() / size as u128),
            format_ops_per_sec(count, duration.as_nanos())
        );

        assert_eq!(count, size as usize);
    }
}

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
//   cargo test --test performance perf_set_operations -- --ignored --nocapture

use crate::benchmarks::format_duration;
use roaring_bitmap::RoaringBitmap;
use std::time::Instant;

#[test]
#[ignore]
fn perf_set_operations() {
    println!("\n=== SET OPERATIONS PERFORMANCE ===");

    let size = 100_000u32;

    let mut bm1 = RoaringBitmap::new();
    let mut bm2 = RoaringBitmap::new();

    // bm1: 0, 2, 4, 6, ...
    for i in 0..size {
        if i % 2 == 0 {
            bm1.insert(i);
        }
    }

    // bm2: 0, 3, 6, 9, ...
    for i in 0..size {
        if i % 3 == 0 {
            bm2.insert(i);
        }
    }

    println!(
        "  Dataset: {} values in bm1, {} values in bm2",
        bm1.len(),
        bm2.len()
    );

    // Union
    {
        let start = Instant::now();
        let result = bm1.union(&bm2);
        let duration = start.elapsed();
        println!(
            "  Union: {} ({} result values)",
            format_duration(duration.as_nanos()),
            result.len()
        );
    }

    // Intersection
    {
        let start = Instant::now();
        let result = bm1.intersection(&bm2);
        let duration = start.elapsed();
        println!(
            "  Intersection: {} ({} result values)",
            format_duration(duration.as_nanos()),
            result.len()
        );
    }

    // Difference
    {
        let start = Instant::now();
        let result = bm1.difference(&bm2);
        let duration = start.elapsed();
        println!(
            "  Difference: {} ({} result values)",
            format_duration(duration.as_nanos()),
            result.len()
        );
    }

    // Symmetric Difference
    {
        let start = Instant::now();
        let result = bm1.symmetric_difference(&bm2);
        let duration = start.elapsed();
        println!(
            "  Symmetric Difference: {} ({} result values)",
            format_duration(duration.as_nanos()),
            result.len()
        );
    }
}

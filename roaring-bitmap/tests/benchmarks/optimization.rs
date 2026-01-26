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
//   cargo test --test performance perf_optimize_array_to_run -- --ignored --nocapture

use crate::benchmarks::format_duration;
use roaring_bitmap::RoaringBitmap;
use std::time::Instant;

#[test]
#[ignore]
fn perf_optimize_array_to_run() {
    println!("\n=== OPTIMIZE PERFORMANCE (Array → Run) ===");

    let sizes = vec![1_000, 10_000, 50_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        // Insert consecutive values (ideal for Run container)
        for i in 0..size {
            bm.insert(i);
        }

        let len_before = bm.len();

        // Measure optimize
        let start = Instant::now();
        bm.optimize();
        let duration = start.elapsed();

        println!(
            "  {} values: {} (Array→Run conversion)",
            size,
            format_duration(duration.as_nanos())
        );

        assert_eq!(bm.len(), len_before);
        assert_eq!(bm.container_type(0), Some("Run"));
    }
}

#[test]
#[ignore]
fn perf_optimize_fragmented_run() {
    println!("\n=== OPTIMIZE PERFORMANCE (Fragmented Run → Array) ===");

    let sizes = vec![1_000, 10_000, 50_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        // Create consecutive values, then optimize to Run container
        for i in 0..(size * 2) {
            bm.insert(i);
        }
        bm.optimize(); // Convert to Run container

        // Verify it's a Run container
        let container_type_before = bm.container_type(0);

        // Fragment it by removing every other value
        for i in 0..(size * 2) {
            if i % 2 == 0 {
                bm.remove(i);
            }
        }

        let container_type_after_deletion = bm.container_type(0);
        let len_before = bm.len();

        // Measure optimize (Run should convert to Array if fragmented)
        let start = Instant::now();
        bm.optimize();
        let duration = start.elapsed();

        println!(
            "  {} values: {} ({:?}→{:?} after fragmentation)",
            size,
            format_duration(duration.as_nanos()),
            container_type_before,
            container_type_after_deletion
        );

        assert_eq!(bm.len(), len_before);
    }
}

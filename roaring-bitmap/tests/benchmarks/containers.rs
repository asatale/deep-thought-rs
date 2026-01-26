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
//   cargo test --test performance perf_container_type_comparison -- --ignored --nocapture

use crate::benchmarks::format_duration;
use roaring_bitmap::RoaringBitmap;
use std::time::Instant;

#[test]
#[ignore]
fn perf_container_type_comparison() {
    println!("\n=== CONTAINER TYPE PERFORMANCE COMPARISON ===");

    // Test 1: Array container (sparse, under threshold)
    {
        let size = 2_000u32; // Keep under 4096 threshold
        let mut bm = RoaringBitmap::new();
        for i in 0..size {
            bm.insert(i * 10); // Sparse, non-consecutive
        }

        let container_type = bm.container_type(0).expect("Container should exist");

        let start = Instant::now();
        for i in 0..size {
            assert!(bm.contains(i * 10));
        }
        let duration = start.elapsed();

        println!(
            "  Array container ({} values): {} per lookup",
            size,
            format_duration(duration.as_nanos() / size as u128)
        );
        assert_eq!(container_type, "Array");
    }

    // Test 2: Bitmap container (dense, non-consecutive)
    {
        let mut bm = RoaringBitmap::new();
        // Insert enough non-consecutive values to trigger Bitmap (need 4096+)
        for i in 0..8192 {
            if i % 2 == 0 {
                bm.insert(i);
            }
        }

        let container_type = bm.container_type(0).expect("Container should exist");
        let size = bm.len();

        let start = Instant::now();
        for i in 0..8192 {
            if i % 2 == 0 {
                assert!(bm.contains(i));
            }
        }
        let duration = start.elapsed();

        println!(
            "  {} container ({} values): {} per lookup",
            container_type,
            size,
            format_duration(duration.as_nanos() / (size as u128))
        );
        assert_eq!(container_type, "Bitmap");
    }

    // Test 3: Run container (consecutive)
    {
        let size = 5_000u32;
        let mut bm = RoaringBitmap::new();
        for i in 0..size {
            bm.insert(i);
        }
        bm.optimize(); // Convert to Run

        let container_type = bm.container_type(0).expect("Container should exist");

        let start = Instant::now();
        for i in 0..size {
            assert!(bm.contains(i));
        }
        let duration = start.elapsed();

        println!(
            "  Run container ({} values): {} per lookup",
            size,
            format_duration(duration.as_nanos() / size as u128)
        );
        assert_eq!(container_type, "Run");
    }
}

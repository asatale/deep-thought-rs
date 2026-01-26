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
//   cargo test --test performance perf_insert_sequential -- --ignored --nocapture

use crate::benchmarks::{format_duration, format_ops_per_sec};
use roaring_bitmap::RoaringBitmap;
use std::time::Instant;

#[test]
#[ignore]
fn perf_insert_sequential() {
    println!("\n=== INSERT PERFORMANCE (Sequential) ===");

    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        let start = Instant::now();
        for i in 0..size {
            bm.insert(i);
        }
        let duration = start.elapsed();

        println!(
            "  {} inserts: {} total, {} per op, {}",
            size,
            format_duration(duration.as_nanos()),
            format_duration(duration.as_nanos() / size as u128),
            format_ops_per_sec(size as usize, duration.as_nanos())
        );

        assert_eq!(bm.len(), size as u64);
    }
}

#[test]
#[ignore]
fn perf_insert_random() {
    println!("\n=== INSERT PERFORMANCE (Random - Sparse) ===");

    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        // Insert with gaps to avoid consecutive sequences
        let start = Instant::now();
        for i in 0..size {
            bm.insert(i * 7 + 13); // Prime numbers to spread values
        }
        let duration = start.elapsed();

        println!(
            "  {} inserts: {} total, {} per op, {}",
            size,
            format_duration(duration.as_nanos()),
            format_duration(duration.as_nanos() / size as u128),
            format_ops_per_sec(size as usize, duration.as_nanos())
        );

        assert_eq!(bm.len(), size as u64);
    }
}

#[test]
#[ignore]
fn perf_remove_sequential() {
    println!("\n=== REMOVE PERFORMANCE (Sequential) ===");

    let sizes = vec![1_000, 10_000, 100_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        // Pre-populate
        for i in 0..size {
            bm.insert(i);
        }

        // Measure removal
        let start = Instant::now();
        for i in 0..size {
            assert!(bm.remove(i));
        }
        let duration = start.elapsed();

        println!(
            "  {} removes: {} total, {} per op, {}",
            size,
            format_duration(duration.as_nanos()),
            format_duration(duration.as_nanos() / size as u128),
            format_ops_per_sec(size as usize, duration.as_nanos())
        );

        assert_eq!(bm.len(), 0);
    }
}

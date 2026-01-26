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
//   cargo test --test performance perf_lookup_sequential -- --ignored --nocapture

use crate::benchmarks::{format_duration, format_ops_per_sec};
use roaring_bitmap::RoaringBitmap;
use std::time::Instant;

#[test]
#[ignore]
fn perf_lookup_sequential() {
    println!("\n=== LOOKUP PERFORMANCE (Sequential - Hit) ===");

    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        // Pre-populate
        for i in 0..size {
            bm.insert(i);
        }

        // Measure lookup
        let start = Instant::now();
        for i in 0..size {
            assert!(bm.contains(i));
        }
        let duration = start.elapsed();

        println!(
            "  {} lookups: {} total, {} per op, {}",
            size,
            format_duration(duration.as_nanos()),
            format_duration(duration.as_nanos() / size as u128),
            format_ops_per_sec(size as usize, duration.as_nanos())
        );
    }
}

#[test]
#[ignore]
fn perf_lookup_sparse() {
    println!("\n=== LOOKUP PERFORMANCE (Sparse - Hit) ===");

    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        // Pre-populate with sparse data
        for i in 0..size {
            bm.insert(i * 7 + 13);
        }

        // Measure lookup
        let start = Instant::now();
        for i in 0..size {
            assert!(bm.contains(i * 7 + 13));
        }
        let duration = start.elapsed();

        println!(
            "  {} lookups: {} total, {} per op, {}",
            size,
            format_duration(duration.as_nanos()),
            format_duration(duration.as_nanos() / size as u128),
            format_ops_per_sec(size as usize, duration.as_nanos())
        );
    }
}

#[test]
#[ignore]
fn perf_lookup_miss() {
    println!("\n=== LOOKUP PERFORMANCE (Miss) ===");

    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    for size in sizes {
        let mut bm = RoaringBitmap::new();

        // Pre-populate with even numbers
        for i in 0..size {
            bm.insert(i * 2);
        }

        // Measure lookup of odd numbers (misses)
        let start = Instant::now();
        for i in 0..size {
            assert!(!bm.contains(i * 2 + 1));
        }
        let duration = start.elapsed();

        println!(
            "  {} lookups: {} total, {} per op, {}",
            size,
            format_duration(duration.as_nanos()),
            format_duration(duration.as_nanos() / size as u128),
            format_ops_per_sec(size as usize, duration.as_nanos())
        );
    }
}

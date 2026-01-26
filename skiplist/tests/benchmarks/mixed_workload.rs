use crate::benchmarks::{format_duration, format_ops_per_sec, time_operation, BenchItem};
use skiplist::{SkipList, SkipListEntry};

#[test]
#[ignore]
fn perf_mixed_insert_lookup() {
    const N: usize = 50_000;
    let mut list = SkipList::new();

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            // Insert
            list.insert(Box::new(BenchItem::new(i))).unwrap();

            // Lookup previous elements
            if i > 0 {
                assert!(list.get(&(i - 1)).is_some());
            }
        }
    });

    println!("\n=== Mixed Insert/Lookup Performance ===");
    println!("Operations: {} inserts + {} lookups", N, N - 1);
    println!("Total time: {}", format_duration(elapsed));
    println!("Throughput: {}", format_ops_per_sec(N + (N - 1), elapsed));
}

#[test]
#[ignore]
fn perf_mixed_insert_remove() {
    const N: usize = 50_000;
    let mut list = SkipList::new();

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            // Insert two elements
            list.insert(Box::new(BenchItem::new(i * 2))).unwrap();
            list.insert(Box::new(BenchItem::new(i * 2 + 1))).unwrap();

            // Remove one element
            if i > 0 {
                list.remove_by_key(&(i - 1));
            }
        }
    });

    println!("\n=== Mixed Insert/Remove Performance ===");
    println!("Operations: {} inserts + {} removes", N * 2, N - 1);
    println!("Total time: {}", format_duration(elapsed));
    println!(
        "Throughput: {}",
        format_ops_per_sec(N * 2 + (N - 1), elapsed)
    );
}

#[test]
#[ignore]
fn perf_mixed_all_operations() {
    const N: usize = 30_000;
    let mut list = SkipList::new();

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            // Insert
            list.insert(Box::new(BenchItem::new(i))).unwrap();

            // Lookup
            if i > 0 {
                let _ = list.get(&(i / 2));
            }

            // Navigate
            if i > 0 {
                let _ = list.first();
            }

            // Remove old elements
            if i > 100 {
                list.remove_by_key(&(i - 100));
            }
        }
    });

    let total_ops = N * 4; // Approximate

    println!("\n=== Mixed All Operations Performance ===");
    println!("Operations: ~{} mixed operations", total_ops);
    println!("Total time: {}", format_duration(elapsed));
    println!("Throughput: {}", format_ops_per_sec(total_ops, elapsed));
}

#[test]
#[ignore]
fn perf_producer_consumer_pattern() {
    const N: usize = 50_000;
    let mut list = SkipList::new();

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            // Producer: Insert new elements
            list.insert(Box::new(BenchItem::new(i))).unwrap();

            // Consumer: Process and remove oldest
            if list.len() > 1000 {
                if let Some(first) = list.first() {
                    let first_key = *first.key();
                    list.remove_by_key(&first_key);
                }
            }
        }
    });

    println!("\n=== Producer/Consumer Pattern Performance ===");
    println!("Operations: {} with sliding window", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_range_scan_pattern() {
    const N: usize = 50_000;
    let list = crate::benchmarks::create_populated_list(N);

    let elapsed = time_operation(|| {
        // Simulate range scans
        for start in (0..N as i32).step_by(100) {
            let mut count = 0;
            let mut current = list.successor(&(start - 1));

            while let Some(item) = current {
                let item_key = *item.key();
                if item_key >= start + 100 {
                    break;
                }
                count += 1;
                current = list.successor(&item_key);
            }

            assert_eq!(count, 100);
        }
    });

    let num_scans = N / 100;
    let total_ops = num_scans * 100;

    println!("\n=== Range Scan Pattern Performance ===");
    println!("Operations: {} scans of 100 elements each", num_scans);
    println!("Total time: {}", format_duration(elapsed));
    println!("Throughput: {}", format_ops_per_sec(total_ops, elapsed));
}

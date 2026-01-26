use crate::benchmarks::{format_duration, format_ops_per_sec, time_operation, BenchItem};
use skiplist::SkipList;

#[test]
#[ignore]
fn perf_insert_sequential() {
    const N: usize = 100_000;
    let mut list = SkipList::new();

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            list.insert(Box::new(BenchItem::new(i))).unwrap();
        }
    });

    println!("\n=== Sequential Insertion Performance ===");
    println!("Operations: {} insertions", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_insert_reverse() {
    const N: usize = 100_000;
    let mut list = SkipList::new();

    let elapsed = time_operation(|| {
        for i in (0..N as i32).rev() {
            list.insert(Box::new(BenchItem::new(i))).unwrap();
        }
    });

    println!("\n=== Reverse Insertion Performance ===");
    println!("Operations: {} insertions", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_insert_random() {
    const N: usize = 100_000;
    let mut list = SkipList::new();

    // Generate pseudo-random sequence using simple LCG
    let mut keys = Vec::with_capacity(N);
    let mut x = 12345u32;
    for _ in 0..N {
        x = x.wrapping_mul(1103515245).wrapping_add(12345);
        keys.push((x / 65536) as i32);
    }

    let elapsed = time_operation(|| {
        for &key in &keys {
            let _ = list.insert(Box::new(BenchItem::new(key)));
        }
    });

    println!("\n=== Random Insertion Performance ===");
    println!("Operations: {} insertions", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_insert_duplicate_attempts() {
    const N: usize = 50_000;
    let mut list = SkipList::new();

    // Pre-populate half
    for i in 0..N as i32 {
        list.insert(Box::new(BenchItem::new(i))).unwrap();
    }

    // Try to insert again (will fail for existing keys)
    let elapsed = time_operation(|| {
        for i in 0..(N * 2) as i32 {
            let _ = list.insert(Box::new(BenchItem::new(i)));
        }
    });

    println!("\n=== Duplicate Insertion Attempts Performance ===");
    println!("Operations: {} insertions (50% duplicates)", N * 2);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / (N * 2) as u128));
    println!("Throughput: {}", format_ops_per_sec(N * 2, elapsed));
}

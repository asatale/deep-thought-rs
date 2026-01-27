use crate::benchmarks::{
    create_populated_list, format_duration, format_ops_per_sec, time_operation,
};
use skiplist::SkipListEntry;

#[test]
#[ignore]
fn perf_first_access() {
    const N: usize = 100_000;
    let list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for _ in 0..N {
            assert!(list.first().is_some());
        }
    });

    println!("\n=== First Element Access Performance ===");
    println!("Operations: {} first() calls", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_successor_sequential() {
    const N: usize = 50_000;
    let list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for i in 0..(N - 1) as i32 {
            assert!(list.successor(&i).is_some());
        }
    });

    println!("\n=== Sequential Successor Performance ===");
    println!("Operations: {} successor calls", N - 1);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / (N - 1) as u128));
    println!("Throughput: {}", format_ops_per_sec(N - 1, elapsed));
}

#[test]
#[ignore]
fn perf_full_iteration() {
    const N: usize = 50_000;
    let list = create_populated_list(N);

    let elapsed = time_operation(|| {
        let mut count = 0;
        let mut current = list.first();

        while let Some(item) = current {
            count += 1;
            let key = *item.key();
            current = list.successor(&key);
        }

        assert_eq!(count, N);
    });

    println!("\n=== Full List Iteration Performance ===");
    println!("Operations: {} elements traversed", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_successor_random() {
    const N: usize = 50_000;
    let list = create_populated_list(N);

    // Generate pseudo-random lookup sequence
    let mut keys = Vec::with_capacity(N);
    let mut x = 98765u32;
    for _ in 0..N {
        x = x.wrapping_mul(1103515245).wrapping_add(12345);
        keys.push(((x / 65536) % N as u32) as i32);
    }

    let elapsed = time_operation(|| {
        for &key in &keys {
            let _ = list.successor(&key);
        }
    });

    println!("\n=== Random Successor Performance ===");
    println!("Operations: {} successor calls", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_successor_sparse() {
    const N: usize = 10_000;
    let mut list = create_populated_list(0);

    // Create sparse list with gaps of 10
    for i in 0..N as i32 {
        list.insert(Box::new(crate::benchmarks::BenchItem::new(i * 10)))
            .unwrap();
    }

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            // Query in between elements (forces search)
            let query_key = i * 10 + 5;
            let _ = list.successor(&query_key);
        }
    });

    println!("\n=== Sparse Successor Performance ===");
    println!("Operations: {} successor calls (sparse keys)", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

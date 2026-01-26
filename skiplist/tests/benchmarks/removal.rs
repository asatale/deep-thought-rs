use crate::benchmarks::{
    create_populated_list, format_duration, format_ops_per_sec, time_operation,
};

#[test]
#[ignore]
fn perf_remove_sequential() {
    const N: usize = 50_000;
    let mut list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            assert!(list.remove(&i).is_some());
        }
    });

    println!("\n=== Sequential Remove Performance ===");
    println!("Operations: {} removals", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
    assert!(list.is_empty());
}

#[test]
#[ignore]
fn perf_remove_reverse() {
    const N: usize = 50_000;
    let mut list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for i in (0..N as i32).rev() {
            assert!(list.remove(&i).is_some());
        }
    });

    println!("\n=== Reverse Remove Performance ===");
    println!("Operations: {} removals", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
    assert!(list.is_empty());
}

#[test]
#[ignore]
fn perf_remove_by_key_sequential() {
    const N: usize = 50_000;
    let mut list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            assert!(list.remove_by_key(&i));
        }
    });

    println!("\n=== Sequential Remove By Key Performance ===");
    println!("Operations: {} removals", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
    assert!(list.is_empty());
}

#[test]
#[ignore]
fn perf_remove_alternating() {
    const N: usize = 50_000;
    let mut list = create_populated_list(N);

    // Remove every other element
    let elapsed = time_operation(|| {
        for i in (0..N as i32).step_by(2) {
            assert!(list.remove_by_key(&i));
        }
    });

    println!("\n=== Alternating Remove Performance ===");
    println!("Operations: {} removals (every other element)", N / 2);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / (N / 2) as u128));
    println!("Throughput: {}", format_ops_per_sec(N / 2, elapsed));
    assert_eq!(list.len(), N / 2);
}

#[test]
#[ignore]
fn perf_remove_missing_keys() {
    const N: usize = 50_000;
    let mut list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            let missing_key = N as i32 + i;
            assert!(!list.remove_by_key(&missing_key));
        }
    });

    println!("\n=== Missing Key Remove Performance ===");
    println!("Operations: {} remove attempts (all miss)", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
    assert_eq!(list.len(), N);
}

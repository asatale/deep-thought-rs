use crate::benchmarks::{
    create_populated_list, format_duration, format_ops_per_sec, time_operation,
};

#[test]
#[ignore]
fn perf_get_sequential() {
    const N: usize = 100_000;
    let list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            assert!(list.get(&i).is_some());
        }
    });

    println!("\n=== Sequential Lookup Performance ===");
    println!("Operations: {} lookups", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_get_random() {
    const N: usize = 100_000;
    let list = create_populated_list(N);

    // Generate pseudo-random lookup sequence
    let mut keys = Vec::with_capacity(N);
    let mut x = 54321u32;
    for _ in 0..N {
        x = x.wrapping_mul(1103515245).wrapping_add(12345);
        keys.push(((x / 65536) % N as u32) as i32);
    }

    let elapsed = time_operation(|| {
        for &key in &keys {
            let _ = list.get(&key);
        }
    });

    println!("\n=== Random Lookup Performance ===");
    println!("Operations: {} lookups", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_get_missing_keys() {
    const N: usize = 100_000;
    let list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            let missing_key = N as i32 + i;
            assert!(list.get(&missing_key).is_none());
        }
    });

    println!("\n=== Missing Key Lookup Performance ===");
    println!("Operations: {} lookups (all miss)", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

#[test]
#[ignore]
fn perf_get_mut() {
    const N: usize = 50_000;
    let mut list = create_populated_list(N);

    let elapsed = time_operation(|| {
        for i in 0..N as i32 {
            if let Some(item) = list.get_mut(&i) {
                item.data[0] = (i % 256) as u8;
            }
        }
    });

    println!("\n=== Mutable Lookup Performance ===");
    println!("Operations: {} mutable lookups", N);
    println!("Total time: {}", format_duration(elapsed));
    println!("Average: {}", format_duration(elapsed / N as u128));
    println!("Throughput: {}", format_ops_per_sec(N, elapsed));
}

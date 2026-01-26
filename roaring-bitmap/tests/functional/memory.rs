use roaring_bitmap::RoaringBitmap;

#[test]
fn memory_usage_empty() {
    let bm = RoaringBitmap::new();
    let usage = bm.memory_usage();

    // Empty Vec: 3 * size_of::<usize>() bytes (ptr, cap, len)
    // Vec capacity: 0
    // No containers
    let expected_vec_size = std::mem::size_of::<Vec<(u16, usize)>>();
    assert_eq!(usage, expected_vec_size);
}

#[test]
fn memory_usage_array_container() {
    let mut bm = RoaringBitmap::new();

    // Insert 100 values
    for i in 0..100 {
        bm.insert(i);
    }

    let usage = bm.memory_usage();

    // RoaringBitmap: size_of::<Vec<_>>()
    // containers Vec: at least 1 * (2 + enum_size) bytes
    // Array Vec: at least 100 * 2 = 200 bytes
    let vec_size = std::mem::size_of::<Vec<(u16, usize)>>();
    let min_expected = vec_size + 200; // Vec metadata + 100 u16 values
    assert!(
        usage >= min_expected,
        "Expected at least {} bytes, got {}",
        min_expected,
        usage
    );

    println!("Array container (100 values): {} bytes", usage);
}

#[test]
fn memory_usage_bitmap_container() {
    let mut bm = RoaringBitmap::new();

    // Insert enough non-consecutive values to trigger bitmap
    for i in 0..8192 {
        if i % 2 == 0 {
            bm.insert(i);
        }
    }

    let usage = bm.memory_usage();

    // RoaringBitmap: size_of::<Vec<_>>()
    // containers Vec: at least 1 element
    // Bitmap: 8192 bytes (fixed)
    // Total should include 8192 bytes
    let vec_size = std::mem::size_of::<Vec<(u16, usize)>>();
    let min_expected = vec_size + 8192;
    assert!(
        usage >= min_expected,
        "Expected at least {} bytes, got {}",
        min_expected,
        usage
    );

    println!("Bitmap container (4096 values): {} bytes", usage);
}

#[test]
fn memory_usage_run_container() {
    let mut bm = RoaringBitmap::new();

    // Insert consecutive values
    for i in 0..10000 {
        bm.insert(i);
    }
    bm.optimize(); // Convert to Run

    let usage = bm.memory_usage();

    // RoaringBitmap: size_of::<Vec<_>>()
    // containers Vec: at least 1 element
    // Run container: 1 run * 4 bytes = 4 bytes (minimal)
    // Total should be much less than Array (10000 * 2 = 20000 bytes)
    let vec_size = std::mem::size_of::<Vec<(u16, usize)>>();
    let max_expected = vec_size + 1000; // Generous upper bound for overhead + 1 run
    assert!(
        usage < max_expected,
        "Expected less than {} bytes, got {} (should be minimal for 1 run)",
        max_expected,
        usage
    );

    println!("Run container (10000 consecutive values): {} bytes", usage);
}

#[test]
fn memory_usage_multiple_containers() {
    let mut bm = RoaringBitmap::new();

    // Container 0: Array (100 sparse values)
    for i in 0..100 {
        bm.insert(i * 10); // Sparse to avoid Run
    }

    // Container 1: Bitmap (4096 non-consecutive values)
    for i in 0..8192 {
        if i % 2 == 0 {
            bm.insert(65536 + i);
        }
    }

    // Container 2: Run (10000 consecutive)
    for i in 0..10000 {
        bm.insert(131072 + i);
    }
    bm.optimize();

    let usage = bm.memory_usage();

    // Should include:
    // - RoaringBitmap: size_of::<Vec<_>>()
    // - Containers Vec: 3 elements
    // - Array: ~200 bytes
    // - Bitmap: 8192 bytes
    // - Run: minimal (1 run)
    let vec_size = std::mem::size_of::<Vec<(u16, usize)>>();
    let min_expected = vec_size + 8192; // At minimum: Vec metadata + bitmap
    assert!(
        usage >= min_expected,
        "Expected at least {} bytes (for bitmap + overhead), got {}",
        min_expected,
        usage
    );

    println!("Multiple containers: {} bytes", usage);
}

#[test]
fn memory_usage_detailed() {
    let mut bm = RoaringBitmap::new();

    // Container 0: Array (sparse, under threshold)
    for i in 0..100 {
        bm.insert(i * 10); // Sparse to avoid Run conversion
    }

    // Container 1: Bitmap (dense, non-consecutive, over threshold)
    for i in 0..8192 {
        if i % 2 == 0 {
            bm.insert(65536 + i);
        }
    }

    // Container 2: Run (consecutive)
    for i in 0..10000 {
        bm.insert(131072 + i);
    }
    bm.optimize();

    let usage = bm.memory_usage_detailed();

    println!("\nDetailed memory usage:");
    println!("  Total: {} bytes", usage.total);
    println!("  Stack: {} bytes", usage.stack);
    println!("  Heap:  {} bytes", usage.heap);
    println!("  Containers:");
    for container in &usage.containers {
        println!(
            "    Container {}: {} - {} bytes",
            container.key, container.container_type, container.memory_bytes
        );
    }

    // Verify consistency
    assert_eq!(usage.total, usage.stack + usage.heap);
    assert_eq!(usage.containers.len(), 3);

    // Verify stack size (Vec metadata: ptr, capacity, length)
    let expected_stack = std::mem::size_of::<Vec<(u16, usize)>>();
    assert_eq!(usage.stack, expected_stack);

    // Verify we have all three container types
    let types: Vec<&str> = usage.containers.iter().map(|c| c.container_type).collect();
    assert!(types.contains(&"Array"), "Should have Array container");
    assert!(types.contains(&"Bitmap"), "Should have Bitmap container");
    assert!(types.contains(&"Run"), "Should have Run container");
}

#[test]
fn memory_usage_comparison() {
    println!("\n=== MEMORY USAGE COMPARISON ===");

    // Test 1: Same data in Array vs Run
    {
        let mut array_bm = RoaringBitmap::new();
        let mut run_bm = RoaringBitmap::new();

        // Insert 3000 consecutive values (under 4096 threshold to stay as Array)
        for i in 0..3000 {
            array_bm.insert(i);
            run_bm.insert(i);
        }

        let array_usage = array_bm.memory_usage();

        run_bm.optimize(); // Convert to Run
        let run_usage = run_bm.memory_usage();

        println!("  3,000 consecutive values:");
        println!("    Array: {} bytes", array_usage);
        println!("    Run:   {} bytes", run_usage);
        println!(
            "    Savings: {} bytes ({:.1}%)",
            array_usage - run_usage,
            (1.0 - run_usage as f64 / array_usage as f64) * 100.0
        );

        // Run should be much smaller (1 run = 4 bytes vs 3000 values * 2 = 6000 bytes)
        assert!(
            run_usage < array_usage / 10,
            "Run should be at least 10x smaller than Array for consecutive data"
        );
    }

    // Test 2: Same data in Array vs Bitmap
    {
        let mut array_bm = RoaringBitmap::new();
        let mut bitmap_bm = RoaringBitmap::new();

        // Insert sparse values (5000)
        for i in 0..10000 {
            if i % 2 == 0 {
                array_bm.insert(i);
                bitmap_bm.insert(i);
            }
        }

        let array_usage = array_bm.memory_usage();
        let bitmap_usage = bitmap_bm.memory_usage();

        println!("  5,000 sparse values:");
        println!("    Array:  {} bytes", array_usage);
        println!("    Bitmap: {} bytes", bitmap_usage);

        if bitmap_usage < array_usage {
            println!(
                "    Savings: {} bytes (Bitmap is smaller)",
                array_usage - bitmap_usage
            );
        } else {
            println!(
                "    Overhead: {} bytes (Array is smaller)",
                bitmap_usage - array_usage
            );
        }
    }
}

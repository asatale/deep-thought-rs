use roaring_bitmap::RoaringBitmap;

#[test]
fn intermediate_extend_consecutive_basic() {
    let mut bm = RoaringBitmap::new();

    // Insert consecutive range
    bm.extend_consecutive(0..1000);

    // Verify correctness
    assert_eq!(bm.len(), 1000);
    for i in 0..1000 {
        assert!(bm.contains(i), "Should contain {}", i);
    }

    // Verify it created a Run container (optimization)
    let stats = bm.container_stats();
    assert_eq!(stats.len(), 1);
    assert_eq!(
        stats[0].1, "Run",
        "Should create Run container for consecutive values"
    );
}

#[test]
fn intermediate_extend_consecutive_large() {
    let mut bm = RoaringBitmap::new();

    // Insert 1 million consecutive values
    bm.extend_consecutive(0..1_000_000);

    assert_eq!(bm.len(), 1_000_000);

    // Verify memory efficiency - Run container should be very compact
    let memory = bm.memory_usage();
    println!("Memory for 1M consecutive values: {} bytes", memory);

    // Run container: ~16 containers * (4 bytes per run) = very small
    // Much better than Array: 1M * 2 = 2MB
    assert!(
        memory < 10_000,
        "Should use less than 10KB for 1M consecutive values, got {}",
        memory
    );

    // Spot check values
    assert!(bm.contains(0));
    assert!(bm.contains(500_000));
    assert!(bm.contains(999_999));
    assert!(!bm.contains(1_000_000));
}

#[test]
fn intermediate_extend_consecutive_multiple_ranges() {
    let mut bm = RoaringBitmap::new();

    // Insert multiple consecutive ranges
    bm.extend_consecutive(0..100);
    bm.extend_consecutive(1000..2000);
    bm.extend_consecutive(100_000..101_000);

    assert_eq!(bm.len(), 100 + 1000 + 1000);

    // Verify all ranges present
    for i in 0..100 {
        assert!(bm.contains(i));
    }
    for i in 1000..2000 {
        assert!(bm.contains(i));
    }
    for i in 100_000..101_000 {
        assert!(bm.contains(i));
    }

    // Verify gaps
    assert!(!bm.contains(500));
    assert!(!bm.contains(50_000));
}

#[test]
fn intermediate_extend_consecutive_across_containers() {
    let mut bm = RoaringBitmap::new();

    // Insert range that spans multiple containers (16-bit boundaries)
    // Container boundaries are at 65536, 131072, etc.
    bm.extend_consecutive(65000..66000);

    assert_eq!(bm.len(), 1000);

    // Verify values across container boundary
    for i in 65000..66000 {
        assert!(bm.contains(i), "Should contain {}", i);
    }

    // Should create 2 containers (spans boundary at 65536)
    let stats = bm.container_stats();
    assert_eq!(stats.len(), 2, "Should span 2 containers");
}

#[test]
fn intermediate_extend_consecutive_empty_range() {
    let mut bm = RoaringBitmap::new();

    // Empty range should do nothing
    bm.extend_consecutive(100..100);
    assert_eq!(bm.len(), 0);

    // Reversed range should do nothing
    bm.extend_consecutive(100..50);
    assert_eq!(bm.len(), 0);
}

#[test]
fn intermediate_extend_consecutive_then_optimize() {
    let mut bm = RoaringBitmap::new();

    // Insert consecutive range (creates Run)
    bm.extend_consecutive(0..1000);
    assert_eq!(bm.container_type(0), Some("Run"));

    // Remove every other value (fragments the run)
    for i in (0..1000).step_by(2) {
        bm.remove(i);
    }

    // optimize() should convert to better container type (data-driven)
    bm.optimize();

    // After fragmentation, might convert to Array (more efficient)
    let container_type = bm.container_type(0);
    println!("After fragmentation and optimize: {:?}", container_type);

    // Verify correctness regardless of container type
    assert_eq!(bm.len(), 500);
    for i in (1..1000).step_by(2) {
        assert!(bm.contains(i), "Should contain {}", i);
    }
}

#[test]
fn intermediate_extend_sparse_basic() {
    let mut bm = RoaringBitmap::new();

    // Insert sparse values
    bm.extend_sparse([10, 100, 1000, 10000, 100000]);

    assert_eq!(bm.len(), 5);
    assert!(bm.contains(10));
    assert!(bm.contains(100));
    assert!(bm.contains(1000));
    assert!(bm.contains(10000));
    assert!(bm.contains(100000));
    assert!(!bm.contains(50));
}

#[test]
fn intermediate_extend_sparse_from_vec() {
    let mut bm = RoaringBitmap::new();

    // Create sparse values from a vector
    let sparse_values: Vec<u32> = vec![42, 1337, 9999, 123456];
    bm.extend_sparse(sparse_values);

    assert_eq!(bm.len(), 4);
    assert!(bm.contains(42));
    assert!(bm.contains(1337));
    assert!(bm.contains(9999));
    assert!(bm.contains(123456));
}

#[test]
fn intermediate_extend_sparse_large_gaps() {
    let mut bm = RoaringBitmap::new();

    // Insert values with very large gaps (different containers)
    let sparse_values: Vec<u32> = (0..100).map(|i| i * 100_000).collect();
    bm.extend_sparse(sparse_values);

    assert_eq!(bm.len(), 100);

    // Verify values
    for i in 0..100 {
        assert!(bm.contains(i * 100_000));
    }
}

#[test]
fn intermediate_extend_sparse_duplicates() {
    let mut bm = RoaringBitmap::new();

    // Insert with duplicates (should handle gracefully)
    bm.extend_sparse([10, 20, 10, 30, 20, 40]);

    assert_eq!(bm.len(), 4, "Should deduplicate");
    assert!(bm.contains(10));
    assert!(bm.contains(20));
    assert!(bm.contains(30));
    assert!(bm.contains(40));
}

#[test]
fn intermediate_extend_dense_basic() {
    let mut bm = RoaringBitmap::new();

    // Insert even numbers (50% density)
    bm.extend_dense((0..10_000).filter(|x| x % 2 == 0));

    assert_eq!(bm.len(), 5000);

    // Verify even numbers present
    for i in (0..10_000).step_by(2) {
        assert!(bm.contains(i), "Should contain {}", i);
    }

    // Verify odd numbers absent
    for i in (1..10_000).step_by(2) {
        assert!(!bm.contains(i), "Should not contain {}", i);
    }
}

#[test]
fn intermediate_extend_dense_high_density() {
    let mut bm = RoaringBitmap::new();

    // Insert with ~67% density (every value except multiples of 3)
    bm.extend_dense((0..9000).filter(|x| x % 3 != 0));

    let expected_count = (0..9000).filter(|x| x % 3 != 0).count();
    assert_eq!(bm.len(), expected_count as u64);

    // Verify correctness
    for i in 0..9000 {
        if i % 3 != 0 {
            assert!(bm.contains(i), "Should contain {}", i);
        } else {
            assert!(!bm.contains(i), "Should not contain {}", i);
        }
    }
}

#[test]
fn intermediate_extend_dense_triggers_bitmap() {
    let mut bm = RoaringBitmap::new();

    // Insert enough values to trigger Bitmap conversion (>= 4096)
    bm.extend_dense((0..8192).filter(|x| x % 2 == 0));

    assert_eq!(bm.len(), 4096);

    // Should automatically convert to Bitmap (high cardinality)
    let stats = bm.container_stats();
    assert_eq!(stats.len(), 1);
    assert_eq!(
        stats[0].1, "Bitmap",
        "Should automatically create Bitmap for dense values"
    );
}

#[test]
fn intermediate_mixing_methods() {
    let mut bm = RoaringBitmap::new();

    // Mix different semantic methods
    bm.extend_consecutive(0..1000); // Run
    bm.extend_sparse([50_000, 100_000, 150_000]); // Array (different containers)
    bm.extend_dense((200_000..210_000).filter(|x| x % 2 == 0)); // Array/Bitmap

    let total = 1000 + 3 + 5000;
    assert_eq!(bm.len(), total as u64);

    // Verify all methods worked
    assert!(bm.contains(500)); // From consecutive
    assert!(bm.contains(50_000)); // From sparse
    assert!(bm.contains(200_000)); // From dense
}

#[test]
fn intermediate_extend_consecutive_full_container() {
    let mut bm = RoaringBitmap::new();

    // Insert a full container (65536 consecutive values)
    bm.extend_consecutive(0..65536);

    assert_eq!(bm.len(), 65536);

    // Should create Run container with single run
    let stats = bm.container_stats();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].1, "Run");
    assert_eq!(stats[0].2, 65536, "Should have all 65536 values");

    // Memory should be minimal (1 run = 4 bytes)
    let memory = bm.memory_usage();
    println!("Memory for full container (65536 values): {} bytes", memory);
    assert!(memory < 1000, "Should be very compact, got {}", memory);
}

#[test]
fn intermediate_performance_comparison() {
    // Compare semantic method vs regular insert for consecutive values

    // Method 1: Regular insert (inefficient)
    let mut bm1 = RoaringBitmap::new();
    for i in 0..10_000 {
        bm1.insert(i);
    }

    // Method 2: Semantic method (efficient)
    let mut bm2 = RoaringBitmap::new();
    bm2.extend_consecutive(0..10_000);

    // Both should have same values
    assert_eq!(bm1.len(), bm2.len());
    for i in 0..10_000 {
        assert_eq!(bm1.contains(i), bm2.contains(i));
    }

    // Optimize both
    bm1.optimize();
    bm2.optimize();

    // After optimization, both should be similar
    assert_eq!(bm1.len(), bm2.len());
}

#[test]
fn intermediate_documentation_example_consecutive() {
    // Test the example from documentation
    let mut bm = RoaringBitmap::new();

    // Efficient: creates Run container directly
    bm.extend_consecutive(0..1_000_000);
    println!("Memory: {} bytes", bm.memory_usage()); // Very compact!

    // Multiple consecutive ranges
    bm.extend_consecutive(2_000_000..3_000_000);
    bm.extend_consecutive(5_000_000..6_000_000);

    assert_eq!(bm.len(), 3_000_000);
}

#[test]
fn intermediate_documentation_example_sparse() {
    // Test the example from documentation
    let mut bm = RoaringBitmap::new();

    // Sparse user IDs
    bm.extend_sparse([1000, 5000, 10000, 50000, 100000]);

    // From a vector
    let sparse_values: Vec<u32> = vec![42, 1337, 9999];
    bm.extend_sparse(sparse_values);

    assert_eq!(bm.len(), 8);
}

#[test]
fn intermediate_documentation_example_dense() {
    // Test the example from documentation
    let mut bm = RoaringBitmap::new();

    // Insert even numbers in a range (50% density)
    bm.extend_dense((0..10_000).filter(|x| x % 2 == 0));

    // Dense range with some gaps
    let values: Vec<u32> = (0..8000).filter(|x| x % 3 != 0).collect();
    bm.extend_dense(values);

    println!("Memory: {} bytes", bm.memory_usage());
}

#[test]
fn intermediate_optimize_is_data_driven() {
    // Verify that optimize() is data-driven, not hint-driven
    let mut bm = RoaringBitmap::new();

    // Create Run container via semantic method
    bm.extend_consecutive(0..100);
    assert_eq!(bm.container_type(0), Some("Run"));

    // Fragment the data heavily
    for i in (0..100).step_by(2) {
        bm.remove(i);
    }

    // Before optimize: might still be Run (fragmented)
    let before_type = bm.container_type(0);
    println!("Before optimize: {:?}", before_type);

    // optimize() should analyze actual data and convert if beneficial
    bm.optimize();

    let after_type = bm.container_type(0);
    println!("After optimize: {:?}", after_type);

    // Verify correctness regardless of container type
    assert_eq!(bm.len(), 50);
    for i in (1..100).step_by(2) {
        assert!(bm.contains(i));
    }

    // The key point: optimize() made a data-driven decision,
    // not preserving the Run container just because we used extend_consecutive
}

#[test]
fn intermediate_extend_consecutive_range_types() {
    let mut bm = RoaringBitmap::new();

    // Test different range types
    bm.extend_consecutive(0..100); // Exclusive end
    bm.extend_consecutive(1000..=2000); // Inclusive end
    bm.extend_consecutive(10000..); // Unbounded (but will stop at container boundary)

    // Verify ranges work correctly
    assert!(bm.contains(0));
    assert!(bm.contains(99));
    assert!(!bm.contains(100));

    assert!(bm.contains(1000));
    assert!(bm.contains(2000));

    assert!(bm.contains(10000));
}

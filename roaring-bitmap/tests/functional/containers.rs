use roaring_bitmap::RoaringBitmap;

// Bitmap Container Tests

#[test]
fn automatic_conversion_to_bitmap_container() {
    let mut bm = RoaringBitmap::new();

    // Insert 4096 values - should trigger conversion to bitmap container
    for i in 0..4096 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 4096);

    // Verify all values are present
    for i in 0..4096 {
        assert!(bm.contains(i), "Value {} should be present", i);
    }

    // Verify values outside range are not present
    assert!(!bm.contains(4096));
    assert!(!bm.contains(5000));
}

#[test]
fn automatic_conversion_from_bitmap_to_array() {
    let mut bm = RoaringBitmap::new();

    // Insert 4096 values to create bitmap container
    for i in 0..4096 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 4096);

    // Remove values to drop below threshold - should convert back to array
    for i in 0..100 {
        assert!(bm.remove(i));
    }

    assert_eq!(bm.len(), 4096 - 100);

    // Verify remaining values are still present
    for i in 100..4096 {
        assert!(bm.contains(i), "Value {} should still be present", i);
    }

    // Verify removed values are gone
    for i in 0..100 {
        assert!(!bm.contains(i), "Value {} should be removed", i);
    }
}

#[test]
fn bitmap_container_operations() {
    let mut bm1 = RoaringBitmap::new();
    let mut bm2 = RoaringBitmap::new();

    // Create two bitmaps with bitmap containers
    for i in 0..5000 {
        bm1.insert(i);
    }

    for i in 4000..9000 {
        bm2.insert(i);
    }

    // Test union
    let union = bm1.union(&bm2);
    assert_eq!(union.len(), 9000);
    for i in 0..9000 {
        assert!(union.contains(i));
    }

    // Test intersection
    let intersection = bm1.intersection(&bm2);
    assert_eq!(intersection.len(), 1000); // 4000-4999 overlap
    for i in 4000..5000 {
        assert!(intersection.contains(i));
    }
    assert!(!intersection.contains(3999));
    assert!(!intersection.contains(5000));

    // Test difference
    let diff = bm1.difference(&bm2);
    assert_eq!(diff.len(), 4000); // 0-3999
    for i in 0..4000 {
        assert!(diff.contains(i));
    }
    assert!(!diff.contains(4000));
}

#[test]
fn mixed_array_and_bitmap_container_operations() {
    let mut array_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    // Create array container (< 4096 elements)
    for i in 0..1000 {
        array_bm.insert(i);
    }

    // Create bitmap container (>= 4096 elements)
    for i in 500..5000 {
        bitmap_bm.insert(i);
    }

    // Test union
    let union = array_bm.union(&bitmap_bm);
    assert_eq!(union.len(), 5000);
    for i in 0..5000 {
        assert!(union.contains(i));
    }

    // Test intersection
    let intersection = array_bm.intersection(&bitmap_bm);
    assert_eq!(intersection.len(), 500); // 500-999 overlap
    for i in 500..1000 {
        assert!(intersection.contains(i));
    }

    // Test difference
    let diff = array_bm.difference(&bitmap_bm);
    assert_eq!(diff.len(), 500); // 0-499
    for i in 0..500 {
        assert!(diff.contains(i));
    }
}

#[test]
fn bitmap_container_with_gaps() {
    let mut bm = RoaringBitmap::new();

    // Create a bitmap container with gaps
    for i in 0..2000 {
        bm.insert(i);
    }
    for i in 5000..7000 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 4000);

    // Verify the gaps
    for i in 0..2000 {
        assert!(bm.contains(i));
    }
    for i in 2000..5000 {
        assert!(!bm.contains(i));
    }
    for i in 5000..7000 {
        assert!(bm.contains(i));
    }
}

#[test]
fn bitmap_container_cardinality_tracking() {
    let mut bm = RoaringBitmap::new();

    // Insert to create bitmap container
    for i in 0..10000 {
        bm.insert(i);
    }
    assert_eq!(bm.len(), 10000);

    // Remove some values
    for i in 0..1000 {
        bm.remove(i);
    }
    assert_eq!(bm.len(), 9000);

    // Add some back
    for i in 0..500 {
        bm.insert(i);
    }
    assert_eq!(bm.len(), 9500);
}

#[test]
fn dense_bitmap_operations() {
    let mut bm = RoaringBitmap::new();

    // Create a very dense bitmap (near maximum for one container)
    for i in 0..60000 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 60000);

    // Remove scattered elements
    for i in (0..60000).step_by(100) {
        bm.remove(i);
    }

    assert_eq!(bm.len(), 60000 - 600);

    // Verify correct elements remain
    for i in 0..60000 {
        if i % 100 == 0 {
            assert!(!bm.contains(i));
        } else {
            assert!(bm.contains(i));
        }
    }
}

// Container Type Conversion Verification Tests

#[test]
fn verify_array_container_before_threshold() {
    let mut bm = RoaringBitmap::new();

    // Insert 4095 values (just below threshold)
    for i in 0..4095 {
        bm.insert(i);
    }

    // Verify it's still an array container
    assert_eq!(bm.container_type(0), Some("Array"));
    assert_eq!(bm.len(), 4095);
}

#[test]
fn verify_conversion_at_threshold() {
    let mut bm = RoaringBitmap::new();

    // Insert 4095 values with gaps (non-consecutive to avoid Run container)
    // Use pattern: insert every other value
    for i in 0..8190 {
        if i % 2 == 0 {
            bm.insert(i);
        }
    }
    assert_eq!(
        bm.container_type(0),
        Some("Array"),
        "Should be Array before threshold"
    );
    assert_eq!(bm.len(), 4095);

    // Insert 4096th value - should trigger conversion to Bitmap
    bm.insert(8190);
    assert_eq!(
        bm.container_type(0),
        Some("Bitmap"),
        "Should be Bitmap at threshold"
    );
    assert_eq!(bm.len(), 4096);
}

#[test]
fn verify_bitmap_container_after_threshold() {
    let mut bm = RoaringBitmap::new();

    // Insert 5000 values with gaps (non-consecutive to avoid Run container)
    // Use pattern: insert every other value
    for i in 0..10000 {
        if i % 2 == 0 {
            bm.insert(i);
        }
    }

    // Verify it's a bitmap container (5000 sparse values should use Bitmap)
    assert_eq!(bm.container_type(0), Some("Bitmap"));
    assert_eq!(bm.len(), 5000);
}

#[test]
fn verify_conversion_back_to_array() {
    let mut bm = RoaringBitmap::new();

    // Create bitmap container with non-consecutive values
    // Insert every other value to avoid Run container
    for i in 0..8192 {
        if i % 2 == 0 {
            bm.insert(i);
        }
    }
    assert_eq!(
        bm.container_type(0),
        Some("Bitmap"),
        "Should be Bitmap initially"
    );
    assert_eq!(bm.len(), 4096);

    // Remove values to drop below threshold
    for i in 0..200 {
        if i % 2 == 0 {
            bm.remove(i);
        }
    }

    // Should convert back to array (4096 - 100 = 3996 < 4096)
    assert_eq!(
        bm.container_type(0),
        Some("Array"),
        "Should convert back to Array"
    );
    assert_eq!(bm.len(), 3996);
}

#[test]
fn verify_multiple_containers_different_types() {
    let mut bm = RoaringBitmap::new();

    // Container 0: Array (small)
    for i in 0..100 {
        bm.insert(i);
    }

    // Container 1: Bitmap (large) - insert every other value to avoid Run
    // Values starting at 65536 (key=1)
    for i in 0..10000 {
        if i % 2 == 0 {
            bm.insert(65536 + i);
        }
    }

    // Container 2: Array (small) - values 131072-131172
    for i in 131072..131172 {
        bm.insert(i);
    }

    let stats = bm.container_stats();
    assert_eq!(stats.len(), 3, "Should have 3 containers");

    // Verify container types
    assert_eq!(
        stats[0],
        (0, "Array", 100),
        "First container should be Array"
    );
    assert_eq!(
        stats[1],
        (1, "Bitmap", 5000),
        "Second container should be Bitmap"
    );
    assert_eq!(
        stats[2],
        (2, "Array", 100),
        "Third container should be Array"
    );
}

#[test]
fn verify_exact_threshold_boundary() {
    let mut bm = RoaringBitmap::new();

    // Test the exact boundary: 4095 -> 4096
    // Use non-consecutive pattern to avoid Run container
    for i in 0..8190 {
        if i % 2 == 0 {
            bm.insert(i);
        }
    }
    assert_eq!(
        bm.container_type(0),
        Some("Array"),
        "Should be Array at 4095"
    );
    assert_eq!(bm.len(), 4095);

    // Insert one more to reach exactly 4096
    bm.insert(8190);
    assert_eq!(
        bm.container_type(0),
        Some("Bitmap"),
        "Should be Bitmap at 4096"
    );
    assert_eq!(bm.len(), 4096);

    // Remove one to drop to 4095
    bm.remove(8190);
    assert_eq!(
        bm.container_type(0),
        Some("Array"),
        "Should be Array at 4095 again"
    );
    assert_eq!(bm.len(), 4095);
}

#[test]
fn verify_non_sequential_inserts_trigger_conversion() {
    let mut bm = RoaringBitmap::new();

    // Insert 4096 values in random order (using step pattern)
    for i in (0..8192).step_by(2) {
        bm.insert(i);
    }

    // Should be bitmap container (4096 values)
    assert_eq!(bm.container_type(0), Some("Bitmap"));
    assert_eq!(bm.len(), 4096);

    // All even values should be present
    for i in (0..8192).step_by(2) {
        assert!(bm.contains(i), "Even value {} should be present", i);
    }

    // All odd values should be absent
    for i in (1..8192).step_by(2) {
        assert!(!bm.contains(i), "Odd value {} should be absent", i);
    }
}

// Run Container Tests

#[test]
fn run_container_consecutive_values() {
    use roaring_bitmap::RoaringBitmap;

    // Note: We can't directly create Run containers yet, as they're not
    // automatically chosen. This test verifies that consecutive values
    // work correctly (they'll be stored in array/bitmap for now)

    let mut bm = RoaringBitmap::new();

    // Insert consecutive sequence
    for i in 100..200 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 100);

    // Verify all values present
    for i in 100..200 {
        assert!(bm.contains(i), "Value {} should be present", i);
    }

    // Verify iteration is correct
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 100);
    assert_eq!(values[0], 100);
    assert_eq!(values[99], 199);
}

#[test]
fn run_container_multiple_sequences() {
    use roaring_bitmap::RoaringBitmap;

    let mut bm = RoaringBitmap::new();

    // Insert multiple consecutive sequences with gaps
    for i in 0..10 {
        bm.insert(i);
    }
    for i in 20..30 {
        bm.insert(i);
    }
    for i in 50..60 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 30);

    // Verify correct values
    for i in 0..10 {
        assert!(bm.contains(i));
    }
    for i in 10..20 {
        assert!(!bm.contains(i));
    }
    for i in 20..30 {
        assert!(bm.contains(i));
    }
    for i in 30..50 {
        assert!(!bm.contains(i));
    }
    for i in 50..60 {
        assert!(bm.contains(i));
    }
}

#[test]
fn run_container_with_removals() {
    use roaring_bitmap::RoaringBitmap;

    let mut bm = RoaringBitmap::new();

    // Insert long consecutive sequence
    for i in 0..100 {
        bm.insert(i);
    }

    // Remove some values to create gaps
    bm.remove(10);
    bm.remove(50);
    bm.remove(99);

    assert_eq!(bm.len(), 97);
    assert!(!bm.contains(10));
    assert!(!bm.contains(50));
    assert!(!bm.contains(99));
    assert!(bm.contains(9));
    assert!(bm.contains(11));
    assert!(bm.contains(49));
    assert!(bm.contains(51));
    assert!(bm.contains(98));
}

#[test]
fn run_container_set_operations() {
    use roaring_bitmap::RoaringBitmap;

    let mut bm1 = RoaringBitmap::new();
    let mut bm2 = RoaringBitmap::new();

    // BM1: 0-49
    for i in 0..50 {
        bm1.insert(i);
    }

    // BM2: 25-74
    for i in 25..75 {
        bm2.insert(i);
    }

    // Union: 0-74
    let union = bm1.union(&bm2);
    assert_eq!(union.len(), 75);
    for i in 0..75 {
        assert!(union.contains(i));
    }

    // Intersection: 25-49
    let intersection = bm1.intersection(&bm2);
    assert_eq!(intersection.len(), 25);
    for i in 25..50 {
        assert!(intersection.contains(i));
    }

    // Difference: 0-24
    let diff = bm1.difference(&bm2);
    assert_eq!(diff.len(), 25);
    for i in 0..25 {
        assert!(diff.contains(i));
    }
}

#[test]
fn run_container_large_consecutive_sequence() {
    use roaring_bitmap::RoaringBitmap;

    let mut bm = RoaringBitmap::new();

    // Insert a very large consecutive sequence
    for i in 0..10000 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 10000);

    // Verify first, middle, and last
    assert!(bm.contains(0));
    assert!(bm.contains(5000));
    assert!(bm.contains(9999));
    assert!(!bm.contains(10000));

    // Verify iteration
    let values: Vec<u32> = bm.iter().take(10).collect();
    assert_eq!(values, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

// Test for Run container efficiency after deletions
#[test]
fn run_container_fragmentation_after_deletions() {
    let mut bm = RoaringBitmap::new();

    // Create a highly efficient Run container: [0-10000] = 1 run, 4 bytes
    for i in 0..10000 {
        bm.insert(i);
    }
    assert_eq!(bm.container_type(0), Some("Run"));
    assert_eq!(bm.len(), 10000);

    // Remove every other value - this fragments the Run container into 5000 runs
    // Run container: 5000 runs * 4 bytes = 20,000 bytes
    // Array container: 5000 values * 2 bytes = 10,000 bytes
    // Bitmap container: 8,192 bytes (fixed)
    for i in 0..10000 {
        if i % 2 == 0 {
            bm.remove(i);
        }
    }

    println!("\nAfter fragmenting deletions:");
    println!("  Container type: {:?}", bm.container_type(0));
    println!("  Length: {}", bm.len());

    // Without optimize(), Run container stays fragmented (inefficient)
    // This is expected with our Hybrid+Lazy approach
    assert_eq!(bm.container_type(0), Some("Run"));
    assert_eq!(bm.len(), 5000);

    // After optimize(), should convert to more efficient representation
    bm.optimize();
    println!("  After optimize: {:?}", bm.container_type(0));

    // With 5000 sparse values, should be Bitmap (8KB) or Array (10KB)
    // Our heuristic converts heavily fragmented Run to Array
    assert!(
        bm.container_type(0) == Some("Array") || bm.container_type(0) == Some("Bitmap"),
        "Should convert fragmented Run to Array or Bitmap"
    );
    assert_eq!(bm.len(), 5000);
}

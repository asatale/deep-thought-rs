use roaring_bitmap::RoaringBitmap;

#[test]
fn batch_remove_range_basic() {
    let mut bm = RoaringBitmap::new();

    // Insert a range and then remove a subrange
    bm.extend_consecutive(0..10_000);
    assert_eq!(bm.len(), 10_000);

    // Remove middle portion
    bm.remove_range(4000..6000);
    assert_eq!(bm.len(), 8_000);

    // Verify removed range
    for i in 4000..6000 {
        assert!(!bm.contains(i), "Value {} should be removed", i);
    }

    // Verify remaining ranges
    for i in 0..4000 {
        assert!(bm.contains(i), "Value {} should remain", i);
    }
    for i in 6000..10_000 {
        assert!(bm.contains(i), "Value {} should remain", i);
    }
}

#[test]
fn batch_remove_range_entire_container() {
    let mut bm = RoaringBitmap::new();

    // Insert across multiple containers
    bm.extend_consecutive(0..200_000);
    let original_len = bm.len();
    assert_eq!(original_len, 200_000);

    // Remove an entire container's worth
    bm.remove_range(65536..131072);
    assert_eq!(bm.len(), original_len - 65536);

    // Verify removed range
    for i in 65536..131072 {
        assert!(!bm.contains(i));
    }

    // Verify remaining values
    for i in 0..65536 {
        assert!(bm.contains(i));
    }
    for i in 131072..200_000 {
        assert!(bm.contains(i));
    }
}

#[test]
fn batch_remove_range_across_containers() {
    let mut bm = RoaringBitmap::new();

    // Insert range spanning multiple containers
    bm.extend_consecutive(50_000..150_000);
    assert_eq!(bm.len(), 100_000);

    // Remove range across container boundaries
    bm.remove_range(60_000..140_000);
    assert_eq!(bm.len(), 20_000);

    // Verify
    for i in 50_000..60_000 {
        assert!(bm.contains(i));
    }
    for i in 60_000..140_000 {
        assert!(!bm.contains(i));
    }
    for i in 140_000..150_000 {
        assert!(bm.contains(i));
    }
}

#[test]
fn batch_remove_range_empty_range() {
    let mut bm = RoaringBitmap::new();
    bm.extend_consecutive(0..1000);

    // Empty range should do nothing
    bm.remove_range(100..100);
    assert_eq!(bm.len(), 1000);

    // Reversed range should do nothing
    bm.remove_range(500..400);
    assert_eq!(bm.len(), 1000);
}

#[test]
fn batch_remove_range_partial_container() {
    let mut bm = RoaringBitmap::new();
    bm.extend_consecutive(0..1000);
    bm.extend_consecutive(2000..3000);

    // Remove part of first container
    bm.remove_range(100..500);
    assert_eq!(bm.len(), 1000 + 1000 - 400);

    // Verify
    for i in 0..100 {
        assert!(bm.contains(i));
    }
    for i in 100..500 {
        assert!(!bm.contains(i));
    }
    for i in 500..1000 {
        assert!(bm.contains(i));
    }
}

#[test]
fn batch_remove_range_inclusive() {
    let mut bm = RoaringBitmap::new();
    bm.extend_consecutive(0..100);

    // Test inclusive range
    bm.remove_range(10..=20);
    assert_eq!(bm.len(), 89); // Removed 11 values (10-20 inclusive)

    assert!(bm.contains(9));
    assert!(!bm.contains(10));
    assert!(!bm.contains(20));
    assert!(bm.contains(21));
}

#[test]
fn batch_remove_sparse_basic() {
    let mut bm = RoaringBitmap::new();
    bm.extend_sparse([10, 100, 1000, 10000, 100000]);
    assert_eq!(bm.len(), 5);

    // Remove some sparse values
    bm.remove_sparse([100, 10000]);
    assert_eq!(bm.len(), 3);

    assert!(bm.contains(10));
    assert!(!bm.contains(100));
    assert!(bm.contains(1000));
    assert!(!bm.contains(10000));
    assert!(bm.contains(100000));
}

#[test]
fn batch_remove_sparse_from_vec() {
    let mut bm = RoaringBitmap::new();
    bm.extend_consecutive(0..1000);

    // Remove specific values from vector
    let to_remove: Vec<u32> = vec![10, 50, 100, 500, 999];
    bm.remove_sparse(to_remove);

    assert_eq!(bm.len(), 995);
    assert!(!bm.contains(10));
    assert!(!bm.contains(50));
    assert!(!bm.contains(100));
    assert!(!bm.contains(500));
    assert!(!bm.contains(999));
}

#[test]
fn batch_remove_sparse_nonexistent() {
    let mut bm = RoaringBitmap::new();
    bm.extend_sparse([1, 2, 3, 4, 5]);

    // Remove values that don't exist
    bm.remove_sparse([10, 20, 30]);
    assert_eq!(bm.len(), 5); // No change

    // Verify original values still present
    for i in 1..=5 {
        assert!(bm.contains(i));
    }
}

#[test]
fn batch_remove_sparse_mixed() {
    let mut bm = RoaringBitmap::new();
    bm.extend_sparse([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    // Remove mix of existing and non-existing
    bm.remove_sparse([2, 15, 5, 25, 8, 35]);
    assert_eq!(bm.len(), 7); // Removed 3 existing values

    assert!(bm.contains(1));
    assert!(!bm.contains(2));
    assert!(bm.contains(3));
    assert!(!bm.contains(5));
    assert!(!bm.contains(8));
    assert!(bm.contains(10));
}

#[test]
fn batch_clear_basic() {
    let mut bm = RoaringBitmap::new();
    bm.extend_consecutive(0..100_000);
    assert_eq!(bm.len(), 100_000);

    bm.clear();
    assert_eq!(bm.len(), 0);
    assert!(bm.is_empty());

    // Verify nothing remains
    assert!(!bm.contains(0));
    assert!(!bm.contains(50_000));
    assert!(!bm.contains(99_999));
}

#[test]
fn batch_clear_empty() {
    let mut bm = RoaringBitmap::new();
    bm.clear(); // Clear empty bitmap
    assert!(bm.is_empty());
}

#[test]
fn batch_clear_and_reuse() {
    let mut bm = RoaringBitmap::new();

    // Use, clear, reuse
    bm.extend_consecutive(0..1000);
    assert_eq!(bm.len(), 1000);

    bm.clear();
    assert!(bm.is_empty());

    // Reuse after clear
    bm.extend_sparse([10, 20, 30]);
    assert_eq!(bm.len(), 3);
    assert!(bm.contains(10));
    assert!(bm.contains(20));
    assert!(bm.contains(30));
}

#[test]
fn batch_api_symmetry() {
    // Test that removal operations mirror insertion helpers

    let mut bm = RoaringBitmap::new();

    // Consecutive: insert then remove
    bm.extend_consecutive(0..10_000);
    bm.remove_range(0..5_000);
    assert_eq!(bm.len(), 5_000);

    // Sparse: insert then remove
    bm.extend_sparse([100_000, 200_000, 300_000]);
    bm.remove_sparse([100_000, 300_000]);
    assert!(bm.contains(200_000));
    assert!(!bm.contains(100_000));
    assert!(!bm.contains(300_000));
}

#[test]
fn batch_removal_time_series_use_case() {
    // Simulate time-series pruning use case
    let mut bm = RoaringBitmap::new();

    // Add timestamps for 100 days (assuming one timestamp per second)
    let seconds_per_day = 86400u32;
    for day in 0..100 {
        let start = day * seconds_per_day;
        let end = start + seconds_per_day;
        bm.extend_consecutive(start..end);
    }

    let total_seconds = 100 * seconds_per_day;
    assert_eq!(bm.len(), total_seconds as u64);

    // Remove old data (first 30 days)
    bm.remove_range(0..(30 * seconds_per_day));
    assert_eq!(bm.len(), (70 * seconds_per_day) as u64);

    // Verify old data removed
    assert!(!bm.contains(0));
    assert!(!bm.contains(30 * seconds_per_day - 1));

    // Verify recent data remains
    assert!(bm.contains(30 * seconds_per_day));
    assert!(bm.contains(99 * seconds_per_day));
}

#[test]
fn batch_removal_sliding_window() {
    // Simulate sliding window operation
    let mut bm = RoaringBitmap::new();

    // Initial window: 0-1000
    bm.extend_consecutive(0..1000);

    // Slide window: remove old, add new
    bm.remove_range(0..100);
    bm.extend_consecutive(1000..1100);

    // Window is now 100-1100
    assert_eq!(bm.len(), 1000);
    assert!(!bm.contains(50));
    assert!(bm.contains(500));
    assert!(bm.contains(1050));
}

#[test]
fn batch_removal_downsampling() {
    // Simulate batch downsampling by removing ranges
    let mut bm = RoaringBitmap::new();

    // Insert data at full resolution
    bm.extend_consecutive(0..10_000);

    // Downsample by removing every other 100-value block
    for i in (0..10_000).step_by(200) {
        bm.remove_range(i..i + 100);
    }

    // Should have removed 50 blocks of 100 = 5000 values
    assert_eq!(bm.len(), 5_000);

    // Verify pattern
    assert!(!bm.contains(0)); // First block removed
    assert!(bm.contains(100)); // Second block remains
    assert!(!bm.contains(200)); // Third block removed
}

#[test]
fn batch_removal_with_optimize() {
    // Test that removal operations work correctly with optimize()
    let mut bm = RoaringBitmap::new();

    // Create Run container
    bm.extend_consecutive(0..10_000);
    assert_eq!(bm.container_type(0), Some("Run"));

    // Remove range (fragments the run)
    bm.remove_range(4000..6000);
    assert_eq!(bm.len(), 8_000);

    // Optimize should convert to more efficient representation
    bm.optimize();

    // Verify correctness after optimization
    assert_eq!(bm.len(), 8_000);
    for i in 0..4000 {
        assert!(bm.contains(i));
    }
    for i in 4000..6000 {
        assert!(!bm.contains(i));
    }
    for i in 6000..10_000 {
        assert!(bm.contains(i));
    }
}

#[test]
fn batch_removal_documentation_examples() {
    // Test examples from README

    // remove_range example
    {
        let mut bm = RoaringBitmap::new();
        bm.extend_consecutive(0..100_000);

        bm.remove_range(1000..2000);
        assert!(!bm.contains(1500));
        assert!(bm.contains(500));
        assert!(bm.contains(2500));

        bm.remove_range(0..10_000);
        assert!(!bm.contains(5000));
    }

    // remove_sparse example
    {
        let mut bm = RoaringBitmap::new();
        bm.extend_sparse([100, 1000, 5000, 10000, 50000]);

        bm.remove_sparse([1000, 10000]);
        assert!(!bm.contains(1000));

        let to_remove: Vec<u32> = vec![100, 5000];
        bm.remove_sparse(to_remove);
        assert_eq!(bm.len(), 1); // Only 50000 remains
    }

    // clear example
    {
        let mut bm = RoaringBitmap::new();
        bm.extend_consecutive(0..100_000);

        bm.clear();
        assert!(bm.is_empty());
    }
}

use crate::functional::{bitmap_of, expect_bitmap};
use roaring_bitmap::RoaringBitmap;

// ============================================================================
// Basic Iterator Tests
// ============================================================================

#[test]
fn iterator_on_empty_bitmap() {
    let bm = RoaringBitmap::new();
    assert_eq!(bm.iter().count(), 0);

    // Empty iterator returns None immediately
    let mut iter = bm.iter();
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None); // Fused behavior
}

#[test]
fn iterator_single_value() {
    let bm = bitmap_of(&[42]);
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values, vec![42]);

    // Verify iteration
    let mut iter = bm.iter();
    assert_eq!(iter.next(), Some(42));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None); // Fused behavior
}

#[test]
fn iterator_is_sorted_and_exact() {
    let mut bm = RoaringBitmap::new();
    for value in (0..100).step_by(7) {
        assert!(bm.insert(value));
    }
    let mut prev = None;
    for value in bm.iter() {
        if let Some(p) = prev {
            assert!(p < value);
        }
        prev = Some(value);
    }
    assert_eq!(bm.iter().count() as u64, bm.len());
}

#[test]
fn iterator_works_after_remove() {
    let mut bm = bitmap_of(&[1, 2, 3, 4, 5]);
    bm.remove(3);
    expect_bitmap(&bm, &[1, 2, 4, 5]);
    bm.remove(1);
    expect_bitmap(&bm, &[2, 4, 5]);
    bm.remove(5);
    expect_bitmap(&bm, &[2, 4]);
}

#[test]
fn iterator_can_be_called_multiple_times() {
    let bm = bitmap_of(&[10, 20, 30]);
    let first: Vec<u32> = bm.iter().collect();
    let second: Vec<u32> = bm.iter().collect();
    assert_eq!(first, second);
    assert_eq!(first, vec![10, 20, 30]);
}

#[test]
fn iterator_empty_after_clearing() {
    let mut bm = bitmap_of(&[1, 2, 3]);
    bm.remove(1);
    bm.remove(2);
    bm.remove(3);
    assert!(bm.is_empty());
    assert_eq!(bm.iter().count(), 0);
}

#[test]
fn iterator_multiple_iterators_concurrent() {
    let bm = bitmap_of(&[1, 5, 10, 15, 20]);

    let mut iter1 = bm.iter();
    let mut iter2 = bm.iter();

    // Advance iter1 partially
    assert_eq!(iter1.next(), Some(1));
    assert_eq!(iter1.next(), Some(5));

    // iter2 should start from beginning
    assert_eq!(iter2.next(), Some(1));
    assert_eq!(iter2.next(), Some(5));
    assert_eq!(iter2.next(), Some(10));

    // Continue iter1
    assert_eq!(iter1.next(), Some(10));
    assert_eq!(iter1.next(), Some(15));
}

#[test]
fn iterator_exhaustive_all_values_once() {
    let mut bm = RoaringBitmap::new();

    // Insert known pattern
    let expected: Vec<u32> = (0..1000).step_by(7).collect();
    for &val in &expected {
        bm.insert(val);
    }

    // Collect and verify
    let mut actual: Vec<u32> = bm.iter().collect();
    actual.sort();

    assert_eq!(actual.len(), expected.len());
    assert_eq!(actual, expected);
}

#[test]
fn iterator_fused_continues_to_return_none() {
    let bm = bitmap_of(&[1, 2]);
    let mut iter = bm.iter();

    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), None);

    // Fused iterator continues to return None
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

// ============================================================================
// Container Boundary Tests
// ============================================================================

#[test]
fn iterator_at_container_boundaries() {
    let mut bm = RoaringBitmap::new();

    // Values at container boundaries
    bm.insert(65535);    // End of container 0
    bm.insert(65536);    // Start of container 1
    bm.insert(131071);   // End of container 1
    bm.insert(131072);   // Start of container 2

    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values, vec![65535, 65536, 131071, 131072]);
}

#[test]
fn iterator_crossing_multiple_container_boundaries() {
    let mut bm = RoaringBitmap::new();

    // Add values in multiple containers
    for container_idx in 0..5 {
        let base = container_idx * 65536;
        bm.insert(base);         // First value in container
        bm.insert(base + 100);   // Middle value
        bm.insert(base + 65535); // Last value in container
    }

    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 15);

    // Verify sorted order
    for i in 1..values.len() {
        assert!(values[i - 1] < values[i]);
    }
}

#[test]
fn iterator_container_boundary_transitions() {
    let mut bm = RoaringBitmap::new();

    // Fill end of one container and start of next
    for i in 65530..65540 {
        bm.insert(i);
    }

    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 10);
    assert_eq!(values[0], 65530);
    assert_eq!(values[4], 65534);
    assert_eq!(values[5], 65535); // Container boundary
    assert_eq!(values[6], 65536); // Next container
    assert_eq!(values[9], 65539);
}

// ============================================================================
// Large Dataset Tests
// ============================================================================

#[test]
fn iterator_large_bitmap_one_million_values() {
    let mut bm = RoaringBitmap::new();

    // Insert 1 million sparse values
    for i in (0..10_000_000).step_by(10) {
        bm.insert(i);
    }

    let count = bm.iter().count();
    assert_eq!(count, 1_000_000);

    // Verify first and last
    let mut iter = bm.iter();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.last(), Some(9_999_990));
}

#[test]
fn iterator_large_consecutive_range() {
    let mut bm = RoaringBitmap::new();

    // Insert large consecutive range across multiple containers
    for i in 0..200_000 {
        bm.insert(i);
    }
    bm.optimize();

    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 200_000);

    // Spot check boundaries
    assert_eq!(values[0], 0);
    assert_eq!(values[65535], 65535);
    assert_eq!(values[65536], 65536);
    assert_eq!(values[131071], 131071);
    assert_eq!(values[131072], 131072);
    assert_eq!(values[199999], 199999);
}

// ============================================================================
// Size Hint Tests (Note: Current iterator implementation doesn't provide size_hint)
// ============================================================================

// TODO: Add size_hint tests when/if size_hint is implemented in the iterator

#[test]
fn iterator_count_method() {
    let bm = bitmap_of(&[1, 2, 5, 10, 100]);
    assert_eq!(bm.iter().count(), 5);

    let mut large_bm = RoaringBitmap::new();
    for i in 0..10000 {
        large_bm.insert(i);
    }
    assert_eq!(large_bm.iter().count(), 10000);
}

// ============================================================================
// Iterator Trait Method Tests
// ============================================================================

#[test]
fn iterator_collect_into_vec() {
    let bm = bitmap_of(&[1, 3, 5, 7, 9]);
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values, vec![1, 3, 5, 7, 9]);
}

#[test]
fn iterator_nth_method() {
    let bm = bitmap_of(&[10, 20, 30, 40, 50]);
    let mut iter = bm.iter();

    assert_eq!(iter.nth(0), Some(10));
    assert_eq!(iter.nth(1), Some(30)); // Skips 20
    assert_eq!(iter.nth(0), Some(40));
    assert_eq!(iter.nth(0), Some(50));
    assert_eq!(iter.nth(0), None);
}

#[test]
fn iterator_last_method() {
    let bm = bitmap_of(&[1, 5, 10, 15, 20, 100]);
    assert_eq!(bm.iter().last(), Some(100));

    let empty = RoaringBitmap::new();
    assert_eq!(empty.iter().last(), None);
}

#[test]
fn iterator_min_max_methods() {
    let bm = bitmap_of(&[5, 2, 9, 1, 7]);

    assert_eq!(bm.iter().min(), Some(1));
    assert_eq!(bm.iter().max(), Some(9));
}

#[test]
fn iterator_sum_method() {
    let bm = bitmap_of(&[1, 2, 3, 4, 5]);
    let sum: u32 = bm.iter().sum();
    assert_eq!(sum, 15);
}

#[test]
fn iterator_any_all_methods() {
    let bm = bitmap_of(&[2, 4, 6, 8, 10]);

    assert!(bm.iter().any(|x| x == 6));
    assert!(!bm.iter().any(|x| x == 5));

    assert!(bm.iter().all(|x| x % 2 == 0));
    assert!(!bm.iter().all(|x| x < 8));
}

#[test]
fn iterator_filter_map() {
    let bm = bitmap_of(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    // Filter even numbers and double them
    let result: Vec<u32> = bm
        .iter()
        .filter(|&x| x % 2 == 0)
        .map(|x| x * 2)
        .collect();

    assert_eq!(result, vec![4, 8, 12, 16, 20]);
}

// ============================================================================
// Container-Specific Iterator Tests
// ============================================================================

#[test]
fn iterator_array_container() {
    // Array container: < 4096 values, sparse
    let mut bm = RoaringBitmap::new();
    for i in (0..100).step_by(10) {
        bm.insert(i);
    }

    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 10);
    for (idx, &val) in values.iter().enumerate() {
        assert_eq!(val, (idx * 10) as u32);
    }
}

#[test]
fn iterator_bitmap_container() {
    // Bitmap container: Dense values
    let mut bm = RoaringBitmap::new();

    // Insert values to create bitmap container
    for i in (0..10000).step_by(2) {
        bm.insert(i);
    }

    // Verify iteration order and completeness
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 5000);

    // Check that all values are even and in order
    for (idx, &val) in values.iter().enumerate() {
        assert_eq!(val, (idx * 2) as u32);
    }
}

#[test]
fn bitmap_container_iteration() {
    let mut bm = RoaringBitmap::new();

    // Insert values to create bitmap container
    for i in (0..10000).step_by(2) {
        bm.insert(i);
    }

    // Verify iteration order and completeness
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 5000);

    // Check that all values are even and in order
    for (idx, &val) in values.iter().enumerate() {
        assert_eq!(val, (idx * 2) as u32);
    }
}

#[test]
fn iterator_run_container() {
    // Run container: Consecutive values
    let mut bm = RoaringBitmap::new();

    // Insert consecutive range to create run container
    for i in 1000..2000 {
        bm.insert(i);
    }
    bm.optimize();

    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 1000);
    assert_eq!(values[0], 1000);
    assert_eq!(values[999], 1999);

    // Verify all consecutive
    for (idx, &val) in values.iter().enumerate() {
        assert_eq!(val, 1000 + idx as u32);
    }
}

#[test]
fn iterator_mixed_container_types() {
    let mut bm = RoaringBitmap::new();

    // Array container: sparse low values
    for i in (0..50).step_by(5) {
        bm.insert(i);
    }

    // Run container: consecutive mid values
    for i in 70000..71000 {
        bm.insert(i);
    }
    bm.optimize();

    // Bitmap container: dense high values
    for i in (100000..110000).step_by(2) {
        bm.insert(i);
    }

    // Verify iteration is sorted across all containers
    let values: Vec<u32> = bm.iter().collect();
    let expected_len = 10 + 1000 + 5000;
    assert_eq!(values.len(), expected_len);

    // Verify sorted order
    for i in 1..values.len() {
        assert!(values[i - 1] < values[i], "Not sorted at index {}", i);
    }
}

#[test]
fn iterator_with_empty_containers_between() {
    let mut bm = RoaringBitmap::new();

    // Container 0: values 0-10
    for i in 0..10 {
        bm.insert(i);
    }

    // Skip containers 1-9 (empty)

    // Container 10: values 655360-655370 (10 * 65536 + 0..10)
    for i in 0..10 {
        bm.insert(10 * 65536 + i);
    }

    // Container 20: values 1310720-1310730 (20 * 65536 + 0..10)
    for i in 0..10 {
        bm.insert(20 * 65536 + i);
    }

    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 30);

    // Verify first container
    for i in 0..10 {
        assert_eq!(values[i], i as u32);
    }

    // Verify second container
    for i in 0..10 {
        assert_eq!(values[10 + i], (10 * 65536 + i) as u32);
    }

    // Verify third container
    for i in 0..10 {
        assert_eq!(values[20 + i], (20 * 65536 + i) as u32);
    }
}

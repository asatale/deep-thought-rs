/// Tests for container-level set operations
/// These tests specifically target different container type combinations
/// to ensure all code paths in Container::union, intersection, difference, etc. are covered

use crate::functional::{bitmap_of, expect_bitmap};
use roaring_bitmap::RoaringBitmap;

// ============================================================================
// Union Tests - Mixed Container Types
// ============================================================================

#[test]
fn union_bitmap_with_bitmap() {
    // Test Bitmap-Bitmap union
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    // Create two bitmap containers
    for i in 0..5000 {
        a.insert(i);
    }

    for i in 4000..9000 {
        b.insert(i);
    }

    // Union should have all values from both
    let union = a.union(&b);
    assert_eq!(union.len(), 9000); // 0..9000

    assert!(union.contains(0));
    assert!(union.contains(4000));
    assert!(union.contains(8999));
    assert!(!union.contains(9000));
}

#[test]
fn union_array_array_converts_to_bitmap() {
    // Create two array containers whose union exceeds threshold (4096)
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    // Insert 2500 values in 'a' (even numbers 0-5000)
    for i in (0..5000).step_by(2) {
        a.insert(i);
    }

    // Insert 2500 values in 'b' (odd numbers 1-5001)
    for i in (1..5001).step_by(2) {
        b.insert(i);
    }

    // Union should have 5000 values and trigger conversion to bitmap container
    let union = a.union(&b);
    assert_eq!(union.len(), 5000);

    // Verify all values present
    for i in 0..5000 {
        assert!(union.contains(i), "Union should contain {}", i);
    }
}

#[test]
fn union_array_with_bitmap() {
    let mut array_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    // Create array container (small, sparse)
    for i in [10, 20, 30, 40, 50] {
        array_bm.insert(i);
    }

    // Create bitmap container (dense, >4096 values)
    for i in 1000..6000 {
        bitmap_bm.insert(i);
    }

    // Union array + bitmap
    let union = array_bm.union(&bitmap_bm);
    assert_eq!(union.len(), 5005); // 5 from array + 5000 from bitmap

    // Verify array values present
    for &i in &[10, 20, 30, 40, 50] {
        assert!(union.contains(i));
    }

    // Verify bitmap values present (spot check)
    assert!(union.contains(1000));
    assert!(union.contains(3000));
    assert!(union.contains(5999));
}

#[test]
fn union_bitmap_with_array() {
    // Test the reverse: bitmap + array
    let mut bitmap_bm = RoaringBitmap::new();
    let mut array_bm = RoaringBitmap::new();

    // Create bitmap container
    for i in 0..5000 {
        bitmap_bm.insert(i);
    }

    // Create array container
    for i in [5001, 5002, 5003] {
        array_bm.insert(i);
    }

    let union = bitmap_bm.union(&array_bm);
    assert_eq!(union.len(), 5003);
    assert!(union.contains(4999));
    assert!(union.contains(5001));
    assert!(union.contains(5003));
}

#[test]
fn union_run_with_run() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    // Create run containers with consecutive values
    for i in 1000..2000 {
        a.insert(i);
    }
    a.optimize();

    for i in 1500..2500 {
        b.insert(i);
    }
    b.optimize();

    // Union of overlapping runs
    let union = a.union(&b);
    assert_eq!(union.len(), 1500); // 1000..2500 = 1500 values

    assert!(union.contains(1000));
    assert!(union.contains(1500));
    assert!(union.contains(2499));
    assert!(!union.contains(999));
    assert!(!union.contains(2500));
}

#[test]
fn union_run_with_array() {
    let mut run_bm = RoaringBitmap::new();
    let mut array_bm = RoaringBitmap::new();

    // Create run container
    for i in 1000..1100 {
        run_bm.insert(i);
    }
    run_bm.optimize();

    // Create array container
    for i in [10, 20, 30, 1050, 2000] {
        array_bm.insert(i);
    }

    // Test run + array
    let union1 = run_bm.union(&array_bm);
    assert_eq!(union1.len(), 104); // 100 from run + 4 new from array (1050 is already in run)
    assert!(union1.contains(10));
    assert!(union1.contains(1000));
    assert!(union1.contains(1050));
    assert!(union1.contains(2000));

    // Test array + run (commutative)
    let union2 = array_bm.union(&run_bm);
    assert_eq!(union2.len(), 104);
    assert_eq!(
        union1.iter().collect::<Vec<_>>(),
        union2.iter().collect::<Vec<_>>()
    );
}

#[test]
fn union_run_with_bitmap() {
    let mut run_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    // Create run container
    for i in 100..200 {
        run_bm.insert(i);
    }
    run_bm.optimize();

    // Create bitmap container
    for i in 1000..6000 {
        bitmap_bm.insert(i);
    }

    // Test run + bitmap
    let union1 = run_bm.union(&bitmap_bm);
    assert_eq!(union1.len(), 5100); // 100 from run + 5000 from bitmap

    // Test bitmap + run (commutative)
    let union2 = bitmap_bm.union(&run_bm);
    assert_eq!(union2.len(), 5100);

    // Verify values
    assert!(union1.contains(100));
    assert!(union1.contains(199));
    assert!(union1.contains(1000));
    assert!(union1.contains(5999));
}

// ============================================================================
// Intersection Tests - Mixed Container Types
// ============================================================================

#[test]
fn intersection_array_with_array() {
    // Basic array-array intersection
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    for i in [1, 3, 5, 7, 9] {
        a.insert(i);
    }

    for i in [2, 3, 5, 8, 9] {
        b.insert(i);
    }

    let intersection = a.intersection(&b);
    assert_eq!(intersection.len(), 3); // 3, 5, 9
    assert!(intersection.contains(3));
    assert!(intersection.contains(5));
    assert!(intersection.contains(9));
}

#[test]
fn intersection_array_with_array_empty() {
    // Disjoint arrays - no intersection
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    for i in [1, 2, 3] {
        a.insert(i);
    }

    for i in [10, 20, 30] {
        b.insert(i);
    }

    let intersection = a.intersection(&b);
    assert_eq!(intersection.len(), 0);
    assert!(intersection.is_empty());
}

#[test]
fn intersection_bitmap_bitmap_converts_to_array() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    // Create two bitmap containers with small intersection
    for i in 0..5000 {
        a.insert(i);
    }

    for i in 4900..10000 {
        b.insert(i);
    }

    // Intersection is only 100 values (4900-4999), should convert to array
    let intersection = a.intersection(&b);
    assert_eq!(intersection.len(), 100);

    for i in 4900..5000 {
        assert!(intersection.contains(i));
    }

    assert!(!intersection.contains(4899));
    assert!(!intersection.contains(5000));
}

#[test]
fn intersection_array_with_bitmap() {
    let mut array_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    // Array container
    for i in [100, 1000, 2000, 3000, 4000, 5000] {
        array_bm.insert(i);
    }

    // Bitmap container
    for i in 900..5100 {
        bitmap_bm.insert(i);
    }

    // Intersection should find values in bitmap
    let intersection = array_bm.intersection(&bitmap_bm);
    assert_eq!(intersection.len(), 5); // 1000, 2000, 3000, 4000, 5000

    assert!(!intersection.contains(100)); // Not in bitmap
    assert!(intersection.contains(1000));
    assert!(intersection.contains(5000));
}

#[test]
fn intersection_bitmap_with_array() {
    // Test reverse: bitmap & array
    let mut bitmap_bm = RoaringBitmap::new();
    let mut array_bm = RoaringBitmap::new();

    // Bitmap
    for i in 0..5000 {
        bitmap_bm.insert(i);
    }

    // Array with some values in bitmap, some not
    for i in [10, 4999, 5000, 5001] {
        array_bm.insert(i);
    }

    let intersection = bitmap_bm.intersection(&array_bm);
    assert_eq!(intersection.len(), 2); // Only 10 and 4999 are in both
    assert!(intersection.contains(10));
    assert!(intersection.contains(4999));
    assert!(!intersection.contains(5000));
}

#[test]
fn intersection_array_bitmap_empty() {
    let mut array_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    // Disjoint sets
    for i in 0..10 {
        array_bm.insert(i);
    }

    for i in 1000..6000 {
        bitmap_bm.insert(i);
    }

    let intersection = array_bm.intersection(&bitmap_bm);
    assert_eq!(intersection.len(), 0);
    assert!(intersection.is_empty());
}

#[test]
fn intersection_run_with_array() {
    let mut run_bm = RoaringBitmap::new();
    let mut array_bm = RoaringBitmap::new();

    // Run container
    for i in 1000..1100 {
        run_bm.insert(i);
    }
    run_bm.optimize();

    // Array with some overlap
    for i in [500, 1020, 1050, 1080, 2000] {
        array_bm.insert(i);
    }

    // Test both directions
    let intersection1 = run_bm.intersection(&array_bm);
    assert_eq!(intersection1.len(), 3); // 1020, 1050, 1080
    assert!(intersection1.contains(1020));
    assert!(intersection1.contains(1050));
    assert!(intersection1.contains(1080));

    let intersection2 = array_bm.intersection(&run_bm);
    assert_eq!(intersection2.len(), 3);
    assert_eq!(
        intersection1.iter().collect::<Vec<_>>(),
        intersection2.iter().collect::<Vec<_>>()
    );
}

#[test]
fn intersection_run_with_bitmap() {
    let mut run_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    // Run container
    for i in 1000..1200 {
        run_bm.insert(i);
    }
    run_bm.optimize();

    // Bitmap with partial overlap
    for i in 1100..6100 {
        bitmap_bm.insert(i);
    }

    // Intersection is 1100-1199 (100 values)
    let intersection1 = run_bm.intersection(&bitmap_bm);
    assert_eq!(intersection1.len(), 100);
    assert!(intersection1.contains(1100));
    assert!(intersection1.contains(1199));
    assert!(!intersection1.contains(1099));
    assert!(!intersection1.contains(1200));

    let intersection2 = bitmap_bm.intersection(&run_bm);
    assert_eq!(intersection2.len(), 100);
}

#[test]
fn intersection_run_with_run() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    // Overlapping runs
    for i in 1000..2000 {
        a.insert(i);
    }
    a.optimize();

    for i in 1500..2500 {
        b.insert(i);
    }
    b.optimize();

    let intersection = a.intersection(&b);
    assert_eq!(intersection.len(), 500); // 1500-1999
    assert!(intersection.contains(1500));
    assert!(intersection.contains(1999));
    assert!(!intersection.contains(1499));
    assert!(!intersection.contains(2000));
}

// ============================================================================
// Difference Tests - Mixed Container Types
// ============================================================================

#[test]
fn difference_array_with_array() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    for i in [1, 2, 3, 4, 5] {
        a.insert(i);
    }

    for i in [3, 4, 6, 7] {
        b.insert(i);
    }

    // a - b: keep 1, 2, 5
    let diff = a.difference(&b);
    assert_eq!(diff.len(), 3);
    assert!(diff.contains(1));
    assert!(diff.contains(2));
    assert!(diff.contains(5));
    assert!(!diff.contains(3));
    assert!(!diff.contains(4));
}

#[test]
fn difference_bitmap_with_bitmap() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    for i in 0..6000 {
        a.insert(i);
    }

    for i in 5000..10000 {
        b.insert(i);
    }

    // a - b: keep 0..5000
    let diff = a.difference(&b);
    assert_eq!(diff.len(), 5000);
    assert!(diff.contains(0));
    assert!(diff.contains(4999));
    assert!(!diff.contains(5000));
}

#[test]
fn difference_array_with_bitmap() {
    let mut array_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    for i in [10, 100, 1000, 2000, 3000] {
        array_bm.insert(i);
    }

    for i in 500..2500 {
        bitmap_bm.insert(i);
    }

    // array - bitmap: keep values not in bitmap
    // bitmap contains 500..2500, so from array: 1000, 2000 are in bitmap
    // Only 10, 100, 3000 remain
    let diff = array_bm.difference(&bitmap_bm);
    assert_eq!(diff.len(), 3); // 10, 100, and 3000 remain
    assert!(diff.contains(10));
    assert!(diff.contains(100));
    assert!(diff.contains(3000));
    assert!(!diff.contains(1000));
    assert!(!diff.contains(2000));
}

#[test]
fn difference_bitmap_with_array() {
    let mut bitmap_bm = RoaringBitmap::new();
    let mut array_bm = RoaringBitmap::new();

    for i in 0..5000 {
        bitmap_bm.insert(i);
    }

    for i in [100, 200, 300] {
        array_bm.insert(i);
    }

    // bitmap - array
    let diff = bitmap_bm.difference(&array_bm);
    assert_eq!(diff.len(), 4997); // 5000 - 3
    assert!(diff.contains(0));
    assert!(diff.contains(99));
    assert!(!diff.contains(100));
    assert!(!diff.contains(200));
    assert!(!diff.contains(300));
    assert!(diff.contains(301));
}

#[test]
fn difference_run_with_array() {
    let mut run_bm = RoaringBitmap::new();
    let mut array_bm = RoaringBitmap::new();

    for i in 1000..1100 {
        run_bm.insert(i);
    }
    run_bm.optimize();

    for i in [1020, 1050, 1080, 2000] {
        array_bm.insert(i);
    }

    let diff = run_bm.difference(&array_bm);
    assert_eq!(diff.len(), 97); // 100 - 3
    assert!(diff.contains(1000));
    assert!(diff.contains(1019));
    assert!(!diff.contains(1020));
    assert!(!diff.contains(1050));
    assert!(!diff.contains(1080));
    assert!(diff.contains(1099));
}

#[test]
fn difference_run_with_bitmap() {
    let mut run_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    for i in 1000..1200 {
        run_bm.insert(i);
    }
    run_bm.optimize();

    for i in 1100..6100 {
        bitmap_bm.insert(i);
    }

    // run - bitmap: keep 1000-1099
    let diff = run_bm.difference(&bitmap_bm);
    assert_eq!(diff.len(), 100);
    assert!(diff.contains(1000));
    assert!(diff.contains(1099));
    assert!(!diff.contains(1100));
}

#[test]
fn difference_run_with_run() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    for i in 1000..2000 {
        a.insert(i);
    }
    a.optimize();

    for i in 1500..2500 {
        b.insert(i);
    }
    b.optimize();

    // a - b: keep 1000-1499
    let diff = a.difference(&b);
    assert_eq!(diff.len(), 500);
    assert!(diff.contains(1000));
    assert!(diff.contains(1499));
    assert!(!diff.contains(1500));
}

// ============================================================================
// Symmetric Difference Tests - Mixed Container Types
// ============================================================================

#[test]
fn symmetric_difference_array_with_array() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    for i in [1, 2, 3, 4] {
        a.insert(i);
    }

    for i in [3, 4, 5, 6] {
        b.insert(i);
    }

    // Symmetric diff: (a - b) ∪ (b - a) = {1, 2} ∪ {5, 6}
    let sym_diff = a.symmetric_difference(&b);
    assert_eq!(sym_diff.len(), 4);
    assert!(sym_diff.contains(1));
    assert!(sym_diff.contains(2));
    assert!(sym_diff.contains(5));
    assert!(sym_diff.contains(6));
    assert!(!sym_diff.contains(3));
    assert!(!sym_diff.contains(4));
}

#[test]
fn symmetric_difference_bitmap_with_bitmap() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    for i in 0..6000 {
        a.insert(i);
    }

    for i in 5000..11000 {
        b.insert(i);
    }

    // Symmetric diff: {0..5000} ∪ {6000..11000}
    let sym_diff = a.symmetric_difference(&b);
    assert_eq!(sym_diff.len(), 10000); // 5000 + 5000

    assert!(sym_diff.contains(0));
    assert!(sym_diff.contains(4999));
    assert!(!sym_diff.contains(5000)); // In both
    assert!(!sym_diff.contains(5999)); // In both
    assert!(sym_diff.contains(6000));
    assert!(sym_diff.contains(10999));
}

#[test]
fn symmetric_difference_array_with_bitmap() {
    let mut array_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    for i in [10, 100, 1000, 2000] {
        array_bm.insert(i);
    }

    for i in 500..2500 {
        bitmap_bm.insert(i);
    }

    // Symmetric diff: (A - B) ∪ (B - A)
    let sym_diff = array_bm.symmetric_difference(&bitmap_bm);

    // Should have: 10, 100 (from array only) + all from bitmap except 1000, 2000
    assert!(sym_diff.contains(10));
    assert!(sym_diff.contains(100));
    assert!(!sym_diff.contains(1000)); // In both
    assert!(!sym_diff.contains(2000)); // In both
    assert!(sym_diff.contains(500));
    assert!(sym_diff.contains(2499));
}

#[test]
fn symmetric_difference_run_with_array() {
    let mut run_bm = RoaringBitmap::new();
    let mut array_bm = RoaringBitmap::new();

    for i in 1000..1100 {
        run_bm.insert(i);
    }
    run_bm.optimize();

    for i in [1050, 1080, 2000, 3000] {
        array_bm.insert(i);
    }

    let sym_diff = run_bm.symmetric_difference(&array_bm);

    // Should have: run values except 1050, 1080 + 2000, 3000 from array
    assert_eq!(sym_diff.len(), 100); // 98 from run + 2 from array
    assert!(sym_diff.contains(1000));
    assert!(!sym_diff.contains(1050));
    assert!(!sym_diff.contains(1080));
    assert!(sym_diff.contains(2000));
    assert!(sym_diff.contains(3000));
}

#[test]
fn symmetric_difference_run_with_bitmap() {
    let mut run_bm = RoaringBitmap::new();
    let mut bitmap_bm = RoaringBitmap::new();

    for i in 1000..1200 {
        run_bm.insert(i);
    }
    run_bm.optimize();

    for i in 1100..1300 {
        bitmap_bm.insert(i);
    }

    let sym_diff = run_bm.symmetric_difference(&bitmap_bm);

    // Should have: 1000-1099 (run only) + 1200-1299 (bitmap only)
    assert_eq!(sym_diff.len(), 200);
    assert!(sym_diff.contains(1000));
    assert!(sym_diff.contains(1099));
    assert!(!sym_diff.contains(1100)); // In both
    assert!(!sym_diff.contains(1199)); // In both
    assert!(sym_diff.contains(1200));
    assert!(sym_diff.contains(1299));
}

#[test]
fn symmetric_difference_run_with_run() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    for i in 1000..2000 {
        a.insert(i);
    }
    a.optimize();

    for i in 1500..2500 {
        b.insert(i);
    }
    b.optimize();

    let sym_diff = a.symmetric_difference(&b);

    // Should have: 1000-1499 (a only) + 2000-2499 (b only)
    assert_eq!(sym_diff.len(), 1000);
    assert!(sym_diff.contains(1000));
    assert!(sym_diff.contains(1499));
    assert!(!sym_diff.contains(1500)); // In both
    assert!(!sym_diff.contains(1999)); // In both
    assert!(sym_diff.contains(2000));
    assert!(sym_diff.contains(2499));
}

use crate::functional::{bitmap_of, expect_bitmap};
use roaring_bitmap::RoaringBitmap;

#[test]
fn operator_union_allocating() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[3, 4, 5]);

    let result = &a | &b;

    expect_bitmap(&result, &[1, 2, 3, 4, 5]);
    // Verify originals unchanged
    expect_bitmap(&a, &[1, 2, 3]);
    expect_bitmap(&b, &[3, 4, 5]);
}

#[test]
fn operator_union_inplace() {
    let mut a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[3, 4, 5]);

    a |= &b;

    expect_bitmap(&a, &[1, 2, 3, 4, 5]);
}

#[test]
fn operator_intersection_allocating() {
    let a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    let result = &a & &b;

    expect_bitmap(&result, &[3, 4]);
}

#[test]
fn operator_intersection_inplace() {
    let mut a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    a &= &b;

    expect_bitmap(&a, &[3, 4]);
}

#[test]
fn operator_symmetric_difference_allocating() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[2, 3, 4]);

    let result = &a ^ &b;

    expect_bitmap(&result, &[1, 4]);
}

#[test]
fn operator_symmetric_difference_inplace() {
    let mut a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[2, 3, 4]);

    a ^= &b;

    expect_bitmap(&a, &[1, 4]);
}

#[test]
fn operator_difference_allocating() {
    let a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    let result = &a - &b;

    expect_bitmap(&result, &[1, 2]);
}

#[test]
fn operator_difference_inplace() {
    let mut a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    a -= &b;

    expect_bitmap(&a, &[1, 2]);
}

#[test]
fn operator_chaining_allocating() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[2, 3, 4]);
    let c = bitmap_of(&[3, 4, 5]);

    // Chain operations using operators
    let result = (&a | &b) & &c;

    expect_bitmap(&result, &[3, 4]);
}

#[test]
fn operator_chaining_inplace() {
    let mut result = bitmap_of(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let filter1 = bitmap_of(&[2, 4, 6, 8, 10]);
    let filter2 = bitmap_of(&[3, 6, 9]);
    let filter3 = bitmap_of(&[6, 12, 18]);

    // Apply filters using in-place operators
    result &= &filter1;
    result &= &filter2;
    result &= &filter3;

    expect_bitmap(&result, &[6]);
}

#[test]
fn operator_complex_expression() {
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6, 7]);
    let c = bitmap_of(&[5, 6, 7, 8, 9]);

    // Complex expression: (A ∪ B) - C
    let result = (&a | &b) - &c;

    expect_bitmap(&result, &[1, 2, 3, 4]);
}

#[test]
fn operator_ergonomics_workflow() {
    // Demonstrate ergonomic workflow with operators
    let mut candidates = RoaringBitmap::new();
    for i in 1..=100 {
        candidates.insert(i);
    }

    let mut evens = RoaringBitmap::new();
    for i in (2..=100).step_by(2) {
        evens.insert(i);
    }

    let mut multiples_of_3 = RoaringBitmap::new();
    for i in (3..=100).step_by(3) {
        multiples_of_3.insert(i);
    }

    // Find numbers that are even OR multiples of 3
    let result = &evens | &multiples_of_3;

    // Verify some expected values
    assert!(result.contains(2)); // Even
    assert!(result.contains(3)); // Multiple of 3
    assert!(result.contains(6)); // Both
    assert!(!result.contains(1)); // Neither
    assert!(!result.contains(5)); // Neither
}

#[test]
fn operator_matches_method_results() {
    // Verify operators produce same results as methods
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6, 7]);

    // Union
    assert_eq!(
        (&a | &b).iter().collect::<Vec<_>>(),
        a.union(&b).iter().collect::<Vec<_>>()
    );

    // Intersection
    assert_eq!(
        (&a & &b).iter().collect::<Vec<_>>(),
        a.intersection(&b).iter().collect::<Vec<_>>()
    );

    // Symmetric difference
    assert_eq!(
        (&a ^ &b).iter().collect::<Vec<_>>(),
        a.symmetric_difference(&b).iter().collect::<Vec<_>>()
    );

    // Difference
    assert_eq!(
        (&a - &b).iter().collect::<Vec<_>>(),
        a.difference(&b).iter().collect::<Vec<_>>()
    );
}

#[test]
fn operator_assign_matches_method_results() {
    // Verify in-place operators produce same results as in-place methods
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6, 7]);

    // Union
    {
        let mut a1 = a.clone();
        let mut a2 = a.clone();
        a1 |= &b;
        a2.union_with(&b);
        assert_eq!(a1.iter().collect::<Vec<_>>(), a2.iter().collect::<Vec<_>>());
    }

    // Intersection
    {
        let mut a1 = a.clone();
        let mut a2 = a.clone();
        a1 &= &b;
        a2.intersect_with(&b);
        assert_eq!(a1.iter().collect::<Vec<_>>(), a2.iter().collect::<Vec<_>>());
    }

    // Symmetric difference
    {
        let mut a1 = a.clone();
        let mut a2 = a.clone();
        a1 ^= &b;
        a2.symmetric_difference_with(&b);
        assert_eq!(a1.iter().collect::<Vec<_>>(), a2.iter().collect::<Vec<_>>());
    }

    // Difference
    {
        let mut a1 = a.clone();
        let mut a2 = a.clone();
        a1 -= &b;
        a2.difference_with(&b);
        assert_eq!(a1.iter().collect::<Vec<_>>(), a2.iter().collect::<Vec<_>>());
    }
}

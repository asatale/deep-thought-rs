use crate::functional::{bitmap_of, expect_bitmap};
use roaring_bitmap::RoaringBitmap;

#[test]
fn inplace_union_with_basic() {
    let mut a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[3, 4, 5]);

    a.union_with(&b);

    expect_bitmap(&a, &[1, 2, 3, 4, 5]);
}

#[test]
fn inplace_union_with_empty() {
    let mut a = bitmap_of(&[1, 2, 3]);
    let empty = RoaringBitmap::new();

    a.union_with(&empty);
    expect_bitmap(&a, &[1, 2, 3]);
}

#[test]
fn inplace_union_with_incremental() {
    // Test building up a union incrementally (common workflow)
    let bitmaps = vec![
        bitmap_of(&[1, 2]),
        bitmap_of(&[3, 4]),
        bitmap_of(&[5, 6]),
        bitmap_of(&[7, 8]),
    ];

    let mut result = RoaringBitmap::new();
    for bitmap in &bitmaps {
        result.union_with(bitmap);
    }

    expect_bitmap(&result, &[1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn inplace_intersect_with_basic() {
    let mut a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6, 7]);

    a.intersect_with(&b);

    expect_bitmap(&a, &[3, 4, 5]);
}

#[test]
fn inplace_intersect_with_disjoint() {
    let mut a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[4, 5, 6]);

    a.intersect_with(&b);

    expect_bitmap(&a, &[]);
    assert!(a.is_empty());
}

#[test]
fn inplace_intersect_with_filters() {
    // Test applying multiple filters (common workflow)
    let mut candidates = bitmap_of(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let filter1 = bitmap_of(&[2, 4, 6, 8, 10]); // Even numbers
    let filter2 = bitmap_of(&[3, 6, 9]); // Multiples of 3
    let filter3 = bitmap_of(&[1, 2, 3, 4, 5, 6]); // <= 6

    candidates.intersect_with(&filter1);
    candidates.intersect_with(&filter2);
    candidates.intersect_with(&filter3);

    // Only 6 satisfies all filters (even, multiple of 3, <= 6)
    expect_bitmap(&candidates, &[6]);
}

#[test]
fn inplace_difference_with_basic() {
    let mut a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6, 7]);

    a.difference_with(&b);

    expect_bitmap(&a, &[1, 2]);
}

#[test]
fn inplace_difference_with_empty() {
    let mut a = bitmap_of(&[1, 2, 3]);
    let empty = RoaringBitmap::new();

    a.difference_with(&empty);
    expect_bitmap(&a, &[1, 2, 3]);
}

#[test]
fn inplace_symmetric_difference_with_basic() {
    let mut a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[2, 3, 4]);

    a.symmetric_difference_with(&b);

    expect_bitmap(&a, &[1, 4]);
}

#[test]
fn inplace_symmetric_difference_with_identical() {
    let mut a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[1, 2, 3]);

    a.symmetric_difference_with(&b);

    expect_bitmap(&a, &[]);
    assert!(a.is_empty());
}

#[test]
fn inplace_operations_preserve_correctness() {
    // Verify in-place operations produce same results as allocating versions
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6, 7]);

    // Test union
    {
        let mut a_mut = a.clone();
        a_mut.union_with(&b);
        let a_alloc = a.union(&b);
        assert_eq!(
            a_mut.iter().collect::<Vec<_>>(),
            a_alloc.iter().collect::<Vec<_>>()
        );
    }

    // Test intersection
    {
        let mut a_mut = a.clone();
        a_mut.intersect_with(&b);
        let a_alloc = a.intersection(&b);
        assert_eq!(
            a_mut.iter().collect::<Vec<_>>(),
            a_alloc.iter().collect::<Vec<_>>()
        );
    }

    // Test difference
    {
        let mut a_mut = a.clone();
        a_mut.difference_with(&b);
        let a_alloc = a.difference(&b);
        assert_eq!(
            a_mut.iter().collect::<Vec<_>>(),
            a_alloc.iter().collect::<Vec<_>>()
        );
    }

    // Test symmetric difference
    {
        let mut a_mut = a.clone();
        a_mut.symmetric_difference_with(&b);
        let a_alloc = a.symmetric_difference(&b);
        assert_eq!(
            a_mut.iter().collect::<Vec<_>>(),
            a_alloc.iter().collect::<Vec<_>>()
        );
    }
}

#[test]
fn inplace_operations_across_containers() {
    // Test in-place operations with values spanning multiple containers
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();

    // Container 0: values 0-100
    for i in 0..100 {
        a.insert(i);
    }
    // Container 1: values 65536-65636
    for i in 65536..65636 {
        a.insert(i);
    }

    // Container 0: values 50-150
    for i in 50..150 {
        b.insert(i);
    }
    // Container 1: values 65586-65686
    for i in 65586..65686 {
        b.insert(i);
    }

    let original_a = a.clone();

    // Test union
    a.union_with(&b);
    assert!(a.len() > original_a.len());
    for i in 0..150 {
        assert!(a.contains(i));
    }
    for i in 65536..65686 {
        assert!(a.contains(i));
    }
}

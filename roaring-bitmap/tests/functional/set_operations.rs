use crate::functional::{bitmap_of, expect_bitmap};
use roaring_bitmap::RoaringBitmap;

#[test]
fn union_with_empty_and_self() {
    let a = bitmap_of(&[1, 2, 3]);
    let empty = RoaringBitmap::new();
    expect_bitmap(&a.union(&empty), &[1, 2, 3]);
    expect_bitmap(&empty.union(&a), &[1, 2, 3]);
    expect_bitmap(&a.union(&a), &[1, 2, 3]);
}

#[test]
fn union_intersection_difference_and_symmetric_difference_overlap() {
    let a = bitmap_of(&[1, 2, 3, 10, 11]);
    let b = bitmap_of(&[3, 4, 5, 10, 20]);
    expect_bitmap(&a.union(&b), &[1, 2, 3, 4, 5, 10, 11, 20]);
    expect_bitmap(&a.intersection(&b), &[3, 10]);
    expect_bitmap(&a.difference(&b), &[1, 2, 11]);
    expect_bitmap(&b.difference(&a), &[4, 5, 20]);
    expect_bitmap(&a.symmetric_difference(&b), &[1, 2, 4, 5, 11, 20]);
    assert_eq!(
        a.union(&b).len(),
        a.len() + b.len() - a.intersection(&b).len()
    );
}

#[test]
fn difference_with_superset_yields_empty() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[1, 2, 3, 4]);
    expect_bitmap(&a.difference(&b), &[]);
    expect_bitmap(&b.difference(&a), &[4]);
}

#[test]
fn symmetric_difference_with_identical_bitmaps_is_empty() {
    let a = bitmap_of(&[100, 101, 102]);
    expect_bitmap(&a.symmetric_difference(&a), &[]);
}

#[test]
fn large_sparse_and_dense_mix() {
    let mut sparse = RoaringBitmap::new();
    for value in (0..1_000_000).step_by(1000) {
        assert!(sparse.insert(value));
    }
    let mut dense = RoaringBitmap::new();
    for value in 500_000..500_500 {
        assert!(dense.insert(value));
    }
    let union = sparse.union(&dense);
    for value in sparse.iter().chain(dense.iter()) {
        assert!(union.contains(value));
    }
    let intersection = sparse.intersection(&dense);
    expect_bitmap(&intersection, &[500_000]);
    let diff = dense.difference(&sparse);
    assert_eq!(diff.len() as usize, dense.len() as usize - 1);
}

#[test]
fn union_is_commutative() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[3, 4, 5]);
    expect_bitmap(&a.union(&b), &[1, 2, 3, 4, 5]);
    expect_bitmap(&b.union(&a), &[1, 2, 3, 4, 5]);
}

#[test]
fn intersection_is_commutative() {
    let a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);
    expect_bitmap(&a.intersection(&b), &[3, 4]);
    expect_bitmap(&b.intersection(&a), &[3, 4]);
}

#[test]
fn symmetric_difference_is_commutative() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[2, 3, 4]);
    expect_bitmap(&a.symmetric_difference(&b), &[1, 4]);
    expect_bitmap(&b.symmetric_difference(&a), &[1, 4]);
}

#[test]
fn union_is_associative() {
    let a = bitmap_of(&[1, 2]);
    let b = bitmap_of(&[2, 3]);
    let c = bitmap_of(&[3, 4]);
    let left = a.union(&b).union(&c);
    let right = a.union(&b.union(&c));
    expect_bitmap(&left, &[1, 2, 3, 4]);
    expect_bitmap(&right, &[1, 2, 3, 4]);
}

#[test]
fn intersection_is_associative() {
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[2, 3, 4, 5, 6]);
    let c = bitmap_of(&[3, 4, 5, 6, 7]);
    let left = a.intersection(&b).intersection(&c);
    let right = a.intersection(&b.intersection(&c));
    expect_bitmap(&left, &[3, 4, 5]);
    expect_bitmap(&right, &[3, 4, 5]);
}

#[test]
fn intersection_is_idempotent() {
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    expect_bitmap(&a.intersection(&a), &[1, 2, 3, 4, 5]);
}

#[test]
fn union_is_idempotent() {
    let a = bitmap_of(&[10, 20, 30]);
    expect_bitmap(&a.union(&a), &[10, 20, 30]);
}

#[test]
fn difference_with_self_is_empty() {
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    expect_bitmap(&a.difference(&a), &[]);
}

#[test]
fn empty_operations_are_identity() {
    let empty = RoaringBitmap::new();
    let a = bitmap_of(&[1, 2, 3]);

    // Union with empty
    expect_bitmap(&a.union(&empty), &[1, 2, 3]);
    expect_bitmap(&empty.union(&a), &[1, 2, 3]);

    // Intersection with empty
    expect_bitmap(&a.intersection(&empty), &[]);
    expect_bitmap(&empty.intersection(&a), &[]);

    // Difference with empty
    expect_bitmap(&a.difference(&empty), &[1, 2, 3]);
    expect_bitmap(&empty.difference(&a), &[]);

    // Symmetric difference with empty
    expect_bitmap(&a.symmetric_difference(&empty), &[1, 2, 3]);
    expect_bitmap(&empty.symmetric_difference(&a), &[1, 2, 3]);
}

#[test]
fn de_morgans_laws() {
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[4, 5, 6, 7, 8]);
    let universe = bitmap_of(&[1, 2, 3, 4, 5, 6, 7, 8]);

    // not(A ∪ B) = not(A) ∩ not(B)
    let union_complement = universe.difference(&a.union(&b));
    let complement_intersection = universe
        .difference(&a)
        .intersection(&universe.difference(&b));
    expect_bitmap(&union_complement, &[]);
    expect_bitmap(&complement_intersection, &[]);

    // not(A ∩ B) = not(A) ∪ not(B)
    let intersection_complement = universe.difference(&a.intersection(&b));
    let complement_union = universe.difference(&a).union(&universe.difference(&b));
    expect_bitmap(&intersection_complement, &[1, 2, 3, 6, 7, 8]);
    expect_bitmap(&complement_union, &[1, 2, 3, 6, 7, 8]);
}

#[test]
fn disjoint_sets_operations() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[10, 20, 30]);

    expect_bitmap(&a.intersection(&b), &[]);
    expect_bitmap(&a.union(&b), &[1, 2, 3, 10, 20, 30]);
    expect_bitmap(&a.difference(&b), &[1, 2, 3]);
    expect_bitmap(&b.difference(&a), &[10, 20, 30]);
    expect_bitmap(&a.symmetric_difference(&b), &[1, 2, 3, 10, 20, 30]);
}

#[test]
fn values_across_container_boundaries() {
    let mut bm = RoaringBitmap::new();
    // Container boundaries are at multiples of 65536 (2^16)
    bm.insert(65535); // Last value in first container
    bm.insert(65536); // First value in second container
    bm.insert(65537);
    assert_eq!(bm.len(), 3);
    expect_bitmap(&bm, &[65535, 65536, 65537]);
}

#[test]
fn operations_across_multiple_containers() {
    let mut a = RoaringBitmap::new();
    let mut b = RoaringBitmap::new();
    // Spread values across multiple containers
    for i in 0..5 {
        a.insert(i * 65536);
        a.insert(i * 65536 + 100);
        b.insert(i * 65536 + 100);
        b.insert(i * 65536 + 200);
    }
    let intersection = a.intersection(&b);
    assert_eq!(intersection.len(), 5); // Only the +100 values overlap
    for i in 0..5 {
        assert!(intersection.contains(i * 65536 + 100));
    }
}

#[test]
fn container_boundary_edge_values() {
    let mut bm = RoaringBitmap::new();
    let boundaries = vec![0, 65535, 65536, 131071, 131072, u32::MAX - 1, u32::MAX];
    for &value in &boundaries {
        bm.insert(value);
    }
    for &value in &boundaries {
        assert!(bm.contains(value));
    }
    assert_eq!(bm.len(), boundaries.len() as u64);
}

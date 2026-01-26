use crate::functional::{bitmap_of, expect_bitmap};
use roaring_bitmap::RoaringBitmap;

#[test]
fn new_bitmap_is_empty() {
    let bm = RoaringBitmap::new();
    assert_eq!(bm.len(), 0);
    assert!(bm.is_empty());
    assert!(!bm.contains(0));
    assert!(bm.iter().next().is_none());
}

#[test]
fn insert_reports_true_only_for_new_values() {
    let mut bm = RoaringBitmap::new();
    assert!(bm.insert(42));
    assert!(!bm.insert(42));
    assert!(bm.insert(7));
    expect_bitmap(&bm, &[7, 42]);
}

#[test]
fn contains_and_remove_work_for_extreme_values() {
    let mut bm = RoaringBitmap::new();
    assert!(bm.insert(0));
    assert!(bm.insert(u32::MAX));
    assert!(bm.contains(0));
    assert!(bm.contains(u32::MAX));
    assert!(bm.remove(0));
    assert!(!bm.contains(0));
    assert!(bm.remove(u32::MAX));
    assert!(!bm.remove(u32::MAX));
    assert!(bm.is_empty());
}

#[test]
fn len_and_is_empty_track_updates() {
    let mut bm = RoaringBitmap::new();
    for value in [1, 2, 3, 4, 5] {
        assert!(bm.insert(value));
    }
    assert_eq!(bm.len(), 5);
    assert!(!bm.is_empty());
    assert!(bm.remove(3));
    assert_eq!(bm.len(), 4);
    assert!(!bm.remove(99));
    expect_bitmap(&bm, &[1, 2, 4, 5]);
}

#[test]
fn repeated_insert_remove_cycles_leave_bitmap_consistent() {
    let mut bm = RoaringBitmap::new();
    for round in 0..10 {
        for value in 1000 * round..1000 * round + 500 {
            assert!(bm.insert(value));
        }
        for value in 1000 * round..1000 * round + 250 {
            assert!(bm.remove(value));
        }
    }
    let collected: Vec<u32> = bm.iter().collect();
    for window in collected.windows(2) {
        assert!(window[0] < window[1]);
    }
    assert_eq!(bm.len() as usize, collected.len());
}

#[test]
fn remove_from_empty_bitmap() {
    let mut bm = RoaringBitmap::new();
    assert!(!bm.remove(42));
    assert!(bm.is_empty());
}

#[test]
fn contains_on_empty_bitmap() {
    let bm = RoaringBitmap::new();
    assert!(!bm.contains(0));
    assert!(!bm.contains(u32::MAX));
    assert!(!bm.contains(12345));
}

#[test]
fn single_element_operations() {
    let a = bitmap_of(&[42]);
    let b = bitmap_of(&[42]);
    let c = bitmap_of(&[99]);

    expect_bitmap(&a.union(&b), &[42]);
    expect_bitmap(&a.intersection(&b), &[42]);
    expect_bitmap(&a.difference(&b), &[]);
    expect_bitmap(&a.symmetric_difference(&b), &[]);

    expect_bitmap(&a.union(&c), &[42, 99]);
    expect_bitmap(&a.intersection(&c), &[]);
    expect_bitmap(&a.difference(&c), &[42]);
    expect_bitmap(&a.symmetric_difference(&c), &[42, 99]);
}

#[test]
fn alternating_insert_remove() {
    let mut bm = RoaringBitmap::new();
    for i in 0..1000 {
        assert!(bm.insert(i));
        assert!(bm.contains(i));
        assert!(bm.remove(i));
        assert!(!bm.contains(i));
    }
    assert!(bm.is_empty());
}

#[test]
fn insert_sequential_vs_random_order() {
    let mut sequential = RoaringBitmap::new();
    let mut random = RoaringBitmap::new();

    let values = vec![42, 7, 1000, 3, 999, 1, 500, 250];

    for &v in &values {
        sequential.insert(v);
    }

    for &v in values.iter().rev() {
        random.insert(v);
    }

    let seq_vec: Vec<u32> = sequential.iter().collect();
    let rand_vec: Vec<u32> = random.iter().collect();
    assert_eq!(seq_vec, rand_vec);
}

#[test]
fn len_overflow_safety() {
    let mut bm = RoaringBitmap::new();
    // Insert many elements to test u64 cardinality
    for i in 0..100_000 {
        bm.insert(i);
    }
    assert_eq!(bm.len(), 100_000);
}

use crate::functional::{bitmap_of, expect_bitmap};
use roaring_bitmap::RoaringBitmap;

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

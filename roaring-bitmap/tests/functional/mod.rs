// Shared test helpers for functional tests
use roaring_bitmap::RoaringBitmap;

/// Helper function to create a bitmap from a slice of values
pub fn bitmap_of(values: &[u32]) -> RoaringBitmap {
    let mut bm = RoaringBitmap::new();
    for &value in values {
        assert!(bm.insert(value), "bitmap_of expects unique values");
    }
    bm
}

/// Helper function to verify a bitmap contains exactly the expected values
pub fn expect_bitmap(bitmap: &RoaringBitmap, expected: &[u32]) {
    let actual: Vec<u32> = bitmap.iter().collect();
    assert_eq!(actual, expected);
    assert_eq!(bitmap.len() as usize, expected.len());
    assert_eq!(bitmap.is_empty(), expected.is_empty());
    for &value in expected {
        assert!(bitmap.contains(value));
    }
}

// Test modules
mod basic_operations;
mod batch_removal;
mod bulk_operations;
mod container_set_operations;
mod containers;
mod iteration;
mod memory;
mod operators;
mod operators_owned;
mod optimization;
mod regression;
mod set_operations;
mod set_operations_inplace;

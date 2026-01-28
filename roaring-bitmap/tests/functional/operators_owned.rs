/// Tests for operator overloads that consume owned values
/// These test the BitOr/BitAnd/BitXor/Sub implementations for:
/// - RoaringBitmap op &RoaringBitmap
/// - &RoaringBitmap op RoaringBitmap
///
/// This is separate from the regular operator tests which use references.

use crate::functional::{bitmap_of, expect_bitmap};
use roaring_bitmap::RoaringBitmap;

// ============================================================================
// Union (BitOr) - Owned Operations
// ============================================================================

#[test]
fn test_union_owned_with_ref() {
    // RoaringBitmap | &RoaringBitmap
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[3, 4, 5]);

    let result = a | &b; // a is consumed, b is borrowed
    expect_bitmap(&result, &[1, 2, 3, 4, 5]);

    // b still exists
    assert_eq!(b.len(), 3);
}

#[test]
fn test_union_ref_with_owned() {
    // &RoaringBitmap | RoaringBitmap
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[3, 4, 5]);

    let result = &a | b; // a is borrowed, b is consumed
    expect_bitmap(&result, &[1, 2, 3, 4, 5]);

    // a still exists
    assert_eq!(a.len(), 3);
}

#[test]
fn test_union_owned_with_ref_empty() {
    let a = RoaringBitmap::new();
    let b = bitmap_of(&[1, 2, 3]);

    let result = a | &b;
    expect_bitmap(&result, &[1, 2, 3]);
}

#[test]
fn test_union_ref_with_owned_empty() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = RoaringBitmap::new();

    let result = &a | b;
    expect_bitmap(&result, &[1, 2, 3]);
}

// ============================================================================
// Intersection (BitAnd) - Owned Operations
// ============================================================================

#[test]
fn test_intersection_owned_with_ref() {
    // RoaringBitmap & &RoaringBitmap
    let a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    let result = a & &b;
    expect_bitmap(&result, &[3, 4]);

    // b still exists
    assert_eq!(b.len(), 4);
}

#[test]
fn test_intersection_ref_with_owned() {
    // &RoaringBitmap & RoaringBitmap
    let a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    let result = &a & b;
    expect_bitmap(&result, &[3, 4]);

    // a still exists
    assert_eq!(a.len(), 4);
}

#[test]
fn test_intersection_owned_with_ref_disjoint() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[4, 5, 6]);

    let result = a & &b;
    expect_bitmap(&result, &[]);
}

#[test]
fn test_intersection_ref_with_owned_disjoint() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[4, 5, 6]);

    let result = &a & b;
    expect_bitmap(&result, &[]);
}

#[test]
fn test_intersection_owned_with_ref_empty() {
    let a = RoaringBitmap::new();
    let b = bitmap_of(&[1, 2, 3]);

    let result = a & &b;
    expect_bitmap(&result, &[]);
}

// ============================================================================
// Symmetric Difference (BitXor) - Owned Operations
// ============================================================================

#[test]
fn test_symmetric_difference_owned_with_ref() {
    // RoaringBitmap ^ &RoaringBitmap
    let a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    let result = a ^ &b;
    expect_bitmap(&result, &[1, 2, 5, 6]);

    // b still exists
    assert_eq!(b.len(), 4);
}

#[test]
fn test_symmetric_difference_ref_with_owned() {
    // &RoaringBitmap ^ RoaringBitmap
    let a = bitmap_of(&[1, 2, 3, 4]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    let result = &a ^ b;
    expect_bitmap(&result, &[1, 2, 5, 6]);

    // a still exists
    assert_eq!(a.len(), 4);
}

#[test]
fn test_symmetric_difference_owned_with_ref_identical() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[1, 2, 3]);

    let result = a ^ &b;
    expect_bitmap(&result, &[]);
}

#[test]
fn test_symmetric_difference_ref_with_owned_identical() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[1, 2, 3]);

    let result = &a ^ b;
    expect_bitmap(&result, &[]);
}

// ============================================================================
// Difference (Sub) - Owned Operations
// ============================================================================

#[test]
fn test_difference_owned_with_ref() {
    // RoaringBitmap - &RoaringBitmap
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    let result = a - &b;
    expect_bitmap(&result, &[1, 2]);

    // b still exists
    assert_eq!(b.len(), 4);
}

#[test]
fn test_difference_ref_with_owned() {
    // &RoaringBitmap - RoaringBitmap
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6]);

    let result = &a - b;
    expect_bitmap(&result, &[1, 2]);

    // a still exists
    assert_eq!(a.len(), 5);
}

#[test]
fn test_difference_owned_with_ref_empty() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = RoaringBitmap::new();

    let result = a - &b;
    expect_bitmap(&result, &[1, 2, 3]);
}

#[test]
fn test_difference_ref_with_owned_empty() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = RoaringBitmap::new();

    let result = &a - b;
    expect_bitmap(&result, &[1, 2, 3]);
}

#[test]
fn test_difference_owned_with_ref_all_removed() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[1, 2, 3, 4, 5]);

    let result = a - &b;
    expect_bitmap(&result, &[]);
}

// ============================================================================
// Complex Expressions with Owned Values
// ============================================================================

#[test]
fn test_complex_expression_with_owned_values() {
    let a = bitmap_of(&[1, 2, 3, 4, 5]);
    let b = bitmap_of(&[3, 4, 5, 6, 7]);
    let c = bitmap_of(&[5, 6, 7, 8, 9]);

    // (a | &b) & &c - consumes a, borrows b and c
    let result = (a | &b) & &c;
    expect_bitmap(&result, &[5, 6, 7]);

    // b and c still exist
    assert_eq!(b.len(), 5);
    assert_eq!(c.len(), 5);
}

#[test]
fn test_chained_operations_with_owned() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[2, 3, 4]);
    let c = bitmap_of(&[3, 4, 5]);

    // a ^ &b = {1, 4} (symmetric difference)
    // {1, 4} - &c = {1} (remove 4 which is in c)
    let temp = a ^ &b; // a consumed, temp = {1, 4}
    let result = temp - &c; // remove c's values from temp
    expect_bitmap(&result, &[1]);
}

#[test]
fn test_move_semantics_with_operators() {
    let a = bitmap_of(&[1, 2, 3]);
    let b = bitmap_of(&[3, 4, 5]);

    // After this, a is moved/consumed
    let _result = a | &b;

    // Cannot use 'a' anymore (this would be a compile error)
    // println!("{}", a.len()); // Error: value borrowed after move

    // But b is still valid
    assert_eq!(b.len(), 3);
}

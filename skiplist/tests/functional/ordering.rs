use crate::functional::{verify_sorted_order, TestItem};
use skiplist::{SkipList, SkipListEntry};

#[test]
fn insertion_order_irrelevant_for_sorted_output() {
    let mut list1 = SkipList::new();
    let mut list2 = SkipList::new();

    // Insert in ascending order
    for key in [1, 2, 3, 4, 5] {
        list1.insert(Box::new(TestItem::with_key(key))).unwrap();
    }

    // Insert in descending order
    for key in [5, 4, 3, 2, 1] {
        list2.insert(Box::new(TestItem::with_key(key))).unwrap();
    }

    // Both should have same sorted order
    verify_sorted_order(&list1);
    verify_sorted_order(&list2);

    // Verify they contain same elements in same order
    let mut iter1 = list1.first();
    let mut iter2 = list2.first();

    while let (Some(item1), Some(item2)) = (iter1, iter2) {
        assert_eq!(item1.key(), item2.key());
        let key1 = *item1.key();
        let key2 = *item2.key();
        iter1 = list1.successor(&key1);
        iter2 = list2.successor(&key2);
    }

    assert!(iter1.is_none() && iter2.is_none());
}

#[test]
fn random_insertion_maintains_order() {
    let mut list = SkipList::new();
    let keys = vec![42, 7, 99, 3, 56, 21, 88, 12, 67, 34];

    for key in keys {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }

    verify_sorted_order(&list);
}

#[test]
fn interleaved_insert_remove_maintains_order() {
    let mut list = SkipList::new();

    // Insert 1, 3, 5, 7, 9
    for key in [1, 3, 5, 7, 9] {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }
    verify_sorted_order(&list);

    // Remove 5
    list.remove_by_key(&5);
    verify_sorted_order(&list);

    // Insert 2, 4, 6, 8
    for key in [2, 4, 6, 8] {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }
    verify_sorted_order(&list);

    // Remove 1, 9
    list.remove_by_key(&1);
    list.remove_by_key(&9);
    verify_sorted_order(&list);
}

#[test]
fn negative_and_positive_keys_ordered_correctly() {
    let mut list = SkipList::new();
    let keys = vec![-50, 25, -10, 100, 0, -100, 50, -25, 10];

    for key in keys {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }

    verify_sorted_order(&list);

    // Verify specific order
    let mut current = list.first();
    let expected = vec![-100, -50, -25, -10, 0, 10, 25, 50, 100];
    let mut index = 0;

    while let Some(item) = current {
        let key = *item.key();
        assert_eq!(key, expected[index]);
        current = list.successor(&key);
        index += 1;
    }
}

#[test]
fn large_dataset_maintains_order() {
    let mut list = SkipList::new();

    // Insert 1000 items in random order
    let keys: Vec<i32> = (0..1000).rev().collect();
    for key in keys {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }

    verify_sorted_order(&list);
    assert_eq!(list.len(), 1000);
}

#[test]
fn successive_ranges_maintain_order() {
    let mut list = SkipList::new();

    // Insert ranges: 100-199, 0-99, 200-299
    for key in 100..200 {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }
    for key in 0..100 {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }
    for key in 200..300 {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }

    verify_sorted_order(&list);
    assert_eq!(list.len(), 300);

    // Verify first and last
    if let Some(first) = list.first() {
        assert_eq!(*first.key(), 0);
    }

    let mut last_key = None;
    let mut current = list.first();
    while let Some(item) = current {
        let key = *item.key();
        last_key = Some(key);
        current = list.successor(&key);
    }
    assert_eq!(last_key, Some(299));
}

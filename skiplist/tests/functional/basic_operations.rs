use crate::functional::{expect_skiplist, skiplist_of, verify_sorted_order, TestItem};
use skiplist::{SkipList, SkipListEntry};

#[test]
fn new_skiplist_is_empty() {
    let list: SkipList<i32, TestItem> = SkipList::new();
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
    assert!(list.get(&0).is_none());
    assert!(list.first().is_none());
}

#[test]
fn insert_single_item() {
    let mut list = SkipList::new();
    let item = Box::new(TestItem::with_key(42));
    assert!(list.insert(item).is_ok());
    assert_eq!(list.len(), 1);
    assert!(!list.is_empty());
    assert!(list.get(&42).is_some());
}

#[test]
fn insert_duplicate_key_fails() {
    let mut list = SkipList::new();
    let item1 = Box::new(TestItem::new(42, "first".to_string()));
    let item2 = Box::new(TestItem::new(42, "second".to_string()));

    assert!(list.insert(item1).is_ok());
    assert_eq!(list.len(), 1);

    // Second insert should fail and return the item
    let result = list.insert(item2);
    assert!(result.is_err());
    if let Err(returned_item) = result {
        assert_eq!(returned_item.key, 42);
        assert_eq!(returned_item.value, "second");
    }

    // List should still have only one item
    assert_eq!(list.len(), 1);
}

#[test]
fn insert_multiple_items_maintains_order() {
    let mut list = SkipList::new();
    let keys = vec![5, 2, 8, 1, 9, 3];

    for key in &keys {
        let item = Box::new(TestItem::with_key(*key));
        assert!(list.insert(item).is_ok());
    }

    assert_eq!(list.len(), keys.len());
    verify_sorted_order(&list);
}

#[test]
fn get_retrieves_correct_item() {
    let list = skiplist_of(&[1, 5, 10, 15, 20]);

    if let Some(item) = list.get(&10) {
        assert_eq!(*item.key(), 10);
        assert_eq!(item.value, "value_10");
    } else {
        panic!("Expected to find key 10");
    }

    assert!(list.get(&99).is_none());
}

#[test]
fn get_mut_allows_value_modification() {
    let mut list = skiplist_of(&[1, 5, 10]);

    if let Some(item) = list.get_mut(&5) {
        item.value = "modified".to_string();
    }

    if let Some(item) = list.get(&5) {
        assert_eq!(item.value, "modified");
    }
}

#[test]
fn remove_existing_key_returns_item() {
    let mut list = skiplist_of(&[1, 5, 10, 15, 20]);

    let removed = list.remove(&10);
    assert!(removed.is_some());
    if let Some(item) = removed {
        assert_eq!(item.key, 10);
        assert_eq!(item.value, "value_10");
    }

    assert_eq!(list.len(), 4);
    assert!(list.get(&10).is_none());
    expect_skiplist(&list, &[1, 5, 15, 20]);
}

#[test]
fn remove_nonexistent_key_returns_none() {
    let mut list = skiplist_of(&[1, 5, 10]);

    let removed = list.remove(&99);
    assert!(removed.is_none());
    assert_eq!(list.len(), 3);
}

#[test]
fn remove_by_key_returns_bool() {
    let mut list = skiplist_of(&[1, 5, 10, 15, 20]);

    assert!(list.remove_by_key(&10));
    assert_eq!(list.len(), 4);
    assert!(list.get(&10).is_none());

    assert!(!list.remove_by_key(&99));
    assert_eq!(list.len(), 4);
}

#[test]
fn remove_first_element() {
    let mut list = skiplist_of(&[1, 5, 10, 15, 20]);

    assert!(list.remove_by_key(&1));
    expect_skiplist(&list, &[5, 10, 15, 20]);
}

#[test]
fn remove_last_element() {
    let mut list = skiplist_of(&[1, 5, 10, 15, 20]);

    assert!(list.remove_by_key(&20));
    expect_skiplist(&list, &[1, 5, 10, 15]);
}

#[test]
fn remove_middle_element() {
    let mut list = skiplist_of(&[1, 5, 10, 15, 20]);

    assert!(list.remove_by_key(&10));
    expect_skiplist(&list, &[1, 5, 15, 20]);
}

#[test]
fn remove_all_elements_one_by_one() {
    let mut list = skiplist_of(&[1, 2, 3, 4, 5]);

    for key in [3, 1, 5, 2, 4] {
        assert!(list.remove_by_key(&key));
    }

    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn len_tracks_insertions_and_removals() {
    let mut list = SkipList::new();
    assert_eq!(list.len(), 0);

    // Insert items
    for key in 1..=10 {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
        assert_eq!(list.len(), key as usize);
    }

    // Remove items
    for key in 1..=5 {
        list.remove_by_key(&key);
        assert_eq!(list.len(), (10 - key) as usize);
    }
}

#[test]
fn repeated_insert_remove_cycles() {
    let mut list = SkipList::new();

    for round in 0..5 {
        // Insert 100 items
        for i in 0..100 {
            let key = round * 1000 + i;
            list.insert(Box::new(TestItem::with_key(key))).unwrap();
        }

        // Remove half of them
        for i in 0..50 {
            let key = round * 1000 + i;
            assert!(list.remove_by_key(&key));
        }
    }

    assert_eq!(list.len(), 250); // 5 rounds * 50 remaining per round
    verify_sorted_order(&list);
}

#[test]
fn with_max_level_creates_skiplist() {
    let list: SkipList<i32, TestItem> = SkipList::with_max_level(8);
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

use crate::functional::{skiplist_of, verify_sorted_order, TestItem};
use skiplist::{SkipList, SkipListEntry, SkipListNode};

#[test]
fn empty_skiplist_operations() {
    let mut list: SkipList<i32, TestItem> = SkipList::new();

    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert!(list.first().is_none());
    assert!(list.get(&0).is_none());
    assert!(list.get_mut(&0).is_none());
    assert!(list.successor(&0).is_none());
    assert!(list.remove(&0).is_none());
    assert!(!list.remove_by_key(&0));
}

#[test]
fn single_element_operations() {
    let mut list = skiplist_of(&[42]);

    assert!(!list.is_empty());
    assert_eq!(list.len(), 1);

    assert!(list.first().is_some());
    assert_eq!(*list.first().unwrap().key(), 42);

    assert!(list.get(&42).is_some());
    assert!(list.get(&0).is_none());

    assert!(list.successor(&42).is_none());
    assert!(list.successor(&0).is_some()); // Should find 42

    // Remove the only element
    assert!(list.remove_by_key(&42));
    assert!(list.is_empty());
}

#[test]
fn extreme_values() {
    let mut list = SkipList::new();

    list.insert(Box::new(TestItem::with_key(i32::MIN))).unwrap();
    list.insert(Box::new(TestItem::with_key(i32::MAX))).unwrap();
    list.insert(Box::new(TestItem::with_key(0))).unwrap();

    verify_sorted_order(&list);

    assert_eq!(*list.first().unwrap().key(), i32::MIN);

    let mut current = list.first();
    let mut last_key = None;
    while let Some(item) = current {
        let key = *item.key();
        last_key = Some(key);
        current = list.successor(&key);
    }
    assert_eq!(last_key, Some(i32::MAX));
}

#[test]
fn consecutive_keys() {
    let mut list = SkipList::new();

    for key in 0..100 {
        list.insert(Box::new(TestItem::with_key(key))).unwrap();
    }

    verify_sorted_order(&list);
    assert_eq!(list.len(), 100);

    // Test successor on consecutive keys
    for key in 0..99 {
        if let Some(next) = list.successor(&key) {
            assert_eq!(*next.key(), key + 1);
        } else {
            panic!("Expected successor for key {}", key);
        }
    }
}

#[test]
fn sparse_keys() {
    let keys = vec![1000, 2000, 3000, 4000, 5000];
    let list = skiplist_of(&keys);

    verify_sorted_order(&list);

    // Test successor jumps over gaps
    assert_eq!(*list.successor(&1000).unwrap().key(), 2000);
    assert_eq!(*list.successor(&1500).unwrap().key(), 2000);
    assert_eq!(*list.successor(&2999).unwrap().key(), 3000);
}

#[test]
fn remove_from_single_element_list() {
    let mut list = skiplist_of(&[42]);

    let removed = list.remove(&42);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().key, 42);
    assert!(list.is_empty());
    assert!(list.first().is_none());
}

#[test]
fn insert_after_removing_all() {
    let mut list = skiplist_of(&[1, 2, 3]);

    // Remove all elements
    list.remove_by_key(&1);
    list.remove_by_key(&2);
    list.remove_by_key(&3);
    assert!(list.is_empty());

    // Insert new elements
    list.insert(Box::new(TestItem::with_key(10))).unwrap();
    list.insert(Box::new(TestItem::with_key(20))).unwrap();

    assert_eq!(list.len(), 2);
    verify_sorted_order(&list);
}

#[test]
fn get_mut_doesnt_break_ordering() {
    let mut list = skiplist_of(&[1, 5, 10, 15, 20]);

    // Modify value but not key
    if let Some(item) = list.get_mut(&10) {
        item.value = "modified".to_string();
    }

    verify_sorted_order(&list);
    assert_eq!(list.get(&10).unwrap().value, "modified");
}

#[test]
fn repeated_lookups_consistent() {
    let list = skiplist_of(&[1, 5, 10, 15, 20]);

    for _ in 0..100 {
        assert!(list.get(&10).is_some());
        assert!(list.get(&99).is_none());
        assert_eq!(*list.first().unwrap().key(), 1);
    }
}

#[test]
fn alternating_insert_remove() {
    let mut list = SkipList::new();

    for i in 0..50 {
        // Insert
        list.insert(Box::new(TestItem::with_key(i))).unwrap();
        assert_eq!(list.len(), 1);

        // Remove
        assert!(list.remove_by_key(&i));
        assert!(list.is_empty());
    }
}

#[test]
fn two_element_list_operations() {
    let mut list = skiplist_of(&[10, 20]);

    assert_eq!(list.len(), 2);
    assert_eq!(*list.first().unwrap().key(), 10);
    assert_eq!(*list.successor(&10).unwrap().key(), 20);
    assert!(list.successor(&20).is_none());

    // Remove first
    list.remove_by_key(&10);
    assert_eq!(list.len(), 1);
    assert_eq!(*list.first().unwrap().key(), 20);

    // Remove second
    list.remove_by_key(&20);
    assert!(list.is_empty());
}

#[test]
fn default_creates_empty_skiplist() {
    let list: SkipList<i32, TestItem> = SkipList::default();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn skiplist_node_default() {
    let node = SkipListNode::default();
    let node_new = SkipListNode::new();

    // Both should be functionally equivalent (empty forward vectors)
    // This is mainly to verify Default trait implementation
    let _ = (node, node_new);
}

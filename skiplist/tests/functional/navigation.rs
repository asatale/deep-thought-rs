use crate::functional::{skiplist_of, TestItem};
use skiplist::{SkipList, SkipListEntry};

#[test]
fn first_returns_smallest_key() {
    let list = skiplist_of(&[10, 5, 20, 1, 15]);

    if let Some(item) = list.first() {
        assert_eq!(*item.key(), 1);
    } else {
        panic!("Expected first element");
    }
}

#[test]
fn first_on_empty_list_returns_none() {
    let list: SkipList<i32, TestItem> = SkipList::new();
    assert!(list.first().is_none());
}

#[test]
fn first_after_removing_first() {
    let mut list = skiplist_of(&[1, 5, 10, 15, 20]);
    list.remove_by_key(&1);

    if let Some(item) = list.first() {
        assert_eq!(*item.key(), 5);
    } else {
        panic!("Expected first element after removal");
    }
}

#[test]
fn successor_returns_next_element() {
    let list = skiplist_of(&[1, 5, 10, 15, 20]);

    if let Some(next) = list.successor(&5) {
        assert_eq!(*next.key(), 10);
    } else {
        panic!("Expected successor of 5");
    }

    if let Some(next) = list.successor(&10) {
        assert_eq!(*next.key(), 15);
    } else {
        panic!("Expected successor of 10");
    }
}

#[test]
fn successor_of_last_element_returns_none() {
    let list = skiplist_of(&[1, 5, 10, 15, 20]);

    assert!(list.successor(&20).is_none());
}

#[test]
fn successor_of_nonexistent_key_finds_next_larger() {
    let list = skiplist_of(&[1, 5, 10, 15, 20]);

    // Key 7 doesn't exist, but 10 is the next larger key
    if let Some(next) = list.successor(&7) {
        assert_eq!(*next.key(), 10);
    } else {
        panic!("Expected successor of non-existent key 7 to be 10");
    }

    // Key 12 doesn't exist, but 15 is the next larger key
    if let Some(next) = list.successor(&12) {
        assert_eq!(*next.key(), 15);
    } else {
        panic!("Expected successor of non-existent key 12 to be 15");
    }
}

#[test]
fn successor_larger_than_all_keys_returns_none() {
    let list = skiplist_of(&[1, 5, 10, 15, 20]);

    assert!(list.successor(&100).is_none());
}

#[test]
fn iterate_entire_list_using_successor() {
    let list = skiplist_of(&[2, 8, 3, 10, 5, 1, 9]);
    let expected = vec![1, 2, 3, 5, 8, 9, 10];

    let mut collected = Vec::new();
    let mut current = list.first();

    while let Some(item) = current {
        let key = *item.key();
        collected.push(key);
        current = list.successor(&key);
    }

    assert_eq!(collected, expected);
}

#[test]
fn successor_after_removal() {
    let mut list = skiplist_of(&[1, 5, 10, 15, 20]);

    // Remove middle element
    list.remove_by_key(&10);

    // Successor of 5 should now be 15
    if let Some(next) = list.successor(&5) {
        assert_eq!(*next.key(), 15);
    } else {
        panic!("Expected successor of 5 to be 15 after removing 10");
    }
}

#[test]
fn successor_chain_consistency() {
    let list = skiplist_of(&[1, 3, 5, 7, 9, 11, 13, 15]);

    let mut current = list.first();
    let mut prev_key: Option<i32> = None;
    let mut count = 0;

    while let Some(item) = current {
        let item_key = *item.key();

        // Verify ordering
        if let Some(prev) = prev_key {
            assert!(prev < item_key, "successor chain not in order");
        }

        prev_key = Some(item_key);
        current = list.successor(&item_key);
        count += 1;
    }

    assert_eq!(count, list.len());
}

#[test]
fn first_and_successor_on_single_element() {
    let list = skiplist_of(&[42]);

    if let Some(first) = list.first() {
        assert_eq!(*first.key(), 42);
        assert!(list.successor(&42).is_none());
    } else {
        panic!("Expected first element");
    }
}

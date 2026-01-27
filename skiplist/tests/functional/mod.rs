// Shared test helpers for functional tests
use skiplist::{SkipList, SkipListEntry, SkipListNode};

/// Test item structure for skiplist tests
#[derive(Debug, Clone)]
pub struct TestItem {
    pub key: i32,
    pub value: String,
    pub skiplist_meta: SkipListNode,
}

impl TestItem {
    /// Create a new test item with the given key and value
    pub fn new(key: i32, value: String) -> Self {
        Self {
            key,
            value,
            skiplist_meta: SkipListNode::new(),
        }
    }

    /// Create a new test item with key and default value
    pub fn with_key(key: i32) -> Self {
        Self::new(key, format!("value_{}", key))
    }
}

impl SkipListEntry for TestItem {
    type Key = i32;

    fn key(&self) -> &Self::Key {
        &self.key
    }

    fn skiplist_node(&self) -> &SkipListNode {
        &self.skiplist_meta
    }

    fn skiplist_node_mut(&mut self) -> &mut SkipListNode {
        &mut self.skiplist_meta
    }
}

/// Helper function to create a skiplist from a slice of keys
pub fn skiplist_of(keys: &[i32]) -> SkipList<i32, TestItem> {
    let mut list = SkipList::new();
    for &key in keys {
        let item = Box::new(TestItem::with_key(key));
        assert!(
            list.insert(item).is_ok(),
            "skiplist_of expects unique keys, found duplicate: {}",
            key
        );
    }
    list
}

/// Helper function to verify a skiplist contains exactly the expected keys in sorted order
pub fn expect_skiplist(list: &SkipList<i32, TestItem>, expected_keys: &[i32]) {
    // Verify length
    assert_eq!(
        list.len(),
        expected_keys.len(),
        "skiplist length mismatch: expected {}, got {}",
        expected_keys.len(),
        list.len()
    );

    // Verify is_empty
    assert_eq!(
        list.is_empty(),
        expected_keys.is_empty(),
        "skiplist is_empty mismatch"
    );

    // Verify each expected key exists
    for &key in expected_keys {
        assert!(
            list.get(&key).is_some(),
            "expected key {} not found in skiplist",
            key
        );
    }

    // Verify keys are in sorted order and match expected
    let mut prev_key: Option<i32> = None;
    let mut current = list.first();
    let mut index = 0;

    while let Some(item) = current {
        let item_key = *item.key();

        // Check we haven't exceeded expected length
        assert!(
            index < expected_keys.len(),
            "skiplist has more elements than expected"
        );

        // Check key matches expected
        assert_eq!(
            item_key, expected_keys[index],
            "key at index {} mismatch: expected {}, got {}",
            index, expected_keys[index], item_key
        );

        // Verify sorted order
        if let Some(prev) = prev_key {
            assert!(
                prev < item_key,
                "skiplist not in sorted order: {} >= {}",
                prev,
                item_key
            );
        }

        prev_key = Some(item_key);
        current = list.successor(&item_key);
        index += 1;
    }

    // Verify we've seen all expected keys
    assert_eq!(
        index,
        expected_keys.len(),
        "skiplist has fewer elements than expected"
    );
}

/// Helper to verify skiplist is in sorted order
pub fn verify_sorted_order(list: &SkipList<i32, TestItem>) {
    let mut prev_key: Option<i32> = None;
    let mut current = list.first();
    let mut count = 0;

    while let Some(item) = current {
        let item_key = *item.key();
        if let Some(prev) = prev_key {
            assert!(
                prev < item_key,
                "skiplist not in sorted order: {} >= {}",
                prev,
                item_key
            );
        }
        prev_key = Some(item_key);
        current = list.successor(&item_key);
        count += 1;
    }

    assert_eq!(count, list.len(), "iteration count doesn't match len()");
}

// Test modules
mod basic_operations;
mod edge_cases;
mod navigation;
mod ordering;

// Shared benchmark helpers
use skiplist::{SkipList, SkipListEntry, SkipListNode};
use std::time::Instant;

/// Test item for benchmarks
#[derive(Debug)]
pub struct BenchItem {
    pub key: i32,
    pub data: [u8; 64], // Some payload
    pub skiplist_meta: SkipListNode,
}

impl BenchItem {
    pub fn new(key: i32) -> Self {
        Self {
            key,
            data: [0u8; 64],
            skiplist_meta: SkipListNode::new(),
        }
    }
}

impl SkipListEntry for BenchItem {
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

/// Helper to format duration in appropriate units
pub fn format_duration(nanos: u128) -> String {
    if nanos < 1_000 {
        format!("{} ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.2} μs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2} ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2} s", nanos as f64 / 1_000_000_000.0)
    }
}

/// Helper to format operations per second
pub fn format_ops_per_sec(ops: usize, nanos: u128) -> String {
    let ops_per_sec = (ops as f64 / nanos as f64) * 1_000_000_000.0;
    if ops_per_sec >= 1_000_000.0 {
        format!("{:.2} M ops/sec", ops_per_sec / 1_000_000.0)
    } else if ops_per_sec >= 1_000.0 {
        format!("{:.2} K ops/sec", ops_per_sec / 1_000.0)
    } else {
        format!("{:.2} ops/sec", ops_per_sec)
    }
}

/// Helper to measure execution time
pub fn time_operation<F>(op: F) -> u128
where
    F: FnOnce(),
{
    let start = Instant::now();
    op();
    start.elapsed().as_nanos()
}

/// Helper to create a pre-populated skiplist for benchmarks
pub fn create_populated_list(size: usize) -> SkipList<i32, BenchItem> {
    let mut list = SkipList::new();
    for i in 0..size as i32 {
        list.insert(Box::new(BenchItem::new(i))).unwrap();
    }
    list
}

// Benchmark modules
mod insertion;
mod lookup;
mod mixed_workload;
mod navigation;
mod removal;

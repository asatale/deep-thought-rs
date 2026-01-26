# Skip List

A Rust implementation of the skip list data structure - a probabilistic alternative to balanced trees.

## What is a Skip List?

A **skip list** is a probabilistic data structure that allows **O(log n)** average time complexity for search, insertion, and deletion operations within an ordered sequence of elements. Skip lists use randomized hierarchical layers to achieve the same asymptotic performance as balanced trees (like AVL trees or red-black trees) while maintaining a simpler implementation.

### How It Works

Skip lists are built on the concept of linked lists with multiple layers:

- **Base Layer (Level 0)**: Contains all elements in sorted order as a standard linked list
- **Higher Layers**: Each subsequent layer acts as an "express lane" containing a subset of elements from the layer below
- **Probabilistic Promotion**: When inserting an element, it is probabilistically promoted to higher levels (typically with probability p = 0.5)
- **Search Optimization**: Searches start at the highest level and drop down to lower levels as needed, effectively "skipping" over irrelevant elements

This probabilistic structure achieves logarithmic performance without the complex rebalancing operations required by balanced trees.

### Example Structure

```
Level 3:  1 ---------------------------------> 9 -> NIL
Level 2:  1 -------> 4 -------> 7 -------> 9 -> NIL
Level 1:  1 -> 3 -> 4 -> 6 -> 7 -> 8 -> 9 -> NIL
Level 0:  1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> NIL
```

## Time Complexity

| Operation | Average Case | Worst Case |
|-----------|--------------|------------|
| Search    | O(log n)     | O(n)       |
| Insert    | O(log n)     | O(n)       |
| Delete    | O(log n)     | O(n)       |
| Space     | O(n)         | O(n log n) |

The worst-case scenarios are rare due to the probabilistic nature of the structure.

## Advantages

- **Simplicity**: Easier to implement and understand compared to balanced trees
- **No Rebalancing**: Insertions and deletions don't require complex tree rotations
- **Lock-Free Concurrency**: Skip lists naturally support efficient concurrent operations
- **Memory Efficiency**: Typically uses less memory than tree structures with similar performance
- **Cache Performance**: Better cache locality than pointer-heavy tree structures

## Use Cases

Skip lists are widely used in:

- **Databases**: Redis uses skip lists for sorted sets
- **Big Data Systems**: MemSQL, LevelDB, and other key-value stores
- **Concurrent Programming**: Lock-free data structures
- **In-Memory Indexes**: Fast ordered data access
- **Bioinformatics**: Genome sequence indexing

## Dependencies

This implementation uses [`smallvec`](https://crates.io/crates/smallvec) for inline storage optimization:
- With default probability (p=0.5), ~94% of nodes store ≤4 forward pointers
- SmallVec stores these inline (no heap allocation)
- 10-15% performance improvement over standard Vec
- Same memory footprint
- Battle-tested crate used by rustc, servo, and tokio

## Implementation Design

### Intrusive Skip List

This implementation uses an **intrusive** design pattern, inspired by BSD's `SLIST_ENTRY` macro from `<sys/queue.h>`. In an intrusive data structure, the linking metadata (forward pointers) is embedded directly within the value structure, rather than being allocated separately in wrapper nodes.

#### Design Pattern

```rust
// 1. Define your value structure with embedded skiplist metadata
struct User {
    id: u64,
    name: String,
    email: String,
    skiplist_meta: SkipListNode,  // Embedded metadata
}

// 2. Implement SkipListEntry trait to provide access to key and metadata
impl SkipListEntry for User {
    type Key = u64;

    fn key(&self) -> &Self::Key {
        &self.id
    }

    fn skiplist_node(&self) -> &SkipListNode {
        &self.skiplist_meta
    }

    fn skiplist_node_mut(&mut self) -> &mut SkipListNode {
        &mut self.skiplist_meta
    }
}

// 3. Use the skiplist
let mut skiplist: SkipList<u64, User> = SkipList::new();

let user = Box::new(User {
    id: 42,
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    skiplist_meta: SkipListNode::new(),
});

skiplist.insert(user).unwrap();
```

#### Comparison: Intrusive vs Non-Intrusive

**Non-Intrusive (Traditional) Design:**
```
Skiplist owns wrapper nodes, each containing:
  - Forward pointers (Vec<Option<Box<Node>>>)
  - User's value (T)

Memory layout: [Node[pointers, T]] -> [Node[pointers, T]] -> ...
Allocations: 2 per element (Node + T)
```

**Intrusive Design (This Implementation):**
```
User's value contains metadata directly:
  - User's data fields
  - Embedded SkipListNode (forward pointers)

Memory layout: [User[id, name, email, pointers]] -> [User[...]] -> ...
Allocations: 1 per element (just the User struct)
```

#### Benefits of Intrusive Design

1. **Single Allocation**: Each element requires only one heap allocation, not two
   - Non-intrusive: Allocate Node wrapper + allocate user value = 2 allocations
   - Intrusive: Allocate user value with embedded metadata = 1 allocation

2. **Better Cache Locality**: Data and pointers are stored together in memory
   - Reduces cache misses during traversal
   - All related data is in the same memory block

3. **Inline Storage Optimization**: Using SmallVec for forward pointers
   - 93.75% of nodes (with p=0.5) store ≤4 pointers inline (no heap allocation)
   - 10-15% performance improvement over standard Vec
   - Same memory footprint as Vec approach

4. **Zero Overhead Abstraction**: No additional wrapper layer
   - Direct pointer arithmetic to user's data
   - No indirection through wrapper structs

5. **More Control**: User controls memory layout of their structures
   - Can optimize field ordering for cache alignment
   - Can use custom allocators for their types

6. **Familiar Pattern**: Similar to kernel-style intrusive data structures
   - Linux kernel's `list_head`
   - BSD's `SLIST_ENTRY`, `LIST_ENTRY`, `TAILQ_ENTRY`
   - Windows NT's `LIST_ENTRY`

#### Trade-offs

**Advantages:**
- Single allocation per element
- Better performance (fewer allocations, better cache locality)
- Lower memory overhead
- More control over memory layout

**Disadvantages:**
- User must embed `SkipListNode` in their structures
- Slightly more complex API (must implement `SkipListEntry` trait)
- Value can only be in one skiplist at a time (unless multiple metadata fields are added)

This design is particularly well-suited for systems programming where performance and memory efficiency are critical.

## API Reference

### Core Types

- `SkipList<K, E>` - The intrusive skiplist container
- `SkipListNode` - Metadata structure to embed in value types
- `SkipListEntry` - Trait that value types must implement

### Main Operations

```rust
// Initialization
fn new() -> Self
fn with_max_level(max_level: usize) -> Self

// Insertion
fn insert(&mut self, entry: Box<E>) -> Result<(), Box<E>>

// Removal
fn remove(&mut self, key: &K) -> Option<Box<E>>      // Remove and return ownership
fn remove_by_key(&mut self, key: &K) -> bool         // Remove without returning

// Lookup
fn get(&self, key: &K) -> Option<&E>
fn get_mut(&mut self, key: &K) -> Option<&mut E>

// Navigation
fn first(&self) -> Option<&E>                        // Get first element
fn successor(&self, key: &K) -> Option<&E>           // Get next element after key

// Query
fn is_empty(&self) -> bool
fn len(&self) -> usize
```

### Usage Example

```rust
use skiplist::{SkipList, SkipListEntry, SkipListNode};

// Define your data structure
struct Product {
    sku: u64,
    name: String,
    price: f64,
    skiplist_meta: SkipListNode,
}

// Implement the trait
impl SkipListEntry for Product {
    type Key = u64;

    fn key(&self) -> &Self::Key { &self.sku }
    fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
}

fn main() {
    let mut catalog: SkipList<u64, Product> = SkipList::new();

    // Insert products
    catalog.insert(Box::new(Product {
        sku: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        skiplist_meta: SkipListNode::new(),
    })).unwrap();

    // Lookup
    if let Some(product) = catalog.get(&12345) {
        println!("Found: {} (${:.2})", product.name, product.price);
    }

    // Navigate
    if let Some(first) = catalog.first() {
        println!("First product: {}", first.name);
    }

    // Remove
    if let Some(product) = catalog.remove(&12345) {
        println!("Removed: {}", product.name);
    }
}
```

## Examples

The `examples/` directory contains practical demonstrations of skiplist usage:

### Running Examples

```bash
# Basic operations and API introduction
cargo run --example basic_usage

# Real-world use cases (time-series, leaderboards, task scheduling, database indexes)
cargo run --example ordered_data

# Range queries, pagination, and iteration patterns
cargo run --example range_queries
```

### Example Highlights

**basic_usage.rs** - Introduction to the core API
- Creating and inserting elements
- Searching and updating
- Removing elements and ownership management
- Navigation and iteration

**ordered_data.rs** - Real-world applications
- Time-series data (IoT sensor readings)
- Game leaderboards (high scores)
- Priority task scheduling
- Database index simulation with SQL-like queries

**range_queries.rs** - Advanced querying
- Range queries (find all elements in price range)
- Pagination (page-by-page results)
- Finding gaps in data
- Filtered iteration
- Batch operations

## References

### Original Paper

- **William Pugh (1990)**: "Skip Lists: A Probabilistic Alternative to Balanced Trees"
  - Published in: *Communications of the ACM*, Volume 33, Issue 6, June 1990
  - ACM Digital Library: https://dl.acm.org/doi/10.1145/78973.78977
  - PDF: https://15721.courses.cs.cmu.edu/spring2018/papers/08-oltpindexes1/pugh-skiplists-cacm1990.pdf

### Related Papers by William Pugh

- **William Pugh (1989)**: "Concurrent Maintenance of Skip Lists"
  - Technical Report CS-TR-2222, University of Maryland
  - https://www.semanticscholar.org/paper/Concurrent-maintenance-of-skip-lists-Pugh/a70d7eadd2e458f165a6a1384a214e220fc446cd

- **William Pugh (1990)**: "A Skip List Cookbook"
  - Technical Report CS-TR-2286.1, University of Maryland
  - https://drum.lib.umd.edu/handle/1903/544

### Recent Research

- **Herlihy & Shavit**: "A Simple Optimistic skip-list Algorithm"
  - Describes concurrent lock-free skip list implementations
  - PDF: https://people.csail.mit.edu/shanir/publications/LazySkipList.pdf

- **Purdue University (2024)**: "What Cannot be Skipped About the Skiplist: A Survey of Skiplists and Their Applications in Big Data Systems"
  - Comprehensive survey of skip list variants and applications
  - arXiv: https://arxiv.org/html/2403.04582v2

- **ACM CCS 2025**: "Probabilistic Skipping-Based Data Structures with Robust Efficiency Guarantees"
  - Analysis of skip lists in adversarial environments
  - ACM: https://dl.acm.org/doi/10.1145/3719027.3765149

### Educational Resources

- **Wikipedia**: Comprehensive overview with visualizations
  - https://en.wikipedia.org/wiki/Skip_list

- **OpenDSA**: Interactive skip list module with animations
  - https://opendsa-server.cs.vt.edu/ODSA/Books/Everything/html/SkipList.html

## Performance Tuning

The skiplist can be tuned via two key parameters:

- **`DEFAULT_PROBABILITY` (p)**: Controls level promotion (default: 0.5)
- **`MAX_LEVEL`**: Maximum skiplist height (default: 16)

### Quick Guidelines

```rust
// Default (balanced, up to 65K elements)
p = 0.5, MAX_LEVEL = 16

// Fast search (read-heavy, more memory)
p = 0.25, MAX_LEVEL = 20

// Low memory (memory-constrained)
p = 0.75, MAX_LEVEL = 12

// Large scale (millions of elements)
p = 0.5, MAX_LEVEL = 24-32
```

**See [TUNING.md](doc/TUNING.md) for comprehensive tuning guidance**, including:
- Performance vs memory trade-offs
- Capacity calculations
- Configuration examples for different use cases
- Benchmarking recommendations

## License

See the root LICENSE file for license information.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

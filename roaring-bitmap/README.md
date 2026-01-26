# Roaring Bitmap Implementation

Comprehensive technical documentation for the Roaring Bitmap data structure implementation.

## Overview

Roaring bitmaps are a compressed bitmap data structure that efficiently stores sets of integers. They provide a space-efficient alternative to traditional bitmaps while maintaining fast operations.

## Basic API

### Construction
- `new()` - Creates an empty roaring bitmap

### Insertion
- `insert(value: u32) -> bool` - Adds a single element to the bitmap, returns `true` if the element was newly inserted

### Query Operations
- `contains(value: u32) -> bool` - Checks if an element exists in the bitmap
- `len() -> u64` - Returns the number of elements in the bitmap (cardinality)
- `is_empty() -> bool` - Returns `true` if the bitmap contains no elements

### Deletion
- `remove(value: u32) -> bool` - Removes a single element from the bitmap, returns `true` if the element was present
- `remove_range(range)` - Efficiently removes consecutive values (mirrors `extend_consecutive`)
- `remove_sparse(values)` - Efficiently removes sparse values (mirrors `extend_sparse`)
- `clear()` - Removes all elements from the bitmap

### Set Operations

**Allocating operations** (create new bitmap):
- `union(&self, other: &RoaringBitmap) -> RoaringBitmap` - Returns the union (OR) of two bitmaps
- `intersection(&self, other: &RoaringBitmap) -> RoaringBitmap` - Returns the intersection (AND) of two bitmaps
- `difference(&self, other: &RoaringBitmap) -> RoaringBitmap` - Returns the difference (AND NOT) of two bitmaps
- `symmetric_difference(&self, other: &RoaringBitmap) -> RoaringBitmap` - Returns the symmetric difference (XOR) of two bitmaps

**In-place operations** (modify existing bitmap):
- `union_with(&mut self, other: &RoaringBitmap)` - In-place union
- `intersect_with(&mut self, other: &RoaringBitmap)` - In-place intersection
- `difference_with(&mut self, other: &RoaringBitmap)` - In-place difference
- `symmetric_difference_with(&mut self, other: &RoaringBitmap)` - In-place symmetric difference

**Operator syntax** (via trait implementations):
- `&a | &b` - Union (allocating), `a |= &b` - Union (in-place)
- `&a & &b` - Intersection (allocating), `a &= &b` - Intersection (in-place)
- `&a ^ &b` - Symmetric difference (allocating), `a ^= &b` - Symmetric difference (in-place)
- `&a - &b` - Difference (allocating), `a -= &b` - Difference (in-place)

### Iteration
- `iter(&self) -> Iter` - Returns an iterator over elements in sorted order

### Optimization
- `optimize(&mut self)` - Optimizes container storage for minimal memory usage

### Bulk Operations (Intermediate API)
**Insertion:**
- `extend_consecutive(range)` - Efficiently insert consecutive values (creates Run containers)
- `extend_sparse(values)` - Efficiently insert sparse values (creates Array containers)
- `extend_dense(values)` - Efficiently insert dense values (smart container choice)

**Removal:**
- `remove_range(range)` - Efficiently remove consecutive values
- `remove_sparse(values)` - Efficiently remove sparse values

### Memory Usage
- `memory_usage(&self) -> usize` - Returns total memory usage in bytes
- `memory_usage_detailed(&self) -> MemoryUsage` - Returns detailed breakdown with struct-based API

## Internal Container Types

This implementation uses three container types internally to optimize memory usage:

### Array Container
- **Storage**: Sorted array of u16 values
- **Size**: `2 × cardinality` bytes
- **Used for**: Sparse data (typically < 4,096 values)
- **Operations**: O(log n) lookup via binary search

### Bitmap Container
- **Storage**: Fixed 8,192-byte bitmap (65,536 bits)
- **Size**: 8,192 bytes (constant)
- **Used for**: Dense data (typically ≥ 4,096 values)
- **Operations**: O(1) lookup, insert, remove

### Run Container
- **Storage**: Run-length encoded consecutive sequences
- **Size**: `4 × number_of_runs` bytes
- **Used for**: Data with long consecutive sequences
- **Operations**: O(n) where n = number of runs
- **Example**: `[1,2,3,4,5,100,101,102]` = 2 runs = 8 bytes

## Intermediate API: Semantic Bulk Operations

For users who understand their data patterns, this implementation provides semantic methods that optimize **insertion performance** by creating appropriate container types directly.

### Methods

#### `extend_consecutive(range)`

Efficiently inserts consecutive values by creating Run containers directly.

**Performance:**
- **Time**: O(n) where n = number of values
- **Memory**: 4 bytes per run (minimal for consecutive data)
- **Best for**: Time series, ID ranges, sequential data

**Example:**
```rust
let mut bm = RoaringBitmap::new();

// Efficient: creates Run container directly
bm.extend_consecutive(0..1_000_000);
println!("Memory: {} bytes", bm.memory_usage()); // Very compact!

// Multiple consecutive ranges
bm.extend_consecutive(2_000_000..3_000_000);
bm.extend_consecutive(5_000_000..6_000_000);
```

#### `extend_sparse(values)`

Efficiently inserts sparse values by creating Array containers directly.

**Performance:**
- **Time**: O(n log n) where n = number of values (sorting)
- **Memory**: 2 bytes per value
- **Best for**: Scattered IDs, random samples, user IDs

**Example:**
```rust
let mut bm = RoaringBitmap::new();

// Sparse user IDs
bm.extend_sparse([1000, 5000, 10000, 50000, 100000]);

// From a vector
let sparse_values: Vec<u32> = vec![42, 1337, 9999];
bm.extend_sparse(sparse_values);
```

#### `extend_dense(values)`

Efficiently inserts dense values with smart container selection based on cardinality.

**Performance:**
- **Time**: O(n) where n = number of values
- **Memory**: Automatically chooses optimal container type
- **Best for**: Unknown patterns, mixed density

**Example:**
```rust
let mut bm = RoaringBitmap::new();

// Insert even numbers in a range (50% density)
bm.extend_dense((0..10_000).filter(|x| x % 2 == 0));

// Dense range with some gaps
let values: Vec<u32> = (0..8000).filter(|x| x % 3 != 0).collect();
bm.extend_dense(values);
```

## In-Place Set Operations and Operator Overloading

This implementation provides both allocating and in-place set operations, along with ergonomic operator syntax.

### Why In-Place Operations Matter

**Without in-place operations** (creates N-1 intermediate bitmaps):
```rust
let mut result = bitmaps[0].clone();
for bitmap in &bitmaps[1..] {
    result = result.union(bitmap);  // Allocates new bitmap each iteration!
}
```

**With in-place operations** (zero intermediate allocations):
```rust
let mut result = bitmaps[0].clone();
for bitmap in &bitmaps[1..] {
    result.union_with(bitmap);  // Modifies in-place, no allocation
}
```

**With operator syntax** (even cleaner):
```rust
let mut result = bitmaps[0].clone();
for bitmap in &bitmaps[1..] {
    result |= bitmap;  // Ergonomic and efficient
}
```

### API Overview

| Operation | Allocating | In-Place Method | Operator (Alloc) | Operator (In-Place) |
|-----------|------------|----------------|------------------|---------------------|
| Union | `a.union(&b)` | `a.union_with(&b)` | `&a \| &b` | `a \|= &b` |
| Intersection | `a.intersection(&b)` | `a.intersect_with(&b)` | `&a & &b` | `a &= &b` |
| Difference | `a.difference(&b)` | `a.difference_with(&b)` | `&a - &b` | `a -= &b` |
| Symmetric Diff | `a.symmetric_difference(&b)` | `a.symmetric_difference_with(&b)` | `&a ^ &b` | `a ^= &b` |

### Performance Characteristics

**In-place operations:**
- **Time**: Same as allocating versions (O(n + m) for most operations)
- **Space**: No new bitmap allocation, modifies existing bitmap
- **Use when**: You want to modify an existing bitmap and don't need the original

**Allocating operations:**
- **Time**: Same as in-place versions
- **Space**: Allocates new bitmap
- **Use when**: You need to keep the original bitmaps unchanged

## Batch Removal Operations

### Methods

#### `remove_range(range)`

Efficiently removes consecutive values.

**Example:**
```rust
let mut bm = RoaringBitmap::new();
bm.extend_consecutive(0..100_000);

// Remove a contiguous block
bm.remove_range(1000..2000);

// Clear old data (time-series pruning)
bm.remove_range(0..10_000);
```

#### `remove_sparse(values)`

Efficiently removes sparse values.

**Example:**
```rust
bm.remove_sparse([1000, 10000]);
```

#### `clear()`

Removes all elements from the bitmap (O(1)).

### API Symmetry

| Insertion | Removal | Use Case |
|-----------|---------|----------|
| `extend_consecutive(range)` | `remove_range(range)` | Consecutive values |
| `extend_sparse(values)` | `remove_sparse(values)` | Sparse values |
| `insert(value)` | `remove(value)` | Single values |
| N/A | `clear()` | Remove everything |

## Memory Usage Tracking

### API

**`memory_usage() -> usize`**

Returns total memory usage in bytes.

**`memory_usage_detailed() -> MemoryUsage`**

Returns a `MemoryUsage` struct with detailed breakdown:
- `total`: Total bytes (stack + heap)
- `stack`: Stack-allocated bytes
- `heap`: Total heap-allocated bytes
- `containers`: Vec of `ContainerStats` for each container

**Example:**
```rust
let usage = bm.memory_usage_detailed();
println!("Total: {} bytes", usage.total);
for container in &usage.containers {
    println!("Container {}: {} - {} bytes",
        container.key, container.container_type, container.memory_bytes);
}
```

## Optimization Strategy: Hybrid + Lazy

### Automatic Conversions (Conservative)

**During Operations:**
- Array → Bitmap at 4,096 values
- Bitmap → Array below 4,096 values
- These happen automatically during insert/remove

**Conservative Philosophy:**
- Only convert when clearly beneficial
- Avoid expensive conversions during hot paths
- Let `optimize()` handle aggressive optimization

### Explicit Optimization

**`optimize()` method:**
- Analyzes actual data patterns
- Converts to most efficient container type
- Example: Fragmented Run → Array/Bitmap

**When to Call optimize():**
1. After bulk operations
2. Before serialization
3. After significant modifications
4. When memory is critical

## Testing

### Running Tests

**All functional tests:**
```bash
cargo test --test roaring_bitmap
```

**Specific category:**
```bash
cargo test --test roaring_bitmap operators
cargo test --test roaring_bitmap batch_removal
cargo test --test roaring_bitmap regression
```

**Performance benchmarks:**
```bash
cargo test --test performance -- --ignored --nocapture
cargo test --test performance insertion -- --ignored --nocapture
```

### Test Organization

- `basic_operations` - Core operations (11 tests)
- `set_operations` - Union, intersection, etc. (19 tests)
- `operators` - Operator overloading (14 tests)
- `bulk_operations` - Bulk insertion (21 tests)
- `batch_removal` - Batch removal (19 tests)
- `containers` - Container types (20 tests)
- `memory` - Memory tracking (7 tests)
- `regression` - Bug fixes (9 tests)

**Total: 137 functional tests, 13 performance benchmarks**

## References

### Academic Papers
- "Roaring Bitmaps: Implementation of an Optimized Software Library" (2018)
- "Better bitmap performance with Roaring bitmaps" (2016)

### Additional Resources
- Official Roaring Bitmap website
- CRoaring - C/C++ implementation
- Rust documentation

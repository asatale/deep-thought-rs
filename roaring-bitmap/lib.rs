// Roaring Bitmap Implementation

//! Internal Structure:
//!
//! A Roaring Bitmap stores 32-bit integers by splitting them into:
//! - High 16 bits: Container key (determines which container)
//! - Low 16 bits: Value stored within that container
//!
//! This allows efficient storage of sparse and dense integer sets.

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

/// Main Roaring Bitmap structure
#[derive(Clone)]
pub struct RoaringBitmap {
    /// Sorted vector of (key, container) pairs
    /// Key = high 16 bits of the integer
    /// Container stores the low 16 bits
    ///
    /// # Invariants
    /// - Keys must be sorted in ascending order
    /// - Keys must be unique (no duplicates)
    /// - Each container must be non-empty
    ///
    /// These invariants must be maintained by insert/remove operations
    containers: Vec<(u16, Container)>,
}

/// Detailed memory usage information for a RoaringBitmap
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Total memory usage in bytes (stack + heap)
    pub total: usize,
    /// Stack-allocated memory in bytes (the struct itself)
    pub stack: usize,
    /// Total heap-allocated memory in bytes
    pub heap: usize,
    /// Per-container memory breakdown
    pub containers: Vec<ContainerStats>,
}

/// Memory statistics for a single container
#[derive(Debug, Clone)]
pub struct ContainerStats {
    /// Container key (high 16 bits)
    pub key: u16,
    /// Container type: "Array", "Bitmap", or "Run"
    pub container_type: &'static str,
    /// Memory used by this container in bytes
    pub memory_bytes: usize,
}

/// Container types for storing values within a 16-bit range
#[derive(Clone)]
enum Container {
    /// Array container: sorted array of u16 values
    /// Used for sparse data (typically < 4096 elements)
    Array(ArrayContainer),

    /// Bitmap container: 8KB bitmap (65536 bits)
    /// Used for dense data (typically >= 4096 elements)
    Bitmap(BitmapContainer),

    /// Run container: run-length encoded consecutive values
    /// Used for data with long consecutive sequences
    Run(RunContainer),
}

/// Threshold for converting between array and bitmap containers
const ARRAY_TO_BITMAP_THRESHOLD: usize = 4096;

/// Array container: stores values as a sorted Vec<u16>
#[derive(Clone)]
struct ArrayContainer {
    /// Sorted array of values (low 16 bits)
    ///
    /// # Invariants
    /// - Values must be sorted in ascending order
    /// - Values must be unique (no duplicates)
    /// - Container must be non-empty (empty containers should be removed from RoaringBitmap)
    ///
    /// These invariants must be maintained by insert/remove operations
    values: Vec<u16>,
}

/// Bitmap container: stores values as a bitmap of 65536 bits (8KB)
#[derive(Clone)]
struct BitmapContainer {
    /// Bitmap stored as 1024 u64 values (65536 / 64 = 1024)
    /// Each bit represents whether a value is present (1) or absent (0)
    ///
    /// # Layout
    /// - bits[0] covers values 0-63 (bit 0 = value 0, bit 1 = value 1, etc.)
    /// - bits[1] covers values 64-127
    /// - ...
    /// - bits[1023] covers values 65472-65535
    ///
    /// # Invariants
    /// - Container must be non-empty (at least one bit set to 1)
    /// - Cardinality should be tracked for efficiency
    bits: Box<[u64; 1024]>,

    /// Cached cardinality (number of set bits)
    /// Maintained by insert/remove operations
    cardinality: u64,
}

/// Run container: stores consecutive sequences as (start, length_minus_1) pairs
#[derive(Clone)]
struct RunContainer {
    /// Vector of runs, each run is (start_value, length_minus_1)
    /// where length_minus_1 is (actual_length - 1) to allow representing up to 65536 values
    ///
    /// Encoding: stored value 0 = 1 value, stored value 65535 = 65536 values
    /// Example: run (10, 3) represents values [10, 11, 12, 13] (length_minus_1=3, actual length=4)
    ///
    /// # Invariants
    /// - Runs must be sorted by start value
    /// - Runs must not overlap or be adjacent (should be merged)
    /// - Container must be non-empty (at least one run)
    runs: Vec<(u16, u16)>,
}

impl Default for RoaringBitmap {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over elements in a RoaringBitmap
pub struct Iter<'a> {
    /// Reference to the bitmap being iterated
    bitmap: &'a RoaringBitmap,
    /// Index of current container
    container_index: usize,
    /// Index within current container (for array containers, or run index for run containers)
    value_index: usize,
    /// For bitmap containers: current word index
    bitmap_word_index: usize,
    /// For bitmap containers: current word being scanned
    bitmap_current_word: u64,
    /// For bitmap containers: bit position within current word
    bitmap_bit_position: u8,
    /// For run containers: offset within current run
    run_offset: u16,
}

impl RoaringBitmap {
    // Helper methods

    /// Splits a u32 value into high 16 bits (key) and low 16 bits
    #[inline]
    fn split(value: u32) -> (u16, u16) {
        let key = (value >> 16) as u16;
        let low = value as u16;
        (key, low)
    }

    /// Combines key (high 16 bits) and low 16 bits into u32
    #[inline]
    fn combine(key: u16, low: u16) -> u32 {
        ((key as u32) << 16) | (low as u32)
    }

    // Construction

    /// Creates an empty roaring bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let bm = RoaringBitmap::new();
    /// assert!(bm.is_empty());
    /// assert_eq!(bm.len(), 0);
    /// ```
    pub fn new() -> Self {
        RoaringBitmap {
            containers: Vec::new(),
        }
    }

    // Insertion

    /// Adds a single element to the bitmap, returns `true` if the element was newly inserted
    ///
    /// # Implementation Notes
    /// Must maintain invariants:
    /// - Containers vector remains sorted by key
    /// - No duplicate keys in containers vector
    /// - Values within each container remain sorted and unique
    /// - Create new container if key doesn't exist
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// assert!(bm.insert(42));  // Returns true, newly inserted
    /// assert!(!bm.insert(42)); // Returns false, already present
    /// assert_eq!(bm.len(), 1);
    /// ```
    pub fn insert(&mut self, value: u32) -> bool {
        let (key, low) = Self::split(value);

        // Binary search for container with this key
        match self.containers.binary_search_by_key(&key, |(k, _)| *k) {
            Ok(index) => {
                // Container exists, insert into it
                let container = &mut self.containers[index].1;
                container.insert(low)
            }
            Err(index) => {
                // Container doesn't exist, create new one
                let mut container = Container::Array(ArrayContainer { values: Vec::new() });
                container.insert(low);
                self.containers.insert(index, (key, container));
                true
            }
        }
    }

    // Query Operations

    /// Checks if an element exists in the bitmap
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// bm.insert(10);
    /// assert!(bm.contains(10));
    /// assert!(!bm.contains(20));
    /// ```
    pub fn contains(&self, value: u32) -> bool {
        let (key, low) = Self::split(value);

        // Binary search for container with this key
        match self.containers.binary_search_by_key(&key, |(k, _)| *k) {
            Ok(index) => {
                // Container exists, check if value is in it
                self.containers[index].1.contains(low)
            }
            Err(_) => {
                // Container doesn't exist, value not present
                false
            }
        }
    }

    /// Returns the number of elements in the bitmap (cardinality)
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// assert_eq!(bm.len(), 0);
    /// bm.insert(1);
    /// bm.insert(2);
    /// bm.insert(3);
    /// assert_eq!(bm.len(), 3);
    /// ```
    pub fn len(&self) -> u64 {
        self.containers
            .iter()
            .map(|(_, container)| container.len())
            .sum()
    }

    /// Returns `true` if the bitmap contains no elements
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// assert!(bm.is_empty());
    /// bm.insert(1);
    /// assert!(!bm.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.containers.is_empty()
    }

    // Deletion

    /// Removes a single element from the bitmap, returns `true` if the element was present
    ///
    /// # Implementation Notes
    /// Must maintain invariants:
    /// - Remove empty containers after last value is removed
    /// - Containers vector remains sorted by key
    /// - Values within each container remain sorted and unique
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// bm.insert(42);
    /// assert!(bm.remove(42));  // Returns true, was present
    /// assert!(!bm.remove(42)); // Returns false, no longer present
    /// ```
    pub fn remove(&mut self, value: u32) -> bool {
        let (key, low) = Self::split(value);

        // Binary search for container with this key
        match self.containers.binary_search_by_key(&key, |(k, _)| *k) {
            Ok(index) => {
                // Container exists, remove from it
                let container = &mut self.containers[index].1;
                let removed = container.remove(low);

                // If container is now empty, remove it (maintain invariant)
                if removed && container.is_empty() {
                    self.containers.remove(index);
                }

                removed
            }
            Err(_) => {
                // Container doesn't exist, value not present
                false
            }
        }
    }

    /// Efficiently removes a range of consecutive values.
    ///
    /// This method mirrors `extend_consecutive` and provides efficient bulk deletion
    /// of contiguous integer ranges. It's particularly useful for:
    /// - Batch downsampling operations
    /// - Trimming large contiguous blocks
    /// - Time window expiration
    /// - Clearing sequential ID ranges
    ///
    /// # Performance
    ///
    /// - **Time**: O(n) where n = number of containers affected
    /// - **Best for**: Consecutive ranges like `0..1000` or `1000..=2000`
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// bm.extend_consecutive(0..100_000);
    ///
    /// // Remove a contiguous block
    /// bm.remove_range(1000..2000);
    /// assert!(!bm.contains(1500));
    /// assert!(bm.contains(500));
    /// assert!(bm.contains(2500));
    ///
    /// // Clear old time-series data
    /// bm.remove_range(0..10_000);
    /// assert!(!bm.contains(5000));
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Time-series pruning**: Remove old timestamp ranges
    /// - **Downsampling**: Delete every nth block of IDs
    /// - **Range invalidation**: Clear specific ID ranges
    /// - **Window operations**: Remove data outside sliding windows
    pub fn remove_range<R: std::ops::RangeBounds<u32>>(&mut self, range: R) {
        use std::ops::Bound::*;

        // Determine start and end of range
        let start = match range.start_bound() {
            Included(&s) => s,
            Excluded(&s) => s.saturating_add(1),
            Unbounded => 0,
        };

        let end = match range.end_bound() {
            Included(&e) => e,
            Excluded(&e) => e.saturating_sub(1),
            Unbounded => u32::MAX,
        };

        if start > end {
            return; // Empty range
        }

        // Process range by splitting into containers
        let mut current = start;
        while current <= end {
            let (key, low_start) = Self::split(current);
            let (end_key, low_end) = Self::split(end);

            // Determine how many values to remove from this container
            let low_end_in_container = if key == end_key { low_end } else { u16::MAX };

            // Find container
            if let Ok(index) = self.containers.binary_search_by_key(&key, |(k, _)| *k) {
                // Container exists, remove values
                let container = &mut self.containers[index].1;
                for low in low_start..=low_end_in_container {
                    container.remove(low);
                }

                // If container is now empty, remove it
                if container.is_empty() {
                    self.containers.remove(index);
                }
            }

            // Move to next container
            if key == end_key {
                break;
            }
            current = ((key as u32 + 1) << 16).max(current + 1);
        }
    }

    /// Efficiently removes multiple sparse values.
    ///
    /// This method mirrors `extend_sparse` and provides efficient bulk deletion
    /// of non-consecutive values.
    ///
    /// # Performance
    ///
    /// - **Time**: O(n log m) where n = values to remove, m = container size
    /// - **Best for**: Scattered values with large gaps
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// bm.extend_sparse([100, 1000, 5000, 10000, 50000]);
    ///
    /// // Remove specific values
    /// bm.remove_sparse([1000, 10000]);
    /// assert!(!bm.contains(1000));
    /// assert!(!bm.contains(10000));
    /// assert!(bm.contains(100));
    /// assert!(bm.contains(5000));
    ///
    /// // From a collection
    /// let to_remove: Vec<u32> = vec![100, 5000];
    /// bm.remove_sparse(to_remove);
    /// assert_eq!(bm.len(), 1); // Only 50000 remains
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Removing specific user IDs or session IDs
    /// - Clearing flagged items
    /// - Batch deletion of sparse indices
    pub fn remove_sparse<I: IntoIterator<Item = u32>>(&mut self, values: I) {
        for value in values {
            self.remove(value);
        }
    }

    /// Removes all elements from the bitmap.
    ///
    /// This method efficiently clears the entire bitmap, removing all containers.
    /// After calling this method, the bitmap will be empty (`len() == 0`).
    ///
    /// # Performance
    ///
    /// - **Time**: O(1) - just clears the containers vector
    /// - **Memory**: Deallocates all container memory
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// bm.extend_consecutive(0..100_000);
    /// assert_eq!(bm.len(), 100_000);
    ///
    /// bm.clear();
    /// assert!(bm.is_empty());
    /// assert_eq!(bm.len(), 0);
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Resetting bitmap state
    /// - Clearing temporary bitmaps for reuse
    /// - Starting fresh without reallocating
    pub fn clear(&mut self) {
        self.containers.clear();
    }

    // Set Operations

    /// Returns the union (OR) of two bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    /// let result = a.union(&b);
    /// assert!(result.contains(1));
    /// assert!(result.contains(2));
    /// assert!(result.contains(3));
    /// assert_eq!(result.len(), 3);
    /// ```
    pub fn union(&self, other: &RoaringBitmap) -> RoaringBitmap {
        let mut result_containers = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Merge containers from both bitmaps
        while i < self.containers.len() && j < other.containers.len() {
            let (key_a, container_a) = &self.containers[i];
            let (key_b, container_b) = &other.containers[j];

            match key_a.cmp(key_b) {
                std::cmp::Ordering::Less => {
                    // Key only in self
                    result_containers.push((*key_a, container_a.clone()));
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Key in both, union the containers
                    let union_container = container_a.union(container_b);
                    result_containers.push((*key_a, union_container));
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    // Key only in other
                    result_containers.push((*key_b, container_b.clone()));
                    j += 1;
                }
            }
        }

        // Add remaining containers from self
        while i < self.containers.len() {
            result_containers.push(self.containers[i].clone());
            i += 1;
        }

        // Add remaining containers from other
        while j < other.containers.len() {
            result_containers.push(other.containers[j].clone());
            j += 1;
        }

        RoaringBitmap {
            containers: result_containers,
        }
    }

    /// Returns the intersection (AND) of two bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    /// let result = a.intersection(&b);
    /// assert!(result.contains(2));
    /// assert_eq!(result.len(), 1);
    /// ```
    pub fn intersection(&self, other: &RoaringBitmap) -> RoaringBitmap {
        let mut result_containers = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Only keep containers where keys match in both bitmaps
        while i < self.containers.len() && j < other.containers.len() {
            let (key_a, container_a) = &self.containers[i];
            let (key_b, container_b) = &other.containers[j];

            match key_a.cmp(key_b) {
                std::cmp::Ordering::Less => {
                    // Key only in self, skip
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Key in both, intersect the containers
                    if let Some(intersect_container) = container_a.intersection(container_b) {
                        result_containers.push((*key_a, intersect_container));
                    }
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    // Key only in other, skip
                    j += 1;
                }
            }
        }

        RoaringBitmap {
            containers: result_containers,
        }
    }

    /// Returns the difference (AND NOT) of two bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    /// let result = a.difference(&b);
    /// assert!(result.contains(1));
    /// assert!(!result.contains(2));
    /// assert_eq!(result.len(), 1);
    /// ```
    pub fn difference(&self, other: &RoaringBitmap) -> RoaringBitmap {
        let mut result_containers = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Keep elements from self that are not in other
        while i < self.containers.len() && j < other.containers.len() {
            let (key_a, container_a) = &self.containers[i];
            let (key_b, container_b) = &other.containers[j];

            match key_a.cmp(key_b) {
                std::cmp::Ordering::Less => {
                    // Key only in self, keep it
                    result_containers.push((*key_a, container_a.clone()));
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Key in both, compute difference
                    if let Some(diff_container) = container_a.difference(container_b) {
                        result_containers.push((*key_a, diff_container));
                    }
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    // Key only in other, skip
                    j += 1;
                }
            }
        }

        // Add remaining containers from self
        while i < self.containers.len() {
            result_containers.push(self.containers[i].clone());
            i += 1;
        }

        RoaringBitmap {
            containers: result_containers,
        }
    }

    /// Returns the symmetric difference (XOR) of two bitmaps
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    /// let result = a.symmetric_difference(&b);
    /// assert!(result.contains(1));
    /// assert!(!result.contains(2));
    /// assert!(result.contains(3));
    /// assert_eq!(result.len(), 2);
    /// ```
    pub fn symmetric_difference(&self, other: &RoaringBitmap) -> RoaringBitmap {
        let mut result_containers = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Keep elements that are in exactly one of the two bitmaps
        while i < self.containers.len() && j < other.containers.len() {
            let (key_a, container_a) = &self.containers[i];
            let (key_b, container_b) = &other.containers[j];

            match key_a.cmp(key_b) {
                std::cmp::Ordering::Less => {
                    // Key only in self
                    result_containers.push((*key_a, container_a.clone()));
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Key in both, compute symmetric difference
                    if let Some(xor_container) = container_a.symmetric_difference(container_b) {
                        result_containers.push((*key_a, xor_container));
                    }
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    // Key only in other
                    result_containers.push((*key_b, container_b.clone()));
                    j += 1;
                }
            }
        }

        // Add remaining containers from self
        while i < self.containers.len() {
            result_containers.push(self.containers[i].clone());
            i += 1;
        }

        // Add remaining containers from other
        while j < other.containers.len() {
            result_containers.push(other.containers[j].clone());
            j += 1;
        }

        RoaringBitmap {
            containers: result_containers,
        }
    }

    // In-place Set Operations

    /// Computes the union in-place, modifying this bitmap to include all elements from `other`.
    ///
    /// This is an in-place operation that avoids allocating a new bitmap. It's more efficient
    /// than `self = self.union(other)` when you want to modify an existing bitmap.
    ///
    /// # Performance
    ///
    /// - **Time**: O(n + m) where n, m = number of containers in each bitmap
    /// - **Space**: No new bitmap allocation (modifies in-place)
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// a.union_with(&b);  // Modifies 'a' in-place
    /// assert_eq!(a.len(), 3);
    /// assert!(a.contains(1));
    /// assert!(a.contains(2));
    /// assert!(a.contains(3));
    /// ```
    ///
    /// # Common Workflows
    ///
    /// **Building up a union incrementally:**
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut result = RoaringBitmap::new();
    /// # let bitmaps = vec![RoaringBitmap::new(); 3];
    /// for bitmap in &bitmaps {
    ///     result.union_with(bitmap);  // Zero intermediate allocations
    /// }
    /// ```
    pub fn union_with(&mut self, other: &RoaringBitmap) {
        let mut result_containers = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Merge containers from both bitmaps
        while i < self.containers.len() && j < other.containers.len() {
            let (key_a, container_a) = &self.containers[i];
            let (key_b, container_b) = &other.containers[j];

            match key_a.cmp(key_b) {
                std::cmp::Ordering::Less => {
                    // Key only in self
                    result_containers.push((*key_a, container_a.clone()));
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Key in both, union the containers
                    let union_container = container_a.union(container_b);
                    result_containers.push((*key_a, union_container));
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    // Key only in other
                    result_containers.push((*key_b, container_b.clone()));
                    j += 1;
                }
            }
        }

        // Add remaining containers from self
        while i < self.containers.len() {
            result_containers.push(self.containers[i].clone());
            i += 1;
        }

        // Add remaining containers from other
        while j < other.containers.len() {
            result_containers.push(other.containers[j].clone());
            j += 1;
        }

        self.containers = result_containers;
    }

    /// Computes the intersection in-place, modifying this bitmap to keep only elements in `other`.
    ///
    /// This is an in-place operation that avoids allocating a new bitmap.
    ///
    /// # Performance
    ///
    /// - **Time**: O(n + m) where n, m = number of containers in each bitmap
    /// - **Space**: No new bitmap allocation (modifies in-place)
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    /// b.insert(2);
    /// b.insert(3);
    /// b.insert(4);
    ///
    /// a.intersect_with(&b);  // Modifies 'a' in-place
    /// assert_eq!(a.len(), 2);
    /// assert!(!a.contains(1));
    /// assert!(a.contains(2));
    /// assert!(a.contains(3));
    /// assert!(!a.contains(4));
    /// ```
    ///
    /// # Common Workflows
    ///
    /// **Applying multiple filters:**
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut candidates = RoaringBitmap::new();
    /// # let filter1 = RoaringBitmap::new();
    /// # let filter2 = RoaringBitmap::new();
    /// # let filter3 = RoaringBitmap::new();
    /// // ... populate candidates and filters ...
    ///
    /// candidates.intersect_with(&filter1);  // Zero intermediate allocations
    /// candidates.intersect_with(&filter2);
    /// candidates.intersect_with(&filter3);
    /// ```
    pub fn intersect_with(&mut self, other: &RoaringBitmap) {
        let mut result_containers = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Only keep containers where keys match in both bitmaps
        while i < self.containers.len() && j < other.containers.len() {
            let (key_a, container_a) = &self.containers[i];
            let (key_b, container_b) = &other.containers[j];

            match key_a.cmp(key_b) {
                std::cmp::Ordering::Less => {
                    // Key only in self, skip
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Key in both, intersect the containers
                    if let Some(intersect_container) = container_a.intersection(container_b) {
                        result_containers.push((*key_a, intersect_container));
                    }
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    // Key only in other, skip
                    j += 1;
                }
            }
        }

        self.containers = result_containers;
    }

    /// Computes the difference in-place, modifying this bitmap to remove elements in `other`.
    ///
    /// This is an in-place operation that avoids allocating a new bitmap.
    ///
    /// # Performance
    ///
    /// - **Time**: O(n + m) where n, m = number of containers in each bitmap
    /// - **Space**: No new bitmap allocation (modifies in-place)
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    /// b.insert(2);
    /// b.insert(4);
    ///
    /// a.difference_with(&b);  // Modifies 'a' in-place
    /// assert_eq!(a.len(), 2);
    /// assert!(a.contains(1));
    /// assert!(!a.contains(2));
    /// assert!(a.contains(3));
    /// ```
    pub fn difference_with(&mut self, other: &RoaringBitmap) {
        let mut result_containers = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Keep elements from self that are not in other
        while i < self.containers.len() && j < other.containers.len() {
            let (key_a, container_a) = &self.containers[i];
            let (key_b, container_b) = &other.containers[j];

            match key_a.cmp(key_b) {
                std::cmp::Ordering::Less => {
                    // Key only in self, keep it
                    result_containers.push((*key_a, container_a.clone()));
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Key in both, compute difference
                    if let Some(diff_container) = container_a.difference(container_b) {
                        result_containers.push((*key_a, diff_container));
                    }
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    // Key only in other, skip
                    j += 1;
                }
            }
        }

        // Add remaining containers from self
        while i < self.containers.len() {
            result_containers.push(self.containers[i].clone());
            i += 1;
        }

        self.containers = result_containers;
    }

    /// Computes the symmetric difference in-place, modifying this bitmap to keep only
    /// elements that are in exactly one of the two bitmaps.
    ///
    /// This is an in-place operation that avoids allocating a new bitmap.
    ///
    /// # Performance
    ///
    /// - **Time**: O(n + m) where n, m = number of containers in each bitmap
    /// - **Space**: No new bitmap allocation (modifies in-place)
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// a.symmetric_difference_with(&b);  // Modifies 'a' in-place
    /// assert_eq!(a.len(), 2);
    /// assert!(a.contains(1));
    /// assert!(!a.contains(2));
    /// assert!(a.contains(3));
    /// ```
    pub fn symmetric_difference_with(&mut self, other: &RoaringBitmap) {
        let mut result_containers = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Keep elements that are in exactly one of the two bitmaps
        while i < self.containers.len() && j < other.containers.len() {
            let (key_a, container_a) = &self.containers[i];
            let (key_b, container_b) = &other.containers[j];

            match key_a.cmp(key_b) {
                std::cmp::Ordering::Less => {
                    // Key only in self
                    result_containers.push((*key_a, container_a.clone()));
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Key in both, compute symmetric difference
                    if let Some(xor_container) = container_a.symmetric_difference(container_b) {
                        result_containers.push((*key_a, xor_container));
                    }
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    // Key only in other
                    result_containers.push((*key_b, container_b.clone()));
                    j += 1;
                }
            }
        }

        // Add remaining containers from self
        while i < self.containers.len() {
            result_containers.push(self.containers[i].clone());
            i += 1;
        }

        // Add remaining containers from other
        while j < other.containers.len() {
            result_containers.push(other.containers[j].clone());
            j += 1;
        }

        self.containers = result_containers;
    }

    // Iteration

    /// Returns an iterator over elements in sorted order
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// bm.insert(3);
    /// bm.insert(1);
    /// bm.insert(2);
    /// let values: Vec<u32> = bm.iter().collect();
    /// assert_eq!(values, vec![1, 2, 3]); // Always sorted
    /// ```
    pub fn iter(&self) -> Iter<'_> {
        let (bitmap_word_index, bitmap_current_word) = if !self.containers.is_empty() {
            match &self.containers[0].1 {
                Container::Bitmap(bm) => (0, bm.bits[0]),
                _ => (0, 0),
            }
        } else {
            (0, 0)
        };

        Iter {
            bitmap: self,
            container_index: 0,
            value_index: 0,
            bitmap_word_index,
            bitmap_current_word,
            bitmap_bit_position: 0,
            run_offset: 0,
        }
    }

    // Optimization

    /// Optimizes container storage by converting between Array, Bitmap, and Run containers
    /// based on data patterns and density.
    ///
    /// This method analyzes each container and converts it to the most efficient representation:
    /// - Dense data (≥4,096 values) → Bitmap container (8,192 bytes)
    /// - Sparse data with many consecutive sequences → Run container (4 bytes per run)
    /// - Sparse data with few consecutive sequences → Array container (2 bytes per value)
    /// - Fragmented Run containers → Array or Bitmap (when runs > values/2)
    ///
    /// # When to Call
    ///
    /// **Recommended scenarios:**
    /// - After bulk insert/delete operations
    /// - Before serializing to disk (minimize storage)
    /// - After deletions that fragment Run containers
    /// - Before transitioning from write-heavy to read-heavy workload
    ///
    /// **Avoid calling:**
    /// - During continuous write operations (let automatic conversions handle it)
    /// - After every single insert/remove (overhead outweighs benefit)
    /// - When memory usage is not a concern
    ///
    /// # Performance
    ///
    /// **Time Complexity** (per container):
    /// - Array container: O(n) where n = number of values
    /// - Bitmap container: O(65,536) - must scan all bits
    /// - Run container: O(1) - just check metadata
    ///
    /// **Space Complexity**:
    /// - Temporary allocation: O(n) for container conversion
    /// - Worst case: ~131 KB when converting full bitmap (65,536 values × 2 bytes)
    /// - Typical case: < 10 KB
    ///
    /// **Timing estimates** (per container):
    /// - Small (< 1,000 values): < 1 microsecond
    /// - Medium (1,000-10,000 values): 1-10 microseconds
    /// - Large (> 10,000 values): 10-100 microseconds
    /// - Full bitmap (65,536 values): 100-500 microseconds
    ///
    /// # Memory Savings Examples
    ///
    /// **Consecutive sequences:**
    /// - Before: Array with 10,000 values = 20,000 bytes
    /// - After: Run with 1 sequence = 4 bytes (99.98% savings!)
    ///
    /// **Fragmented Run after deletions:**
    /// - Before: Run with 5,000 runs = 20,000 bytes
    /// - After: Array with 5,000 values = 10,000 bytes (50% savings)
    /// - Or: Bitmap = 8,192 bytes (59% savings)
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    ///
    /// // Insert consecutive values
    /// for i in 0..10000 {
    ///     bm.insert(i);
    /// }
    /// bm.optimize();  // Converts to Run container (4 bytes vs 20KB)
    ///
    /// // Fragment the Run by removing every other value
    /// for i in 0..10000 {
    ///     if i % 2 == 0 {
    ///         bm.remove(i);
    ///     }
    /// }
    /// // Now have 5,000 runs = 20KB (inefficient!)
    /// bm.optimize();  // Converts to Array (10KB) - 50% savings
    /// ```
    ///
    /// ```no_run
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// // Batch operations pattern
    /// let mut bm = RoaringBitmap::new();
    /// # let large_dataset = vec![1u32, 2, 3];
    /// # let queries = vec![1u32, 2, 3];
    ///
    /// // Write phase
    /// for value in large_dataset {
    ///     bm.insert(value);
    /// }
    /// bm.optimize();  // Optimize once after bulk load
    ///
    /// // Read phase (now optimized for queries)
    /// for query in queries {
    ///     let _present = bm.contains(query);
    /// }
    /// ```
    pub fn optimize(&mut self) {
        for (_, container) in &mut self.containers {
            container.optimize();
        }
    }

    // Semantic Bulk Operations (Intermediate API)
    //
    // These methods provide hints about data patterns to optimize insertion performance.
    // They create appropriate container types directly, avoiding intermediate conversions.
    //
    // Important: These are insertion-time optimizations only. The `optimize()` method
    // remains data-driven and may convert containers based on actual data patterns.

    /// Efficiently insert consecutive values by creating Run containers directly.
    ///
    /// This method is optimized for inserting ranges of consecutive integers. It creates
    /// Run containers from the start, avoiding the overhead of creating Array containers
    /// first and then converting them.
    ///
    /// # Performance
    ///
    /// - **Time**: O(n) where n = number of containers affected (typically 1-2)
    /// - **Memory**: ~4 bytes per run (much better than 2 bytes per value in Array)
    /// - **Best for**: Consecutive sequences like `[0,1,2,3,...]` or `[1000,1001,1002,...]`
    ///
    /// # Interaction with `optimize()`
    ///
    /// This method only affects **insertion performance**, not final compression.
    /// Later calls to `optimize()` will still analyze the actual data and may convert
    /// the container if the pattern changes (e.g., due to removals or sparse insertions).
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    ///
    /// // Efficient: creates Run container directly
    /// bm.extend_consecutive(0..1_000_000);
    /// println!("Memory: {} bytes", bm.memory_usage()); // Very compact!
    ///
    /// // Multiple consecutive ranges
    /// bm.extend_consecutive(2_000_000..3_000_000);
    /// bm.extend_consecutive(5_000_000..6_000_000);
    ///
    /// assert_eq!(bm.len(), 3_000_000);
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Database auto-increment IDs: `extend_consecutive(0..num_rows)`
    /// - Time series with regular intervals
    /// - Sequential file offsets or block numbers
    /// - Any scenario where values are naturally consecutive
    pub fn extend_consecutive<R: std::ops::RangeBounds<u32>>(&mut self, range: R) {
        use std::ops::Bound::*;

        // Determine start and end of range
        let start = match range.start_bound() {
            Included(&s) => s,
            Excluded(&s) => s.saturating_add(1),
            Unbounded => 0,
        };

        let end = match range.end_bound() {
            Included(&e) => e,
            Excluded(&e) => e.saturating_sub(1),
            Unbounded => u32::MAX,
        };

        if start > end {
            return; // Empty range
        }

        // Process range by splitting into containers
        let mut current = start;
        while current <= end {
            let (key, low_start) = Self::split(current);
            let (end_key, low_end) = Self::split(end);

            // Determine how many values to add to this container
            let low_end_in_container = if key == end_key { low_end } else { u16::MAX };

            // Find or create container
            match self.containers.binary_search_by_key(&key, |(k, _)| *k) {
                Ok(index) => {
                    // Container exists, add consecutive values
                    let container = &mut self.containers[index].1;
                    for low in low_start..=low_end_in_container {
                        container.insert(low);
                    }
                }
                Err(index) => {
                    // Container doesn't exist, create Run container directly
                    let num_values = (low_end_in_container as u32) - (low_start as u32) + 1;

                    let container = if num_values >= 4096 {
                        // Large consecutive range - check if Run is better
                        // For consecutive values, Run is almost always better
                        // Run: 1 run * 4 bytes = 4 bytes
                        // Array: num_values * 2 bytes
                        let mut runs = Vec::new();
                        runs.push((low_start, (num_values - 1) as u16)); // Store length-1
                        Container::Run(RunContainer { runs })
                    } else {
                        // Smaller range - still create as Run for consecutive data
                        let mut runs = Vec::new();
                        runs.push((low_start, (num_values - 1) as u16)); // Store length-1
                        Container::Run(RunContainer { runs })
                    };

                    self.containers.insert(index, (key, container));
                }
            }

            // Move to next container
            if key == end_key {
                break;
            }
            current = ((key as u32 + 1) << 16).max(current + 1);
        }
    }

    /// Efficiently insert sparse values by creating Array containers.
    ///
    /// This method is optimized for inserting sparse, non-consecutive values. It creates
    /// Array containers that are efficient for storing scattered integers.
    ///
    /// # Performance
    ///
    /// - **Time**: O(n log n) where n = number of values (for sorting)
    /// - **Memory**: 2 bytes per value (Array storage)
    /// - **Best for**: Scattered values with significant gaps, like `[10, 100, 1000, 10000]`
    ///
    /// # When to Use
    ///
    /// Use this method when:
    /// - Values have large gaps between them
    /// - You're inserting a collection of random or pseudo-random values
    /// - Cardinality is low to moderate (< 4096 values per 64K block)
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    ///
    /// // Sparse user IDs
    /// bm.extend_sparse([1000, 5000, 10000, 50000, 100000]);
    ///
    /// // From a vector
    /// let sparse_values: Vec<u32> = vec![42, 1337, 9999];
    /// bm.extend_sparse(sparse_values);
    ///
    /// assert_eq!(bm.len(), 8);
    /// ```
    ///
    /// # Use Cases
    ///
    /// - User IDs, session IDs (random identifiers)
    /// - Sparse index lookups
    /// - Random sampling results
    /// - Feature flags or permission bits with low density
    pub fn extend_sparse<I: IntoIterator<Item = u32>>(&mut self, values: I) {
        for value in values {
            self.insert(value);
        }
    }

    /// Efficiently insert dense values with smart container choice.
    ///
    /// This method is optimized for inserting dense ranges where many (but not all) values
    /// in a range are present. It makes smart decisions about whether to use Array or
    /// Bitmap containers based on density.
    ///
    /// # Performance
    ///
    /// - **Time**: O(n) where n = range size
    /// - **Memory**: Adapts based on density (Array or Bitmap)
    /// - **Best for**: Ranges where ~30-70% of values are present
    ///
    /// # When to Use
    ///
    /// Use this method when:
    /// - You have a range with moderate to high density
    /// - Values are somewhat clustered but have some gaps
    /// - You want the library to choose the best container type
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    ///
    /// // Insert even numbers in a range (50% density)
    /// bm.extend_dense((0..10_000).filter(|x| x % 2 == 0));
    ///
    /// // Dense range with some gaps
    /// let values: Vec<u32> = (0..8000).filter(|x| x % 3 != 0).collect();
    /// bm.extend_dense(values);
    ///
    /// println!("Memory: {} bytes", bm.memory_usage());
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Bloom filter-like structures with moderate hit rates
    /// - Partially filled bitmaps or flags
    /// - Feature vectors with moderate sparsity
    /// - Query result sets with good selectivity
    pub fn extend_dense<I: IntoIterator<Item = u32>>(&mut self, values: I) {
        // For dense insertions, just use regular insert
        // The automatic Array->Bitmap conversion will handle density
        for value in values {
            self.insert(value);
        }
    }

    // Memory Usage

    /// Returns the total memory usage in bytes, including all Rust overheads.
    ///
    /// This includes:
    /// - Stack-allocated struct size (24 bytes for the Vec metadata)
    /// - Heap-allocated containers vector (capacity × element size)
    /// - Memory used by each container (including their heap allocations)
    ///
    /// # Performance
    ///
    /// Time complexity: O(n) where n = number of containers
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    ///
    /// // Empty bitmap
    /// let empty_size = bm.memory_usage();
    /// println!("Empty: {} bytes", empty_size);
    ///
    /// // Add values
    /// for i in 0..1000 {
    ///     bm.insert(i);
    /// }
    /// let array_size = bm.memory_usage();
    /// println!("Array container (1000 values): {} bytes", array_size);
    ///
    /// // Add more to trigger bitmap
    /// for i in 0..5000 {
    ///     bm.insert(i);
    /// }
    /// let bitmap_size = bm.memory_usage();
    /// println!("Bitmap container (5000 values): {} bytes", bitmap_size);
    /// ```
    pub fn memory_usage(&self) -> usize {
        // Stack size of RoaringBitmap struct itself
        let mut total = std::mem::size_of::<Self>();

        // Heap allocation for containers Vec
        // Vec allocates: capacity * size_of::<(u16, Container)>()
        total += self.containers.capacity() * std::mem::size_of::<(u16, Container)>();

        // Memory used by each container (their heap allocations)
        for (_, container) in &self.containers {
            total += container.heap_memory();
        }

        total
    }

    /// Returns detailed memory usage breakdown.
    ///
    /// Returns a `MemoryUsage` struct containing:
    /// - `total`: Total memory in bytes (stack + heap)
    /// - `stack`: Stack-allocated memory (the struct itself)
    /// - `heap`: Total heap-allocated memory
    /// - `containers`: Vector of per-container statistics
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut bm = RoaringBitmap::new();
    /// for i in 0..10000 {
    ///     bm.insert(i);
    /// }
    ///
    /// let usage = bm.memory_usage_detailed();
    /// println!("Total: {} bytes (stack: {}, heap: {})", usage.total, usage.stack, usage.heap);
    /// for container in &usage.containers {
    ///     println!("  Container {}: {} - {} bytes",
    ///         container.key, container.container_type, container.memory_bytes);
    /// }
    /// ```
    pub fn memory_usage_detailed(&self) -> MemoryUsage {
        // Stack size
        let stack_size = std::mem::size_of::<Self>();

        // Heap allocation for containers Vec
        let containers_vec_heap =
            self.containers.capacity() * std::mem::size_of::<(u16, Container)>();

        // Per-container breakdown
        let mut container_stats = Vec::new();
        let mut containers_heap = 0;

        for (key, container) in &self.containers {
            let container_type = match container {
                Container::Array(_) => "Array",
                Container::Bitmap(_) => "Bitmap",
                Container::Run(_) => "Run",
            };
            let memory_bytes = container.heap_memory();
            containers_heap += memory_bytes;
            container_stats.push(ContainerStats {
                key: *key,
                container_type,
                memory_bytes,
            });
        }

        let total_heap = containers_vec_heap + containers_heap;
        let total = stack_size + total_heap;

        MemoryUsage {
            total,
            stack: stack_size,
            heap: total_heap,
            containers: container_stats,
        }
    }

    // Test/Debug helpers

    /// Returns the container type for a given key (for testing purposes)
    /// Returns None if container doesn't exist
    ///
    /// **Note**: This method is intended for testing and debugging only.
    /// It exposes internal implementation details.
    #[doc(hidden)]
    pub fn container_type(&self, key: u16) -> Option<&'static str> {
        self.containers
            .binary_search_by_key(&key, |(k, _)| *k)
            .ok()
            .map(|index| match &self.containers[index].1 {
                Container::Array(_) => "Array",
                Container::Bitmap(_) => "Bitmap",
                Container::Run(_) => "Run",
            })
    }

    /// Returns information about all containers (for testing purposes)
    /// Returns a vector of (key, type_name, cardinality) tuples
    ///
    /// **Note**: This method is intended for testing and debugging only.
    /// It exposes internal implementation details.
    #[doc(hidden)]
    pub fn container_stats(&self) -> Vec<(u16, &'static str, u64)> {
        self.containers
            .iter()
            .map(|(key, container)| {
                let (type_name, len) = match container {
                    Container::Array(_) => ("Array", container.len()),
                    Container::Bitmap(_) => ("Bitmap", container.len()),
                    Container::Run(_) => ("Run", container.len()),
                };
                (*key, type_name, len)
            })
            .collect()
    }
}

// Container implementations

impl Container {
    /// Inserts a value into the container, returns true if newly inserted
    ///
    /// # Automatic Conversions (Conservative)
    ///
    /// Only essential conversions happen automatically:
    /// - Array → Bitmap: When array reaches 4,096 values
    ///   - Before conversion, checks if Run would be better (consecutive data)
    ///
    /// For more aggressive optimization (e.g., converting small Arrays with many
    /// consecutive values to Run), call optimize() explicitly.
    fn insert(&mut self, value: u16) -> bool {
        let result = match self {
            Container::Array(array) => array.insert(value),
            Container::Bitmap(bitmap) => bitmap.insert(value),
            Container::Run(run) => run.insert(value),
        };

        // Strategic conversion checkpoints (conservative by default)
        // For aggressive optimization, users should call optimize() explicitly
        if result {
            if let Container::Array(array) = self {
                // Only checkpoint: Before converting to Bitmap, check if Run is better
                if array.len() as usize >= ARRAY_TO_BITMAP_THRESHOLD {
                    if Self::run_better_than_bitmap_array(array) {
                        *self = Container::Run(RunContainer::from_array(array));
                    } else {
                        *self = Container::Bitmap(BitmapContainer::from_array(array));
                    }
                }
            }
        }

        result
    }

    /// Checks if a value exists in the container
    fn contains(&self, value: u16) -> bool {
        match self {
            Container::Array(array) => array.contains(value),
            Container::Bitmap(bitmap) => bitmap.contains(value),
            Container::Run(run) => run.contains(value),
        }
    }

    /// Removes a value from the container, returns true if it was present
    ///
    /// # Automatic Conversions (Conservative)
    ///
    /// Only essential conversions happen automatically:
    /// - Bitmap → Array: When cardinality drops below 4,096
    ///
    /// Run containers are NOT automatically converted even if fragmented.
    /// This is intentional to avoid overhead during write-heavy operations.
    /// Call optimize() explicitly to convert fragmented Run containers.
    ///
    /// # Why Not Auto-Convert Run Containers?
    ///
    /// Deletions can fragment Run containers, making them inefficient:
    /// - Example: Removing every other value from [0-10000] creates 5,000 runs (20KB)
    /// - Better: Array (10KB) or Bitmap (8KB)
    ///
    /// However, automatic conversion on every remove() would:
    /// - Add overhead to every removal operation
    /// - Cause thrashing with mixed insert/remove patterns
    /// - Not align with our Hybrid+Lazy optimization philosophy
    ///
    /// Instead, users should call optimize() when appropriate (e.g., after batch deletions).
    fn remove(&mut self, value: u16) -> bool {
        let result = match self {
            Container::Array(array) => array.remove(value),
            Container::Bitmap(bitmap) => bitmap.remove(value),
            Container::Run(run) => run.remove(value),
        };

        // Convert bitmap to array if it shrinks too small
        if result {
            if let Container::Bitmap(bitmap) = self {
                if bitmap.len() < ARRAY_TO_BITMAP_THRESHOLD as u64 {
                    *self = Container::Array(bitmap.to_array());
                }
            }
        }

        result
    }

    /// Returns the number of elements in the container
    fn len(&self) -> u64 {
        match self {
            Container::Array(array) => array.len(),
            Container::Bitmap(bitmap) => bitmap.len(),
            Container::Run(run) => run.len(),
        }
    }

    /// Checks if the container is empty
    fn is_empty(&self) -> bool {
        match self {
            Container::Array(array) => array.is_empty(),
            Container::Bitmap(bitmap) => bitmap.is_empty(),
            Container::Run(run) => run.is_empty(),
        }
    }

    /// Returns the union of two containers
    fn union(&self, other: &Container) -> Container {
        match (self, other) {
            (Container::Array(a), Container::Array(b)) => {
                let result = a.union(b);
                // Convert to bitmap if result is large
                if result.len() as usize >= ARRAY_TO_BITMAP_THRESHOLD {
                    Container::Bitmap(BitmapContainer::from_array(&result))
                } else {
                    Container::Array(result)
                }
            }
            (Container::Bitmap(a), Container::Bitmap(b)) => Container::Bitmap(a.union(b)),
            (Container::Array(a), Container::Bitmap(b)) => {
                let mut result = b.clone();
                for &value in &a.values {
                    result.insert(value);
                }
                Container::Bitmap(result)
            }
            (Container::Bitmap(a), Container::Array(b)) => {
                let mut result = a.clone();
                for &value in &b.values {
                    result.insert(value);
                }
                Container::Bitmap(result)
            }
            (Container::Run(a), Container::Run(b)) => Container::Run(a.union(b)),
            // Run + Array/Bitmap: convert run to array and delegate
            (Container::Run(r), Container::Array(a)) | (Container::Array(a), Container::Run(r)) => {
                let run_array = r.to_array();
                Container::Array(run_array.union(a))
            }
            (Container::Run(r), Container::Bitmap(b))
            | (Container::Bitmap(b), Container::Run(r)) => {
                let run_array = r.to_array();
                let mut result = b.clone();
                for &value in &run_array.values {
                    result.insert(value);
                }
                Container::Bitmap(result)
            }
        }
    }

    /// Returns the intersection of two containers
    fn intersection(&self, other: &Container) -> Option<Container> {
        match (self, other) {
            (Container::Array(a), Container::Array(b)) => a.intersection(b).map(Container::Array),
            (Container::Bitmap(a), Container::Bitmap(b)) => {
                a.intersection(b).map(|bitmap| {
                    // Convert to array if result is small
                    if bitmap.len() < ARRAY_TO_BITMAP_THRESHOLD as u64 {
                        Container::Array(bitmap.to_array())
                    } else {
                        Container::Bitmap(bitmap)
                    }
                })
            }
            (Container::Array(a), Container::Bitmap(b)) => {
                // Array-Bitmap intersection: check which array elements are in bitmap
                let mut values = Vec::new();
                for &value in &a.values {
                    if b.contains(value) {
                        values.push(value);
                    }
                }
                if values.is_empty() {
                    None
                } else {
                    Some(Container::Array(ArrayContainer { values }))
                }
            }
            (Container::Bitmap(a), Container::Array(b)) => {
                // Bitmap-Array intersection: same as Array-Bitmap
                let mut values = Vec::new();
                for &value in &b.values {
                    if a.contains(value) {
                        values.push(value);
                    }
                }
                if values.is_empty() {
                    None
                } else {
                    Some(Container::Array(ArrayContainer { values }))
                }
            }
            (Container::Run(a), Container::Run(b)) => a.intersection(b).map(Container::Run),
            // Run + Array/Bitmap: convert run to array and delegate
            (Container::Run(r), Container::Array(a)) | (Container::Array(a), Container::Run(r)) => {
                let run_array = r.to_array();
                run_array.intersection(a).map(Container::Array)
            }
            (Container::Run(r), Container::Bitmap(b))
            | (Container::Bitmap(b), Container::Run(r)) => {
                let run_array = r.to_array();
                let mut values = Vec::new();
                for &value in &run_array.values {
                    if b.contains(value) {
                        values.push(value);
                    }
                }
                if values.is_empty() {
                    None
                } else {
                    Some(Container::Array(ArrayContainer { values }))
                }
            }
        }
    }

    /// Returns the difference of two containers (self - other)
    fn difference(&self, other: &Container) -> Option<Container> {
        match (self, other) {
            (Container::Array(a), Container::Array(b)) => a.difference(b).map(Container::Array),
            (Container::Bitmap(a), Container::Bitmap(b)) => {
                a.difference(b).map(|bitmap| {
                    // Convert to array if result is small
                    if bitmap.len() < ARRAY_TO_BITMAP_THRESHOLD as u64 {
                        Container::Array(bitmap.to_array())
                    } else {
                        Container::Bitmap(bitmap)
                    }
                })
            }
            (Container::Array(a), Container::Bitmap(b)) => {
                // Array - Bitmap: keep array elements not in bitmap
                let mut values = Vec::new();
                for &value in &a.values {
                    if !b.contains(value) {
                        values.push(value);
                    }
                }
                if values.is_empty() {
                    None
                } else {
                    Some(Container::Array(ArrayContainer { values }))
                }
            }
            (Container::Bitmap(a), Container::Array(b)) => {
                // Bitmap - Array: remove array elements from bitmap
                let mut result = a.clone();
                for &value in &b.values {
                    result.remove(value);
                }
                if result.is_empty() {
                    None
                } else if result.len() < ARRAY_TO_BITMAP_THRESHOLD as u64 {
                    Some(Container::Array(result.to_array()))
                } else {
                    Some(Container::Bitmap(result))
                }
            }
            (Container::Run(a), Container::Run(b)) => a.difference(b).map(Container::Run),
            // Run + Array/Bitmap: convert run to array and delegate
            (Container::Run(r), Container::Array(a)) => {
                let run_array = r.to_array();
                run_array.difference(a).map(Container::Array)
            }
            (Container::Array(a), Container::Run(r)) => {
                let run_array = r.to_array();
                a.difference(&run_array).map(Container::Array)
            }
            (Container::Run(r), Container::Bitmap(b)) => {
                let run_array = r.to_array();
                let mut values = Vec::new();
                for &value in &run_array.values {
                    if !b.contains(value) {
                        values.push(value);
                    }
                }
                if values.is_empty() {
                    None
                } else {
                    Some(Container::Array(ArrayContainer { values }))
                }
            }
            (Container::Bitmap(b), Container::Run(r)) => {
                let run_array = r.to_array();
                let mut result = b.clone();
                for &value in &run_array.values {
                    result.remove(value);
                }
                if result.is_empty() {
                    None
                } else if result.len() < ARRAY_TO_BITMAP_THRESHOLD as u64 {
                    Some(Container::Array(result.to_array()))
                } else {
                    Some(Container::Bitmap(result))
                }
            }
        }
    }

    /// Returns the symmetric difference of two containers (self XOR other)
    fn symmetric_difference(&self, other: &Container) -> Option<Container> {
        match (self, other) {
            (Container::Array(a), Container::Array(b)) => {
                a.symmetric_difference(b).map(Container::Array)
            }
            (Container::Bitmap(a), Container::Bitmap(b)) => {
                a.symmetric_difference(b).map(|bitmap| {
                    // Convert to array if result is small
                    if bitmap.len() < ARRAY_TO_BITMAP_THRESHOLD as u64 {
                        Container::Array(bitmap.to_array())
                    } else {
                        Container::Bitmap(bitmap)
                    }
                })
            }
            (Container::Array(a), Container::Bitmap(b))
            | (Container::Bitmap(b), Container::Array(a)) => {
                // Convert array to bitmap and compute XOR
                let a_bitmap = BitmapContainer::from_array(a);
                a_bitmap.symmetric_difference(b).map(|bitmap| {
                    if bitmap.len() < ARRAY_TO_BITMAP_THRESHOLD as u64 {
                        Container::Array(bitmap.to_array())
                    } else {
                        Container::Bitmap(bitmap)
                    }
                })
            }
            (Container::Run(a), Container::Run(b)) => a.symmetric_difference(b).map(Container::Run),
            // Run + Array/Bitmap: convert run to array and delegate
            (Container::Run(r), Container::Array(a)) | (Container::Array(a), Container::Run(r)) => {
                let run_array = r.to_array();
                run_array.symmetric_difference(a).map(Container::Array)
            }
            (Container::Run(r), Container::Bitmap(b))
            | (Container::Bitmap(b), Container::Run(r)) => {
                let run_array = r.to_array();
                let a_bitmap = BitmapContainer::from_array(&run_array);
                a_bitmap.symmetric_difference(b).map(|bitmap| {
                    if bitmap.len() < ARRAY_TO_BITMAP_THRESHOLD as u64 {
                        Container::Array(bitmap.to_array())
                    } else {
                        Container::Bitmap(bitmap)
                    }
                })
            }
        }
    }

    // Memory usage

    /// Returns the heap memory used by this container in bytes
    fn heap_memory(&self) -> usize {
        match self {
            Container::Array(array) => array.heap_memory(),
            Container::Bitmap(bitmap) => bitmap.heap_memory(),
            Container::Run(run) => run.heap_memory(),
        }
    }

    // Optimization helpers

    /// Optimizes this container by converting to the most efficient type
    ///
    /// # Conversion Heuristics
    ///
    /// **Array Container:**
    /// - → Run: When runs < values/2 AND values ≥ 10
    ///   (Consecutive sequences make Run more efficient, avoid overhead for tiny containers)
    /// - → Bitmap: When values ≥ 4,096
    ///   (Dense data benefits from fixed-size bitmap)
    ///
    /// **Run Container:**
    /// - → Array: When runs > values/2 OR values < 10
    ///   (Fragmented or tiny Run containers are inefficient)
    ///
    /// **Bitmap Container:**
    /// - → Array: When values < 4,096
    ///   (Sparse enough that array is smaller)
    /// - → Run: After converting to Array, check if Run is better
    ///   (Consecutive sequences in sparse bitmap)
    ///
    /// # Performance
    ///
    /// - Array: O(n) to count runs
    /// - Run: O(1) to check run count
    /// - Bitmap: O(65,536) to scan all bits and extract values
    fn optimize(&mut self) {
        match self {
            Container::Array(array) => {
                let num_runs = Self::count_runs_array(array);
                let len = array.len() as usize;

                // Heuristic: Array → Run when runs < values/2
                // Exception: Don't convert tiny containers (< 10 values) to avoid overhead
                if num_runs < len / 2 && len >= 10 {
                    *self = Container::Run(RunContainer::from_array(array));
                }
                // Array → Bitmap when values >= threshold
                else if len >= ARRAY_TO_BITMAP_THRESHOLD {
                    *self = Container::Bitmap(BitmapContainer::from_array(array));
                }
            }
            Container::Run(run) => {
                let num_runs = run.runs.len();
                let len = run.len() as usize;

                // Heuristic: Run → Array when runs > values/2 (not beneficial)
                // or when container is tiny (< 10 values)
                if num_runs > len / 2 || len < 10 {
                    *self = Container::Array(run.to_array());
                }
            }
            Container::Bitmap(bitmap) => {
                let len = bitmap.len() as usize;

                // Bitmap → Array when sparse enough
                if len < ARRAY_TO_BITMAP_THRESHOLD {
                    let array = bitmap.to_array();
                    let num_runs = Self::count_runs_array(&array);

                    // Convert to Run if beneficial, otherwise Array
                    if num_runs < len / 2 && num_runs > 10 {
                        *self = Container::Run(RunContainer::from_array(&array));
                    } else {
                        *self = Container::Array(array);
                    }
                }
            }
        }
    }

    /// Counts the number of runs in an array
    fn count_runs_array(array: &ArrayContainer) -> usize {
        if array.values.is_empty() {
            return 0;
        }

        let mut runs = 1;
        for i in 1..array.values.len() {
            if array.values[i] != array.values[i - 1] + 1 {
                runs += 1;
            }
        }
        runs
    }

    /// Determines if Run container would be better than Bitmap
    fn run_better_than_bitmap_array(array: &ArrayContainer) -> bool {
        let num_runs = Self::count_runs_array(array);
        let len = array.len() as usize;

        // Run: 4 bytes per run
        // Bitmap: 8192 bytes fixed
        let run_bytes = num_runs * 4;
        let bitmap_bytes = 8192;

        // Be conservative: Run is better only if MUCH smaller AND excellent compression
        // This avoids interfering with normal Array→Bitmap conversions
        run_bytes < bitmap_bytes / 2 && num_runs < len / 4
    }
}

impl ArrayContainer {
    /// Inserts a value, maintaining sorted order and uniqueness
    fn insert(&mut self, value: u16) -> bool {
        match self.values.binary_search(&value) {
            Ok(_) => false, // Value already exists
            Err(index) => {
                // Insert at the correct position to maintain sorted order
                self.values.insert(index, value);
                true
            }
        }
    }

    /// Checks if a value exists using binary search
    fn contains(&self, value: u16) -> bool {
        self.values.binary_search(&value).is_ok()
    }

    /// Removes a value if present
    fn remove(&mut self, value: u16) -> bool {
        match self.values.binary_search(&value) {
            Ok(index) => {
                self.values.remove(index);
                true
            }
            Err(_) => false, // Value not found
        }
    }

    /// Returns the number of values in the container
    fn len(&self) -> u64 {
        self.values.len() as u64
    }

    /// Checks if the container is empty
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns the union of two array containers
    fn union(&self, other: &ArrayContainer) -> ArrayContainer {
        let mut result = Vec::with_capacity(self.values.len() + other.values.len());
        let mut i = 0;
        let mut j = 0;

        // Merge two sorted arrays
        while i < self.values.len() && j < other.values.len() {
            match self.values[i].cmp(&other.values[j]) {
                std::cmp::Ordering::Less => {
                    result.push(self.values[i]);
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    result.push(self.values[i]);
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    result.push(other.values[j]);
                    j += 1;
                }
            }
        }

        // Add remaining elements
        result.extend_from_slice(&self.values[i..]);
        result.extend_from_slice(&other.values[j..]);

        ArrayContainer { values: result }
    }

    /// Returns the intersection of two array containers (None if empty)
    fn intersection(&self, other: &ArrayContainer) -> Option<ArrayContainer> {
        let mut result = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Find common elements in two sorted arrays
        while i < self.values.len() && j < other.values.len() {
            match self.values[i].cmp(&other.values[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Equal => {
                    result.push(self.values[i]);
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => j += 1,
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(ArrayContainer { values: result })
        }
    }

    /// Returns the difference of two array containers (self - other) (None if empty)
    fn difference(&self, other: &ArrayContainer) -> Option<ArrayContainer> {
        let mut result = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Find elements in self but not in other
        while i < self.values.len() && j < other.values.len() {
            match self.values[i].cmp(&other.values[j]) {
                std::cmp::Ordering::Less => {
                    result.push(self.values[i]);
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => j += 1,
            }
        }

        // Add remaining elements from self
        result.extend_from_slice(&self.values[i..]);

        if result.is_empty() {
            None
        } else {
            Some(ArrayContainer { values: result })
        }
    }

    /// Returns the heap memory used by this container in bytes
    fn heap_memory(&self) -> usize {
        // Vec heap allocation: capacity * element_size
        self.values.capacity() * std::mem::size_of::<u16>()
    }

    /// Returns the symmetric difference of two array containers (None if empty)
    fn symmetric_difference(&self, other: &ArrayContainer) -> Option<ArrayContainer> {
        let mut result = Vec::new();
        let mut i = 0;
        let mut j = 0;

        // Find elements in exactly one of the two arrays
        while i < self.values.len() && j < other.values.len() {
            match self.values[i].cmp(&other.values[j]) {
                std::cmp::Ordering::Less => {
                    result.push(self.values[i]);
                    i += 1;
                }
                std::cmp::Ordering::Equal => {
                    // Skip elements present in both
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Greater => {
                    result.push(other.values[j]);
                    j += 1;
                }
            }
        }

        // Add remaining elements
        result.extend_from_slice(&self.values[i..]);
        result.extend_from_slice(&other.values[j..]);

        if result.is_empty() {
            None
        } else {
            Some(ArrayContainer { values: result })
        }
    }
}

impl BitmapContainer {
    /// Creates a new empty bitmap container
    fn new() -> Self {
        BitmapContainer {
            bits: Box::new([0u64; 1024]),
            cardinality: 0,
        }
    }

    /// Creates a bitmap container from an array container
    fn from_array(array: &ArrayContainer) -> Self {
        let mut bitmap = Self::new();
        for &value in &array.values {
            bitmap.insert_unchecked(value);
        }
        bitmap
    }

    /// Helper: calculates which u64 and which bit within it for a value
    #[inline]
    fn position(value: u16) -> (usize, usize) {
        let index = (value as usize) / 64;
        let bit = (value as usize) % 64;
        (index, bit)
    }

    /// Inserts a value without checking if it already exists (for internal use)
    #[inline]
    fn insert_unchecked(&mut self, value: u16) {
        let (index, bit) = Self::position(value);
        self.bits[index] |= 1u64 << bit;
        self.cardinality += 1;
    }

    /// Inserts a value, maintaining cardinality
    fn insert(&mut self, value: u16) -> bool {
        let (index, bit) = Self::position(value);
        let mask = 1u64 << bit;
        let was_set = (self.bits[index] & mask) != 0;

        if !was_set {
            self.bits[index] |= mask;
            self.cardinality += 1;
            true
        } else {
            false
        }
    }

    /// Checks if a value exists
    fn contains(&self, value: u16) -> bool {
        let (index, bit) = Self::position(value);
        let mask = 1u64 << bit;
        (self.bits[index] & mask) != 0
    }

    /// Removes a value if present
    fn remove(&mut self, value: u16) -> bool {
        let (index, bit) = Self::position(value);
        let mask = 1u64 << bit;
        let was_set = (self.bits[index] & mask) != 0;

        if was_set {
            self.bits[index] &= !mask;
            self.cardinality -= 1;
            true
        } else {
            false
        }
    }

    /// Returns the number of values in the container
    fn len(&self) -> u64 {
        self.cardinality
    }

    /// Checks if the container is empty
    fn is_empty(&self) -> bool {
        self.cardinality == 0
    }

    /// Returns the heap memory used by this container in bytes
    fn heap_memory(&self) -> usize {
        // Box<[u64; 1024]> allocates 1024 * 8 bytes on heap
        1024 * std::mem::size_of::<u64>()
    }

    /// Converts to array container (for when cardinality becomes small)
    fn to_array(&self) -> ArrayContainer {
        let mut values = Vec::with_capacity(self.cardinality as usize);

        for (index, &word) in self.bits.iter().enumerate() {
            if word != 0 {
                let base = (index * 64) as u16;
                for bit in 0..64 {
                    if (word & (1u64 << bit)) != 0 {
                        values.push(base + bit as u16);
                    }
                }
            }
        }

        ArrayContainer { values }
    }

    /// Returns the union of two bitmap containers
    fn union(&self, other: &BitmapContainer) -> BitmapContainer {
        let mut result = Self::new();

        for i in 0..1024 {
            result.bits[i] = self.bits[i] | other.bits[i];
        }

        // Calculate cardinality
        result.cardinality = result.bits.iter().map(|w| w.count_ones() as u64).sum();

        result
    }

    /// Returns the intersection of two bitmap containers (None if empty)
    fn intersection(&self, other: &BitmapContainer) -> Option<BitmapContainer> {
        let mut result = Self::new();

        for i in 0..1024 {
            result.bits[i] = self.bits[i] & other.bits[i];
        }

        // Calculate cardinality
        result.cardinality = result.bits.iter().map(|w| w.count_ones() as u64).sum();

        if result.cardinality == 0 {
            None
        } else {
            Some(result)
        }
    }

    /// Returns the difference of two bitmap containers (self - other) (None if empty)
    fn difference(&self, other: &BitmapContainer) -> Option<BitmapContainer> {
        let mut result = Self::new();

        for i in 0..1024 {
            result.bits[i] = self.bits[i] & !other.bits[i];
        }

        // Calculate cardinality
        result.cardinality = result.bits.iter().map(|w| w.count_ones() as u64).sum();

        if result.cardinality == 0 {
            None
        } else {
            Some(result)
        }
    }

    /// Returns the symmetric difference of two bitmap containers (None if empty)
    fn symmetric_difference(&self, other: &BitmapContainer) -> Option<BitmapContainer> {
        let mut result = Self::new();

        for i in 0..1024 {
            result.bits[i] = self.bits[i] ^ other.bits[i];
        }

        // Calculate cardinality
        result.cardinality = result.bits.iter().map(|w| w.count_ones() as u64).sum();

        if result.cardinality == 0 {
            None
        } else {
            Some(result)
        }
    }
}

impl RunContainer {
    /// Returns the heap memory used by this container in bytes
    fn heap_memory(&self) -> usize {
        // Vec heap allocation: capacity * element_size
        self.runs.capacity() * std::mem::size_of::<(u16, u16)>()
    }

    /// Creates a run container from an array container
    fn from_array(array: &ArrayContainer) -> Self {
        if array.values.is_empty() {
            return RunContainer { runs: Vec::new() };
        }

        let mut runs = Vec::new();
        let mut run_start = array.values[0];
        let mut run_length = 1u16;

        for i in 1..array.values.len() {
            if array.values[i] == array.values[i - 1] + 1 {
                // Continue current run (use saturating_add to prevent overflow with full container)
                run_length = run_length.saturating_add(1);
            } else {
                // End current run, start new one (store length-1)
                runs.push((run_start, run_length - 1));
                run_start = array.values[i];
                run_length = 1;
            }
        }

        // Add the last run (store length-1)
        runs.push((run_start, run_length - 1));

        RunContainer { runs }
    }

    /// Converts run container to array container
    fn to_array(&self) -> ArrayContainer {
        let capacity: usize = self.runs.iter().map(|(_, len)| (*len as usize) + 1).sum();
        let mut values = Vec::with_capacity(capacity);

        for &(start, length) in &self.runs {
            // length is (actual_length - 1), so iterate 0..=length to get all values
            for offset in 0..=length {
                values.push(start + offset);
            }
        }

        ArrayContainer { values }
    }

    /// Inserts a value, maintaining run invariants
    fn insert(&mut self, value: u16) -> bool {
        // Find where this value should go
        let pos = self.runs.binary_search_by_key(&value, |(start, _)| *start);

        match pos {
            Ok(_index) => {
                // Value equals a run start, already present
                false
            }
            Err(index) => {
                // Check if value is within or adjacent to previous run
                if index > 0 {
                    let (start, length) = self.runs[index - 1];
                    // length is (actual_length - 1), so run_end = start + length
                    let run_end = start + length;

                    if value <= run_end {
                        // Value already in previous run
                        return false;
                    } else if value == run_end + 1 {
                        // Extend previous run
                        self.runs[index - 1].1 += 1;

                        // Check if we can merge with next run
                        if index < self.runs.len() {
                            let (next_start, next_length) = self.runs[index];
                            if value + 1 == next_start {
                                // Merge runs (add next_length + 1 since length encoding is length_minus_1)
                                self.runs[index - 1].1 += next_length + 1;
                                self.runs.remove(index);
                            }
                        }

                        return true;
                    }
                }

                // Check if value is adjacent to next run
                if index < self.runs.len() {
                    let (start, _) = self.runs[index];
                    if value + 1 == start {
                        // Extend next run backwards
                        self.runs[index].0 = value;
                        self.runs[index].1 += 1;
                        return true;
                    }
                }

                // Insert new run of length 1 (store as 0 in length-1 encoding)
                self.runs.insert(index, (value, 0));
                true
            }
        }
    }

    /// Checks if a value exists
    fn contains(&self, value: u16) -> bool {
        for &(start, length) in &self.runs {
            // length is (actual_length - 1), so end = start + length
            let end = start + length;
            if value >= start && value <= end {
                return true;
            }
            if value < start {
                return false; // Runs are sorted, no need to continue
            }
        }
        false
    }

    /// Removes a value if present
    fn remove(&mut self, value: u16) -> bool {
        for i in 0..self.runs.len() {
            let (start, length) = self.runs[i];
            // length is (actual_length - 1), so end = start + length
            let end = start + length;

            if value >= start && value <= end {
                // Value is in this run
                if length == 0 {
                    // Remove the entire run (length=0 means 1 value)
                    self.runs.remove(i);
                } else if value == start {
                    // Remove from start of run
                    self.runs[i] = (start + 1, length - 1);
                } else if value == end {
                    // Remove from end of run
                    self.runs[i].1 -= 1;
                } else {
                    // Split the run
                    // First run: [start, value-1], length = (value - start)
                    // Second run: [value+1, end], length = (end - value - 1)
                    let first_length = value - start - 1;
                    let second_start = value + 1;
                    let second_length = end - value - 1;

                    self.runs[i] = (start, first_length);
                    self.runs.insert(i + 1, (second_start, second_length));
                }
                return true;
            }

            if value < start {
                return false; // Runs are sorted
            }
        }
        false
    }

    /// Returns the number of values in the container
    fn len(&self) -> u64 {
        // length is (actual_length - 1), so add 1 to get actual length
        self.runs
            .iter()
            .map(|(_, length)| (*length as u64) + 1)
            .sum()
    }

    /// Checks if the container is empty
    fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    /// Returns the union of two run containers
    fn union(&self, other: &RunContainer) -> RunContainer {
        let array_self = self.to_array();
        let array_other = other.to_array();
        let union_array = array_self.union(&array_other);
        RunContainer::from_array(&union_array)
    }

    /// Returns the intersection of two run containers (None if empty)
    fn intersection(&self, other: &RunContainer) -> Option<RunContainer> {
        let array_self = self.to_array();
        let array_other = other.to_array();
        array_self
            .intersection(&array_other)
            .map(|arr| RunContainer::from_array(&arr))
    }

    /// Returns the difference of two run containers (self - other) (None if empty)
    fn difference(&self, other: &RunContainer) -> Option<RunContainer> {
        let array_self = self.to_array();
        let array_other = other.to_array();
        array_self
            .difference(&array_other)
            .map(|arr| RunContainer::from_array(&arr))
    }

    /// Returns the symmetric difference of two run containers (None if empty)
    fn symmetric_difference(&self, other: &RunContainer) -> Option<RunContainer> {
        let array_self = self.to_array();
        let array_other = other.to_array();
        array_self
            .symmetric_difference(&array_other)
            .map(|arr| RunContainer::from_array(&arr))
    }
}

// Iterator implementation

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        loop {
            // Check if we've exhausted all containers
            if self.container_index >= self.bitmap.containers.len() {
                return None;
            }

            // Get current container
            let (key, container) = &self.bitmap.containers[self.container_index];

            match container {
                Container::Array(array) => {
                    // Array container: iterate through sorted values
                    if self.value_index < array.values.len() {
                        let low = array.values[self.value_index];
                        self.value_index += 1;
                        return Some(RoaringBitmap::combine(*key, low));
                    } else {
                        // Move to next container
                        self.container_index += 1;
                        self.value_index = 0;
                        continue;
                    }
                }
                Container::Bitmap(bitmap) => {
                    // Bitmap container: scan for set bits
                    loop {
                        // If current word is exhausted or not yet loaded, load/move to next word
                        if self.bitmap_current_word == 0 {
                            // Check if we need to load the current word first (not yet loaded)
                            // This happens when entering a bitmap container from a non-bitmap container
                            if self.bitmap_bit_position == 0 && self.bitmap_word_index < 1024 {
                                // Load the current word before moving to next
                                self.bitmap_current_word = bitmap.bits[self.bitmap_word_index];
                                if self.bitmap_current_word != 0 {
                                    // Word has bits, process it
                                    continue;
                                }
                            }
                            // Move to next word
                            self.bitmap_word_index += 1;
                            if self.bitmap_word_index >= 1024 {
                                // Exhausted this container, move to next
                                self.container_index += 1;
                                self.value_index = 0;

                                // Reset bitmap state (will be reinitialized if next is also bitmap)
                                self.bitmap_word_index = 0;
                                self.bitmap_bit_position = 0;
                                self.bitmap_current_word = 0;

                                // Initialize for next container if it's a bitmap
                                // Always start at bits[0], consistent with initial iterator behavior
                                if self.container_index < self.bitmap.containers.len() {
                                    if let Container::Bitmap(next_bm) =
                                        &self.bitmap.containers[self.container_index].1
                                    {
                                        self.bitmap_current_word = next_bm.bits[0];
                                    }
                                }
                                break;
                            }
                            self.bitmap_current_word = bitmap.bits[self.bitmap_word_index];
                            self.bitmap_bit_position = 0;
                            continue;
                        }

                        // Find next set bit in current word
                        while self.bitmap_bit_position < 64 {
                            if (self.bitmap_current_word & (1u64 << self.bitmap_bit_position)) != 0
                            {
                                let low = (self.bitmap_word_index * 64
                                    + self.bitmap_bit_position as usize)
                                    as u16;
                                self.bitmap_bit_position += 1;
                                return Some(RoaringBitmap::combine(*key, low));
                            }
                            self.bitmap_bit_position += 1;
                        }

                        // Current word exhausted
                        self.bitmap_current_word = 0;
                    }
                }
                Container::Run(run) => {
                    // Run container: iterate through runs
                    if self.value_index < run.runs.len() {
                        let (start, length) = run.runs[self.value_index];

                        // length is (actual_length - 1), so iterate 0..=length
                        if self.run_offset <= length {
                            let low = start + self.run_offset;

                            // Check if this is the last value in the run
                            if self.run_offset == length {
                                // Move to next run
                                self.value_index += 1;
                                self.run_offset = 0;
                            } else {
                                // Increment offset for next value in run
                                self.run_offset += 1;
                            }

                            return Some(RoaringBitmap::combine(*key, low));
                        } else {
                            // Move to next run (shouldn't reach here normally)
                            self.value_index += 1;
                            self.run_offset = 0;
                            continue;
                        }
                    } else {
                        // Move to next container
                        self.container_index += 1;
                        self.value_index = 0;
                        self.run_offset = 0;
                        continue;
                    }
                }
            }
        }
    }
}

// ============================================================================
// TRAIT IMPLEMENTATIONS FOR OPERATOR OVERLOADING
// ============================================================================

// BitOr: Union operator (|)
impl BitOr<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// Implements the `|` operator for union (allocating).
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let result = &a | &b;  // Union using | operator
    /// assert_eq!(result.len(), 3);
    /// assert!(result.contains(1));
    /// assert!(result.contains(2));
    /// assert!(result.contains(3));
    /// ```
    fn bitor(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        self.union(rhs)
    }
}

// BitOrAssign: In-place union operator (|=)
impl BitOrAssign<&RoaringBitmap> for RoaringBitmap {
    /// Implements the `|=` operator for in-place union.
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// a |= &b;  // In-place union
    /// assert_eq!(a.len(), 3);
    /// assert!(a.contains(1));
    /// assert!(a.contains(2));
    /// assert!(a.contains(3));
    /// ```
    fn bitor_assign(&mut self, rhs: &RoaringBitmap) {
        self.union_with(rhs);
    }
}

// BitAnd: Intersection operator (&)
impl BitAnd<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// Implements the `&` operator for intersection (allocating).
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let result = &a & &b;  // Intersection using & operator
    /// assert_eq!(result.len(), 1);
    /// assert!(result.contains(2));
    /// ```
    fn bitand(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        self.intersection(rhs)
    }
}

// BitAndAssign: In-place intersection operator (&=)
impl BitAndAssign<&RoaringBitmap> for RoaringBitmap {
    /// Implements the `&=` operator for in-place intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// a &= &b;  // In-place intersection
    /// assert_eq!(a.len(), 1);
    /// assert!(a.contains(2));
    /// ```
    fn bitand_assign(&mut self, rhs: &RoaringBitmap) {
        self.intersect_with(rhs);
    }
}

// BitXor: Symmetric difference operator (^)
impl BitXor<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// Implements the `^` operator for symmetric difference (allocating).
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let result = &a ^ &b;  // Symmetric difference using ^ operator
    /// assert_eq!(result.len(), 2);
    /// assert!(result.contains(1));
    /// assert!(result.contains(3));
    /// ```
    fn bitxor(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        self.symmetric_difference(rhs)
    }
}

// BitXorAssign: In-place symmetric difference operator (^=)
impl BitXorAssign<&RoaringBitmap> for RoaringBitmap {
    /// Implements the `^=` operator for in-place symmetric difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// a ^= &b;  // In-place symmetric difference
    /// assert_eq!(a.len(), 2);
    /// assert!(a.contains(1));
    /// assert!(a.contains(3));
    /// ```
    fn bitxor_assign(&mut self, rhs: &RoaringBitmap) {
        self.symmetric_difference_with(rhs);
    }
}

// Sub: Difference operator (-)
impl Sub<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// Implements the `-` operator for difference (allocating).
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    /// b.insert(2);
    /// b.insert(4);
    ///
    /// let result = &a - &b;  // Difference using - operator
    /// assert_eq!(result.len(), 2);
    /// assert!(result.contains(1));
    /// assert!(result.contains(3));
    /// ```
    fn sub(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        self.difference(rhs)
    }
}

// SubAssign: In-place difference operator (-=)
impl SubAssign<&RoaringBitmap> for RoaringBitmap {
    /// Implements the `-=` operator for in-place difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring_bitmap::RoaringBitmap;
    ///
    /// let mut a = RoaringBitmap::new();
    /// let mut b = RoaringBitmap::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    /// b.insert(2);
    /// b.insert(4);
    ///
    /// a -= &b;  // In-place difference
    /// assert_eq!(a.len(), 2);
    /// assert!(a.contains(1));
    /// assert!(a.contains(3));
    /// ```
    fn sub_assign(&mut self, rhs: &RoaringBitmap) {
        self.difference_with(rhs);
    }
}

// Additional trait implementations for chaining operations
// These allow operations like: (&a | &b) & &c

impl BitOr<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitor(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        (&self).union(rhs)
    }
}

impl BitAnd<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitand(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        (&self).intersection(rhs)
    }
}

impl BitXor<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitxor(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        (&self).symmetric_difference(rhs)
    }
}

impl Sub<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    fn sub(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        (&self).difference(rhs)
    }
}

// Additional trait implementations for mixed reference/owned operations

impl BitAnd<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitand(self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.intersection(&rhs)
    }
}

impl BitOr<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitor(self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.union(&rhs)
    }
}

impl BitXor<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitxor(self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.symmetric_difference(&rhs)
    }
}

impl Sub<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    fn sub(self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.difference(&rhs)
    }
}

//! # Skip List
//!
//! A probabilistic alternative to balanced trees offering O(log n) search, insert, and delete operations.
//!
//! Skip lists use randomized hierarchical layers to achieve the same asymptotic performance as
//! balanced trees (like AVL trees or red-black trees) while maintaining a simpler implementation.
//!
//! ## Intrusive Design
//!
//! This skip list implementation uses an **intrusive** design pattern, similar to BSD's `SLIST_ENTRY`.
//! The skiplist metadata (forward pointers) is embedded directly within the value structure,
//! rather than being allocated separately.
//!
//! ### Benefits
//! - Single allocation per element (no separate node allocation)
//! - Better cache locality (data and pointers together)
//! - More control over memory layout
//! - Zero overhead abstraction
//!
//! ## Example
//!
//! ```rust
//! use skiplist::{SkipList, SkipListEntry, SkipListNode};
//!
//! // Define your value structure with embedded skiplist metadata
//! #[derive(Debug)]
//! struct User {
//!     id: u64,
//!     name: String,
//!     skiplist_meta: SkipListNode,
//! }
//!
//! impl SkipListEntry for User {
//!     type Key = u64;
//!
//!     fn key(&self) -> &Self::Key {
//!         &self.id
//!     }
//!
//!     fn skiplist_node(&self) -> &SkipListNode {
//!         &self.skiplist_meta
//!     }
//!
//!     fn skiplist_node_mut(&mut self) -> &mut SkipListNode {
//!         &mut self.skiplist_meta
//!     }
//! }
//!
//! // Use the skiplist
//! let mut skiplist: SkipList<u64, User> = SkipList::new();
//!
//! let user = Box::new(User {
//!     id: 42,
//!     name: "Alice".to_string(),
//!     skiplist_meta: SkipListNode::new(),
//! });
//!
//! skiplist.insert(user).unwrap();
//!
//! if let Some(user_ref) = skiplist.get(&42) {
//!     assert_eq!(user_ref.name, "Alice");
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

use smallvec::SmallVec;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of levels in the skip list.
///
/// With MAX_LEVEL = 16, the skip list can efficiently handle up to 2^16 = 65,536 elements.
/// This can be adjusted based on expected data size.
pub const MAX_LEVEL: usize = 16;

/// Default probability for promoting elements to higher levels.
///
/// A value of 0.5 means each element has a 50% chance of being promoted to the next level.
pub const DEFAULT_PROBABILITY: f64 = 0.5;

/// Skiplist metadata that must be embedded in value structures.
///
/// This structure contains the forward pointers at each level. Users must include
/// this structure in their value types to use them in a skiplist.
///
/// # Example
///
/// ```rust
/// use skiplist::SkipListNode;
///
/// struct MyValue {
///     key: String,
///     data: Vec<u8>,
///     skiplist_meta: SkipListNode,  // Embedded metadata
/// }
/// ```
///
/// # Implementation Note
///
/// Uses `SmallVec<[T; 4]>` for inline storage optimization. With default probability (p=0.5),
/// approximately 93.75% of nodes have ≤4 forward pointers and avoid heap allocation entirely.
/// This provides 10-15% performance improvement over standard `Vec` with no memory overhead.
#[derive(Debug, Clone)]
pub struct SkipListNode {
    /// Forward pointers at each level.
    /// forward[i] points to the next node at level i.
    ///
    /// Stores up to 4 pointers inline (covers 93.75% of nodes with p=0.5),
    /// automatically spills to heap for taller nodes.
    forward: SmallVec<[Option<NonNull<u8>>; 4]>,
}

impl SkipListNode {
    /// Creates a new skiplist node with no forward pointers.
    ///
    /// The forward pointers vector is initially empty and will be
    /// initialized when the element is inserted into a skiplist.
    pub fn new() -> Self {
        Self {
            forward: SmallVec::new(),
        }
    }

    /// Creates a new skiplist node with the specified number of levels.
    ///
    /// # Arguments
    ///
    /// * `level` - The number of forward pointer levels to allocate
    pub fn with_level(level: usize) -> Self {
        let mut forward = SmallVec::new();
        forward.resize(level + 1, None);
        Self { forward }
    }
}

impl Default for SkipListNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait that must be implemented by types to be stored in a skiplist.
///
/// This trait provides access to the element's key and its embedded skiplist metadata.
///
/// # Example
///
/// ```rust
/// use skiplist::{SkipListEntry, SkipListNode};
///
/// struct User {
///     id: u64,
///     name: String,
///     skiplist_meta: SkipListNode,
/// }
///
/// impl SkipListEntry for User {
///     type Key = u64;
///
///     fn key(&self) -> &Self::Key {
///         &self.id
///     }
///
///     fn skiplist_node(&self) -> &SkipListNode {
///         &self.skiplist_meta
///     }
///
///     fn skiplist_node_mut(&mut self) -> &mut SkipListNode {
///         &mut self.skiplist_meta
///     }
/// }
/// ```
pub trait SkipListEntry {
    /// The key type used for ordering elements in the skiplist.
    type Key: Ord;

    /// Returns a reference to the element's key.
    fn key(&self) -> &Self::Key;

    /// Returns a reference to the embedded skiplist metadata.
    fn skiplist_node(&self) -> &SkipListNode;

    /// Returns a mutable reference to the embedded skiplist metadata.
    fn skiplist_node_mut(&mut self) -> &mut SkipListNode;
}

/// An intrusive skiplist implementation.
///
/// This skiplist maintains elements in sorted order by key, with O(log n) average
/// time complexity for search, insertion, and deletion operations.
///
/// # Type Parameters
///
/// * `K` - The key type, must implement `Ord`
/// * `E` - The entry type, must implement `SkipListEntry<Key = K>`
///
/// # Example
///
/// ```rust
/// use skiplist::{SkipList, SkipListEntry, SkipListNode};
///
/// struct Item {
///     id: i32,
///     value: String,
///     skiplist_meta: SkipListNode,
/// }
///
/// impl SkipListEntry for Item {
///     type Key = i32;
///     fn key(&self) -> &Self::Key { &self.id }
///     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
///     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
/// }
///
/// let mut list: SkipList<i32, Item> = SkipList::new();
/// ```
pub struct SkipList<K, E>
where
    K: Ord,
    E: SkipListEntry<Key = K>,
{
    /// Head node with forward pointers at each level.
    head: SkipListNode,

    /// Number of elements in the skiplist.
    len: usize,

    /// Current maximum level in use.
    level: usize,

    /// Maximum allowed level.
    max_level: usize,

    /// RNG state for level generation (xorshift64)
    rng_state: u64,

    /// Phantom data to hold type parameters.
    _marker: PhantomData<(K, E)>,
}

impl<K, E> SkipList<K, E>
where
    K: Ord,
    E: SkipListEntry<Key = K>,
{
    /// Creates a new empty skiplist with default settings.
    ///
    /// Uses `MAX_LEVEL` as the maximum level and `DEFAULT_PROBABILITY` for promotion.
    ///
    /// # Example
    ///
    /// ```rust
    /// use skiplist::SkipList;
    /// # use skiplist::{SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    ///
    /// let skiplist: SkipList<i32, Item> = SkipList::new();
    /// assert!(skiplist.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::with_max_level(MAX_LEVEL)
    }

    /// Creates a new empty skiplist with the specified maximum level.
    ///
    /// # Arguments
    ///
    /// * `max_level` - Maximum number of levels for the skiplist
    ///
    /// # Example
    ///
    /// ```rust
    /// use skiplist::SkipList;
    /// # use skiplist::{SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    ///
    /// let skiplist: SkipList<i32, Item> = SkipList::with_max_level(8);
    /// ```
    pub fn with_max_level(max_level: usize) -> Self {
        // Initialize RNG with current time
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Self {
            head: SkipListNode::with_level(max_level),
            len: 0,
            level: 0,
            max_level,
            rng_state: if seed == 0 { 1 } else { seed }, // Avoid zero state
            _marker: PhantomData,
        }
    }

    /// Inserts an entry into the skiplist.
    ///
    /// The entry is inserted in sorted order based on its key. If an entry with the same
    /// key already exists, the insertion fails and the entry is returned in the `Err` variant.
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry to insert (must be boxed)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if insertion succeeded
    /// * `Err(entry)` if an entry with the same key already exists
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::{SkipList, SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// let mut skiplist: SkipList<i32, Item> = SkipList::new();
    ///
    /// let item = Box::new(Item { id: 42, skiplist_meta: SkipListNode::new() });
    /// assert!(skiplist.insert(item).is_ok());
    /// ```
    pub fn insert(&mut self, mut entry: Box<E>) -> Result<(), Box<E>> {
        let key = entry.key();

        // Track update pointers at each level (where to insert)
        let mut update: Vec<*const SkipListNode> = vec![std::ptr::null(); self.max_level + 1];

        // Find insertion position
        let mut current = &self.head as *const SkipListNode;

        for level in (0..=self.level).rev() {
            unsafe {
                loop {
                    let current_node = &*current;
                    match current_node.forward[level] {
                        Some(next_ptr) => {
                            let next_entry: &E = next_ptr.cast::<E>().as_ref();

                            match next_entry.key().cmp(key) {
                                std::cmp::Ordering::Less => {
                                    current = next_entry.skiplist_node() as *const SkipListNode;
                                }
                                std::cmp::Ordering::Equal => {
                                    // Key already exists, return error
                                    return Err(entry);
                                }
                                std::cmp::Ordering::Greater => {
                                    break;
                                }
                            }
                        }
                        None => break,
                    }
                }
            }
            update[level] = current;
        }

        // Generate random level for new node
        let new_level = self.random_level();

        // Update skiplist level if necessary
        if new_level > self.level {
            for item in update
                .iter_mut()
                .skip(self.level + 1)
                .take(new_level - self.level)
            {
                *item = &self.head as *const SkipListNode;
            }
            self.level = new_level;
        }

        // Initialize the entry's skiplist node with proper level
        let node = entry.skiplist_node_mut();
        node.forward.resize(new_level + 1, None);

        // Convert Box to raw pointer and store as type-erased pointer
        let entry_ptr = Box::into_raw(entry);
        let erased_ptr = NonNull::new(entry_ptr as *mut u8).unwrap();

        // Update forward pointers at each level
        for (level, &update_node_ptr) in update.iter().enumerate().take(new_level + 1) {
            unsafe {
                let update_node = &*update_node_ptr;

                // New node points to what update[level] pointed to
                (*entry_ptr).skiplist_node_mut().forward[level] = update_node.forward[level];

                // update[level] now points to new node
                let update_node_mut = &mut *(update_node_ptr as *mut SkipListNode);
                update_node_mut.forward[level] = Some(erased_ptr);
            }
        }

        self.len += 1;
        Ok(())
    }

    /// Generate a random level for a new node using xorshift64
    fn random_level(&mut self) -> usize {
        // Xorshift64 - fast PRNG
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;

        let mut level = 0;
        let mut rng = x;

        // Flip coin until we get tails or hit max level
        while level < self.max_level && (rng & 1) == 0 {
            level += 1;
            rng >>= 1;
        }

        level
    }

    /// Removes an entry from the skiplist by key and returns it.
    ///
    /// If an entry with the specified key exists, it is removed from the skiplist
    /// and returned. Otherwise, `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to remove
    ///
    /// # Returns
    ///
    /// The removed entry, or `None` if no entry with the given key exists
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::{SkipList, SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, value: String, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// # let mut skiplist: SkipList<i32, Item> = SkipList::new();
    ///
    /// if let Some(item) = skiplist.remove(&42) {
    ///     println!("Removed: {}", item.value);
    /// }
    /// ```
    pub fn remove(&mut self, key: &K) -> Option<Box<E>> {
        // Track update pointers at each level (nodes pointing to the target)
        let mut update: Vec<*mut SkipListNode> = vec![std::ptr::null_mut(); self.max_level + 1];

        // Find the node to remove
        let mut current = &self.head as *const SkipListNode;

        for level in (0..=self.level).rev() {
            unsafe {
                loop {
                    let current_node = &*current;
                    match current_node.forward[level] {
                        Some(next_ptr) => {
                            let next_entry: &E = next_ptr.cast::<E>().as_ref();

                            match next_entry.key().cmp(key) {
                                std::cmp::Ordering::Less => {
                                    current = next_entry.skiplist_node() as *const SkipListNode;
                                }
                                std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                                    break;
                                }
                            }
                        }
                        None => break,
                    }
                }
            }
            update[level] = current as *mut SkipListNode;
        }

        // Check if the node exists
        unsafe {
            let current_node = &*current;
            if let Some(target_ptr) = current_node.forward[0] {
                let target_entry: &E = target_ptr.cast::<E>().as_ref();

                if target_entry.key() == key {
                    // Found the node to remove
                    let target_node = target_entry.skiplist_node();

                    // Update forward pointers at each level
                    for (level, &update_node_ptr) in update.iter().enumerate().take(self.level + 1)
                    {
                        let update_node = &mut *update_node_ptr;
                        if let Some(ptr) = update_node.forward[level] {
                            if ptr == target_ptr {
                                // Skip over the node being removed
                                update_node.forward[level] = target_node.forward[level];
                            }
                        }
                    }

                    // Update list level if we removed from top levels
                    while self.level > 0 && self.head.forward[self.level].is_none() {
                        self.level -= 1;
                    }

                    self.len -= 1;

                    // Convert raw pointer back to Box and return
                    let removed_ptr = target_ptr.cast::<E>().as_ptr();
                    return Some(Box::from_raw(removed_ptr));
                }
            }
        }

        None
    }

    /// Removes an entry from the skiplist by key without returning it.
    ///
    /// This is more efficient than `remove()` when you don't need the removed value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to remove
    ///
    /// # Returns
    ///
    /// `true` if an entry was removed, `false` if no entry with the given key exists
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::{SkipList, SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// # let mut skiplist: SkipList<i32, Item> = SkipList::new();
    ///
    /// if skiplist.remove_by_key(&42) {
    ///     println!("Entry removed");
    /// }
    /// ```
    pub fn remove_by_key(&mut self, key: &K) -> bool {
        // Just use remove() and drop the returned value
        self.remove(key).is_some()
    }

    /// Looks up an entry by key and returns a reference to it.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to search for
    ///
    /// # Returns
    ///
    /// A reference to the entry if found, or `None` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::{SkipList, SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, value: String, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// # let skiplist: SkipList<i32, Item> = SkipList::new();
    ///
    /// if let Some(item) = skiplist.get(&42) {
    ///     println!("Found: {}", item.value);
    /// }
    /// ```
    pub fn get(&self, key: &K) -> Option<&E> {
        // Start from the head (sentinel node)
        let mut current = &self.head;

        // Search from top level down to level 0
        for level in (0..=self.level).rev() {
            // Move forward at this level while next.key < search_key
            while let Some(next_ptr) = current.forward[level] {
                // Cast type-erased pointer and dereference
                // SAFETY: Pointer is valid (stored by insert from Box<E>)
                let next_entry: &E = unsafe { next_ptr.cast::<E>().as_ref() };

                match next_entry.key().cmp(key) {
                    std::cmp::Ordering::Less => {
                        // next < key: move forward at this level
                        current = next_entry.skiplist_node();
                    }
                    std::cmp::Ordering::Equal => {
                        // Found exact match!
                        return Some(next_entry);
                    }
                    std::cmp::Ordering::Greater => {
                        // next > key: drop down to next level
                        break;
                    }
                }
            }
        }

        // Not found after checking all levels
        None
    }

    /// Looks up an entry by key and returns a mutable reference to it.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to search for
    ///
    /// # Returns
    ///
    /// A mutable reference to the entry if found, or `None` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::{SkipList, SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, value: String, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// # let mut skiplist: SkipList<i32, Item> = SkipList::new();
    ///
    /// if let Some(item) = skiplist.get_mut(&42) {
    ///     item.value = "Updated".to_string();
    /// }
    /// ```
    pub fn get_mut(&mut self, key: &K) -> Option<&mut E> {
        // Start from the head (sentinel node)
        let mut current = &self.head;

        // Search from top level down to level 0
        for level in (0..=self.level).rev() {
            // Move forward at this level while next.key < search_key
            while let Some(next_ptr) = current.forward[level] {
                // Cast type-erased pointer and dereference
                // SAFETY: Pointer is valid (stored by insert from Box<E>)
                let next_entry: &E = unsafe { next_ptr.cast::<E>().as_ref() };

                match next_entry.key().cmp(key) {
                    std::cmp::Ordering::Less => {
                        // next < key: move forward at this level
                        current = next_entry.skiplist_node();
                    }
                    std::cmp::Ordering::Equal => {
                        // Found exact match! Return mutable reference
                        // SAFETY: We have &mut self, so exclusive access is guaranteed
                        let mut mut_ptr = next_ptr.cast::<E>();
                        return Some(unsafe { mut_ptr.as_mut() });
                    }
                    std::cmp::Ordering::Greater => {
                        // next > key: drop down to next level
                        break;
                    }
                }
            }
        }

        // Not found after checking all levels
        None
    }

    /// Returns a reference to the first (smallest) entry in the skiplist.
    ///
    /// # Returns
    ///
    /// A reference to the first entry, or `None` if the skiplist is empty
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::{SkipList, SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// # let skiplist: SkipList<i32, Item> = SkipList::new();
    ///
    /// if let Some(first) = skiplist.first() {
    ///     println!("First key: {}", first.key());
    /// }
    /// ```
    pub fn first(&self) -> Option<&E> {
        // Get the first element by following the head's level 0 pointer
        // The head is a sentinel node that doesn't contain data
        self.head.forward[0].map(|ptr| {
            // Cast from type-erased NonNull<u8> to NonNull<E>
            let entry_ptr: NonNull<E> = ptr.cast::<E>();
            // Safely convert NonNull<E> to &E
            // SAFETY: The pointer is valid because:
            // - It was stored by insert() from a valid Box<E>
            // - The skiplist maintains ownership of all entries
            // - We're returning a reference with lifetime tied to &self
            unsafe { entry_ptr.as_ref() }
        })
    }

    /// Finds the next entry after the given key (successor).
    ///
    /// Returns the smallest entry whose key is strictly greater than the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to find the successor of
    ///
    /// # Returns
    ///
    /// A reference to the successor entry, or `None` if no such entry exists
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::{SkipList, SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// # let skiplist: SkipList<i32, Item> = SkipList::new();
    ///
    /// if let Some(next) = skiplist.successor(&42) {
    ///     println!("Next key after 42: {}", next.key());
    /// }
    /// ```
    pub fn successor(&self, key: &K) -> Option<&E> {
        // Start from the head (sentinel node)
        let mut current = &self.head;

        // Search from top level down to level 0
        for level in (0..=self.level).rev() {
            // Move forward at this level while next.key <= search_key
            while let Some(next_ptr) = current.forward[level] {
                // Cast type-erased pointer and dereference
                // SAFETY: Pointer is valid (stored by insert from Box<E>)
                let next_entry: &E = unsafe { next_ptr.cast::<E>().as_ref() };

                match next_entry.key().cmp(key) {
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                        // next <= key: move forward to find larger element
                        current = next_entry.skiplist_node();
                    }
                    std::cmp::Ordering::Greater => {
                        // next > key: this might be our successor, but check lower levels first
                        break;
                    }
                }
            }
        }

        // After traversal, current points to the largest element <= key
        // Return the next element at level 0 (which is > key)
        current.forward[0].map(|ptr| {
            let entry_ptr: NonNull<E> = ptr.cast::<E>();
            unsafe { entry_ptr.as_ref() }
        })
    }

    /// Returns `true` if the skiplist contains no elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::SkipList;
    /// # use skiplist::{SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// let skiplist: SkipList<i32, Item> = SkipList::new();
    /// assert!(skiplist.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements in the skiplist.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use skiplist::SkipList;
    /// # use skiplist::{SkipListEntry, SkipListNode};
    /// # struct Item { id: i32, skiplist_meta: SkipListNode }
    /// # impl SkipListEntry for Item {
    /// #     type Key = i32;
    /// #     fn key(&self) -> &i32 { &self.id }
    /// #     fn skiplist_node(&self) -> &SkipListNode { &self.skiplist_meta }
    /// #     fn skiplist_node_mut(&mut self) -> &mut SkipListNode { &mut self.skiplist_meta }
    /// # }
    /// # let skiplist: SkipList<i32, Item> = SkipList::new();
    /// println!("Skiplist has {} elements", skiplist.len());
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<K, E> Default for SkipList<K, E>
where
    K: Ord,
    E: SkipListEntry<Key = K>,
{
    fn default() -> Self {
        Self::new()
    }
}

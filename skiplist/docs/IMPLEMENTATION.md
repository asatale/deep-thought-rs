# Skip List Implementation Details

This document provides detailed explanations of the skiplist implementation, including data structure layout, algorithms, and design decisions.

## Table of Contents

- [Data Structure Layout](#data-structure-layout)
- [Type System](#type-system)
- [Function Implementations](#function-implementations)
  - [first()](#first---getting-the-first-element)
  - [get()](#get---searching-for-an-element)
  - [insert()](#insert---adding-elements)
  - [remove()](#remove---removing-and-returning-elements)
  - [remove_by_key()](#remove_by_key---removing-without-return)
  - [get_mut()](#get_mut---mutable-access)
  - [successor()](#successor---finding-next-element)

---

## Data Structure Layout

### SkipListNode Structure

```rust
pub struct SkipListNode {
    forward: Vec<Option<NonNull<u8>>>,
}
```

**Breaking down `Vec<Option<NonNull<u8>>>`:**

```
Vec<Option<NonNull<u8>>>
│   │      │       │
│   │      │       └─ u8: Type-erased pointer
│   │      └───────── NonNull: Non-null raw pointer
│   └──────────────── Option: May or may not point to another node
└──────────────────── Vec: Dynamic array, one entry per level
```

- **`Vec`**: Dynamic array where `forward[i]` is the pointer at level `i`
- **`Option`**: `Some(ptr)` if there's a next node, `None` if this is the last node at this level
- **`NonNull`**: Guaranteed non-null raw pointer (more efficient than Box/Rc)
- **`u8`**: Type erasure - allows `SkipListNode` to be embedded in any struct without generics

### Skip List Levels

```
Level 3:  Head ---------------------------------> Node(9) -> None
Level 2:  Head -------> Node(4) -------> Node(7) -> Node(9) -> None
Level 1:  Head -> Node(3) -> Node(4) -> Node(6) -> Node(7) -> Node(8) -> Node(9) -> None
Level 0:  Head -> Node(2) -> Node(3) -> Node(4) -> Node(5) -> Node(6) -> Node(7) -> Node(8) -> Node(9) -> None
          │
          └─ Sentinel node (no data, only forward pointers)
```

### Memory Layout - Intrusive Design

**User's structure:**
```rust
struct User {
    id: u64,
    name: String,
    skiplist_meta: SkipListNode,  // Embedded metadata
}
```

**In memory:**
```
Box<User> allocated on heap:
┌─────────────────────────┐
│ id: 42                  │
│ name: "Alice"           │
│ skiplist_meta:          │
│   forward[0]: Some(→)   │──┐
│   forward[1]: Some(→)   │  │
│   forward[2]: None      │  │
└─────────────────────────┘  │
                             │
                             ├─→ Next User at level 0
                             └─→ Next User at level 1
```

**Single allocation per entry** - data and skiplist metadata together!

### Type Erasure and Casting

```rust
// When storing (in insert):
let user_ptr: *mut User = Box::into_raw(user_box);
let erased_ptr: NonNull<u8> = NonNull::new(user_ptr as *mut u8).unwrap();
node.forward[i] = Some(erased_ptr);

// When retrieving (in first, get, etc):
let erased_ptr: NonNull<u8> = node.forward[i].unwrap();
let user_ptr: NonNull<User> = erased_ptr.cast::<User>();
let user_ref: &User = unsafe { user_ptr.as_ref() };
```

**Why this works:**
- Pointer address stays the same (just changes type annotation)
- No runtime cost for casting
- Allows `SkipListNode` to be non-generic

---

## Type System

### SkipList Structure

```rust
pub struct SkipList<K, E>
where
    K: Ord,
    E: SkipListEntry<Key = K>,
{
    head: SkipListNode,        // Sentinel node
    len: usize,                // Number of elements
    level: usize,              // Current max level in use
    max_level: usize,          // Maximum allowed level
    probability: f64,          // Level promotion probability (0.5)
    _marker: PhantomData<(K, E)>,
}
```

### SkipListEntry Trait

```rust
pub trait SkipListEntry {
    type Key: Ord;

    fn key(&self) -> &Self::Key;
    fn skiplist_node(&self) -> &SkipListNode;
    fn skiplist_node_mut(&mut self) -> &mut SkipListNode;
}
```

Users must implement this trait to provide:
1. Access to the key (for ordering)
2. Access to embedded skiplist metadata

---

## Function Implementations

### `first()` - Getting the First Element

#### Signature
```rust
pub fn first(&self) -> Option<&E>
```

#### Purpose
Returns a reference to the smallest element (first in sorted order) in the skiplist.

#### Implementation
```rust
pub fn first(&self) -> Option<&E> {
    self.head.forward[0].map(|ptr| {
        let entry_ptr: NonNull<E> = ptr.cast::<E>();
        unsafe { entry_ptr.as_ref() }
    })
}
```

#### Algorithm Steps

1. **Access head's level 0 pointer**: `self.head.forward[0]`
   - The head is a sentinel (dummy) node with no data
   - Level 0 is the base level containing all elements in sorted order
   - Type: `Option<NonNull<u8>>`

2. **Map the Option**: `.map(|ptr| { ... })`
   - If `Some(ptr)`: transform the pointer to a reference
   - If `None`: return `None` (empty list)

3. **Cast pointer**: `ptr.cast::<E>()`
   - Convert from type-erased `NonNull<u8>` to `NonNull<E>`
   - Just changes type annotation, zero runtime cost

4. **Dereference**: `unsafe { entry_ptr.as_ref() }`
   - Convert `NonNull<E>` to `&E`
   - Requires `unsafe` because we're dereferencing a raw pointer
   - **Safety invariants:**
     - Pointer came from valid `Box<E>` in `insert()`
     - Skiplist owns all entries (won't be freed while borrowed)
     - Reference lifetime tied to `&self`

#### Visualization

```
Empty list:
Head
  forward[0] → None     ← first() returns None

Non-empty list:
Head (sentinel)
  forward[0] → User{id:1, name:"Alice"}  ← first() returns Some(&this)
               forward[0] → User{id:5, name:"Bob"}
                            forward[0] → User{id:10, name:"Carol"}
                                         forward[0] → None
```

#### Time Complexity
- **O(1)** - Single pointer dereference

#### Example Usage
```rust
let list = skiplist_of(&[5, 2, 8, 1, 9]);

if let Some(first) = list.first() {
    assert_eq!(*first.key(), 1);  // Always the smallest key
}
```

---

### `get()` - Searching for an Element

#### Signature
```rust
pub fn get(&self, key: &K) -> Option<&E>
```

#### Purpose
Searches for an element by key and returns a reference to it if found.

#### Implementation
```rust
pub fn get(&self, key: &K) -> Option<&E> {
    // Start from the highest level
    let mut current = &self.head;

    // Traverse down through levels
    for level in (0..=self.level).rev() {
        // Move forward at this level while next.key < search_key
        while let Some(next_ptr) = current.forward[level] {
            let next_entry: &E = unsafe { next_ptr.cast::<E>().as_ref() };

            match next_entry.key().cmp(key) {
                std::cmp::Ordering::Less => {
                    // next < key: move forward
                    current = next_entry.skiplist_node();
                }
                std::cmp::Ordering::Equal => {
                    // Found it!
                    return Some(next_entry);
                }
                std::cmp::Ordering::Greater => {
                    // next > key: drop down to next level
                    break;
                }
            }
        }
    }

    // Not found
    None
}
```

#### Algorithm Steps

**Skip list search algorithm:**

1. **Start at top level**: Begin at head node, highest level
2. **For each level (top to bottom)**:
   - Move forward while `next.key < search_key`
   - If `next.key == search_key`: Found! Return it
   - If `next.key > search_key` or end of level: Drop down one level
3. **After checking all levels**: Not found, return `None`

#### Detailed Walkthrough

**Example: Search for key 7 in this skiplist**

```
Level 2:  Head -------> Node(4) -------> Node(9) -> None
Level 1:  Head -> Node(3) -> Node(4) -> Node(6) -> Node(9) -> None
Level 0:  Head -> Node(2) -> Node(3) -> Node(4) -> Node(5) -> Node(6) -> Node(7) -> Node(9) -> None
```

**Search path for key=7:**

```
Step 1: Level 2, current=Head
  - Check Node(4): 4 < 7, move forward to Node(4)
  - Check Node(9): 9 > 7, drop to Level 1

Step 2: Level 1, current=Node(4)
  - Check Node(6): 6 < 7, move forward to Node(6)
  - Check Node(9): 9 > 7, drop to Level 0

Step 3: Level 0, current=Node(6)
  - Check Node(7): 7 == 7, FOUND! Return Some(&Node(7))
```

**Search path visualization:**
```
Level 2:  Head ──→ Node(4) ──→ Node(9)
               ↓          ↓
Level 1:  ...  Node(4) ──→ Node(6) ──→ Node(9)
                             ↓
Level 0:  ...        Node(6) ──→ Node(7) ← Found!
```

#### Pointer Operations Explained

```rust
while let Some(next_ptr) = current.forward[level] {
    // next_ptr: NonNull<u8> (type-erased)

    let next_entry: &E = unsafe { next_ptr.cast::<E>().as_ref() };
    // 1. cast::<E>() - converts NonNull<u8> to NonNull<E>
    // 2. as_ref() - dereferences to get &E

    match next_entry.key().cmp(key) {
        Ordering::Less => {
            // Move current forward
            current = next_entry.skiplist_node();
        }
        Ordering::Equal => {
            return Some(next_entry);
        }
        Ordering::Greater => {
            break;  // Drop to next level
        }
    }
}
```

#### Time Complexity
- **Average: O(log n)**
- **Worst case: O(n)** (very rare due to probabilistic balancing)

**Why O(log n)?**
- Each level skips approximately half the elements
- Similar to binary search but on a linked structure

#### Example Usage
```rust
let list = skiplist_of(&[1, 5, 10, 15, 20]);

// Found
if let Some(item) = list.get(&10) {
    assert_eq!(*item.key(), 10);
    println!("Value: {}", item.value);
}

// Not found
assert!(list.get(&99).is_none());
```

---

### `get_mut()` - Mutable Access

#### Signature
```rust
pub fn get_mut(&mut self, key: &K) -> Option<&mut E>
```

#### Purpose
Searches for an element by key and returns a **mutable reference** to it if found.

#### Implementation
```rust
pub fn get_mut(&mut self, key: &K) -> Option<&mut E> {
    let mut current = &self.head;

    for level in (0..=self.level).rev() {
        while let Some(next_ptr) = current.forward[level] {
            let next_entry: &E = unsafe { next_ptr.cast::<E>().as_ref() };

            match next_entry.key().cmp(key) {
                std::cmp::Ordering::Less => {
                    current = next_entry.skiplist_node();
                }
                std::cmp::Ordering::Equal => {
                    // Found! Return mutable reference
                    let mut mut_ptr = next_ptr.cast::<E>();
                    return Some(unsafe { mut_ptr.as_mut() });
                }
                std::cmp::Ordering::Greater => {
                    break;
                }
            }
        }
    }

    None
}
```

#### Algorithm
**Identical to `get()`** - same search algorithm, different return type.

The only difference is in the return statement:
```rust
// get() returns immutable reference:
unsafe { next_ptr.cast::<E>().as_ref() }

// get_mut() returns mutable reference:
let mut mut_ptr = next_ptr.cast::<E>();
unsafe { mut_ptr.as_mut() }
```

#### Key Differences from `get()`

| Aspect | `get()` | `get_mut()` |
|--------|---------|-------------|
| Self parameter | `&self` | `&mut self` |
| Return type | `Option<&E>` | `Option<&mut E>` |
| Pointer conversion | `as_ref()` | `as_mut()` |
| Allows modification | No | Yes |

#### Safety Considerations

**Why `get_mut()` is safe:**

1. **Exclusive access**: `&mut self` guarantees no other references exist
2. **Valid pointer**: Pointer comes from valid `Box<E>` in `insert()`
3. **Lifetime**: Returned `&mut E` lifetime is tied to `&mut self`
4. **Key unchanged**: User can modify value, but not the key (key is accessed via immutable method `key()`)

**Important:** While users can modify the entry's data through `&mut E`, they **cannot modify the key** because `SkipListEntry::key()` returns `&Key` (immutable). This preserves skiplist ordering invariants.

#### Example Usage
```rust
let mut list = skiplist_of(&[1, 5, 10]);

// Modify value without changing key
if let Some(item) = list.get_mut(&5) {
    item.value = "modified".to_string();
    // item.key cannot be changed (immutable reference)
}

// Verify modification
assert_eq!(list.get(&5).unwrap().value, "modified");
```

#### Time Complexity
- **Average: O(log n)**
- **Worst case: O(n)**

Same as `get()` - just returns mutable reference.

---

### `successor()` - Finding Next Element

#### Signature
```rust
pub fn successor(&self, key: &K) -> Option<&E>
```

#### Purpose
Finds the **next element after the given key** - the smallest element whose key is **strictly greater** than the search key.

**Important distinction:**
- `get(key)` finds element with key **equal** to search key
- `successor(key)` finds element with key **greater than** search key

#### Implementation
```rust
pub fn successor(&self, key: &K) -> Option<&E> {
    let mut current = &self.head;

    for level in (0..=self.level).rev() {
        while let Some(next_ptr) = current.forward[level] {
            let next_entry: &E = unsafe { next_ptr.cast::<E>().as_ref() };

            match next_entry.key().cmp(key) {
                std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                    // next <= key: keep moving forward
                    current = next_entry.skiplist_node();
                }
                std::cmp::Ordering::Greater => {
                    // next > key: might be successor, check lower levels
                    break;
                }
            }
        }
    }

    // current now points to largest element <= key
    // Return next element at level 0 (which is > key)
    current.forward[0].map(|ptr| {
        let entry_ptr: NonNull<E> = ptr.cast::<E>();
        unsafe { entry_ptr.as_ref() }
    })
}
```

#### Algorithm Steps

1. **Search like `get()`, but keep going on equality**
   - In `get()`: Stop when `next == key`
   - In `successor()`: Continue when `next <= key`

2. **Find largest element ≤ key**
   - Move forward while `next <= key`
   - Drop level when `next > key`

3. **Return next element**
   - After traversal, `current` points to largest element ≤ key
   - Return `current.forward[0]` which is the next element

#### Detailed Example

**Skip list:**
```
Level 1:  Head -> 3 -> 6 -> 9 -> None
Level 0:  Head -> 2 -> 3 -> 5 -> 6 -> 7 -> 9 -> None
```

**Case 1: `successor(5)` - Key exists**
```
Search for 5:
  Head -> 3 (3≤5, move) -> 6 (6>5, drop) -> 5 (5≤5, move) -> 7

current = Node(5)
return current.forward[0] = Node(7) ✓
```

**Case 2: `successor(4)` - Key doesn't exist**
```
Search for 4:
  Head -> 3 (3≤4, move) -> 6 (6>4, drop) -> 5 (5>4, drop)

current = Node(3)
return current.forward[0] = Node(5) ✓
```

**Case 3: `successor(9)` - Last element**
```
Search for 9:
  Head -> 3 -> 6 -> 9 (9≤9, move) -> None

current = Node(9)
return current.forward[0] = None ✓
```

**Case 4: `successor(10)` - Larger than all**
```
Search for 10:
  Head -> 3 -> 6 -> 9 (9≤10, move) -> None

current = Node(9)
return current.forward[0] = None ✓
```

#### Key Difference: `Less | Equal` vs `Less`

```rust
// successor(): Keep going when equal
match next_entry.key().cmp(key) {
    Less | Equal => current = next_entry.skiplist_node(),
    Greater => break,
}

// get(): Stop when equal
match next_entry.key().cmp(key) {
    Less => current = next_entry.skiplist_node(),
    Equal => return Some(next_entry),
    Greater => break,
}
```

#### Use Cases

**Iteration:**
```rust
let mut current = list.first();
while let Some(item) = current {
    println!("{}", item.key());
    current = list.successor(item.key());
}
```

**Range queries:**
```rust
// Find all elements in range [start, end)
let mut current = list.get(&start).or_else(|| list.successor(&start));
while let Some(item) = current {
    if *item.key() >= end {
        break;
    }
    println!("{}", item.key());
    current = list.successor(item.key());
}
```

**Finding gaps:**
```rust
// Check if 5 exists, otherwise get next element
if let Some(item) = list.get(&5) {
    println!("Found: {}", item.key());
} else if let Some(next) = list.successor(&5) {
    println!("Not found, next is: {}", next.key());
}
```

#### Time Complexity
- **Average: O(log n)**
- **Worst case: O(n)**

Same as `get()` - same search algorithm.

---

### `insert()` - Adding Elements

#### Signature
```rust
pub fn insert(&mut self, entry: Box<E>) -> Result<(), Box<E>>
```

#### Purpose
Inserts a new element into the skiplist in sorted order. Returns an error if an element with the same key already exists.

#### Implementation Overview

**Insert algorithm has 4 main steps:**

1. **Find insertion position** - Search for where the key should go
2. **Generate random level** - Probabilistically determine node height
3. **Initialize node** - Set up forward pointers in the entry
4. **Update pointers** - Link the new node into the skiplist

#### Full Implementation
```rust
pub fn insert(&mut self, mut entry: Box<E>) -> Result<(), Box<E>> {
    let key = entry.key();

    // Step 1: Track update pointers at each level
    let mut update: Vec<*const SkipListNode> = vec![std::ptr::null(); self.max_level + 1];
    let mut current = &self.head as *const SkipListNode;

    // Find insertion position (like get(), but track all levels)
    for level in (0..=self.level).rev() {
        unsafe {
            loop {
                let current_node = &*current;
                match current_node.forward[level] {
                    Some(next_ptr) => {
                        let next_entry: &E = next_ptr.cast::<E>().as_ref();
                        match next_entry.key().cmp(key) {
                            Less => current = next_entry.skiplist_node() as *const _,
                            Equal => return Err(entry),  // Duplicate key
                            Greater => break,
                        }
                    }
                    None => break,
                }
            }
        }
        update[level] = current;
    }

    // Step 2: Generate random level for new node
    let new_level = self.random_level();

    // Update skiplist level if necessary
    if new_level > self.level {
        for level in (self.level + 1)..=new_level {
            update[level] = &self.head as *const SkipListNode;
        }
        self.level = new_level;
    }

    // Step 3: Initialize entry's skiplist node
    let node = entry.skiplist_node_mut();
    node.forward = vec![None; new_level + 1];

    // Step 4: Convert Box to raw pointer
    let entry_ptr = Box::into_raw(entry);
    let erased_ptr = NonNull::new(entry_ptr as *mut u8).unwrap();

    // Update forward pointers at each level
    for level in 0..=new_level {
        unsafe {
            let update_node_ptr = update[level];
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
```

#### Detailed Step-by-Step

**Step 1: Find Insertion Position**

Like `get()`, but we track the predecessor at **every level**:

```
Insert key=6 into:
Level 1:  Head -> 3 -> 9 -> None
Level 0:  Head -> 2 -> 3 -> 5 -> 9 -> None

Search path:
  Level 1: Head -> 3 (3<6, move) -> 9 (9>6, drop)
           update[1] = Node(3)

  Level 0: at Node(3) -> 5 (5<6, move) -> 9 (9>6, stop)
           update[0] = Node(5)

Result: Insert between Node(5) and Node(9)
```

**Step 2: Generate Random Level**

Probabilistic balancing using coin flips:

```rust
fn random_level(&self) -> usize {
    let mut level = 0;

    // Flip coin until tails or max level
    while level < self.max_level && (rng & 1) == 0 {
        level += 1;
        rng >>= 1;
    }

    level
}
```

- **p=0.5**: Each element has 50% chance of being promoted to next level
- **Expected height**: O(log n)
- **Keeps skiplist balanced** (on average)

**Level distribution:**
- ~50% of nodes at level 0 only
- ~25% of nodes reach level 1
- ~12.5% of nodes reach level 2
- ~6.25% of nodes reach level 3
- etc.

**Step 3: Initialize New Node**

```rust
// Create forward pointer array for new node
node.forward = vec![None; new_level + 1];

// Convert Box<E> to raw pointer
let entry_ptr = Box::into_raw(entry);
let erased_ptr = NonNull::new(entry_ptr as *mut u8).unwrap();
```

**Step 4: Update Pointers**

At each level, splice the new node in:

```
Before (level 0):
  Node(5) -> Node(9)

After inserting Node(6) at level 0:
  Node(5) -> Node(6) -> Node(9)

Implementation:
  Node(6).forward[0] = Node(5).forward[0]  // Node(6) -> Node(9)
  Node(5).forward[0] = Node(6)             // Node(5) -> Node(6)
```

#### Visualization: Complete Insert Example

**Insert 6 with random_level=2:**

```
Before:
Level 2:  Head -------> 9 -> None
Level 1:  Head -> 3 -> 9 -> None
Level 0:  Head -> 2 -> 3 -> 5 -> 9 -> None

After:
Level 2:  Head -------> 6 -------> 9 -> None
Level 1:  Head -> 3 -> 6 -------> 9 -> None
Level 0:  Head -> 2 -> 3 -> 5 -> 6 -> 9 -> None
                             ^new node^
```

**Pointer updates:**
```
update[2] = Head     →  Head.forward[2] = Node(6)
                        Node(6).forward[2] = Node(9)

update[1] = Node(3)  →  Node(3).forward[1] = Node(6)
                        Node(6).forward[1] = Node(9)

update[0] = Node(5)  →  Node(5).forward[0] = Node(6)
                        Node(6).forward[0] = Node(9)
```

#### Memory Management

**Box to Raw Pointer:**
```rust
let entry_ptr = Box::into_raw(entry);
```
- Transfers ownership from `Box` to raw pointer
- Memory is **not freed** automatically
- Skiplist now owns the memory
- Will be freed in `remove()` via `Box::from_raw()`

#### Time Complexity
- **Average: O(log n)**
- **Worst case: O(n)** (very rare)

**Cost breakdown:**
- Search: O(log n)
- Level generation: O(1)
- Pointer updates: O(log n) expected

---

### `remove()` - Removing and Returning Elements

#### Signature
```rust
pub fn remove(&mut self, key: &K) -> Option<Box<E>>
```

#### Purpose
Removes an element by key and returns ownership of it (as `Box<E>`).

#### Implementation
```rust
pub fn remove(&mut self, key: &K) -> Option<Box<E>> {
    // Track update pointers at each level
    let mut update: Vec<*mut SkipListNode> = vec![std::ptr::null_mut(); self.max_level + 1];
    let mut current = &self.head as *const SkipListNode;

    // Find the node to remove (like insert search)
    for level in (0..=self.level).rev() {
        unsafe {
            loop {
                let current_node = &*current;
                match current_node.forward[level] {
                    Some(next_ptr) => {
                        let next_entry: &E = next_ptr.cast::<E>().as_ref();
                        match next_entry.key().cmp(key) {
                            Less => current = next_entry.skiplist_node() as *const _,
                            Equal | Greater => break,
                        }
                    }
                    None => break,
                }
            }
        }
        update[level] = current as *mut SkipListNode;
    }

    // Check if node exists
    unsafe {
        let current_node = &*current;
        if let Some(target_ptr) = current_node.forward[0] {
            let target_entry: &E = target_ptr.cast::<E>().as_ref();

            if target_entry.key() == key {
                let target_node = target_entry.skiplist_node();

                // Update forward pointers to skip over removed node
                for level in 0..=self.level {
                    let update_node = &mut *update[level];
                    if let Some(ptr) = update_node.forward[level] {
                        if ptr == target_ptr {
                            update_node.forward[level] = target_node.forward[level];
                        }
                    }
                }

                // Update list level if top levels now empty
                while self.level > 0 && self.head.forward[self.level].is_none() {
                    self.level -= 1;
                }

                self.len -= 1;

                // Convert raw pointer back to Box
                let removed_ptr = target_ptr.cast::<E>().as_ptr();
                return Some(Box::from_raw(removed_ptr));
            }
        }
    }

    None
}
```

#### Algorithm Steps

1. **Find node** - Search like `insert()`, tracking predecessors
2. **Verify existence** - Check if key exists at level 0
3. **Update pointers** - Unlink node at all levels
4. **Adjust level** - Lower skiplist level if top levels empty
5. **Return ownership** - Convert raw pointer back to `Box`

#### Visualization

**Remove 5 from:**
```
Level 1:  Head -> 3 -> 5 -> 9 -> None
Level 0:  Head -> 2 -> 3 -> 5 -> 7 -> 9 -> None
```

**Step 1: Find update pointers**
```
update[1] = Node(3)  (points to Node(5) at level 1)
update[0] = Node(3)  (points to Node(5) at level 0)
```

**Step 2: Update pointers**
```
Level 1:  Node(3).forward[1] = Node(5).forward[1]  // Node(3) -> Node(9)
Level 0:  Node(3).forward[0] = Node(5).forward[0]  // Node(3) -> Node(7)
```

**Result:**
```
Level 1:  Head -> 3 -> 9 -> None
Level 0:  Head -> 2 -> 3 -> 7 -> 9 -> None
          (Node(5) removed and returned as Box<E>)
```

#### Memory Management

**Raw Pointer to Box:**
```rust
let removed_ptr = target_ptr.cast::<E>().as_ptr();
return Some(Box::from_raw(removed_ptr));
```
- Converts raw pointer back to `Box<E>`
- Transfers ownership to caller
- Caller is responsible for dropping the `Box`
- Memory will be freed when `Box` goes out of scope

#### Time Complexity
- **Average: O(log n)**
- **Worst case: O(n)**

---

### `remove_by_key()` - Removing Without Return

#### Signature
```rust
pub fn remove_by_key(&mut self, key: &K) -> bool
```

#### Purpose
Removes an element by key without returning it. More efficient when you don't need the value.

#### Implementation
```rust
pub fn remove_by_key(&mut self, key: &K) -> bool {
    self.remove(key).is_some()
}
```

**Simple wrapper around `remove()`:**
- Calls `remove(key)`
- Drops the returned `Box<E>` (if any)
- Returns `true` if element was removed, `false` otherwise

#### Use When
- You don't need the removed value
- Just checking if removal succeeded
- Cleaning up entries

#### Example
```rust
if list.remove_by_key(&42) {
    println!("Removed successfully");
} else {
    println!("Key not found");
}
```

#### Time Complexity
- **Average: O(log n)**
- **Worst case: O(n)**

Same as `remove()` since it just wraps it.

---

## Implementation Summary

| Function | Purpose | Returns | Complexity |
|----------|---------|---------|------------|
| `first()` | Get first element | `Option<&E>` | O(1) |
| `get()` | Search by key | `Option<&E>` | O(log n) |
| `get_mut()` | Search with mutation | `Option<&mut E>` | O(log n) |
| `successor()` | Find next element | `Option<&E>` | O(log n) |
| `insert()` | Add new element | `Result<(), Box<E>>` | O(log n) |
| `remove()` | Remove and return | `Option<Box<E>>` | O(log n) |
| `remove_by_key()` | Remove without return | `bool` | O(log n) |

**All operations achieve O(log n) average time complexity through probabilistic balancing!**


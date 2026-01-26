/// Example demonstrating basic skiplist operations
///
/// Run with: cargo run --example basic_usage

use skiplist::{SkipList, SkipListEntry, SkipListNode};

// Define a simple User structure with embedded skiplist metadata
#[derive(Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
    skiplist_meta: SkipListNode,
}

impl User {
    fn new(id: u64, name: &str, email: &str) -> Box<Self> {
        Box::new(User {
            id,
            name: name.to_string(),
            email: email.to_string(),
            skiplist_meta: SkipListNode::new(),
        })
    }
}

// Implement the SkipListEntry trait
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

fn main() {
    println!("=== Skip List: Basic Usage Examples ===\n");

    // Example 1: Creating and inserting
    example_insert();

    // Example 2: Searching
    example_search();

    // Example 3: Updating values
    example_update();

    // Example 4: Removing elements
    example_remove();

    // Example 5: Navigating elements
    example_navigation();

    // Example 6: Working with ownership
    example_ownership();
}

fn example_insert() {
    println!("Example 1: Creating and Inserting");
    println!("----------------------------------");

    let mut users: SkipList<u64, User> = SkipList::new();

    // Insert users
    let alice = User::new(101, "Alice", "alice@example.com");
    let bob = User::new(103, "Bob", "bob@example.com");
    let charlie = User::new(102, "Charlie", "charlie@example.com");

    println!("Inserting users...");
    users.insert(alice).unwrap();
    users.insert(bob).unwrap();
    users.insert(charlie).unwrap();

    println!("  Total users: {}", users.len());
    println!("  Is empty: {}", users.is_empty());

    // Try inserting duplicate
    let duplicate = User::new(101, "Duplicate Alice", "dupe@example.com");
    match users.insert(duplicate) {
        Ok(()) => println!("  Duplicate inserted (shouldn't happen)"),
        Err(returned_user) => {
            println!("  Duplicate rejected: ID {} ({})", returned_user.id, returned_user.name)
        }
    }

    println!();
}

fn example_search() {
    println!("Example 2: Searching");
    println!("--------------------");

    let mut users: SkipList<u64, User> = SkipList::new();
    users.insert(User::new(101, "Alice", "alice@example.com")).unwrap();
    users.insert(User::new(102, "Bob", "bob@example.com")).unwrap();
    users.insert(User::new(103, "Charlie", "charlie@example.com")).unwrap();

    // Search for existing user
    if let Some(user) = users.get(&102) {
        println!("  Found: {} ({})", user.name, user.email);
    }

    // Search for non-existing user
    if users.get(&999).is_none() {
        println!("  User 999 not found (as expected)");
    }

    // Get first user
    if let Some(first) = users.first() {
        println!("  First user: {} (ID: {})", first.name, first.id);
    }

    println!();
}

fn example_update() {
    println!("Example 3: Updating Values");
    println!("---------------------------");

    let mut users: SkipList<u64, User> = SkipList::new();
    users.insert(User::new(101, "Alice", "alice@example.com")).unwrap();
    users.insert(User::new(102, "Bob", "bob@example.com")).unwrap();

    println!("Before update:");
    if let Some(user) = users.get(&101) {
        println!("  User 101: {} ({})", user.name, user.email);
    }

    // Update user's data (not the key!)
    if let Some(user) = users.get_mut(&101) {
        user.name = "Alice Smith".to_string();
        user.email = "alice.smith@example.com".to_string();
    }

    println!("After update:");
    if let Some(user) = users.get(&101) {
        println!("  User 101: {} ({})", user.name, user.email);
    }

    println!();
}

fn example_remove() {
    println!("Example 4: Removing Elements");
    println!("-----------------------------");

    let mut users: SkipList<u64, User> = SkipList::new();
    users.insert(User::new(101, "Alice", "alice@example.com")).unwrap();
    users.insert(User::new(102, "Bob", "bob@example.com")).unwrap();
    users.insert(User::new(103, "Charlie", "charlie@example.com")).unwrap();

    println!("Initial count: {}", users.len());

    // Remove and get ownership back
    if let Some(removed_user) = users.remove(&102) {
        println!("  Removed: {} (ID: {})", removed_user.name, removed_user.id);
        // We now own the Box<User> and can do whatever we want with it
        drop(removed_user); // Explicitly drop to show ownership
    }

    println!("After removal: {}", users.len());

    // Remove without getting value back (more efficient)
    let existed = users.remove_by_key(&103);
    println!("  Removed user 103: {}", existed);
    println!("Final count: {}", users.len());

    println!();
}

fn example_navigation() {
    println!("Example 5: Navigating Elements");
    println!("-------------------------------");

    let mut users: SkipList<u64, User> = SkipList::new();
    users.insert(User::new(101, "Alice", "alice@example.com")).unwrap();
    users.insert(User::new(103, "Bob", "bob@example.com")).unwrap();
    users.insert(User::new(105, "Charlie", "charlie@example.com")).unwrap();
    users.insert(User::new(107, "Diana", "diana@example.com")).unwrap();

    // Get first element
    println!("First user: {}", users.first().unwrap().name);

    // Find successor (next element after a key)
    if let Some(next) = users.successor(&103) {
        println!("User after 103: {} (ID: {})", next.name, next.id);
    }

    // Find successor for non-existent key (finds next greater)
    if let Some(next) = users.successor(&104) {
        println!("User after 104 (doesn't exist): {} (ID: {})", next.name, next.id);
    }

    // Iterate through all users
    println!("\nAll users in order:");
    let mut current = users.first();
    while let Some(user) = current {
        println!("  - {} (ID: {})", user.name, user.id);
        current = users.successor(user.key());
    }

    println!();
}

fn example_ownership() {
    println!("Example 6: Working with Ownership");
    println!("----------------------------------");

    let mut users: SkipList<u64, User> = SkipList::new();

    // Insert transfers ownership to skiplist
    let alice = User::new(101, "Alice", "alice@example.com");
    users.insert(alice).unwrap();
    // alice is now moved and cannot be used

    // Get returns borrowed reference
    {
        let alice_ref = users.get(&101).unwrap();
        println!("  Borrowed: {} ({})", alice_ref.name, alice_ref.email);
    } // Borrow ends here

    // Remove transfers ownership back to caller
    let alice_owned = users.remove(&101).unwrap();
    println!("  Owned again: {} ({})", alice_owned.name, alice_owned.email);

    // Now we can do anything with alice_owned
    drop(alice_owned); // Explicitly drop to demonstrate ownership

    println!("  User removed and dropped");
    println!();
}

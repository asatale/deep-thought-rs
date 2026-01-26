/// Example demonstrating real-world use cases with ordered data
///
/// Run with: cargo run --example ordered_data

use skiplist::{SkipList, SkipListEntry, SkipListNode};

fn main() {
    println!("=== Skip List: Real-World Ordered Data Examples ===\n");

    // Example 1: Time-series data (sensor readings)
    example_timeseries();

    // Example 2: Leaderboard (high scores)
    example_leaderboard();

    // Example 3: Priority queue (task scheduler)
    example_priority_queue();

    // Example 4: Database index simulation
    example_database_index();
}

// ============================================================================
// Example 1: Time-Series Data
// ============================================================================

#[derive(Debug)]
#[allow(dead_code)]
struct SensorReading {
    timestamp: u64,
    sensor_id: String,
    temperature: f64,
    humidity: f64,
    skiplist_meta: SkipListNode,
}

impl SensorReading {
    fn new(timestamp: u64, sensor_id: &str, temperature: f64, humidity: f64) -> Box<Self> {
        Box::new(SensorReading {
            timestamp,
            sensor_id: sensor_id.to_string(),
            temperature,
            humidity,
            skiplist_meta: SkipListNode::new(),
        })
    }
}

impl SkipListEntry for SensorReading {
    type Key = u64;
    fn key(&self) -> &Self::Key {
        &self.timestamp
    }
    fn skiplist_node(&self) -> &SkipListNode {
        &self.skiplist_meta
    }
    fn skiplist_node_mut(&mut self) -> &mut SkipListNode {
        &mut self.skiplist_meta
    }
}

fn example_timeseries() {
    println!("Example 1: Time-Series Data (IoT Sensor Readings)");
    println!("--------------------------------------------------");

    let mut readings: SkipList<u64, SensorReading> = SkipList::new();

    // Simulate sensor readings arriving out of order
    let base_time = 1609459200; // Jan 1, 2021, 00:00:00 UTC

    readings
        .insert(SensorReading::new(
            base_time + 300,
            "sensor-01",
            22.5,
            45.0,
        ))
        .unwrap();
    readings
        .insert(SensorReading::new(
            base_time + 100,
            "sensor-01",
            21.0,
            48.0,
        ))
        .unwrap();
    readings
        .insert(SensorReading::new(
            base_time + 500,
            "sensor-01",
            23.0,
            42.0,
        ))
        .unwrap();
    readings
        .insert(SensorReading::new(
            base_time + 200,
            "sensor-01",
            21.5,
            47.0,
        ))
        .unwrap();

    println!("Collected {} readings", readings.len());

    // Query: Find earliest reading
    if let Some(earliest) = readings.first() {
        println!(
            "  Earliest: t={} ({}°C, {}% humidity)",
            earliest.timestamp, earliest.temperature, earliest.humidity
        );
    }

    // Query: Find first reading after timestamp
    let query_time = base_time + 250;
    if let Some(next) = readings.successor(&query_time) {
        println!(
            "  First reading after t={}: t={} ({}°C)",
            query_time, next.timestamp, next.temperature
        );
    }

    // Query: Iterate through all readings in chronological order
    println!("\n  All readings (chronological):");
    let mut current = readings.first();
    while let Some(reading) = current {
        let offset = reading.timestamp - base_time;
        println!(
            "    +{}s: {}°C, {}%",
            offset, reading.temperature, reading.humidity
        );
        current = readings.successor(reading.key());
    }

    println!();
}

// ============================================================================
// Example 2: Leaderboard
// ============================================================================

#[derive(Debug)]
#[allow(dead_code)]
struct PlayerScore {
    score: u64,
    player_name: String,
    timestamp: u64,
    skiplist_meta: SkipListNode,
}

impl PlayerScore {
    fn new(score: u64, player_name: &str, timestamp: u64) -> Box<Self> {
        Box::new(PlayerScore {
            score,
            player_name: player_name.to_string(),
            timestamp,
            skiplist_meta: SkipListNode::new(),
        })
    }
}

impl SkipListEntry for PlayerScore {
    type Key = u64;
    fn key(&self) -> &Self::Key {
        &self.score
    }
    fn skiplist_node(&self) -> &SkipListNode {
        &self.skiplist_meta
    }
    fn skiplist_node_mut(&mut self) -> &mut SkipListNode {
        &mut self.skiplist_meta
    }
}

fn example_leaderboard() {
    println!("Example 2: Game Leaderboard (High Scores)");
    println!("------------------------------------------");

    let mut leaderboard: SkipList<u64, PlayerScore> = SkipList::new();

    // Players submit scores
    leaderboard.insert(PlayerScore::new(1500, "Alice", 1001)).unwrap();
    leaderboard.insert(PlayerScore::new(2300, "Bob", 1002)).unwrap();
    leaderboard.insert(PlayerScore::new(1800, "Charlie", 1003)).unwrap();
    leaderboard.insert(PlayerScore::new(2100, "Diana", 1004)).unwrap();
    leaderboard.insert(PlayerScore::new(1200, "Eve", 1005)).unwrap();

    println!("Leaderboard has {} entries", leaderboard.len());

    // Show top scores (iterate from highest)
    println!("\n  Top Scores:");
    let mut rank = 1;
    let mut current = leaderboard.first();
    while let Some(entry) = current {
        println!(
            "    #{}: {} - {} points",
            rank, entry.player_name, entry.score
        );
        rank += 1;
        current = leaderboard.successor(entry.key());
    }

    // Find first player above threshold
    let threshold = 2000;
    if let Some(entry) = leaderboard.successor(&threshold) {
        println!(
            "\n  First player above {}: {} with {} points",
            threshold, entry.player_name, entry.score
        );
    }

    // Remove lowest score
    if let Some(lowest) = leaderboard.first() {
        let score_to_remove = *lowest.key();
        if let Some(removed) = leaderboard.remove(&score_to_remove) {
            println!(
                "  Removed lowest score: {} ({} points)",
                removed.player_name, removed.score
            );
        }
    }

    println!();
}

// ============================================================================
// Example 3: Priority Queue
// ============================================================================

#[derive(Debug)]
#[allow(dead_code)]
struct Task {
    priority: u64, // Lower number = higher priority
    task_name: String,
    created_at: u64,
    skiplist_meta: SkipListNode,
}

impl Task {
    fn new(priority: u64, task_name: &str, created_at: u64) -> Box<Self> {
        Box::new(Task {
            priority,
            task_name: task_name.to_string(),
            created_at,
            skiplist_meta: SkipListNode::new(),
        })
    }
}

impl SkipListEntry for Task {
    type Key = u64;
    fn key(&self) -> &Self::Key {
        &self.priority
    }
    fn skiplist_node(&self) -> &SkipListNode {
        &self.skiplist_meta
    }
    fn skiplist_node_mut(&mut self) -> &mut SkipListNode {
        &mut self.skiplist_meta
    }
}

fn example_priority_queue() {
    println!("Example 3: Priority Task Scheduler");
    println!("-----------------------------------");

    let mut task_queue: SkipList<u64, Task> = SkipList::new();

    // Add tasks with different priorities
    task_queue.insert(Task::new(5, "Send email", 1000)).unwrap();
    task_queue.insert(Task::new(1, "Critical security patch", 1001)).unwrap();
    task_queue.insert(Task::new(3, "Backup database", 1002)).unwrap();
    task_queue.insert(Task::new(8, "Clean temp files", 1003)).unwrap();
    task_queue.insert(Task::new(2, "Process payments", 1004)).unwrap();

    println!("Task queue has {} pending tasks", task_queue.len());

    // Process tasks in priority order
    println!("\n  Processing tasks (highest priority first):");
    let mut processed = 0;
    while let Some(task) = task_queue.first() {
        let priority = *task.key();
        println!(
            "    [Priority {}] Processing: {}",
            priority, task.task_name
        );

        // Remove task after processing
        task_queue.remove(&priority);
        processed += 1;

        // Stop after processing 3 tasks for demo
        if processed >= 3 {
            break;
        }
    }

    println!(
        "\n  Remaining tasks: {}",
        task_queue.len()
    );

    println!();
}

// ============================================================================
// Example 4: Database Index Simulation
// ============================================================================

#[derive(Debug)]
struct Record {
    id: u64,
    username: String,
    email: String,
    created_at: u64,
    skiplist_meta: SkipListNode,
}

impl Record {
    fn new(id: u64, username: &str, email: &str, created_at: u64) -> Box<Self> {
        Box::new(Record {
            id,
            username: username.to_string(),
            email: email.to_string(),
            created_at,
            skiplist_meta: SkipListNode::new(),
        })
    }
}

impl SkipListEntry for Record {
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

fn example_database_index() {
    println!("Example 4: Database Index Simulation");
    println!("-------------------------------------");

    let mut index: SkipList<u64, Record> = SkipList::new();

    // Simulate inserting records (like a primary key index)
    index.insert(Record::new(1001, "alice", "alice@example.com", 1000)).unwrap();
    index.insert(Record::new(1005, "bob", "bob@example.com", 1001)).unwrap();
    index.insert(Record::new(1003, "charlie", "charlie@example.com", 1002)).unwrap();
    index.insert(Record::new(1007, "diana", "diana@example.com", 1003)).unwrap();
    index.insert(Record::new(1002, "eve", "eve@example.com", 1004)).unwrap();

    println!("Database has {} records", index.len());

    // Query: Find record by ID
    println!("\n  Query: SELECT * FROM users WHERE id = 1003");
    if let Some(record) = index.get(&1003) {
        println!(
            "    Result: {} ({}) - created {}",
            record.username, record.email, record.created_at
        );
    }

    // Query: Range scan (e.g., for pagination)
    println!("\n  Query: SELECT * FROM users WHERE id >= 1003 LIMIT 3");
    let mut count = 0;
    let mut current = index.get(&1003).or_else(|| index.successor(&1003));
    while let Some(record) = current {
        println!(
            "    ID {}: {} ({})",
            record.id, record.username, record.email
        );
        count += 1;
        if count >= 3 {
            break;
        }
        current = index.successor(record.key());
    }

    // Query: Count records in range
    println!("\n  Query: SELECT COUNT(*) FROM users WHERE id BETWEEN 1003 AND 1006");
    let mut count = 0;
    let mut current = index.get(&1003).or_else(|| index.successor(&1003));
    while let Some(record) = current {
        if *record.key() > 1006 {
            break;
        }
        count += 1;
        current = index.successor(record.key());
    }
    println!("    Count: {}", count);

    // Delete record
    println!("\n  Query: DELETE FROM users WHERE id = 1005");
    if let Some(deleted) = index.remove(&1005) {
        println!("    Deleted: {} ({})", deleted.username, deleted.email);
    }
    println!("    Remaining records: {}", index.len());

    println!();
}

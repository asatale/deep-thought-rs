/// Example demonstrating in-place set operations and operator overloading
use roaring_bitmap::RoaringBitmap;

fn main() {
    println!("=== In-Place Set Operations and Operator Overloading ===\n");

    // Example 1: Incremental Union (Common Workflow)
    println!("Example 1: Building Union Incrementally");
    println!("---------------------------------------");

    let sources = vec![
        create_bitmap(&[1, 2, 3]),
        create_bitmap(&[3, 4, 5]),
        create_bitmap(&[5, 6, 7]),
        create_bitmap(&[7, 8, 9]),
    ];

    // Old way (creates intermediate bitmaps)
    let mut result_old = RoaringBitmap::new();
    for bitmap in &sources {
        result_old = result_old.union(bitmap); // Allocates each time
    }
    println!("Old way (allocating): {:?}", collect_values(&result_old));

    // New way with in-place method
    let mut result_method = RoaringBitmap::new();
    for bitmap in &sources {
        result_method.union_with(bitmap); // No allocation
    }
    println!("New way (in-place):   {:?}", collect_values(&result_method));

    // New way with operator
    let mut result_operator = RoaringBitmap::new();
    for bitmap in &sources {
        result_operator |= bitmap; // Clean and efficient
    }
    println!(
        "New way (operator):   {:?}\n",
        collect_values(&result_operator)
    );

    // Example 2: Applying Multiple Filters
    println!("Example 2: Sequential Filtering");
    println!("--------------------------------");

    let mut candidates = create_bitmap(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let even_filter = create_bitmap(&[2, 4, 6, 8, 10]);
    let small_filter = create_bitmap(&[1, 2, 3, 4, 5, 6]);
    let prime_filter = create_bitmap(&[2, 3, 5, 7]);

    println!("Starting candidates: {:?}", collect_values(&candidates));

    candidates &= &even_filter;
    println!("After even filter:   {:?}", collect_values(&candidates));

    candidates &= &small_filter;
    println!("After small filter:  {:?}", collect_values(&candidates));

    candidates &= &prime_filter;
    println!(
        "After prime filter:  {:?} (only 2 matches all)\n",
        collect_values(&candidates)
    );

    // Example 3: Complex Set Expressions
    println!("Example 3: Complex Set Expressions");
    println!("-----------------------------------");

    let active_users = create_bitmap(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let premium_users = create_bitmap(&[2, 4, 6, 8, 10]);
    let trial_users = create_bitmap(&[1, 3, 5, 7, 9]);
    let banned_users = create_bitmap(&[5, 10]);

    // Find active users who are (premium OR trial) but NOT banned
    let eligible = (&active_users & (&premium_users | &trial_users)) - &banned_users;
    println!("Active: {:?}", collect_values(&active_users));
    println!("Premium: {:?}", collect_values(&premium_users));
    println!("Trial: {:?}", collect_values(&trial_users));
    println!("Banned: {:?}", collect_values(&banned_users));
    println!("Eligible: {:?}\n", collect_values(&eligible));

    // Example 4: Operator Chaining
    println!("Example 4: Operator Chaining");
    println!("-----------------------------");

    let a = create_bitmap(&[1, 2, 3, 4, 5]);
    let b = create_bitmap(&[3, 4, 5, 6, 7]);
    let c = create_bitmap(&[5, 6, 7, 8, 9]);

    println!("A: {:?}", collect_values(&a));
    println!("B: {:?}", collect_values(&b));
    println!("C: {:?}", collect_values(&c));

    let union_then_intersect = (&a | &b) & &c;
    println!("(A ∪ B) ∩ C: {:?}", collect_values(&union_then_intersect));

    let intersect_then_union = (&a & &b) | &c;
    println!("(A ∩ B) ∪ C: {:?}", collect_values(&intersect_then_union));

    let complex = ((&a | &b) - &c) ^ &c;
    println!("((A ∪ B) - C) ⊕ C: {:?}\n", collect_values(&complex));

    // Example 5: Performance Comparison
    println!("Example 5: Performance Demonstration");
    println!("-------------------------------------");

    // Create larger bitmaps for more realistic scenario
    let mut large_sources = Vec::new();
    for i in 0..100 {
        let mut bm = RoaringBitmap::new();
        for j in (i * 100)..((i + 1) * 100) {
            bm.insert(j);
        }
        large_sources.push(bm);
    }

    // Method 1: Allocating (creates 99 intermediate bitmaps)
    let start = std::time::Instant::now();
    let mut result_alloc = RoaringBitmap::new();
    for bitmap in &large_sources {
        result_alloc = result_alloc.union(bitmap);
    }
    let alloc_time = start.elapsed();

    // Method 2: In-place (zero intermediate bitmaps)
    let start = std::time::Instant::now();
    let mut result_inplace = RoaringBitmap::new();
    for bitmap in &large_sources {
        result_inplace |= bitmap;
    }
    let inplace_time = start.elapsed();

    println!("Unioning 100 bitmaps (10,000 values total):");
    println!("  Allocating approach: {:?}", alloc_time);
    println!("  In-place approach:   {:?}", inplace_time);
    println!(
        "  Speedup: {:.2}x",
        alloc_time.as_nanos() as f64 / inplace_time.as_nanos() as f64
    );
    println!("  Both results have {} elements", result_inplace.len());

    // Example 6: Real-World Use Cases
    println!("\nExample 6: Real-World Use Cases");
    println!("--------------------------------");

    // Use case 1: User segmentation
    println!("Use Case 1: User Segmentation");
    let mut target_audience = create_bitmap(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let age_18_35 = create_bitmap(&[1, 2, 3, 4, 5, 6, 7]);
    let interested_in_tech = create_bitmap(&[2, 4, 5, 7, 8, 9]);
    let active_last_30_days = create_bitmap(&[1, 2, 3, 5, 6, 8, 9]);

    target_audience &= &age_18_35;
    target_audience &= &interested_in_tech;
    target_audience &= &active_last_30_days;

    println!("  Target audience size: {}", target_audience.len());
    println!("  User IDs: {:?}", collect_values(&target_audience));

    // Use case 2: Access control
    println!("\nUse Case 2: Access Control");
    let resource_viewers = create_bitmap(&[1, 2, 3, 4, 5]);
    let resource_editors = create_bitmap(&[2, 4]);
    let suspended_users = create_bitmap(&[3]);

    let can_view = &resource_viewers - &suspended_users;
    let can_edit = &resource_editors - &suspended_users;

    println!("  Can view: {:?}", collect_values(&can_view));
    println!("  Can edit: {:?}", collect_values(&can_edit));

    // Use case 3: A/B testing
    println!("\nUse Case 3: A/B Testing");
    let group_a = create_bitmap(&[1, 3, 5, 7, 9]);
    let group_b = create_bitmap(&[2, 4, 6, 8, 10]);
    let converted = create_bitmap(&[1, 2, 5, 8]);

    let a_converted = &group_a & &converted;
    let b_converted = &group_b & &converted;

    println!(
        "  Group A conversions: {} / {}",
        a_converted.len(),
        group_a.len()
    );
    println!(
        "  Group B conversions: {} / {}",
        b_converted.len(),
        group_b.len()
    );
}

// Helper functions

fn create_bitmap(values: &[u32]) -> RoaringBitmap {
    let mut bm = RoaringBitmap::new();
    for &val in values {
        bm.insert(val);
    }
    bm
}

fn collect_values(bitmap: &RoaringBitmap) -> Vec<u32> {
    bitmap.iter().collect()
}

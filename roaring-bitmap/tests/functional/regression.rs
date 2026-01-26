use roaring_bitmap::RoaringBitmap;

// ============================================================================
// REGRESSION TESTS FOR RUNCONTAINER INSERT/MERGE
// These tests directly exercise RunContainer code paths to catch bugs in
// run merging logic and overflow handling.
// ============================================================================

#[test]
fn regression_run_merge_math_error() {
    // This test catches the bug where merging adjacent runs incorrectly
    // adds next_length instead of next_length + 1
    //
    // Scenario:
    // - Run 1: (0, 2) = values [0, 1, 2] (length=3, stored as length-1=2)
    // - Insert value 3, extending Run 1 to (0, 3) = [0, 1, 2, 3]
    // - Run 2: (4, 1) = values [4, 5] (length=2, stored as length-1=1)
    // - When inserting 3, it should merge both runs
    // - Result should be (0, 5) = [0, 1, 2, 3, 4, 5] (all 6 values)
    //
    // Bug: Would produce (0, 4) = [0, 1, 2, 3, 4] (missing value 5)

    let mut bm = RoaringBitmap::new();

    // Create first run: [0, 1, 2]
    bm.insert(0);
    bm.insert(1);
    bm.insert(2);

    // Create second run: [4, 5]
    bm.insert(4);
    bm.insert(5);

    // Optimize to potentially convert to Run containers
    // (Note: optimize() may or may not choose Run for small datasets)
    bm.optimize();

    // Verify we have exactly 5 values before the merge
    assert_eq!(bm.len(), 5);

    // Now insert value 3 to trigger potential run merging
    // This tests the merge math regardless of container type
    assert!(bm.insert(3));

    // Verify all 6 values are present (catches the merge bug)
    assert_eq!(bm.len(), 6, "Should have 6 values after merge");
    assert!(bm.contains(0));
    assert!(bm.contains(1));
    assert!(bm.contains(2));
    assert!(bm.contains(3));
    assert!(bm.contains(4));
    assert!(
        bm.contains(5),
        "Value 5 should be present (merge bug check)"
    );

    // Verify through iteration
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(
        values,
        vec![0, 1, 2, 3, 4, 5],
        "All values should be present in order"
    );
}

#[test]
fn regression_run_merge_multiple_gaps() {
    // Test merging runs with multiple gaps being filled
    let mut bm = RoaringBitmap::new();

    // Create runs: [0-2], [5-7], [10-12]
    for i in 0..=2 {
        bm.insert(i);
    }
    for i in 5..=7 {
        bm.insert(i);
    }
    for i in 10..=12 {
        bm.insert(i);
    }

    bm.optimize();
    // Note: Container type may vary based on optimization heuristics
    assert_eq!(bm.len(), 9);

    // Fill first gap [3, 4] - should merge first two runs
    bm.insert(3);
    bm.insert(4);

    assert_eq!(bm.len(), 11);
    assert!(bm.contains(3));
    assert!(bm.contains(4));

    // Fill second gap [8, 9] - should merge all into one run
    bm.insert(8);
    bm.insert(9);

    assert_eq!(bm.len(), 13);

    // Verify all values [0-12] are present
    for i in 0..=12 {
        assert!(bm.contains(i), "Value {} should be present", i);
    }

    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values, (0..=12).collect::<Vec<u32>>());
}

#[test]
fn regression_full_container_overflow() {
    // Test inserting all 65,536 consecutive values in a single container
    // This catches the overflow bug where run_length (u16) would overflow
    // when converting from array to run with 65,536 values
    //
    // Bug: run_length += 1 would overflow at 65,536 (max u16 is 65,535)

    let mut bm = RoaringBitmap::new();

    // Insert all values from 0 to 65,535 (full container)
    for i in 0u32..=65535u32 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 65536, "Should have all 65,536 values");

    // Optimize to convert to Run container
    // This should trigger the from_array conversion that had the overflow bug
    bm.optimize();

    // Verify container type (should be Run for consecutive values)
    assert_eq!(
        bm.container_type(0),
        Some("Run"),
        "Full consecutive container should be Run type"
    );

    // Verify all values are still present
    assert_eq!(
        bm.len(),
        65536,
        "Should still have all 65,536 values after optimize"
    );

    // Spot check some values
    assert!(bm.contains(0), "Should contain first value");
    assert!(bm.contains(32767), "Should contain middle value");
    assert!(bm.contains(65535), "Should contain last value");

    // Verify through iteration (sampling to avoid slow test)
    let values: Vec<u32> = bm.iter().step_by(1000).collect();
    assert_eq!(
        values.len(),
        66,
        "Should have 66 values when sampling every 1000th"
    );
    assert_eq!(values[0], 0, "First sampled value should be 0");
    assert_eq!(values[65], 65000, "Last sampled value should be 65000");
}

#[test]
fn regression_run_merge_at_boundaries() {
    // Test run merging at u16 boundaries (edge cases)
    let mut bm = RoaringBitmap::new();

    // Create runs near boundaries
    // Run 1: [0, 1, 2]
    for i in 0..=2 {
        bm.insert(i);
    }

    // Run 2: [65533, 65534, 65535] (end of u16 range)
    for i in 65533..=65535 {
        bm.insert(i);
    }

    bm.optimize();

    // Verify both runs exist
    assert_eq!(bm.len(), 6);
    assert!(bm.contains(0));
    assert!(bm.contains(2));
    assert!(bm.contains(65533));
    assert!(bm.contains(65535));

    // Insert in middle to create a third run
    for i in 100..=102 {
        bm.insert(i);
    }

    assert_eq!(bm.len(), 9);

    // Verify iteration works correctly across all runs
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 9);
    assert_eq!(values[0], 0);
    assert_eq!(values[1], 1);
    assert_eq!(values[2], 2);
    assert_eq!(values[3], 100);
    assert_eq!(values[4], 101);
    assert_eq!(values[5], 102);
    assert_eq!(values[6], 65533);
    assert_eq!(values[7], 65534);
    assert_eq!(values[8], 65535);
}

#[test]
fn regression_run_merge_single_value_gap() {
    // Test merging runs separated by a single value
    let mut bm = RoaringBitmap::new();

    // Create two runs with a 1-value gap: [0-5] and [7-10]
    for i in 0..=5 {
        bm.insert(i);
    }
    for i in 7..=10 {
        bm.insert(i);
    }

    bm.optimize();
    assert_eq!(bm.len(), 10);

    // Insert the gap value (6) to trigger merge
    assert!(bm.insert(6));
    assert_eq!(bm.len(), 11);

    // Verify all values [0-10] present
    for i in 0..=10 {
        assert!(bm.contains(i), "Value {} should be present", i);
    }

    // Verify through iteration
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values, (0..=10).collect::<Vec<u32>>());
}

// ============================================================================
// REGRESSION TESTS FOR MULTI-CONTAINER ITERATION
// These tests exercise iterator logic across different container types
// to catch bugs in iterator state management.
// ============================================================================

#[test]
fn regression_iterate_array_bitmap_run() {
    // Test iteration across all three container types: Array + Bitmap + Run
    // This catches bugs in iterator state initialization when switching
    // between container types (especially bitmap state management)

    let mut bm = RoaringBitmap::new();

    // Container 0 (key=0): Array container
    // Insert sparse values [0, 10, 20, ..., 990] = 100 values
    for i in 0..100 {
        bm.insert(i * 10);
    }

    // Container 1 (key=1): Bitmap container
    // Insert dense non-consecutive values [65536, 65538, 65540, ..., 73726] = 4096 values
    for i in 0..8192 {
        if i % 2 == 0 {
            bm.insert(65536 + i);
        }
    }

    // Container 2 (key=2): Run container (via optimize)
    // Insert consecutive values [131072, 131073, ..., 132071] = 1000 values
    for i in 0..1000 {
        bm.insert(131072 + i);
    }

    // Optimize to ensure container 2 becomes Run type
    bm.optimize();

    // Verify container types
    let stats = bm.container_stats();
    assert_eq!(stats.len(), 3, "Should have exactly 3 containers");
    assert_eq!(stats[0].1, "Array", "Container 0 should be Array");
    assert_eq!(stats[1].1, "Bitmap", "Container 1 should be Bitmap");
    assert_eq!(stats[2].1, "Run", "Container 2 should be Run");

    // Verify total count
    assert_eq!(bm.len(), 100 + 4096 + 1000, "Should have 5196 total values");

    // Test iteration across all containers
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 5196, "Iterator should return all 5196 values");

    // Verify first container (Array) values
    assert_eq!(values[0], 0, "First value should be 0");
    assert_eq!(values[1], 10, "Second value should be 10");
    assert_eq!(
        values[99], 990,
        "Last value of first container should be 990"
    );

    // Verify second container (Bitmap) values start correctly
    assert_eq!(
        values[100], 65536,
        "First value of second container should be 65536"
    );
    assert_eq!(
        values[101], 65538,
        "Second value of second container should be 65538"
    );

    // Verify third container (Run) values start correctly
    let run_start_idx = 100 + 4096; // After Array and Bitmap containers
    assert_eq!(
        values[run_start_idx], 131072,
        "First value of third container should be 131072"
    );
    assert_eq!(
        values[run_start_idx + 1],
        131073,
        "Values in Run container should be consecutive"
    );
    assert_eq!(
        values[run_start_idx + 999],
        132071,
        "Last value of third container should be 132071"
    );

    // Verify strict ordering across all containers
    for i in 1..values.len() {
        assert!(
            values[i - 1] < values[i],
            "Values should be strictly increasing: {} >= {}",
            values[i - 1],
            values[i]
        );
    }
}

#[test]
fn regression_iterate_bitmap_transitions() {
    // Test iterator state when transitioning between multiple bitmap containers
    // This catches the bug where bitmap_current_word isn't properly initialized

    let mut bm = RoaringBitmap::new();

    // Container 0: Bitmap (key=0) - first word is zero, later words have values
    // Insert enough to trigger bitmap (need >= 4096 values)
    for i in 64..8256 {
        // Skip first 64 values (first word = 0)
        if i % 2 == 0 {
            bm.insert(i);
        }
    }

    // Container 1: Bitmap (key=1) - also starts with zero word
    // Insert enough to trigger bitmap (need >= 4096 values)
    for i in 65600..78192 {
        // Values starting after first word
        if i % 3 == 0 {
            bm.insert(i);
        }
    }

    // Verify both are bitmap containers
    bm.optimize(); // Ensure optimization doesn't change types
    let stats = bm.container_stats();
    assert_eq!(stats.len(), 2);
    assert_eq!(stats[0].1, "Bitmap");
    assert_eq!(stats[1].1, "Bitmap");

    // Iterate and verify no values are skipped
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(
        values.len(),
        bm.len() as usize,
        "Iterator should return exactly len() values"
    );

    // Verify values are strictly ordered
    for i in 1..values.len() {
        assert!(
            values[i - 1] < values[i],
            "Values must be strictly increasing"
        );
    }

    // Verify first container values are present
    assert_eq!(values[0], 64, "First value should be 64 (not skipped)");

    // Verify transition to second container is correct
    let container_1_starts = values.iter().position(|&v| v >= 65536).unwrap();
    assert_eq!(
        values[container_1_starts], 65601,
        "First value of second container should be present"
    );
}

#[test]
fn regression_iterate_empty_containers_between() {
    // Test iteration when there are gaps in container keys
    // (e.g., containers at keys 0 and 2, but not 1)

    let mut bm = RoaringBitmap::new();

    // Container 0: Array
    for i in 0..50 {
        bm.insert(i);
    }

    // Skip container 1 (keys 65536-131071)

    // Container 2: Run (consecutive)
    for i in 0..500 {
        bm.insert(131072 + i);
    }

    bm.optimize();

    let stats = bm.container_stats();
    assert_eq!(stats.len(), 2, "Should have 2 containers (0 and 2)");
    assert_eq!(stats[0].0, 0, "First container key should be 0");
    assert_eq!(stats[1].0, 2, "Second container key should be 2");

    // Iterate and verify all values present
    let values: Vec<u32> = bm.iter().collect();
    assert_eq!(values.len(), 550, "Should have 550 values total");

    // Verify first container
    assert_eq!(values[0], 0);
    assert_eq!(values[49], 49);

    // Verify second container
    assert_eq!(values[50], 131072);
    assert_eq!(values[549], 131571);

    // Verify strict ordering
    for i in 1..values.len() {
        assert!(values[i - 1] < values[i]);
    }
}

#[test]
fn regression_iterate_all_three_types_mixed_order() {
    // Create a complex scenario with multiple containers of each type
    // to thoroughly test iterator state management

    let mut bm = RoaringBitmap::new();

    // Key 0: Run (consecutive)
    for i in 0..500 {
        bm.insert(i);
    }

    // Key 1: Array (sparse)
    for i in 0..100 {
        bm.insert(65536 + i * 100);
    }

    // Key 2: Bitmap (dense)
    for i in 0..8192 {
        if i % 2 == 0 {
            bm.insert(131072 + i);
        }
    }

    // Key 3: Run again
    for i in 0..1000 {
        bm.insert(196608 + i);
    }

    // Key 4: Array again
    for i in 0..200 {
        bm.insert(262144 + i * 50);
    }

    bm.optimize();

    let stats = bm.container_stats();
    assert_eq!(stats.len(), 5, "Should have 5 containers");

    // Verify types
    assert_eq!(stats[0].1, "Run", "Container 0 should be Run");
    assert_eq!(stats[1].1, "Array", "Container 1 should be Array");
    assert_eq!(stats[2].1, "Bitmap", "Container 2 should be Bitmap");
    assert_eq!(stats[3].1, "Run", "Container 3 should be Run");
    assert_eq!(stats[4].1, "Array", "Container 4 should be Array");

    // Iterate and verify
    let values: Vec<u32> = bm.iter().collect();
    let expected_count = 500 + 100 + 4096 + 1000 + 200;
    assert_eq!(
        values.len(),
        expected_count,
        "Should have {} values",
        expected_count
    );

    // Verify strict ordering across all transitions
    for i in 1..values.len() {
        assert!(
            values[i - 1] < values[i],
            "Values must be strictly increasing at index {}: {} >= {}",
            i,
            values[i - 1],
            values[i]
        );
    }

    // Spot check boundary transitions
    assert_eq!(values[0], 0, "First value of Run container");
    assert_eq!(values[499], 499, "Last value of first Run container");
    assert_eq!(values[500], 65536, "First value of Array container");
    assert_eq!(values[599], 75436, "Last value of Array container");
    assert_eq!(values[600], 131072, "First value of Bitmap container");
}

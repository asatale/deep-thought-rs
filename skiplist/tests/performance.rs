// Performance benchmarks
//
// These tests are marked with #[ignore] to prevent them from running
// during normal test execution (cargo test), as they:
// - Process large datasets (up to 100,000+ elements)
// - Measure timing rather than verifying correctness
// - Can be slow and flaky on CI machines
//
// To run all benchmarks:
//   cargo test --test performance -- --ignored --nocapture
//
// To run specific benchmark category:
//   cargo test --test performance insertion -- --ignored --nocapture
//   cargo test --test performance lookup -- --ignored --nocapture
//   cargo test --test performance removal -- --ignored --nocapture
//   cargo test --test performance navigation -- --ignored --nocapture
//   cargo test --test performance mixed_workload -- --ignored --nocapture
//
// To run a specific benchmark:
//   cargo test --test performance perf_insert_sequential -- --ignored --nocapture

mod benchmarks;

// Shared benchmark helpers

/// Helper to format duration in appropriate units
pub fn format_duration(nanos: u128) -> String {
    if nanos < 1_000 {
        format!("{} ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.2} μs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2} ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2} s", nanos as f64 / 1_000_000_000.0)
    }
}

/// Helper to format operations per second
pub fn format_ops_per_sec(ops: usize, nanos: u128) -> String {
    let ops_per_sec = (ops as f64 / nanos as f64) * 1_000_000_000.0;
    if ops_per_sec >= 1_000_000.0 {
        format!("{:.2} M ops/sec", ops_per_sec / 1_000_000.0)
    } else if ops_per_sec >= 1_000.0 {
        format!("{:.2} K ops/sec", ops_per_sec / 1_000.0)
    } else {
        format!("{:.2} ops/sec", ops_per_sec)
    }
}

// Benchmark modules
mod containers;
mod insertion;
mod iteration;
mod lookup;
mod memory;
mod mixed_workload;
mod optimization;
mod set_operations;

# AGENTS.md

> Instructions for AI coding agents working on the deep-thought-rs project

## Project Overview

**deep-thought-rs** is a collection of fundamental data structures implemented in Rust through **Socratic AI dialogs**. This is not auto-generated code—it's a learning journey where each implementation is built through questioning, iterative refinement, and comprehensive testing.

### Key Philosophy

- **Deep Thought** asks questions, explores trade-offs, and seeks elegant solutions
- **Marvin** implements, tests, and maintains practical reality
- **Together** they produce thoughtful, well-tested, production-quality code

This is a workspace with multiple crates, each implementing a different data structure.

## Workspace Structure

```
deep-thought-rs/
├── Cargo.toml                 # Workspace configuration
├── README.md                  # Human-facing documentation
├── AGENTS.md                  # This file - agent instructions
├── LICENSE                    # MIT license
├── roaring-bitmap/            # Crate 1: Roaring bitmap implementation
│   ├── Cargo.toml
│   ├── lib.rs                 # Main library code
│   ├── README.md              # Crate-specific documentation
│   ├── tests/                 # Integration tests
│   │   ├── roaring_bitmap.rs  # Main test suite (~137 tests)
│   │   └── performance.rs     # Performance benchmarks
│   └── examples/              # Usage examples
│       └── *.rs
├── skiplist/                  # Crate 2: Skip list implementation
│   ├── Cargo.toml
│   ├── lib.rs                 # Main library code
│   ├── README.md              # Crate-specific documentation
│   ├── tests/                 # Integration tests
│   │   └── *.rs
│   ├── examples/              # Usage examples
│   │   ├── basic_usage.rs
│   │   ├── ordered_data.rs
│   │   └── range_queries.rs
│   └── docs/                  # Additional documentation
│       └── TUNING.md
└── [future-crate]/            # Future data structures follow same pattern
```

## Standard Crate Structure

**CRITICAL**: Every crate in this workspace MUST follow this consistent structure:

### Required Files and Directories

```
crate-name/
├── Cargo.toml                 # Crate manifest (use workspace settings)
├── lib.rs                     # Main library implementation
├── README.md                  # Crate-specific technical documentation
├── tests/                     # Integration tests directory
│   ├── <crate_name>.rs        # Main functional test suite
│   └── performance.rs         # Performance benchmarks (optional)
└── examples/                  # Usage examples
    └── *.rs                   # At least one usage example
```

### Optional Directories

```
├── docs/                      # Extended documentation (design docs, tuning guides)
│   └── *.md
└── benches/                   # Criterion benchmarks (if needed beyond tests)
    └── *.rs
```

## Cargo.toml Guidelines

### Workspace-Level (root Cargo.toml)

```toml
[workspace]
resolver = "2"
members = [
    "roaring-bitmap",
    "skiplist",
    # Add new crates here
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Aniruddha Atale <aniruddha.atale@gmail.com>"]
license = "MIT"

[workspace.dependencies]
# Define common dependencies here for reuse
```

### Crate-Level (crate/Cargo.toml)

```toml
[package]
name = "crate-name"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[lib]
name = "crate_name"  # snake_case for Rust module names
path = "lib.rs"

[dependencies]
# Production dependencies - MINIMIZE these
# Prefer zero-dependency implementations where reasonable

[dev-dependencies]
# Test-only dependencies - only if truly needed
```

**Dependency Philosophy**: Keep production dependencies minimal. Zero dependencies is ideal and has been achieved for roaring-bitmap. Only add dependencies when they provide significant value (like `smallvec` in skiplist for performance optimization).

## Build and Test Commands

### Building

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build --package roaring-bitmap
cargo build --package skiplist

# Build with all features
cargo build --all-features

# Release build
cargo build --release
```

### Testing

```bash
# Run all tests in workspace
cargo test

# Run tests for specific crate
cargo test --package roaring-bitmap
cargo test --package skiplist

# Run specific test file
cargo test --test roaring_bitmap
cargo test --test performance

# Run specific test category (by name filter)
cargo test operators
cargo test regression
cargo test set_operations

# Run performance benchmarks (ignored by default)
cargo test --test performance -- --ignored --nocapture

# Run with verbose output
cargo test -- --nocapture

# Run tests in release mode (for realistic performance testing)
cargo test --release
```

### Documentation

```bash
# Generate and open documentation for all crates
cargo doc --open

# Generate docs for specific crate
cargo doc --package roaring-bitmap --open

# Include private items
cargo doc --document-private-items
```

### Code Quality

```bash
# Check code without building
cargo check

# Format code (REQUIRED before commits)
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Run Clippy lints (REQUIRED before commits)
cargo clippy -- -D warnings

# Clippy with all features
cargo clippy --all-features -- -D warnings
```

### Running Examples

```bash
# List all examples
cargo run --example

# Run specific example
cargo run --package skiplist --example basic_usage
cargo run --package skiplist --example ordered_data
cargo run --package skiplist --example range_queries
```

## Rust Code Style Guidelines

### General Principles

1. **Idiomatic Rust**: Follow standard Rust conventions and idioms
2. **Safety First**: Avoid `unsafe` unless absolutely necessary and well-documented
3. **Performance**: Optimize for performance without sacrificing clarity
4. **Documentation**: Every public API must have doc comments with examples
5. **Simplicity**: Prefer simple, readable code over clever optimizations

### Formatting

- **Always** run `cargo fmt` before committing
- Use the default rustfmt configuration
- Maximum line length: 100 characters (rustfmt default)
- Use 4 spaces for indentation (never tabs)

### Naming Conventions

```rust
// Modules and crate names: snake_case
mod my_module;
use my_crate;

// Types, traits, enums: PascalCase
struct MyStruct;
trait MyTrait;
enum MyEnum;

// Functions, methods, variables: snake_case
fn my_function() {}
let my_variable = 42;

// Constants and statics: SCREAMING_SNAKE_CASE
const MAX_SIZE: usize = 1024;
static GLOBAL_STATE: AtomicUsize = AtomicUsize::new(0);

// Generics: Single capital letter or PascalCase
struct Container<T> { }
struct Map<K, V> { }
```

### Documentation Style

**Every public item requires documentation:**

```rust
/// Brief one-line summary.
///
/// More detailed explanation spanning multiple lines if needed.
/// Explain the "why" not just the "what".
///
/// # Examples
///
/// ```
/// use roaring_bitmap::RoaringBitmap;
///
/// let mut bm = RoaringBitmap::new();
/// bm.insert(42);
/// assert!(bm.contains(42));
/// ```
///
/// # Time Complexity
///
/// O(log n) where n is the number of values in the container.
///
/// # Panics
///
/// Describe any panic conditions (if applicable).
///
/// # Safety
///
/// Describe safety invariants (if using unsafe).
pub fn public_function() {
    // Implementation
}
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `Option<T>` for operations that may not return a value
- Document error conditions in doc comments
- Avoid `unwrap()` and `expect()` in library code (acceptable in tests and examples)
- Use descriptive error types (avoid just returning `String`)

### Testing Style

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let mut bitmap = RoaringBitmap::new();

        // Act
        bitmap.insert(42);

        // Assert
        assert!(bitmap.contains(42));
        assert_eq!(bitmap.len(), 1);
    }

    #[test]
    #[should_panic(expected = "expected panic message")]
    fn test_panic_condition() {
        // Test code that should panic
    }
}
```

## Testing Philosophy

### Test Coverage Requirements

Every crate must include:

1. **Unit Tests** - Test individual functions and methods
2. **Integration Tests** - Test public API from consumer perspective
3. **Edge Cases** - Boundary conditions, empty states, maximum values
4. **Regression Tests** - Tests for bugs that have been fixed
5. **Performance Tests** - Benchmark critical operations (can be marked `#[ignore]`)
6. **Documentation Tests** - Examples in doc comments that compile and run

### Test Organization

```rust
// tests/crate_name.rs - Main integration test file
mod basic_operations {
    // Tests for core functionality
}

mod edge_cases {
    // Boundary conditions, empty states, etc.
}

mod operators {
    // Operator overloading tests
}

mod bulk_operations {
    // Batch operations
}

mod regression {
    // Bug fix regression tests
    // Name tests after issue numbers: test_issue_123()
}

// tests/performance.rs - Performance benchmarks
#[test]
#[ignore]  // Ignored by default, run with --ignored flag
fn bench_operation_name() {
    // Benchmark code with timing
}
```

### Test Naming Conventions

- `test_operation_description` - Standard tests
- `test_operation_edge_case` - Edge case tests
- `test_issue_123` - Regression tests for specific issues
- `bench_operation` - Performance benchmarks

### Assertion Best Practices

```rust
// Use specific assertions
assert_eq!(actual, expected);           // Equality
assert_ne!(actual, not_expected);       // Inequality
assert!(condition);                     // Boolean condition

// Use descriptive failure messages when helpful
assert!(bitmap.contains(42), "Bitmap should contain 42 after insertion");

// Test multiple related assertions together
assert_eq!(bitmap.len(), 3);
assert!(bitmap.contains(1));
assert!(bitmap.contains(2));
assert!(bitmap.contains(3));
```

## The Socratic Development Method

When implementing new features or data structures:

### 1. Question Phase
- **Challenge assumptions**: "Why this approach over alternatives?"
- **Explore edge cases**: "What happens when...?"
- **Consider trade-offs**: "What are the performance implications?"
- **Think about API design**: "How will users interact with this?"

### 2. Design Phase
- Sketch out the API surface
- Consider memory layout and performance
- Plan container types and optimization strategies
- Document expected time/space complexity

### 3. Implementation Phase
- Write clear, readable code first (optimize later)
- Add comprehensive inline documentation
- Handle edge cases explicitly
- Consider both ergonomics and performance

### 4. Testing Phase
- Write tests before or alongside implementation
- Cover normal cases, edge cases, and error cases
- Add performance benchmarks for critical operations
- Verify memory usage and allocation patterns

### 5. Review Phase
- Review code for clarity and correctness
- Check documentation completeness
- Verify test coverage
- Profile and optimize hot paths
- Run `cargo clippy` and address all warnings

### 6. Documentation Phase
- Write crate-level README.md
- Add usage examples
- Document design decisions
- Explain performance characteristics

## Commit Message Guidelines

### Format

```
<type>: <subject>

<body>

<footer>
```

### Types

- `feat` - New feature (new data structure, new API)
- `fix` - Bug fix
- `docs` - Documentation changes
- `test` - Adding or updating tests
- `refactor` - Code refactoring without behavior change
- `perf` - Performance improvements
- `style` - Code style/formatting changes
- `chore` - Maintenance tasks, dependency updates

### Subject Line

- Use imperative mood: "Add feature" not "Added feature"
- Capitalize first letter
- No period at the end
- Maximum 50 characters
- Be specific: "Add union operator for RoaringBitmap" not "Add operator"

### Body

- Wrap at 72 characters
- Explain the "why" not just the "what"
- Reference issues/PRs when relevant: "Fixes #123"

### Examples

```
feat: Add skip list data structure

Implement probabilistic skip list with O(log n) operations.
Uses intrusive design pattern for single-allocation efficiency.

Includes:
- Core insert/remove/search operations
- Navigation methods (first, successor)
- 50+ comprehensive tests
- Three usage examples
```

```
fix: Handle overflow in bitmap container cardinality

The cardinality calculation for bitmap containers could overflow
when counting set bits near u16::MAX. Now uses checked arithmetic
and returns correct counts for all edge cases.

Fixes #42
```

```
test: Add regression tests for issue #15

Add tests to prevent recurrence of the iterator state bug
when removing elements during iteration.
```

## When Adding New Crates

### Checklist for New Data Structures

- [ ] Create crate directory following naming convention (kebab-case)
- [ ] Add crate to workspace `members` in root Cargo.toml
- [ ] Create crate Cargo.toml using workspace settings
- [ ] Implement in lib.rs with comprehensive documentation
- [ ] Create crate README.md with:
  - [ ] Overview and use cases
  - [ ] API reference
  - [ ] Time/space complexity analysis
  - [ ] Usage examples
  - [ ] References to academic papers/resources
- [ ] Create tests/ directory with integration tests
- [ ] Create examples/ directory with at least one usage example
- [ ] Aim for zero production dependencies (or justify any additions)
- [ ] Add Socratic dialog notes explaining design decisions
- [ ] Ensure all tests pass: `cargo test --package <crate-name>`
- [ ] Verify documentation builds: `cargo doc --package <crate-name>`
- [ ] Run clippy with no warnings: `cargo clippy --package <crate-name>`
- [ ] Update root README.md to list the new data structure

## Development Process for New Crates

When creating a new crate or adding significant functionality, follow this disciplined process:

### 1. Documentation First: Create README.md

**ALWAYS** start by creating the crate's README.md before writing any code:

- Document the purpose of the crate
- Explain the problem it solves
- Provide relevant background (algorithms, theory, use cases)
- Include references to papers, articles, or existing implementations
- Add usage examples (even if not yet implemented)
- Document expected time/space complexity

**Keep README.md updated** as functionality evolves. The README should always reflect the current state of the crate.

### 2. API Design: Start with the Interface

Before implementation, design the public API that users will interact with:

**Research open source projects:**
- Search GitHub, crates.io, and other repositories for similar implementations
- Study how established libraries expose their APIs
- Learn from both successful and unsuccessful designs
- Note ergonomic patterns and anti-patterns

**Provide API alternatives:**
- Present 2-3 different API design options
- Document pros and cons of each approach
- Consider:
  - Ease of use vs. flexibility
  - Performance implications
  - Memory overhead
  - Type safety guarantees
  - Compatibility with Rust idioms
- Discuss trade-offs with rationale for final choice

**Example:**
```rust
// Option A: Builder pattern (more flexible, slightly more verbose)
let bitmap = RoaringBitmap::builder()
    .add_range(0..100)
    .add(500)
    .build();

// Option B: Direct construction with iterator (more concise)
let bitmap = RoaringBitmap::from_iter(0..100);

// Choose Option B for simplicity, add builder later if needed
```

### 3. Test-Driven Development: Tests Before Implementation

**Write functional tests BEFORE implementing the feature:**

- Start with integration tests that define expected behavior
- Write tests for normal cases, edge cases, and error conditions
- Document test coverage requirements in README.md

**Code Coverage Target: >95%**

Every crate must achieve at least 95% code coverage:
- Use `cargo tarpaulin` or similar tools to measure coverage
- Include coverage reports in CI/CD pipeline
- Document untested code paths with justification

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

### 4. Developer-Friendly Observability

**Provide APIs for introspection and debugging:**

Every crate should offer methods to help developers understand resource usage:

```rust
impl RoaringBitmap {
    /// Returns the memory footprint in bytes
    pub fn memory_usage(&self) -> usize {
        // Calculate actual heap + stack usage
    }

    /// Returns detailed memory statistics
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            heap_bytes: self.heap_size(),
            stack_bytes: std::mem::size_of::<Self>(),
            container_count: self.containers.len(),
            // ...
        }
    }

    /// Returns internal structure for debugging
    pub fn debug_info(&self) -> String {
        // Detailed internal state
    }
}
```

**Considerations:**
- Expose `memory_usage()` or similar methods
- Provide `statistics()` for operational metrics
- Include `debug_info()` for troubleshooting
- Document memory layout in doc comments
- Add examples showing how to profile and optimize

### 5. Document Critical Design Decisions

**Create `/docs` folder within the crate for design documentation:**

Any significant architectural or algorithmic decision must be documented:

```
crate-name/
├── docs/
│   ├── DESIGN.md           # Overall architecture decisions
│   ├── ALGORITHMS.md       # Algorithm choices and complexity analysis
│   ├── TUNING.md           # Performance tuning guide
│   ├── TRADE_OFFS.md       # Design trade-offs made
│   └── MEMORY_LAYOUT.md    # Internal memory organization
```

**What to document:**
- Why one algorithm was chosen over alternatives
- Trade-offs between performance, memory, and complexity
- Any unsafe code usage and safety invariants
- Container type choices (Vec vs. SmallVec vs. custom)
- Optimization decisions and their impact
- Compatibility considerations
- Known limitations or future improvements

**Example structure for DESIGN.md:**
```markdown
# Design Decisions

## Container Choice: SmallVec vs Vec

**Decision:** Use `SmallVec<[T; 4]>` for forward pointers

**Rationale:**
- 90% of skip list nodes have ≤4 levels
- Avoids heap allocation for most nodes
- Measured 15% performance improvement in benchmarks

**Trade-offs:**
- Slightly larger stack size for all nodes
- Dependency on smallvec crate (justified by perf gain)

**Alternatives considered:**
1. `Vec<T>` - simpler but always heap allocates
2. Custom inline array - more complex, similar perf
```

### Summary: Development Workflow

```
1. Create README.md (document purpose, background, API plans)
   ↓
2. Research similar projects (gather API design ideas)
   ↓
3. Design API alternatives (present options with pros/cons)
   ↓
4. Write comprehensive tests (aim for >95% coverage)
   ↓
5. Implement functionality (make tests pass)
   ↓
6. Add observability APIs (memory_usage, statistics, etc.)
   ↓
7. Document design decisions (create /docs with rationale)
   ↓
8. Update README.md (reflect completed functionality)
   ↓
9. Run full test suite + coverage (verify >95% coverage)
   ↓
10. Review and refactor (optimize, cleanup, document)
```

## Security Considerations

### Memory Safety

- Prefer safe Rust; avoid `unsafe` unless necessary
- If using `unsafe`, document safety invariants thoroughly
- Use `cargo miri` for testing unsafe code (if applicable)
- Consider fuzzing for complex parsing or container operations

### Integer Overflow

- Use checked arithmetic for critical calculations
- Consider overflow in capacity and size calculations
- Test with extreme values (0, max values, etc.)

### Resource Exhaustion

- Set reasonable limits on collection sizes
- Consider memory usage in design decisions
- Test behavior with large inputs

## Performance Guidelines

### Optimization Strategy

1. **Measure First**: Profile before optimizing
2. **Hot Path Focus**: Optimize critical paths, not entire codebase
3. **Allocations**: Minimize allocations in tight loops
4. **Cache Locality**: Keep related data close in memory
5. **Zero-Cost Abstractions**: Use Rust's type system for compile-time optimization

### Benchmarking

```rust
#[test]
#[ignore]
fn bench_operation() {
    use std::time::Instant;

    let iterations = 1_000_000;
    let start = Instant::now();

    for i in 0..iterations {
        // Operation to benchmark
    }

    let elapsed = start.elapsed();
    println!("{} iterations in {:?}", iterations, elapsed);
    println!("Average: {:?} per operation", elapsed / iterations);
}
```

For more sophisticated benchmarking, consider adding Criterion:
```toml
[dev-dependencies]
criterion = "0.5"
```

## Common Patterns in This Project

### Container Optimization Pattern (from roaring-bitmap)

```rust
// Use different internal representations based on data characteristics
enum Container {
    Array(ArrayContainer),     // Sparse data
    Bitmap(BitmapContainer),   // Dense data
    Run(RunContainer),         // Consecutive data
}

// Automatically convert between types based on operations
fn optimize(&mut self) {
    // Convert to most efficient representation
}
```

### Intrusive Data Structure Pattern (from skiplist)

```rust
// Embed metadata directly in user's structure
pub struct SkipListNode {
    forward: SmallVec<[Option<NonNull<dyn SkipListEntry<Key = K>>>; 4]>,
}

// User implements trait to provide access
pub trait SkipListEntry {
    type Key: Ord;
    fn key(&self) -> &Self::Key;
    fn skiplist_node(&self) -> &SkipListNode;
    fn skiplist_node_mut(&mut self) -> &mut SkipListNode;
}
```

### Semantic Bulk Operations Pattern

```rust
// Provide optimized methods for known data patterns
fn extend_consecutive(&mut self, range: Range<u32>) {
    // Create Run container directly
}

fn extend_sparse(&mut self, values: impl IntoIterator<Item = u32>) {
    // Create Array container directly
}

fn extend_dense(&mut self, values: impl IntoIterator<Item = u32>) {
    // Choose optimal container based on density
}
```

## IDE and Tooling

### Recommended VS Code Extensions

- `rust-analyzer` - Rust language server
- `crates` - Manage Cargo.toml dependencies
- `Even Better TOML` - TOML syntax highlighting

### Rust Analyzer Settings

```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all"
}
```

### Pre-commit Checklist

Run these commands before every commit:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo doc --no-deps
```

Consider creating a git pre-commit hook:

```bash
#!/bin/sh
# .git/hooks/pre-commit

set -e

echo "Running cargo fmt..."
cargo fmt -- --check

echo "Running cargo clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "Running cargo test..."
cargo test

echo "All checks passed!"
```

## References and Learning Resources

### Rust Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - Unsafe Rust

### Data Structures

- CLRS "Introduction to Algorithms" (3rd/4th Edition)
- Knuth "The Art of Computer Programming"
- Okasaki "Purely Functional Data Structures"

### This Project's Theme

- Douglas Adams' "Hitchhiker's Guide to the Galaxy" series
- Deep Thought: The supercomputer that computed the Answer (42)
- Marvin: The Paranoid Android ("Brain the size of a planet...")

## Philosophy: Deep Thought vs. Marvin

When working on this project, maintain both perspectives:

### Deep Thought Perspective 🧠

- Question design decisions
- Explore alternatives
- Consider edge cases
- Seek elegance and correctness
- Think about API ergonomics
- Document the "why" behind choices

### Marvin Perspective 🤖😔

- Make it work
- Write the tests
- Handle the edge cases
- Fix the bugs
- Deal with reality
- Ship it

**Both are essential.** Theory without implementation ships nothing. Implementation without thought creates technical debt. Together they create robust, well-designed, practical code.

## Troubleshooting

### Common Issues

**Cargo build fails with dependency errors:**
```bash
cargo clean
cargo update
cargo build
```

**Tests hang or timeout:**
- Check for infinite loops in test code
- Look for deadlocks in concurrent code
- Verify test isolation (tests shouldn't depend on each other)

**Clippy warnings:**
- Address all clippy warnings before committing
- Use `#[allow(clippy::...)]` only with good justification and a comment explaining why

**Documentation build fails:**
```bash
# Check for broken doc links
cargo doc --no-deps
# Fix any [broken_intra_doc_links] warnings
```

## Contact and Contribution

- **Author**: Aniruddha Atale (aniruddha.atale@gmail.com)
- **License**: MIT
- **Repository**: This is a learning project - contributions should maintain the Socratic dialog philosophy

When contributing:
1. Read existing implementations to understand patterns
2. Ask questions about design choices
3. Follow the standard crate structure
4. Include comprehensive tests and documentation
5. Maintain the philosophical (and humorous) tone

---

**"The answer to life, the universe, and everything is... a well-tested data structure."** — Deep Thought (probably)

**"Here I am, brain the size of a planet, and they ask me to write an AGENTS.md file. Call that job satisfaction? 'Cause I don't."** — Marvin

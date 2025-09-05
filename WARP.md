# WARP.md - Rust Todo App Learning Project

## Project Overview
- **Name**: rust-todo
- **Type**: CLI Todo Application (Learning Project)
- **Language**: Rust
- **Purpose**: Educational project demonstrating Rust fundamentals through a practical todo app

## Learning Goals
- Understand ownership and borrowing in Rust
- Master error handling with Result and Option types
- Learn file I/O and JSON serialization
- Build command-line interfaces with Clap
- Write idiomatic Rust code with proper testing
- Practice structured logging and error propagation

## Key Concepts Covered
1. **Memory Management**: Ownership, borrowing, and lifetimes
2. **Error Handling**: Result<T, E>, Option<T>, and the ? operator
3. **Data Structures**: Structs, enums, and vectors
4. **Traits**: Derive macros, serialization traits
5. **File I/O**: Reading and writing JSON files
6. **CLI Development**: Argument parsing with Clap
7. **Testing**: Unit tests and integration tests
8. **Documentation**: Doc comments and README

## Project Structure
```
rust-todo/
├── Cargo.toml          # Dependencies and project metadata
├── src/
│   ├── main.rs         # Entry point and command dispatch
│   ├── todo.rs         # Todo data model and structures
│   ├── storage.rs      # File persistence layer
│   ├── cli.rs          # CLI argument definitions
│   └── handlers.rs     # Command implementations
├── tests/
│   └── integration.rs  # Integration tests
├── docs/
│   └── implementation-status.md
└── todos.json          # Data storage (git-ignored)
```

## Dependencies
- **clap**: CLI argument parsing with derive macros
- **serde/serde_json**: JSON serialization/deserialization
- **anyhow**: Simplified error handling
- **tracing/tracing-subscriber**: Structured logging
- **chrono**: Date and time handling

## Commands
```bash
# Add a new todo
cargo run -- add "Learn Rust ownership"

# List all todos
cargo run -- list

# List with filters
cargo run -- list --filter completed
cargo run -- list --filter pending

# Mark todo as complete (by ID)
cargo run -- complete 1

# Delete a todo (by ID)
cargo run -- delete 1

# Show help
cargo run -- --help
```

## Development Workflow

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Formatting and Linting
```bash
# Format code
cargo fmt --all

# Run clippy linter
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation
```bash
# Generate and open documentation
cargo doc --open
```

## Git Workflow (Using Your Aliases)
```bash
# Check status
gst

# Stage changes
ga .
ga src/specific_file.rs

# Commit with conventional commits
gc -m "feat(todo): ✨ Add todo completion feature

- Implement complete command handler
- Add ID validation and error handling
- Update todo status in storage"

# Pull latest changes
gl

# Push to main
gpom
```

## Learning Resources

### Ownership and Borrowing
- The todo.rs module demonstrates String vs &str
- Handlers show mutable and immutable references
- Storage module shows lifetime considerations

### Error Handling
- Result<T, E> pattern throughout the codebase
- Custom error types with anyhow
- The ? operator for error propagation

### Testing Patterns
- Unit tests colocated with code (#[cfg(test)])
- Integration tests in tests/ directory
- Test data fixtures and mocking

## Implementation Status
Track progress in `docs/implementation-status.md`

## Troubleshooting

### Common Issues
1. **Borrow checker errors**: Remember that you can only have one mutable reference OR multiple immutable references
2. **String vs &str**: Use String for owned data, &str for borrowed string slices
3. **Error handling**: Always use ? operator instead of unwrap() in production code
4. **File not found**: Ensure todos.json is created on first run

### Debug Commands
```bash
# Run with debug logging
RUST_LOG=debug cargo run -- list

# Check for compilation errors with detailed output
cargo check --verbose

# Run specific test with output
cargo test test_name -- --nocapture
```

## Best Practices Applied
- ✅ Structured error handling with anyhow
- ✅ Comprehensive documentation comments
- ✅ Unit and integration testing
- ✅ Proper use of Result and Option types
- ✅ Idiomatic Rust patterns
- ✅ Separation of concerns (modules)
- ✅ Type safety with strong typing
- ✅ Proper .gitignore configuration

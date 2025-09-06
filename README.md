# Rust Todo App 🦀

A command-line todo application built in Rust as a learning project. This project demonstrates fundamental Rust concepts including ownership, error handling, file I/O, and CLI development.

## Features

- ✨ Add new todos with unique IDs
- 📋 List todos with filtering options (all/completed/pending)
- ✅ Mark todos as complete
- 🗑️ Delete todos
- 💾 Persistent storage using JSON
- 🎨 Colorful terminal output
- 📝 Comprehensive error handling
- 🧪 Full test coverage
- 🖥️ **NEW: Interactive TUI mode with vim-style navigation**

## Installation

### Prerequisites
- Rust 1.70 or higher
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/joshuadanpeterson/rust-todo.git
cd rust-todo

# Build the project
cargo build --release

# The binary will be available at target/release/rust-todo
```

## Usage

### Add a New Todo
```bash
rust-todo add "Learn Rust ownership concepts"
```

### List All Todos
```bash
rust-todo list
```

### List with Filters
```bash
# Show only completed todos
rust-todo list --filter completed

# Show only pending todos
rust-todo list --filter pending
```

### Mark a Todo as Complete
```bash
# Complete todo with ID 1
rust-todo complete 1
```

### Delete a Todo
```bash
# Delete todo with ID 1
rust-todo delete 1
```

### Get Help
```bash
rust-todo --help
rust-todo <command> --help
```

### Interactive TUI Mode 🖥️
```bash
# Launch the interactive terminal UI
rust-todo tui
# or
rust-todo interactive
```

#### TUI Keyboard Shortcuts:
- **Navigation**: `j/↓` (down), `k/↑` (up), `g` (top), `G` (bottom)
- **Actions**: `i` (add todo), `Enter` (toggle complete), `d` (delete), `e` (edit)
- **Filters**: `f` (cycle filters), `1/2/3` (all/completed/pending)
- **Other**: `h` (help), `q` (quit), `Esc` (cancel)

## Project Structure

```
rust-todo/
├── src/
│   ├── main.rs         # Application entry point
│   ├── todo.rs         # Todo data structures
│   ├── storage.rs      # File persistence
│   ├── cli.rs          # CLI definitions
│   └── handlers.rs     # Command handlers
├── tests/
│   └── integration.rs  # Integration tests
└── Cargo.toml          # Dependencies
```

## Learning Concepts

This project demonstrates:

- **Ownership & Borrowing**: Understanding Rust's memory management
- **Error Handling**: Using `Result<T, E>` and `Option<T>`
- **Structs & Enums**: Building data models
- **Traits**: Serialization with Serde
- **File I/O**: Reading and writing JSON
- **CLI Development**: Building with Clap
- **Testing**: Unit and integration tests

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt --all
```

### Linting
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Building Documentation
```bash
cargo doc --open
```

## Contributing

This is a learning project, but suggestions and improvements are welcome! Feel free to open an issue or submit a pull request.

## License

MIT License - See LICENSE file for details

## Author

Joshua Peterson (@joshuadanpeterson)

## Acknowledgments

- The Rust community for excellent documentation
- Clap for the amazing CLI framework
- Serde for seamless serialization

// src/cli.rs - Command Line Interface Module
// This module defines the CLI structure using Clap's derive macros

use clap::{Parser, Subcommand, ValueEnum};
use crate::todo::TodoFilter;

/// Todo CLI Application
/// 
/// A simple todo list manager built in Rust for learning purposes.
/// 
/// # Key Concepts:
/// 
/// ## Clap Derive Macros
/// - `#[derive(Parser)]`: Automatically generates argument parsing code
/// - Clap reads doc comments (///) as help text
/// - Field names become argument names
/// - Types determine how arguments are parsed
/// 
/// ## Structure-Based CLI Design
/// - The struct represents the entire CLI application
/// - Fields represent global options
/// - Subcommands are defined in a separate enum
#[derive(Parser)]
#[command(name = "rust-todo")]
#[command(author = "Joshua Peterson")]
#[command(version = "1.0")]
#[command(about = "A simple todo list manager", long_about = None)]
pub struct Cli {
    /// The command to execute
    /// 
    /// # Key Concepts:
    /// - `#[command(subcommand)]`: Tells Clap this field contains subcommands
    /// - The type must be an enum with `#[derive(Subcommand)]`
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose output
    /// 
    /// # Key Concepts:
    /// - `#[arg(short, long)]`: Creates both -v and --verbose flags
    /// - bool type makes this a flag (present/absent)
    /// - Optional fields use Option<T>
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// Available commands
/// 
/// # Key Concepts:
/// 
/// ## Enum as Subcommands
/// - Each variant becomes a subcommand
/// - Variant names are converted to kebab-case (AddTodo -> add-todo)
/// - Can override with #[command(name = "...")]
/// 
/// ## Structured Arguments
/// - Each variant can have different fields/arguments
/// - Arguments are type-checked at compile time
/// - No string parsing needed in our code
#[derive(Subcommand)]
pub enum Commands {
    /// Add a new todo item
    /// 
    /// # Example:
    /// ```
    /// rust-todo add "Learn Rust ownership"
    /// ```
    Add {
        /// Description of the todo item
        /// 
        /// # Key Concepts:
        /// - Positional argument (no flag needed)
        /// - String type for text input
        /// - Required (not Option<T>)
        description: String,
        
        /// Priority level for the todo (1-5)
        /// 
        /// # Key Concepts:
        /// - Optional argument with Option<T>
        /// - value_parser validates the range
        /// - short and long flags (-p, --priority)
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=5))]
        priority: Option<u8>,
    },
    
    /// List all todo items
    /// 
    /// # Examples:
    /// ```
    /// rust-todo list
    /// rust-todo list --filter completed
    /// rust-todo list -f pending
    /// ```
    List {
        /// Filter todos by status
        /// 
        /// # Key Concepts:
        /// - Custom enum for filter values
        /// - ValueEnum trait for parsing
        /// - Optional with default behavior
        #[arg(short, long, value_enum)]
        filter: Option<FilterArg>,
        
        /// Show detailed information
        /// 
        /// # Key Concepts:
        /// - Boolean flag for toggling behavior
        /// - Combines well with other options
        #[arg(short = 'd', long)]
        detailed: bool,
    },
    
    /// Mark a todo item as complete
    /// 
    /// # Example:
    /// ```
    /// rust-todo complete 1
    /// ```
    Complete {
        /// ID of the todo to complete
        /// 
        /// # Key Concepts:
        /// - Numeric parsing handled automatically
        /// - Type safety: must be valid u32
        /// - Positional argument
        id: u32,
    },
    
    /// Delete a todo item
    /// 
    /// # Example:
    /// ```
    /// rust-todo delete 1
    /// rust-todo delete 1 --force
    /// ```
    Delete {
        /// ID of the todo to delete
        id: u32,
        
        /// Skip confirmation prompt
        /// 
        /// # Key Concepts:
        /// - Dangerous operations should require confirmation
        /// - --force flag to bypass safety checks
        /// - Common pattern in CLI tools
        #[arg(short, long)]
        force: bool,
    },
    
    /// Clear all completed todos
    /// 
    /// # Example:
    /// ```
    /// rust-todo clear
    /// rust-todo clear --force
    /// ```
    Clear {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show statistics about your todos
    /// 
    /// # Example:
    /// ```
    /// rust-todo stats
    /// ```
    Stats,
    
    /// Export todos to a different format
    /// 
    /// # Example:
    /// ```
    /// rust-todo export --format markdown
    /// ```
    Export {
        /// Export format
        #[arg(short, long, value_enum, default_value = "json")]
        format: ExportFormat,
        
        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Import todos from a file
    /// 
    /// # Example:
    /// ```
    /// rust-todo import todos_backup.json
    /// ```
    Import {
        /// Path to the file to import
        file: String,
        
        /// Merge with existing todos instead of replacing
        #[arg(short, long)]
        merge: bool,
    },
    
    /// Launch interactive TUI mode
    /// 
    /// # Example:
    /// ```
    /// rust-todo tui
    /// rust-todo interactive
    /// ```
    #[command(alias = "interactive")]
    Tui,
}

/// Filter arguments for the list command
/// 
/// # Key Concepts:
/// 
/// ## ValueEnum Trait
/// - Allows enum to be parsed from command line strings
/// - Automatically generates valid values for help text
/// - Case-insensitive by default
/// 
/// ## Mapping to Domain Types
/// - This enum maps to our TodoFilter enum
/// - Separation of concerns: CLI types vs domain types
/// - Allows for different representations
#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum FilterArg {
    /// Show all todos
    All,
    /// Show only completed todos
    Completed,
    /// Show only pending todos
    Pending,
}

// Implement conversion from FilterArg to TodoFilter
// This keeps our CLI types separate from domain types
impl From<FilterArg> for TodoFilter {
    fn from(arg: FilterArg) -> Self {
        match arg {
            FilterArg::All => TodoFilter::All,
            FilterArg::Completed => TodoFilter::Completed,
            FilterArg::Pending => TodoFilter::Pending,
        }
    }
}

/// Export format options
/// 
/// # Key Concepts:
/// - Extensible design: easy to add new formats
/// - Each format has its own serialization logic
#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum ExportFormat {
    /// JSON format (default)
    Json,
    /// Markdown format for documentation
    Markdown,
    /// CSV format for spreadsheets
    Csv,
    /// Plain text format
    Text,
}

/// Validates and processes CLI arguments
/// 
/// # Key Concepts:
/// 
/// ## Entry Point
/// - This function is typically called from main()
/// - Returns parsed arguments or exits with error
/// - Handles --help and --version automatically
pub fn parse_args() -> Cli {
    Cli::parse()
}

/// Helper function to get user confirmation
/// 
/// # Arguments
/// * `prompt` - The question to ask the user
/// 
/// # Returns
/// * `bool` - true if user confirms, false otherwise
/// 
/// # Key Concepts:
/// - Interactive CLI elements
/// - stdin/stdout handling
/// - Error recovery (invalid input)
pub fn get_confirmation(prompt: &str) -> bool {
    use std::io::{self, Write};
    
    print!("{} [y/N]: ", prompt);
    // Flush to ensure prompt appears before input
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    // Check if input starts with 'y' or 'Y'
    matches!(input.trim().to_lowercase().chars().next(), Some('y'))
}

/// Formats a priority value for display
/// 
/// # Key Concepts:
/// - Consistent formatting across the application
/// - Visual indicators for priority levels
pub fn format_priority(priority: Option<u8>) -> String {
    match priority {
        Some(1) => "🔵 Low".to_string(),
        Some(2) => "🟢 Normal".to_string(),
        Some(3) => "🟡 Medium".to_string(),
        Some(4) => "🟠 High".to_string(),
        Some(5) => "🔴 Critical".to_string(),
        Some(p) => format!("Priority {}", p),
        None => "No priority".to_string(),
    }
}

// Unit tests for CLI module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_arg_conversion() {
        assert_eq!(TodoFilter::from(FilterArg::All), TodoFilter::All);
        assert_eq!(TodoFilter::from(FilterArg::Completed), TodoFilter::Completed);
        assert_eq!(TodoFilter::from(FilterArg::Pending), TodoFilter::Pending);
    }
    
    #[test]
    fn test_format_priority() {
        assert_eq!(format_priority(Some(1)), "🔵 Low");
        assert_eq!(format_priority(Some(5)), "🔴 Critical");
        assert_eq!(format_priority(None), "No priority");
    }
    
    // Note: We can't easily test parse_args() in unit tests
    // because it reads from std::env::args()
    // This would be tested in integration tests
    
    #[test]
    fn test_confirmation_parsing() {
        // Test the logic without actual I/O
        let test_cases = vec![
            ("y", true),
            ("Y", true),
            ("yes", true),
            ("YES", true),
            ("n", false),
            ("N", false),
            ("no", false),
            ("", false),
            ("maybe", false),
        ];
        
        for (input, expected) in test_cases {
            let result = matches!(input.to_lowercase().chars().next(), Some('y'));
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
}

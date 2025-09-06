// Module declarations
// These tell Rust to include these files as part of our program
// pub makes them accessible to integration tests
pub mod todo;
pub mod storage;
pub mod cli;
pub mod handlers;
pub mod tui;

// Import necessary items
use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};
use tracing::{info, error};

use cli::parse_args;
use handlers::handle_command;

/// Main entry point of the application
/// 
/// # Key Rust Concepts:
/// 
/// ## Result Type in Main
/// - main() can return Result<(), Error>
/// - Allows using ? operator in main
/// - Automatically prints errors on failure
/// 
/// ## Error Handling Philosophy
/// - Errors bubble up from handlers
/// - Context added at each level
/// - Final error displayed to user
/// 
/// ## Logging vs Printing
/// - println! for user-facing output
/// - tracing for debugging/monitoring
/// - Controlled by RUST_LOG environment variable
fn main() -> Result<()> {
    // Initialize the tracing subscriber for logging
    // This sets up structured logging throughout the application
    init_tracing();
    
    // Log application start
    info!("Starting rust-todo application");
    
    // Parse command-line arguments
    // This will exit with help/error if arguments are invalid
    let cli = parse_args();
    
    // Enable debug logging if verbose flag is set
    if cli.verbose {
        tracing::subscriber::set_global_default(
            fmt::Subscriber::builder()
                .with_env_filter(EnvFilter::new("debug"))
                .finish()
        ).expect("Failed to set verbose logging");
        info!("Verbose mode enabled");
    }
    
    // Handle the command
    // Errors will bubble up and be displayed
    match handle_command(cli.command) {
        Ok(()) => {
            info!("Command completed successfully");
        }
        Err(e) => {
            error!("Command failed: {:?}", e);
            // Re-throw the error so main returns it
            return Err(e);
        }
    }
    
    Ok(())
}

/// Initializes the tracing subscriber for structured logging
/// 
/// # Key Concepts:
/// 
/// ## Environment-based Configuration
/// - RUST_LOG controls log level
/// - Default to "info" if not set
/// - Examples: RUST_LOG=debug, RUST_LOG=rust_todo=trace
/// 
/// ## Structured Logging
/// - Better than println! for debugging
/// - Can be filtered by module/level
/// - Includes timestamps and source location
fn init_tracing() {
    // Try to get filter from environment, default to "info"
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    // Initialize the subscriber
    fmt()
        .with_env_filter(filter)
        .with_target(false) // Don't show module paths in output
        .with_thread_ids(false) // Don't show thread IDs
        .with_file(false) // Don't show source file
        .with_line_number(false) // Don't show line numbers
        .compact() // Use compact formatting
        .init();
}

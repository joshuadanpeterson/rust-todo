// tests/integration.rs - Integration Tests
// These tests verify the entire application works end-to-end

// Import necessary items from the main crate
// The crate name comes from Cargo.toml's [package] name
use rust_todo::cli::{Commands, ExportFormat};
use rust_todo::handlers::handle_command;
use rust_todo::storage::{load_todos, save_todos};
use rust_todo::todo::{TodoFilter, TodoList};

use anyhow::Result;
use std::fs;
use std::path::Path;

// Test-specific storage file to avoid conflicts
const TEST_STORAGE_FILE: &str = "test_todos.json";

/// Helper function to clean up test files
///
/// # Key Testing Concepts:
/// - Each test should start with a clean state
/// - Clean up after tests to avoid side effects
/// - Use different file names for test vs production
fn cleanup_test_files() {
    let _ = fs::remove_file(TEST_STORAGE_FILE);
    let _ = fs::remove_file("todos.json");
    let _ = fs::remove_file("test_export.json");
    let _ = fs::remove_file("test_export.md");
    let _ = fs::remove_file("test_export.csv");
}

/// Helper to set up a test environment
///
/// # Key Concepts:
/// - Test fixtures: Predefined test data
/// - Isolation: Each test gets its own environment
/// - Reproducibility: Same setup every time
fn setup_test_todos() -> TodoList {
    let mut todos = TodoList::new();
    todos.add_todo("Test todo 1".to_string(), None);
    todos.add_todo("Test todo 2".to_string(), Some(3));
    todos.add_todo("Test todo 3".to_string(), Some(5));
    todos
}

// Integration test module
// Tests are in a separate binary from the main application
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test the complete add -> list -> complete -> delete workflow
    ///
    /// # Key Testing Concepts:
    /// - End-to-end testing: Test the full user workflow
    /// - State verification: Check state after each operation
    /// - Multiple assertions: Verify all aspects of the operation
    #[test]
    fn test_complete_workflow() -> Result<()> {
        cleanup_test_files();

        // Add a todo
        handle_command(Commands::Add {
            description: "Integration test todo".to_string(),
            priority: Some(3),
        })?;

        // Verify it was added
        let todos = load_todos()?;
        assert_eq!(todos.todos.len(), 1);
        assert_eq!(todos.todos[0].description, "Integration test todo");
        assert_eq!(todos.todos[0].priority, Some(3));
        assert!(!todos.todos[0].completed);

        // Complete the todo
        let id = todos.todos[0].id;
        handle_command(Commands::Complete { id })?;

        // Verify it was completed
        let todos = load_todos()?;
        assert!(todos.todos[0].completed);
        assert!(todos.todos[0].completed_at.is_some());

        // Delete the todo
        handle_command(Commands::Delete { id, force: true })?;

        // Verify it was deleted
        let todos = load_todos()?;
        assert_eq!(todos.todos.len(), 0);

        cleanup_test_files();
        Ok(())
    }

    /// Test filtering functionality
    ///
    /// # Key Concepts:
    /// - Setup test data with known state
    /// - Test each filter option
    /// - Verify correct items are returned
    #[test]
    fn test_filtering() -> Result<()> {
        cleanup_test_files();

        // Set up test data
        let mut todos = setup_test_todos();

        // Complete one todo
        if let Some(todo) = todos.todos.get_mut(0) {
            todo.complete();
        }

        save_todos(&todos)?;

        // Test All filter
        let all = todos.filter_todos(TodoFilter::All);
        assert_eq!(all.len(), 3);

        // Test Completed filter
        let completed = todos.filter_todos(TodoFilter::Completed);
        assert_eq!(completed.len(), 1);
        assert!(completed[0].completed);

        // Test Pending filter
        let pending = todos.filter_todos(TodoFilter::Pending);
        assert_eq!(pending.len(), 2);
        assert!(pending.iter().all(|t| !t.completed));

        cleanup_test_files();
        Ok(())
    }

    /// Test priority handling
    ///
    /// # Key Concepts:
    /// - Edge cases: Test boundary values
    /// - Optional values: Test with and without
    #[test]
    fn test_priority_handling() -> Result<()> {
        cleanup_test_files();

        // Add todos with various priorities
        handle_command(Commands::Add {
            description: "No priority".to_string(),
            priority: None,
        })?;

        handle_command(Commands::Add {
            description: "Low priority".to_string(),
            priority: Some(1),
        })?;

        handle_command(Commands::Add {
            description: "High priority".to_string(),
            priority: Some(5),
        })?;

        let todos = load_todos()?;
        assert_eq!(todos.todos.len(), 3);

        // Verify priorities
        assert_eq!(todos.todos[0].priority, None);
        assert_eq!(todos.todos[1].priority, Some(1));
        assert_eq!(todos.todos[2].priority, Some(5));

        cleanup_test_files();
        Ok(())
    }

    /// Test export functionality
    ///
    /// # Key Concepts:
    /// - File I/O testing: Verify files are created
    /// - Content validation: Check exported data
    /// - Multiple formats: Test each export type
    #[test]
    fn test_export_formats() -> Result<()> {
        cleanup_test_files();

        // Set up test data
        let todos = setup_test_todos();
        save_todos(&todos)?;

        // Test JSON export
        handle_command(Commands::Export {
            format: ExportFormat::Json,
            output: Some("test_export.json".to_string()),
        })?;
        assert!(Path::new("test_export.json").exists());

        // Verify JSON content is valid
        let json_content = fs::read_to_string("test_export.json")?;
        let parsed: TodoList = serde_json::from_str(&json_content)?;
        assert_eq!(parsed.todos.len(), 3);

        // Test Markdown export
        handle_command(Commands::Export {
            format: ExportFormat::Markdown,
            output: Some("test_export.md".to_string()),
        })?;
        assert!(Path::new("test_export.md").exists());

        // Verify Markdown contains expected content
        let md_content = fs::read_to_string("test_export.md")?;
        assert!(md_content.contains("# Todo List"));
        assert!(md_content.contains("Test todo 1"));

        // Test CSV export
        handle_command(Commands::Export {
            format: ExportFormat::Csv,
            output: Some("test_export.csv".to_string()),
        })?;
        assert!(Path::new("test_export.csv").exists());

        // Verify CSV has header
        let csv_content = fs::read_to_string("test_export.csv")?;
        assert!(csv_content.starts_with("ID,Description,Priority,Completed,Created,Completed At"));

        cleanup_test_files();
        Ok(())
    }

    /// Test import functionality
    ///
    /// # Key Concepts:
    /// - Round-trip testing: Export then import
    /// - Data integrity: Ensure nothing is lost
    /// - Merge vs replace: Test both modes
    #[test]
    fn test_import() -> Result<()> {
        cleanup_test_files();

        // Create and save initial todos
        let original = setup_test_todos();
        save_todos(&original)?;

        // Export to JSON
        handle_command(Commands::Export {
            format: ExportFormat::Json,
            output: Some("test_export.json".to_string()),
        })?;

        // Clear current todos
        let empty = TodoList::new();
        save_todos(&empty)?;

        // Import back
        handle_command(Commands::Import {
            file: "test_export.json".to_string(),
            merge: false,
        })?;

        // Verify todos were restored
        let imported = load_todos()?;
        assert_eq!(imported.todos.len(), 3);
        assert_eq!(imported.todos[0].description, "Test todo 1");

        cleanup_test_files();
        Ok(())
    }

    /// Test clear command
    ///
    /// # Key Concepts:
    /// - Selective operations: Only affect completed todos
    /// - State preservation: Pending todos remain
    #[test]
    fn test_clear_completed() -> Result<()> {
        cleanup_test_files();

        // Set up todos with mixed states
        let mut todos = setup_test_todos();
        todos.todos[0].complete();
        todos.todos[1].complete();
        save_todos(&todos)?;

        // Clear completed todos
        handle_command(Commands::Clear { force: true })?;

        // Verify only pending todos remain
        let remaining = load_todos()?;
        assert_eq!(remaining.todos.len(), 1);
        assert!(!remaining.todos[0].completed);
        assert_eq!(remaining.todos[0].description, "Test todo 3");

        cleanup_test_files();
        Ok(())
    }

    /// Test error handling
    ///
    /// # Key Concepts:
    /// - Negative testing: Test error cases
    /// - Error propagation: Ensure errors bubble up
    /// - Graceful failure: App should handle errors well
    #[test]
    fn test_error_handling() {
        cleanup_test_files();

        // Try to complete non-existent todo
        let result = handle_command(Commands::Complete { id: 999 });
        assert!(result.is_err());

        // Try to delete non-existent todo
        let result = handle_command(Commands::Delete {
            id: 999,
            force: true,
        });
        assert!(result.is_err());

        // Try to add empty description
        let result = handle_command(Commands::Add {
            description: "".to_string(),
            priority: None,
        });
        assert!(result.is_err());

        // Try to import non-existent file
        let result = handle_command(Commands::Import {
            file: "non_existent.json".to_string(),
            merge: false,
        });
        assert!(result.is_err());

        cleanup_test_files();
    }

    /// Test persistence across sessions
    ///
    /// # Key Concepts:
    /// - Session persistence: Data survives restarts
    /// - ID continuity: IDs continue from where they left off
    #[test]
    fn test_persistence() -> Result<()> {
        cleanup_test_files();

        // First "session" - add todos
        handle_command(Commands::Add {
            description: "First session todo".to_string(),
            priority: None,
        })?;

        let first_load = load_todos()?;
        let first_id = first_load.todos[0].id;
        let next_id = first_load.next_id;

        // Second "session" - load and add more
        let second_load = load_todos()?;
        assert_eq!(second_load.todos.len(), 1);
        assert_eq!(second_load.todos[0].id, first_id);
        assert_eq!(second_load.next_id, next_id);

        handle_command(Commands::Add {
            description: "Second session todo".to_string(),
            priority: None,
        })?;

        // Verify IDs are sequential
        let final_load = load_todos()?;
        assert_eq!(final_load.todos.len(), 2);
        assert_eq!(final_load.todos[1].id, next_id);

        cleanup_test_files();
        Ok(())
    }

    /// Test concurrent operations (basic)
    ///
    /// # Key Concepts:
    /// - Race conditions: Multiple operations in sequence
    /// - Data consistency: Final state should be correct
    ///
    /// Note: This is a simple test. Real concurrent testing
    /// would require threads and more complex synchronization
    #[test]
    fn test_rapid_operations() -> Result<()> {
        cleanup_test_files();

        // Rapidly add multiple todos
        for i in 1..=5 {
            handle_command(Commands::Add {
                description: format!("Rapid todo {}", i),
                priority: None,
            })?;
        }

        let todos = load_todos()?;
        assert_eq!(todos.todos.len(), 5);

        // Verify all todos have unique IDs
        let mut ids: Vec<u32> = todos.todos.iter().map(|t| t.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 5, "All IDs should be unique");

        cleanup_test_files();
        Ok(())
    }
}

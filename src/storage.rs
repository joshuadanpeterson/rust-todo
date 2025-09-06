// src/storage.rs - File Storage Module
// This module handles persisting todos to disk and loading them back

use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use tracing::{debug, info, warn};

// Import our Todo types from the todo module
use crate::todo::TodoList;

/// The file where we store our todos
/// 
/// # Key Concepts:
/// - `const`: Compile-time constant (value must be known at compile time)
/// - `&str`: String slice - a view into string data
/// - Constants are typically UPPER_SNAKE_CASE in Rust
const STORAGE_FILE: &str = "todos.json";

/// Saves the todo list to a JSON file
/// 
/// # Arguments
/// * `todos` - Reference to the TodoList to save
/// 
/// # Returns
/// * `Result<()>` - Ok(()) on success, or an error
/// 
/// # Key Rust Concepts:
/// 
/// ## Result<T, E> Type
/// - Result is an enum with two variants: Ok(T) and Err(E)
/// - Used for operations that can fail
/// - Forces you to handle errors explicitly
/// - `Result<()>` means Ok contains nothing (unit type)
/// 
/// ## The ? Operator
/// - Unwraps Ok values or returns early with Err
/// - Can only be used in functions that return Result or Option
/// - Makes error handling much cleaner than match statements
/// 
/// ## References and Borrowing
/// - `&TodoList` borrows the todo list without taking ownership
/// - The caller keeps ownership and can use it after this function
pub fn save_todos(todos: &TodoList) -> Result<()> {
    // Log what we're doing (debug level)
    debug!("Saving {} todos to {}", todos.todos.len(), STORAGE_FILE);
    
    // Serialize the todos to JSON
    // serde_json::to_string_pretty creates formatted JSON for readability
    let json = serde_json::to_string_pretty(todos)
        // .context() adds context to errors for better debugging
        // This is from the anyhow crate
        .context("Failed to serialize todos to JSON")?;
    
    // Write the JSON to file
    // fs::write creates or overwrites the file atomically
    fs::write(STORAGE_FILE, json)
        .context("Failed to write todos to file")?;
    
    info!("Successfully saved {} todos", todos.todos.len());
    
    // Return Ok with unit type ()
    // () is Rust's unit type, similar to void in other languages
    Ok(())
}

/// Loads the todo list from a JSON file
/// 
/// # Returns
/// * `Result<TodoList>` - The loaded TodoList or an error
/// 
/// # Key Concepts:
/// 
/// ## Path Handling
/// - Path::new() creates a Path from a string
/// - Path provides cross-platform file system operations
/// - .exists() checks if the file exists without opening it
/// 
/// ## Error Recovery
/// - We return an empty TodoList if the file doesn't exist
/// - This is a design choice - first run shouldn't be an error
/// 
/// ## String vs Vec<u8>
/// - fs::read_to_string() reads the file as UTF-8 text
/// - fs::read() would read as raw bytes (Vec<u8>)
pub fn load_todos() -> Result<TodoList> {
    // Create a Path object for cross-platform compatibility
    let path = Path::new(STORAGE_FILE);
    
    // Check if the file exists
    // If not, return an empty TodoList (not an error)
    if !path.exists() {
        info!("No existing todo file found, starting with empty list");
        return Ok(TodoList::new());
    }
    
    debug!("Loading todos from {}", STORAGE_FILE);
    
    // Read the file contents as a string
    // This can fail if:
    // - File permissions deny access
    // - File is not valid UTF-8
    // - I/O error occurs
    let contents = fs::read_to_string(path)
        .context("Failed to read todo file")?;
    
    // Parse the JSON into a TodoList
    // serde_json handles the deserialization based on our derive macros
    let todos: TodoList = serde_json::from_str(&contents)
        .context("Failed to parse todo JSON")?;
    
    info!("Successfully loaded {} todos", todos.todos.len());
    
    Ok(todos)
}

/// Ensures the storage file exists with an empty list
/// 
/// This is useful for initialization
/// 
/// # Key Concepts:
/// 
/// ## Idempotency
/// - This function can be called multiple times safely
/// - If the file exists, it does nothing
/// - If it doesn't exist, it creates it
pub fn ensure_storage_exists() -> Result<()> {
    let path = Path::new(STORAGE_FILE);
    
    if !path.exists() {
        debug!("Creating initial storage file");
        let empty_list = TodoList::new();
        save_todos(&empty_list)?;
    }
    
    Ok(())
}

/// Deletes the storage file (useful for testing or reset)
/// 
/// # Key Concepts:
/// 
/// ## Error Handling Patterns
/// - We use if let Ok() to ignore errors when file doesn't exist
/// - This is intentional - deleting a non-existent file is success
/// - Alternative would be match with explicit error handling
pub fn delete_storage() -> Result<()> {
    let path = Path::new(STORAGE_FILE);
    
    if path.exists() {
        fs::remove_file(path)
            .context("Failed to delete storage file")?;
        warn!("Deleted storage file");
    }
    
    Ok(())
}

/// Gets information about the storage file
/// 
/// # Returns
/// * `Option<StorageInfo>` - Information about the file if it exists
/// 
/// # Key Concepts:
/// 
/// ## Metadata
/// - File metadata includes size, permissions, timestamps
/// - Accessing metadata doesn't require opening the file
/// - More efficient than reading the file to check size
pub fn get_storage_info() -> Option<StorageInfo> {
    let path = Path::new(STORAGE_FILE);
    
    if !path.exists() {
        return None;
    }
    
    // Get file metadata
    // We use .ok()? to convert Result to Option
    // If metadata fails, we return None
    let metadata = fs::metadata(path).ok()?;
    
    Some(StorageInfo {
        file_size: metadata.len(),
        file_path: STORAGE_FILE.to_string(),
    })
}

/// Information about the storage file
#[derive(Debug)]
pub struct StorageInfo {
    /// Size of the file in bytes
    pub file_size: u64,
    /// Path to the file
    pub file_path: String,
}

// Unit tests for the storage module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::TodoList;
    use std::fs;
    use std::sync::Mutex;
    
    // Use a mutex to ensure tests don't interfere with each other
    // This is necessary because all tests share the same file
    static TEST_MUTEX: Mutex<()> = Mutex::new(());
    
    // Helper function to clean up test files
    fn cleanup_test_file() {
        let _ = fs::remove_file(STORAGE_FILE);
    }
    
    #[test]
    fn test_save_and_load_empty_list() {
        let _guard = TEST_MUTEX.lock().unwrap();
        cleanup_test_file();
        
        // Create and save an empty list
        let todos = TodoList::new();
        save_todos(&todos).expect("Failed to save");
        
        // Load it back
        let loaded = load_todos().expect("Failed to load");
        assert_eq!(loaded.todos.len(), 0);
        assert_eq!(loaded.next_id, 1); // Check next_id is preserved
        
        cleanup_test_file();
    }
    
    #[test]
    fn test_save_and_load_with_todos() {
        let _guard = TEST_MUTEX.lock().unwrap();
        cleanup_test_file();
        
        // Create a list with some todos
        let mut todos = TodoList::new();
        todos.add_todo("Test todo 1".to_string());
        todos.add_todo("Test todo 2".to_string());
        
        // Save it
        save_todos(&todos).expect("Failed to save");
        
        // Load it back
        let loaded = load_todos().expect("Failed to load");
        assert_eq!(loaded.todos.len(), 2);
        assert_eq!(loaded.todos[0].description, "Test todo 1");
        assert_eq!(loaded.todos[1].description, "Test todo 2");
        assert_eq!(loaded.next_id, 3); // Next ID should be 3 after adding 2 todos
        
        cleanup_test_file();
    }
    
    #[test]
    fn test_load_nonexistent_file() {
        let _guard = TEST_MUTEX.lock().unwrap();
        cleanup_test_file();
        
        // Should return empty list, not error
        let todos = load_todos().expect("Should handle missing file");
        assert_eq!(todos.todos.len(), 0);
    }
    
    #[test]
    fn test_ensure_storage_exists() {
        let _guard = TEST_MUTEX.lock().unwrap();
        cleanup_test_file();
        
        // Ensure storage exists
        ensure_storage_exists().expect("Failed to ensure storage");
        
        // File should now exist
        assert!(Path::new(STORAGE_FILE).exists());
        
        // Should be able to load an empty list
        let todos = load_todos().expect("Failed to load");
        assert_eq!(todos.todos.len(), 0);
        
        cleanup_test_file();
    }
    
    #[test]
    fn test_storage_info() {
        let _guard = TEST_MUTEX.lock().unwrap();
        cleanup_test_file();
        
        // No info when file doesn't exist
        assert!(get_storage_info().is_none());
        
        // Create a file with some todos
        let mut todos = TodoList::new();
        todos.add_todo("Test".to_string());
        save_todos(&todos).expect("Failed to save");
        
        // Now we should get info
        let info = get_storage_info().expect("Should have info");
        assert!(info.file_size > 0);
        assert_eq!(info.file_path, STORAGE_FILE);
        
        cleanup_test_file();
    }
    
    #[test]
    fn test_delete_storage() {
        let _guard = TEST_MUTEX.lock().unwrap();
        cleanup_test_file();
        
        // Create a file
        let todos = TodoList::new();
        save_todos(&todos).expect("Failed to save");
        assert!(Path::new(STORAGE_FILE).exists());
        
        // Delete it
        delete_storage().expect("Failed to delete");
        assert!(!Path::new(STORAGE_FILE).exists());
        
        // Deleting again should not error
        delete_storage().expect("Should handle missing file");
    }
}

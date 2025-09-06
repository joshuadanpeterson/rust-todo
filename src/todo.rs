// src/todo.rs - Todo Data Model
// This module defines the core data structures for our todo application

// We need to import these traits from the serde crate
// 'use' statements bring items into scope
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A single Todo item
/// 
/// This struct represents a todo task with all its associated data.
/// We use the `derive` macro to automatically implement traits.
/// 
/// # Key Rust Concepts:
/// 
/// ## Ownership and String vs &str
/// - `String`: An owned, heap-allocated, growable UTF-8 string
/// - `&str`: A borrowed string slice (reference to string data)
/// - We use `String` here because each Todo owns its data
/// 
/// ## Derive Macros
/// The `#[derive(...)]` attribute automatically implements traits for us:
/// - `Debug`: Allows us to print the struct with {:?} for debugging
/// - `Clone`: Creates a deep copy of the struct
/// - `Serialize/Deserialize`: Converts to/from JSON (from serde)
/// - `PartialEq`: Allows comparison with == operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    /// Unique identifier for the todo
    /// We use u32 (unsigned 32-bit integer) for simplicity
    /// In a real app, you might use UUID
    pub id: u32,
    
    /// The todo title (short summary)
    /// String is owned by this Todo instance
    /// When the Todo is dropped, the String is freed
    pub description: String,
    
    /// Detailed description/notes for the todo
    /// Optional - not all todos need detailed descriptions
    pub details: Option<String>,
    
    /// Whether the todo has been completed
    /// bool is a primitive type (true/false)
    pub completed: bool,
    
    /// When the todo was created
    /// DateTime<Utc> represents a timestamp in UTC timezone
    /// The <Utc> is a generic type parameter
    pub created_at: DateTime<Utc>,
    
    /// When the todo was completed (if applicable)
    /// Option<T> is Rust's way of handling nullable values
    /// - Some(value): Contains a value
    /// - None: No value present
    /// This prevents null pointer errors at compile time!
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Due date for the todo
    /// Optional - not all todos have due dates
    pub due_date: Option<DateTime<Utc>>,
    
    /// Priority level (1-5, where 5 is highest)
    /// Optional field - not all todos need priorities
    pub priority: Option<u8>,
}

// Implementation block for Todo
// This is where we define methods (functions associated with the struct)
impl Todo {
    /// Creates a new Todo with the given description
    /// 
    /// # Arguments
    /// * `description` - The todo text
    /// * `priority` - Optional priority level (1-5)
    /// 
    /// # Returns
    /// A new Todo instance with a generated ID
    /// 
    /// # Key Concepts:
    /// - `pub fn`: Public function accessible from outside the module
    /// - `String` parameter: Takes ownership of the description
    /// - `Self`: Refers to the type we're implementing (Todo)
    pub fn new(id: u32, description: String, priority: Option<u8>) -> Self {
        // 'Self' is shorthand for 'Todo' within impl blocks
        Self {
            id,
            // Field init shorthand: when variable name matches field name
            description,
            details: None,      // No detailed description initially
            completed: false,
            created_at: Utc::now(),
            completed_at: None, // No completion time initially
            due_date: None,     // No due date initially
            priority,
        }
    }
    
    /// Creates a new Todo with all fields
    pub fn new_with_details(
        id: u32,
        description: String,
        details: Option<String>,
        due_date: Option<DateTime<Utc>>,
        priority: Option<u8>,
    ) -> Self {
        Self {
            id,
            description,
            details,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
            due_date,
            priority,
        }
    }
    
    /// Marks the todo as complete
    /// 
    /// # Key Concepts:
    /// - `&mut self`: Mutable reference to self
    ///   - & means we're borrowing, not taking ownership
    ///   - mut means we can modify the borrowed value
    /// - This allows us to modify the Todo without consuming it
    pub fn complete(&mut self) {
        self.completed = true;
        self.completed_at = Some(Utc::now());
    }
    
    /// Checks if the todo is overdue
    pub fn is_overdue(&self) -> bool {
        if self.completed {
            return false;
        }
        
        if let Some(due) = self.due_date {
            due < Utc::now()
        } else {
            false
        }
    }
    
    /// Checks if the todo is due soon (within 24 hours)
    pub fn is_due_soon(&self) -> bool {
        if self.completed || self.is_overdue() {
            return false;
        }
        
        if let Some(due) = self.due_date {
            let hours_until_due = (due - Utc::now()).num_hours();
            hours_until_due >= 0 && hours_until_due <= 24
        } else {
            false
        }
    }
    
    /// Gets a formatted due date string
    pub fn format_due_date(&self) -> Option<String> {
        self.due_date.map(|date| {
            let today = Utc::now().date_naive();
            let due_date = date.date_naive();
            
            if due_date == today {
                "Today".to_string()
            } else if due_date == today.succ_opt().unwrap() {
                "Tomorrow".to_string()
            } else {
                date.format("%b %d").to_string()
            }
        })
    }
}

/// A collection of todos
/// 
/// This wrapper struct manages multiple todos and provides
/// convenient methods for common operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    /// Vec<T> is Rust's dynamic array (vector)
    /// It can grow and shrink at runtime
    /// The todos are owned by this TodoList
    pub todos: Vec<Todo>,
    
    /// Counter for generating unique IDs
    /// We track this separately to ensure uniqueness
    /// We need to serialize this to maintain ID continuity across sessions
    pub next_id: u32,
}

impl TodoList {
    /// Creates a new, empty TodoList
    /// 
    /// # Key Concepts:
    /// - No parameters needed, so no parentheses after 'new'
    /// - Returns an owned TodoList instance
    pub fn new() -> Self {
        Self {
            todos: Vec::new(), // Create an empty vector
            next_id: 1,        // Start IDs at 1
        }
    }
    
    /// Adds a new todo to the list
    /// 
    /// # Arguments
    /// * `description` - The todo text
    /// * `priority` - Optional priority level
    /// 
    /// # Returns
    /// The ID of the newly created todo
    /// 
    /// # Key Concepts:
    /// - `&mut self`: We need to modify the list
    /// - `String` parameter: Takes ownership of the description
    /// - The todo is moved into the vector (ownership transfer)
    pub fn add_todo(&mut self, description: String, priority: Option<u8>) -> u32 {
        let todo = Todo::new(self.next_id, description, priority);
        let id = todo.id;
        
        // push() adds an element to the end of the vector
        // The todo is moved into the vector (ownership transferred)
        self.todos.push(todo);
        
        // Increment the ID counter for next time
        self.next_id += 1;
        
        // Return the ID of the todo we just added
        id
    }
    
    /// Finds a todo by ID and returns a mutable reference to it
    /// 
    /// # Returns
    /// - `Option<&mut Todo>`: Either Some(reference) or None
    /// 
    /// # Key Concepts:
    /// - Return type `Option<&mut Todo>` handles the case where ID doesn't exist
    /// - `iter_mut()`: Creates an iterator of mutable references
    /// - `find()`: Returns the first element matching the condition
    /// - Closure `|todo| todo.id == id`: Anonymous function for filtering
    pub fn find_todo_mut(&mut self, id: u32) -> Option<&mut Todo> {
        self.todos.iter_mut().find(|todo| todo.id == id)
    }
    
    /// Removes a todo by ID
    /// 
    /// # Returns
    /// - `bool`: true if todo was found and removed, false otherwise
    /// 
    /// # Key Concepts:
    /// - `retain()`: Keeps only elements that match the condition
    /// - The closure returns true for todos we want to keep
    /// - This is more idiomatic than finding index and removing
    pub fn remove_todo(&mut self, id: u32) -> bool {
        let original_len = self.todos.len();
        
        // Keep all todos except the one with matching ID
        self.todos.retain(|todo| todo.id != id);
        
        // If length changed, we removed something
        self.todos.len() < original_len
    }
    
    /// Gets all todos matching a filter
    /// 
    /// # Arguments
    /// * `filter` - The filter to apply
    /// 
    /// # Returns
    /// A vector of references to matching todos
    /// 
    /// # Key Concepts:
    /// - Returns `Vec<&Todo>`: Vector of borrowed references
    /// - References allow multiple parts of code to read the same data
    /// - `collect()`: Transforms an iterator into a collection
    pub fn filter_todos(&self, filter: TodoFilter) -> Vec<&Todo> {
        self.todos
            .iter() // Create an iterator over references
            .filter(|todo| match filter {
                // Pattern matching: a powerful Rust feature
                // Each arm of the match must cover a possible value
                TodoFilter::All => true,
                TodoFilter::Completed => todo.completed,
                TodoFilter::Pending => !todo.completed,
            })
            .collect() // Collect iterator results into a Vec
    }
}

/// Filter options for listing todos
/// 
/// # Key Concepts:
/// - `enum`: Defines a type that can be one of several variants
/// - Each variant is a possible value of the enum
/// - Enums are great for representing a fixed set of options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TodoFilter {
    All,
    Completed,
    Pending,
}

// Implement Default trait for TodoList
// This allows TodoList::default() to create a new instance
impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for the todo module
// Tests are included in the same file but only compiled in test mode
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_todo() {
        let todo = Todo::new(1, "Learn Rust".to_string(), None);
        assert_eq!(todo.id, 1);
        assert_eq!(todo.description, "Learn Rust");
        assert!(!todo.completed);
        assert!(todo.completed_at.is_none());
        assert!(todo.priority.is_none());
    }
    
    #[test]
    fn test_create_todo_with_priority() {
        let todo = Todo::new(1, "Important task".to_string(), Some(5));
        assert_eq!(todo.priority, Some(5));
    }
    
    #[test]
    fn test_complete_todo() {
        let mut todo = Todo::new(1, "Learn Rust".to_string(), None);
        todo.complete();
        assert!(todo.completed);
        assert!(todo.completed_at.is_some());
    }
    
    #[test]
    fn test_todo_list_add() {
        let mut list = TodoList::new();
        let id = list.add_todo("First todo".to_string(), None);
        assert_eq!(id, 1);
        assert_eq!(list.todos.len(), 1);
        
        let id2 = list.add_todo("Second todo".to_string(), Some(3));
        assert_eq!(id2, 2);
        assert_eq!(list.todos.len(), 2);
        assert_eq!(list.todos[1].priority, Some(3));
    }
    
    #[test]
    fn test_todo_list_find_and_complete() {
        let mut list = TodoList::new();
        let id = list.add_todo("Test todo".to_string(), None);
        
        // Find and complete the todo
        if let Some(todo) = list.find_todo_mut(id) {
            todo.complete();
        }
        
        // Verify it's completed
        let todo = list.find_todo_mut(id).expect("Todo should exist");
        assert!(todo.completed);
    }
    
    #[test]
    fn test_todo_list_remove() {
        let mut list = TodoList::new();
        let id = list.add_todo("To be removed".to_string(), None);
        
        assert!(list.remove_todo(id));
        assert_eq!(list.todos.len(), 0);
        assert!(!list.remove_todo(id)); // Should return false now
    }
    
    #[test]
    fn test_todo_list_filter() {
        let mut list = TodoList::new();
        let id1 = list.add_todo("Todo 1".to_string(), None);
        let _id2 = list.add_todo("Todo 2".to_string(), None);
        
        // Complete the first todo
        if let Some(todo) = list.find_todo_mut(id1) {
            todo.complete();
        }
        
        let all = list.filter_todos(TodoFilter::All);
        assert_eq!(all.len(), 2);
        
        let completed = list.filter_todos(TodoFilter::Completed);
        assert_eq!(completed.len(), 1);
        
        let pending = list.filter_todos(TodoFilter::Pending);
        assert_eq!(pending.len(), 1);
    }
}

// src/handlers.rs - Command Handlers Module
// This module contains the business logic for each CLI command

use anyhow::{Context, Result, bail};
use tracing::{info, debug, warn};

use crate::cli::{Commands, FilterArg, ExportFormat, get_confirmation, format_priority};
use crate::storage::{load_todos, save_todos, ensure_storage_exists};
use crate::todo::{TodoList, TodoFilter};

/// Handles the execution of CLI commands
/// 
/// # Arguments
/// * `command` - The parsed command from the CLI
/// 
/// # Returns
/// * `Result<()>` - Success or error with context
/// 
/// # Key Concepts:
/// 
/// ## Pattern Matching on Enums
/// - `match` ensures all command variants are handled
/// - Compiler enforces exhaustive matching
/// - Each arm extracts the relevant data
/// 
/// ## Error Propagation
/// - Each handler returns Result
/// - Errors bubble up with context
/// - Main function handles final error display
pub fn handle_command(command: Commands) -> Result<()> {
    // Ensure storage file exists before any operation
    ensure_storage_exists()?;
    
    match command {
        Commands::Add { description, priority } => {
            handle_add(description, priority)
        }
        Commands::List { filter, detailed } => {
            handle_list(filter, detailed)
        }
        Commands::Complete { id } => {
            handle_complete(id)
        }
        Commands::Delete { id, force } => {
            handle_delete(id, force)
        }
        Commands::Clear { force } => {
            handle_clear(force)
        }
        Commands::Stats => {
            handle_stats()
        }
        Commands::Export { format, output } => {
            handle_export(format, output)
        }
        Commands::Import { file, merge } => {
            handle_import(file, merge)
        }
    }
}

/// Handles adding a new todo
/// 
/// # Key Concepts:
/// 
/// ## Mutable Borrowing
/// - Load todos (owned)
/// - Modify the list (mutable)
/// - Save back to disk
/// 
/// ## String Ownership
/// - `description` is moved into the todo
/// - No cloning needed - efficient
fn handle_add(description: String, priority: Option<u8>) -> Result<()> {
    debug!("Adding new todo: {}", description);
    
    // Validate description is not empty
    if description.trim().is_empty() {
        bail!("Todo description cannot be empty");
    }
    
    // Load existing todos
    let mut todos = load_todos()
        .context("Failed to load todos")?;
    
    // Add the new todo
    let id = todos.add_todo(description.clone(), priority);
    
    // Save the updated list
    save_todos(&todos)
        .context("Failed to save todos")?;
    
    // Print success message with priority if set
    let priority_str = if let Some(p) = priority {
        format!(" with {}", format_priority(Some(p)))
    } else {
        String::new()
    };
    
    println!("✅ Added todo #{}: \"{}\"{}",
             id, description, priority_str);
    
    info!("Successfully added todo #{}", id);
    Ok(())
}

/// Handles listing todos
/// 
/// # Key Concepts:
/// 
/// ## References and Iterators
/// - filter_todos returns Vec<&Todo>
/// - References avoid copying data
/// - Efficient for read-only operations
/// 
/// ## Formatting Output
/// - Different formats for detailed/simple view
/// - Status indicators for visual clarity
fn handle_list(filter: Option<FilterArg>, detailed: bool) -> Result<()> {
    debug!("Listing todos with filter: {:?}", filter);
    
    let todos = load_todos()
        .context("Failed to load todos")?;
    
    // Convert CLI filter to domain filter
    let filter = filter.map(Into::into).unwrap_or(TodoFilter::All);
    
    // Get filtered todos
    let filtered = todos.filter_todos(filter);
    
    if filtered.is_empty() {
        println!("No todos found.");
        return Ok(());
    }
    
    // Print header
    println!("\n📋 Todo List");
    println!("{}", "─".repeat(50));
    
    // Print each todo
    for todo in filtered {
        let status = if todo.completed { "✅" } else { "⬜" };
        let priority_display = if detailed && todo.priority.is_some() {
            format!(" {}", format_priority(todo.priority))
        } else {
            String::new()
        };
        
        if detailed {
            // Detailed view with timestamps
            println!("\n{} [#{}] {}{}",
                     status, todo.id, todo.description, priority_display);
            println!("   Created: {}", 
                     todo.created_at.format("%Y-%m-%d %H:%M"));
            if let Some(completed_at) = todo.completed_at {
                println!("   Completed: {}", 
                         completed_at.format("%Y-%m-%d %H:%M"));
            }
        } else {
            // Simple view
            println!("{} [#{}] {}{}",
                     status, todo.id, todo.description, priority_display);
        }
    }
    
    // Print summary
    let total = todos.todos.len();
    let completed = todos.todos.iter().filter(|t| t.completed).count();
    println!("\n{}", "─".repeat(50));
    println!("Total: {} | Completed: {} | Pending: {}",
             total, completed, total - completed);
    
    Ok(())
}

/// Handles completing a todo
/// 
/// # Key Concepts:
/// 
/// ## Option Handling
/// - find_todo_mut returns Option<&mut Todo>
/// - Use match or if let for safe access
/// - Handle the None case gracefully
/// 
/// ## Mutable References
/// - find_todo_mut returns a mutable reference
/// - Allows modifying the todo in place
fn handle_complete(id: u32) -> Result<()> {
    debug!("Completing todo #{}", id);
    
    let mut todos = load_todos()
        .context("Failed to load todos")?;
    
    // Find and complete the todo
    // Using if let for cleaner error handling
    if let Some(todo) = todos.find_todo_mut(id) {
        if todo.completed {
            println!("ℹ️  Todo #{} is already completed", id);
            return Ok(());
        }
        
        let description = todo.description.clone();
        todo.complete();
        
        // Save the updated list
        save_todos(&todos)
            .context("Failed to save todos")?;
        
        println!("✅ Completed todo #{}: \"{}\"", id, description);
        info!("Completed todo #{}", id);
    } else {
        bail!("Todo with ID {} not found", id);
    }
    
    Ok(())
}

/// Handles deleting a todo
/// 
/// # Key Concepts:
/// 
/// ## User Confirmation
/// - Dangerous operations need confirmation
/// - --force flag bypasses the prompt
/// - Common pattern in CLI tools
/// 
/// ## Error Recovery
/// - Check if todo exists before confirming
/// - Provide clear error messages
fn handle_delete(id: u32, force: bool) -> Result<()> {
    debug!("Deleting todo #{} (force: {})", id, force);
    
    let mut todos = load_todos()
        .context("Failed to load todos")?;
    
    // Check if todo exists and get its description for confirmation
    let description = todos.todos
        .iter()
        .find(|t| t.id == id)
        .map(|t| t.description.clone())
        .ok_or_else(|| anyhow::anyhow!("Todo with ID {} not found", id))?;
    
    // Ask for confirmation unless --force is used
    if !force {
        let prompt = format!("Delete todo #{}: \"{}\"?", id, description);
        if !get_confirmation(&prompt) {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }
    
    // Remove the todo
    if todos.remove_todo(id) {
        save_todos(&todos)
            .context("Failed to save todos")?;
        
        println!("🗑️  Deleted todo #{}: \"{}\"", id, description);
        info!("Deleted todo #{}", id);
    } else {
        bail!("Failed to delete todo #{}", id);
    }
    
    Ok(())
}

/// Handles clearing completed todos
/// 
/// # Key Concepts:
/// 
/// ## Bulk Operations
/// - Filter and retain in one operation
/// - Efficient for large lists
/// - Clear feedback on what was removed
fn handle_clear(force: bool) -> Result<()> {
    debug!("Clearing completed todos (force: {})", force);
    
    let mut todos = load_todos()
        .context("Failed to load todos")?;
    
    // Count completed todos
    let completed_count = todos.todos.iter()
        .filter(|t| t.completed)
        .count();
    
    if completed_count == 0 {
        println!("No completed todos to clear.");
        return Ok(());
    }
    
    // Ask for confirmation unless --force is used
    if !force {
        let prompt = format!("Clear {} completed todo(s)?", completed_count);
        if !get_confirmation(&prompt) {
            println!("Clear operation cancelled.");
            return Ok(());
        }
    }
    
    // Remove completed todos
    todos.todos.retain(|todo| !todo.completed);
    
    save_todos(&todos)
        .context("Failed to save todos")?;
    
    println!("🧹 Cleared {} completed todo(s)", completed_count);
    info!("Cleared {} completed todos", completed_count);
    
    Ok(())
}

/// Handles showing statistics
/// 
/// # Key Concepts:
/// 
/// ## Data Analysis
/// - Iterate once, collect multiple metrics
/// - Use iterators for functional style
/// - Present data in readable format
fn handle_stats() -> Result<()> {
    debug!("Generating statistics");
    
    let todos = load_todos()
        .context("Failed to load todos")?;
    
    if todos.todos.is_empty() {
        println!("No todos to analyze.");
        return Ok(());
    }
    
    // Calculate statistics
    let total = todos.todos.len();
    let completed = todos.todos.iter().filter(|t| t.completed).count();
    let pending = total - completed;
    
    // Priority breakdown
    let mut priority_counts = [0; 6]; // Index 0 for None, 1-5 for priorities
    for todo in &todos.todos {
        match todo.priority {
            None => priority_counts[0] += 1,
            Some(p) if p <= 5 => priority_counts[p as usize] += 1,
            _ => {} // Invalid priority, ignore
        }
    }
    
    // Calculate completion rate
    let completion_rate = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    
    // Display statistics
    println!("\n📊 Todo Statistics");
    println!("{}", "═".repeat(50));
    println!("Total todos:      {}", total);
    println!("Completed:        {} ({:.1}%)", completed, completion_rate);
    println!("Pending:          {}", pending);
    
    println!("\n📈 Priority Breakdown:");
    if priority_counts[0] > 0 {
        println!("  No priority:    {}", priority_counts[0]);
    }
    for i in 1..=5 {
        if priority_counts[i] > 0 {
            println!("  {}:     {}", 
                     format_priority(Some(i as u8)), 
                     priority_counts[i]);
        }
    }
    
    // Find oldest pending todo
    if let Some(oldest) = todos.todos.iter()
        .filter(|t| !t.completed)
        .min_by_key(|t| t.created_at) {
        println!("\n⏰ Oldest pending todo:");
        println!("  [#{}] {} (created {})",
                 oldest.id,
                 oldest.description,
                 oldest.created_at.format("%Y-%m-%d"));
    }
    
    println!("{}", "═".repeat(50));
    
    Ok(())
}

/// Handles exporting todos
/// 
/// # Key Concepts:
/// 
/// ## Output Flexibility
/// - Write to file or stdout
/// - Different formats for different uses
/// - Preserve all data for reimport
fn handle_export(format: ExportFormat, output: Option<String>) -> Result<()> {
    debug!("Exporting todos as {:?} to {:?}", format, output);
    
    let todos = load_todos()
        .context("Failed to load todos")?;
    
    // Generate export content based on format
    let content = match format {
        ExportFormat::Json => {
            // Pretty JSON for readability
            serde_json::to_string_pretty(&todos)
                .context("Failed to serialize to JSON")?
        }
        ExportFormat::Markdown => {
            generate_markdown(&todos)
        }
        ExportFormat::Csv => {
            generate_csv(&todos)?
        }
        ExportFormat::Text => {
            generate_text(&todos)
        }
    };
    
    // Write to file or stdout
    if let Some(path) = output {
        std::fs::write(&path, content)
            .context(format!("Failed to write to {}", path))?;
        println!("📤 Exported todos to {}", path);
    } else {
        // Write to stdout
        print!("{}", content);
    }
    
    Ok(())
}

/// Generates Markdown format
fn generate_markdown(todos: &TodoList) -> String {
    let mut output = String::from("# Todo List\n\n");
    
    if todos.todos.is_empty() {
        output.push_str("No todos.\n");
        return output;
    }
    
    // Pending todos
    output.push_str("## Pending\n\n");
    for todo in todos.todos.iter().filter(|t| !t.completed) {
        output.push_str(&format!("- [ ] [#{}] {}", 
                                 todo.id, todo.description));
        if let Some(p) = todo.priority {
            output.push_str(&format!(" _{}_", format_priority(Some(p))));
        }
        output.push('\n');
    }
    
    // Completed todos
    output.push_str("\n## Completed\n\n");
    for todo in todos.todos.iter().filter(|t| t.completed) {
        output.push_str(&format!("- [x] [#{}] {}\n", 
                                 todo.id, todo.description));
    }
    
    output
}

/// Generates CSV format
fn generate_csv(todos: &TodoList) -> Result<String> {
    let mut output = String::from("ID,Description,Priority,Completed,Created,Completed At\n");
    
    for todo in &todos.todos {
        output.push_str(&format!(
            "{},\"{}\",{},{},{},{}\n",
            todo.id,
            todo.description.replace('"', "\"\""), // Escape quotes
            todo.priority.map_or(String::new(), |p| p.to_string()),
            todo.completed,
            todo.created_at.format("%Y-%m-%d %H:%M:%S"),
            todo.completed_at
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default()
        ));
    }
    
    Ok(output)
}

/// Generates plain text format
fn generate_text(todos: &TodoList) -> String {
    let mut output = String::new();
    
    for todo in &todos.todos {
        let status = if todo.completed { "[DONE]" } else { "[TODO]" };
        output.push_str(&format!("{} #{}: {}\n", 
                                 status, todo.id, todo.description));
    }
    
    output
}

/// Handles importing todos
/// 
/// # Key Concepts:
/// 
/// ## Data Merging
/// - Option to merge or replace
/// - Handle ID conflicts
/// - Preserve data integrity
fn handle_import(file: String, merge: bool) -> Result<()> {
    debug!("Importing todos from {} (merge: {})", file, merge);
    
    // Read the import file
    let content = std::fs::read_to_string(&file)
        .context(format!("Failed to read {}", file))?;
    
    // Parse as TodoList (assuming JSON format)
    let imported: TodoList = serde_json::from_str(&content)
        .context("Failed to parse import file as JSON")?;
    
    if merge {
        // Merge with existing todos
        let mut todos = load_todos()
            .context("Failed to load existing todos")?;
        
        // Store count before moving the vector
        let import_count = imported.todos.len();
        
        // Add imported todos with new IDs
        for mut todo in imported.todos {
            todo.id = todos.next_id;
            todos.todos.push(todo);
            todos.next_id += 1;
        }
        
        save_todos(&todos)
            .context("Failed to save merged todos")?;
        
        println!("📥 Imported and merged {} todo(s)", import_count);
    } else {
        // Replace existing todos
        save_todos(&imported)
            .context("Failed to save imported todos")?;
        
        println!("📥 Imported {} todo(s) (replaced existing)", 
                 imported.todos.len());
        warn!("Replaced existing todos with imported data");
    }
    
    Ok(())
}

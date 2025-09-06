// src/tui/mod.rs - Terminal User Interface Module
// This module provides an interactive terminal interface for the todo app

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::storage::{load_todos, save_todos};
use crate::todo::{TodoFilter, TodoList};

/// The main TUI application state
/// 
/// # Key TUI Concepts:
/// 
/// ## State Management
/// - TUIs are state machines that react to events
/// - State changes trigger re-renders
/// - All UI state must be explicitly tracked
/// 
/// ## Event Loop
/// - Continuously poll for keyboard/mouse events
/// - Update state based on events
/// - Render the new state to screen
pub struct App {
    /// The todo list data
    todos: TodoList,
    
    /// Current input mode
    input_mode: InputMode,
    
    /// Text being typed in input field
    input: String,
    
    /// Cursor position in input field
    cursor_position: usize,
    
    /// Currently selected todo index
    selected_index: Option<usize>,
    
    /// Current filter for displaying todos
    filter: TodoFilter,
    
    /// Status message to display
    status_message: Option<String>,
    
    /// Should the app exit?
    should_quit: bool,
    
    /// Show help popup?
    show_help: bool,
}

/// Input modes for the TUI
/// 
/// # Key Concepts:
/// - Modal interface like Vim
/// - Different modes have different key bindings
/// - Visual feedback shows current mode
#[derive(Debug, Clone, Copy, PartialEq)]
enum InputMode {
    /// Normal mode - navigate and execute commands
    Normal,
    /// Insert mode - typing new todo
    Insert,
    /// Editing existing todo
    Editing,
}

impl App {
    /// Creates a new TUI application instance
    pub fn new() -> Result<Self> {
        let todos = load_todos()?;
        let selected_index = if todos.todos.is_empty() { None } else { Some(0) };
        
        Ok(Self {
            todos,
            input_mode: InputMode::Normal,
            input: String::new(),
            cursor_position: 0,
            selected_index,
            filter: TodoFilter::All,
            status_message: Some("Welcome! Press 'h' for help".to_string()),
            should_quit: false,
            show_help: false,
        })
    }
    
    /// Runs the TUI application
    /// 
    /// # Key TUI Concepts:
    /// 
    /// ## Terminal Modes
    /// - Raw mode: Direct keyboard input without line buffering
    /// - Alternate screen: Preserves terminal content when app exits
    /// - Mouse capture: Optional mouse support
    /// 
    /// ## Render Loop
    /// - Clear screen -> Draw widgets -> Present
    /// - Only re-render when state changes
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        
        // Create terminal backend
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Run the app
        let res = self.run_app(&mut terminal);
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        // Return result
        res
    }
    
    /// Main application loop
    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Draw the UI
            terminal.draw(|f| self.draw(f))?;
            
            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // Handle keyboard input based on current mode
                    match self.input_mode {
                        InputMode::Normal => self.handle_normal_mode(key)?,
                        InputMode::Insert => self.handle_insert_mode(key)?,
                        InputMode::Editing => self.handle_editing_mode(key)?,
                    }
                }
            }
            
            // Check if we should quit
            if self.should_quit {
                // Save before quitting
                save_todos(&self.todos)?;
                break;
            }
        }
        
        Ok(())
    }
    
    /// Main drawing function
    /// 
    /// # Layout Concepts:
    /// - Constraints define how space is divided
    /// - Layouts can be nested for complex UIs
    /// - Widgets are rendered into rectangular areas
    fn draw(&mut self, frame: &mut Frame) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(5),      // Todo list
                Constraint::Length(3),   // Input
                Constraint::Length(2),   // Status bar
            ])
            .split(frame.size());
        
        // Draw title
        self.draw_title(frame, chunks[0]);
        
        // Draw todo list
        self.draw_todo_list(frame, chunks[1]);
        
        // Draw input area
        self.draw_input(frame, chunks[2]);
        
        // Draw status bar
        self.draw_status_bar(frame, chunks[3]);
        
        // Draw help popup if needed
        if self.show_help {
            self.draw_help_popup(frame);
        }
    }
    
    /// Draw the title bar
    fn draw_title(&self, frame: &mut Frame, area: Rect) {
        let title = format!(
            " 📋 Rust Todo TUI - Filter: {} ",
            match self.filter {
                TodoFilter::All => "All",
                TodoFilter::Completed => "Completed",
                TodoFilter::Pending => "Pending",
            }
        );
        
        let title_widget = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(title_widget, area);
    }
    
    /// Draw the todo list
    fn draw_todo_list(&mut self, frame: &mut Frame, area: Rect) {
        // Get filtered todos
        let filtered_indices: Vec<(usize, &crate::todo::Todo)> = self.todos.todos
            .iter()
            .enumerate()
            .filter(|(_, todo)| match self.filter {
                TodoFilter::All => true,
                TodoFilter::Completed => todo.completed,
                TodoFilter::Pending => !todo.completed,
            })
            .collect();
        
        // Create list items
        let items: Vec<ListItem> = filtered_indices
            .iter()
            .map(|(_, todo)| {
                let status = if todo.completed { "✅" } else { "⬜" };
                let priority = match todo.priority {
                    Some(1) => " 🔵",
                    Some(2) => " 🟢",
                    Some(3) => " 🟡",
                    Some(4) => " 🟠",
                    Some(5) => " 🔴",
                    _ => "",
                };
                
                let content = format!("{} [#{}] {}{}", status, todo.id, todo.description, priority);
                
                let style = if todo.completed {
                    Style::default().fg(Color::Gray).add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(Color::White)
                };
                
                ListItem::new(content).style(style)
            })
            .collect();
        
        // Create list widget
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Todos ")
                .border_style(Style::default().fg(
                    if self.input_mode == InputMode::Normal { Color::Yellow } else { Color::White }
                )))
            .highlight_style(Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        // Create list state
        let mut state = ListState::default();
        
        // Map selected index to filtered list
        if let Some(selected) = self.selected_index {
            let filtered_index = filtered_indices
                .iter()
                .position(|(idx, _)| *idx == selected);
            state.select(filtered_index);
        }
        
        // Render the list
        frame.render_stateful_widget(list, area, &mut state);
    }
    
    /// Draw the input area
    fn draw_input(&self, frame: &mut Frame, area: Rect) {
        let input_title = match self.input_mode {
            InputMode::Normal => " Commands (press 'i' to add todo) ",
            InputMode::Insert => " Adding Todo (press Esc to cancel) ",
            InputMode::Editing => " Editing Todo (press Esc to cancel) ",
        };
        
        let style = match self.input_mode {
            InputMode::Normal => Style::default().fg(Color::White),
            InputMode::Insert | InputMode::Editing => Style::default().fg(Color::Yellow),
        };
        
        let input = Paragraph::new(self.input.as_str())
            .style(style)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(style));
        
        frame.render_widget(input, area);
        
        // Show cursor when in insert mode
        if self.input_mode == InputMode::Insert || self.input_mode == InputMode::Editing {
            frame.set_cursor(
                area.x + self.cursor_position as u16 + 1,
                area.y + 1,
            );
        }
    }
    
    /// Draw the status bar
    fn draw_status_bar(&self, frame: &mut Frame, area: Rect) {
        let mode = match self.input_mode {
            InputMode::Normal => "NORMAL",
            InputMode::Insert => "INSERT",
            InputMode::Editing => "EDITING",
        };
        
        let stats = format!(
            " Mode: {} | Total: {} | Completed: {} | Pending: {} ",
            mode,
            self.todos.todos.len(),
            self.todos.todos.iter().filter(|t| t.completed).count(),
            self.todos.todos.iter().filter(|t| !t.completed).count(),
        );
        
        let message = if let Some(msg) = &self.status_message {
            format!(" | {} ", msg)
        } else {
            String::new()
        };
        
        let status = Paragraph::new(format!("{}{}", stats, message))
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .alignment(Alignment::Left);
        
        frame.render_widget(status, area);
    }
    
    /// Draw help popup
    fn draw_help_popup(&self, frame: &mut Frame) {
        let area = centered_rect(60, 80, frame.size());
        
        let help_text = vec![
            Line::from(vec![Span::styled("Keyboard Shortcuts", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(vec![Span::styled("Navigation:", Style::default().add_modifier(Modifier::UNDERLINED))]),
            Line::from("  j/↓     - Move down"),
            Line::from("  k/↑     - Move up"),
            Line::from("  g       - Go to top"),
            Line::from("  G       - Go to bottom"),
            Line::from(""),
            Line::from(vec![Span::styled("Actions:", Style::default().add_modifier(Modifier::UNDERLINED))]),
            Line::from("  i       - Insert new todo"),
            Line::from("  Enter   - Complete/uncomplete todo"),
            Line::from("  d       - Delete todo"),
            Line::from("  e       - Edit todo"),
            Line::from("  p       - Set priority (1-5)"),
            Line::from(""),
            Line::from(vec![Span::styled("Filters:", Style::default().add_modifier(Modifier::UNDERLINED))]),
            Line::from("  f       - Cycle through filters"),
            Line::from("  1       - Show all todos"),
            Line::from("  2       - Show completed only"),
            Line::from("  3       - Show pending only"),
            Line::from(""),
            Line::from(vec![Span::styled("Other:", Style::default().add_modifier(Modifier::UNDERLINED))]),
            Line::from("  h/?     - Toggle this help"),
            Line::from("  q       - Quit"),
            Line::from("  Esc     - Cancel/close"),
        ];
        
        let help = Paragraph::new(help_text)
            .block(Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)))
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        
        // Clear the area and render help
        frame.render_widget(Clear, area);
        frame.render_widget(help, area);
    }
    
    /// Handle normal mode key events
    fn handle_normal_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            // Movement
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('g') => self.move_to_top(),
            KeyCode::Char('G') => self.move_to_bottom(),
            
            // Actions
            KeyCode::Char('i') => {
                self.input_mode = InputMode::Insert;
                self.input.clear();
                self.cursor_position = 0;
                self.status_message = Some("Enter todo description".to_string());
            }
            KeyCode::Enter => self.toggle_complete()?,
            KeyCode::Char('d') => self.delete_selected()?,
            KeyCode::Char('e') => self.start_editing()?,
            
            // Filters
            KeyCode::Char('f') => self.cycle_filter(),
            KeyCode::Char('1') => self.filter = TodoFilter::All,
            KeyCode::Char('2') => self.filter = TodoFilter::Completed,
            KeyCode::Char('3') => self.filter = TodoFilter::Pending,
            
            // Priority
            KeyCode::Char('p') => self.prompt_priority()?,
            
            // Help
            KeyCode::Char('h') | KeyCode::Char('?') => self.show_help = !self.show_help,
            
            // Quit
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handle insert mode key events
    fn handle_insert_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    self.todos.add_todo(self.input.clone(), None);
                    save_todos(&self.todos)?;
                    self.status_message = Some(format!("Added: {}", self.input));
                    self.input.clear();
                    self.cursor_position = 0;
                    self.input_mode = InputMode::Normal;
                    
                    // Select the new todo
                    if !self.todos.todos.is_empty() {
                        self.selected_index = Some(self.todos.todos.len() - 1);
                    }
                }
            }
            KeyCode::Esc => {
                self.input.clear();
                self.cursor_position = 0;
                self.input_mode = InputMode::Normal;
                self.status_message = Some("Cancelled".to_string());
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handle editing mode key events
    fn handle_editing_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if let Some(idx) = self.selected_index {
                    if idx < self.todos.todos.len() {
                        self.todos.todos[idx].description = self.input.clone();
                        save_todos(&self.todos)?;
                        self.status_message = Some("Todo updated".to_string());
                    }
                }
                self.input.clear();
                self.cursor_position = 0;
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.input.clear();
                self.cursor_position = 0;
                self.input_mode = InputMode::Normal;
                self.status_message = Some("Edit cancelled".to_string());
            }
            _ => {
                // Reuse insert mode handling for text input
                self.handle_insert_mode(key)?;
            }
        }
        
        Ok(())
    }
    
    /// Move selection up or down
    fn move_selection(&mut self, delta: isize) {
        if self.todos.todos.is_empty() {
            return;
        }
        
        let len = self.todos.todos.len();
        
        if let Some(current) = self.selected_index {
            let new_index = if delta > 0 {
                (current + delta as usize).min(len - 1)
            } else {
                current.saturating_sub(delta.abs() as usize)
            };
            self.selected_index = Some(new_index);
        } else {
            self.selected_index = Some(0);
        }
    }
    
    /// Move to top of list
    fn move_to_top(&mut self) {
        if !self.todos.todos.is_empty() {
            self.selected_index = Some(0);
        }
    }
    
    /// Move to bottom of list
    fn move_to_bottom(&mut self) {
        if !self.todos.todos.is_empty() {
            self.selected_index = Some(self.todos.todos.len() - 1);
        }
    }
    
    /// Toggle completion status of selected todo
    fn toggle_complete(&mut self) -> Result<()> {
        if let Some(idx) = self.selected_index {
            if idx < self.todos.todos.len() {
                if self.todos.todos[idx].completed {
                    self.todos.todos[idx].completed = false;
                    self.todos.todos[idx].completed_at = None;
                    self.status_message = Some("Todo marked as pending".to_string());
                } else {
                    self.todos.todos[idx].complete();
                    self.status_message = Some("Todo completed!".to_string());
                }
                save_todos(&self.todos)?;
            }
        }
        Ok(())
    }
    
    /// Delete selected todo
    fn delete_selected(&mut self) -> Result<()> {
        if let Some(idx) = self.selected_index {
            if idx < self.todos.todos.len() {
                let id = self.todos.todos[idx].id;
                let desc = self.todos.todos[idx].description.clone();
                
                if self.todos.remove_todo(id) {
                    save_todos(&self.todos)?;
                    self.status_message = Some(format!("Deleted: {}", desc));
                    
                    // Adjust selection
                    if self.todos.todos.is_empty() {
                        self.selected_index = None;
                    } else if idx >= self.todos.todos.len() {
                        self.selected_index = Some(self.todos.todos.len() - 1);
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Start editing selected todo
    fn start_editing(&mut self) -> Result<()> {
        if let Some(idx) = self.selected_index {
            if idx < self.todos.todos.len() {
                self.input = self.todos.todos[idx].description.clone();
                self.cursor_position = self.input.len();
                self.input_mode = InputMode::Editing;
                self.status_message = Some("Editing todo".to_string());
            }
        }
        Ok(())
    }
    
    /// Cycle through filters
    fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            TodoFilter::All => TodoFilter::Completed,
            TodoFilter::Completed => TodoFilter::Pending,
            TodoFilter::Pending => TodoFilter::All,
        };
        self.status_message = Some(format!("Filter: {:?}", self.filter));
    }
    
    /// Prompt for priority setting
    fn prompt_priority(&mut self) -> Result<()> {
        self.status_message = Some("Press 1-5 to set priority, 0 to clear".to_string());
        // Note: Priority handling would be in the next key event
        Ok(())
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

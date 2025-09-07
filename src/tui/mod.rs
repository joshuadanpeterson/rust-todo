// src/tui/mod.rs - Terminal User Interface Module
// This module provides an interactive terminal interface for the todo app

mod theme;

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
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use self::theme::{Icons, Theme};
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

    /// Theme for the UI
    theme: Theme,

    /// Show detailed descriptions
    show_details: bool,
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
    /// Editing existing todo title
    Editing,
    /// Editing todo details/notes
    EditingDetails,
    /// Editing due date for a todo
    EditingDueDate,
    /// Setting priority for a todo
    SettingPriority,
}

impl App {
    /// Creates a new TUI application instance
    pub fn new() -> Result<Self> {
        let todos = load_todos()?;
        let selected_index = if todos.todos.is_empty() {
            None
        } else {
            Some(0)
        };

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
            theme: Theme::modern_dark(),
            show_details: false,
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
                        InputMode::EditingDetails => self.handle_editing_details_mode(key)?,
                        InputMode::EditingDueDate => self.handle_due_date_mode(key)?,
                        InputMode::SettingPriority => self.handle_priority_mode(key)?,
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
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Todo list
                Constraint::Length(3), // Input
                Constraint::Length(2), // Status bar
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
        let filter_text = self.get_filter_name();

        let title_spans = vec![
            Span::raw(" "),
            Span::styled(Icons::SPARKLE, Style::default().fg(self.theme.accent)),
            Span::raw(" "),
            Span::styled("Rust Todo", self.theme.title_style()),
            Span::raw(" "),
            Span::styled("│", Style::default().fg(self.theme.bg_highlight)),
            Span::raw(" Filter: "),
            Span::styled(filter_text, Style::default().fg(self.theme.primary_light)),
            Span::raw(" "),
        ];

        let title_widget = Paragraph::new(Line::from(title_spans))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.theme.primary))
                    .style(Style::default().bg(self.theme.bg_secondary)),
            );

        frame.render_widget(title_widget, area);
    }

    /// Draw the todo list
    fn draw_todo_list(&mut self, frame: &mut Frame, area: Rect) {
        // Use the filter_todos method for consistent filtering
        let filtered_todos = self.todos.filter_todos(self.filter);
        
        // Map filtered todos back to their indices
        let filtered_indices: Vec<(usize, &crate::todo::Todo)> = self.todos.todos
            .iter()
            .enumerate()
            .filter(|(_, todo)| filtered_todos.contains(todo))
            .collect();

        // Create list items with beautiful styling
        let items: Vec<ListItem> = filtered_indices
            .iter()
            .map(|(_, todo)| {
                let checkbox = if todo.completed {
                    Icons::CHECKBOX_CHECKED
                } else {
                    Icons::CHECKBOX_EMPTY
                };

                // Create priority indicator with colored squares for maximum visibility
                let priority_indicator = if let Some(p) = todo.priority {
                    // Use filled squares with vibrant colors for each priority level
                    let priority_icon = Icons::SQUARE; // Filled square for all priorities
                    let priority_label = match p {
                        1 => "[1]",
                        2 => "[2]",
                        3 => "[3]",
                        4 => "[4]",
                        5 => "[5]",
                        _ => "",
                    };

                    vec![
                        Span::raw(" "),
                        Span::styled(
                            priority_icon,
                            Style::default()
                                .fg(self.theme.priority_color(todo.priority))
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            priority_label,
                            Style::default()
                                .fg(self.theme.text_muted)
                                .add_modifier(Modifier::DIM),
                        ),
                    ]
                } else {
                    vec![]
                };

                // Build the line with multiple styled spans
                let mut spans = vec![
                    Span::styled(
                        checkbox,
                        if todo.completed {
                            Style::default().fg(self.theme.success)
                        } else {
                            Style::default().fg(self.theme.text_muted)
                        },
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("#{}", todo.id),
                        Style::default()
                            .fg(self.theme.text_muted)
                            .add_modifier(Modifier::DIM),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        &todo.description,
                        if todo.completed {
                            self.theme.completed_style()
                        } else {
                            Style::default().fg(self.theme.text_primary)
                        },
                    ),
                ];

                // Add priority indicator if present
                spans.extend(priority_indicator);

                // Add due date if present
                if let Some(due_str) = todo.format_due_date() {
                    let due_color = if todo.is_overdue() {
                        self.theme.error
                    } else if todo.is_due_soon() {
                        self.theme.warning
                    } else {
                        self.theme.text_muted
                    };

                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(Icons::CLOCK, Style::default().fg(due_color)));
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(due_str, Style::default().fg(due_color)));
                }

                // Create main line
                let mut lines = vec![Line::from(spans)];

                // Add details if enabled and present
                if self.show_details {
                    if let Some(ref details) = todo.details {
                        lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(
                                details,
                                Style::default()
                                    .fg(self.theme.text_secondary)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                        ]));
                    }
                }

                ListItem::new(lines)
            })
            .collect();

        // Create list widget with beautiful styling
        let highlight_symbol = format!("{} ", Icons::ARROW_RIGHT);
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(vec![
                        Span::raw(" "),
                        Span::styled(Icons::LIGHTNING, Style::default().fg(self.theme.warning)),
                        Span::raw(" Tasks "),
                    ])
                    .border_style(
                        self.theme
                            .border_style(self.input_mode == InputMode::Normal),
                    )
                    .style(Style::default().bg(self.theme.bg_primary)),
            )
            .highlight_style(self.theme.selected_style())
            .highlight_symbol(&highlight_symbol);

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
        let (input_icon, input_title, is_active) = match self.input_mode {
            InputMode::Normal => (Icons::BULLET, "Commands (press 'i' to add todo)", false),
            InputMode::Insert => (
                Icons::ROCKET,
                "Adding Todo (use :1-5 for priority | Esc to cancel)",
                true,
            ),
            InputMode::Editing => (Icons::DIAMOND, "Editing Todo Title (Esc to cancel)", true),
            InputMode::EditingDetails => (
                Icons::BULLET,
                "Editing Todo Details/Notes (Esc to cancel)",
                true,
            ),
            InputMode::EditingDueDate => (
                Icons::CLOCK,
                "Set Due Date: today, tomorrow, or YYYY-MM-DD (Esc to cancel)",
                true,
            ),
            InputMode::SettingPriority => (
                Icons::STAR,
                "Set Priority: 1-5 or 0 to clear (Esc to cancel)",
                true,
            ),
        };

        let input_style = if is_active {
            Style::default().fg(self.theme.accent)
        } else {
            Style::default().fg(self.theme.text_secondary)
        };

        let input = Paragraph::new(self.input.as_str())
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(vec![
                        Span::raw(" "),
                        Span::styled(input_icon, Style::default().fg(self.theme.primary)),
                        Span::raw(" "),
                        Span::styled(input_title, Style::default().fg(self.theme.text_secondary)),
                        Span::raw(" "),
                    ])
                    .border_style(self.theme.border_style(is_active))
                    .style(Style::default().bg(self.theme.bg_secondary)),
            );

        frame.render_widget(input, area);

        // Show cursor when in text input modes
        if self.input_mode == InputMode::Insert 
            || self.input_mode == InputMode::Editing 
            || self.input_mode == InputMode::EditingDetails
            || self.input_mode == InputMode::EditingDueDate 
        {
            frame.set_cursor(area.x + self.cursor_position as u16 + 1, area.y + 1);
        }
    }

    /// Draw the status bar
    fn draw_status_bar(&self, frame: &mut Frame, area: Rect) {
        let (mode_icon, mode_text) = match self.input_mode {
            InputMode::Normal => (Icons::CIRCLE, "NORMAL"),
            InputMode::Insert => (Icons::ROCKET, "INSERT"),
            InputMode::Editing => (Icons::DIAMOND, "EDIT"),
            InputMode::EditingDetails => (Icons::BULLET, "DETAILS"),
            InputMode::EditingDueDate => (Icons::CLOCK, "DUE DATE"),
            InputMode::SettingPriority => (Icons::STAR, "PRIORITY"),
        };

        let total = self.todos.todos.len();
        let completed = self.todos.todos.iter().filter(|t| t.completed).count();
        let pending = self.todos.todos.iter().filter(|t| !t.completed).count();

        // Build status bar with styled spans
        let mut status_spans = vec![
            Span::raw(" "),
            Span::styled(mode_icon, Style::default().fg(self.theme.accent)),
            Span::raw(" "),
            Span::styled(
                mode_text,
                Style::default()
                    .fg(self.theme.primary_light)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(self.theme.bg_highlight)),
            Span::styled(
                Icons::CHECKBOX_EMPTY,
                Style::default().fg(self.theme.text_muted),
            ),
            Span::styled(
                format!(" {} Total", total),
                Style::default().fg(self.theme.text_secondary),
            ),
            Span::styled(" │ ", Style::default().fg(self.theme.bg_highlight)),
            Span::styled(
                Icons::CHECKBOX_CHECKED,
                Style::default().fg(self.theme.success),
            ),
            Span::styled(
                format!(" {} Done", completed),
                Style::default().fg(self.theme.success),
            ),
            Span::styled(" │ ", Style::default().fg(self.theme.bg_highlight)),
            Span::styled(Icons::CIRCLE, Style::default().fg(self.theme.warning)),
            Span::styled(
                format!(" {} Pending", pending),
                Style::default().fg(self.theme.warning),
            ),
        ];

        // Add status message if present
        if let Some(msg) = &self.status_message {
            status_spans.push(Span::styled(
                " │ ",
                Style::default().fg(self.theme.bg_highlight),
            ));
            status_spans.push(Span::styled(
                Icons::SPARKLE,
                Style::default().fg(self.theme.info),
            ));
            status_spans.push(Span::raw(" "));
            status_spans.push(Span::styled(msg, Style::default().fg(self.theme.info)));
        }

        let status = Paragraph::new(Line::from(status_spans))
            .style(Style::default().bg(self.theme.bg_secondary))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(self.theme.bg_highlight)),
            );

        frame.render_widget(status, area);
    }

    /// Draw help popup
    fn draw_help_popup(&self, frame: &mut Frame) {
        let area = centered_rect(65, 85, frame.size());

        let help_text = vec![
            Line::from(vec![
                Span::styled(Icons::SPARKLE, Style::default().fg(self.theme.accent)),
                Span::raw(" "),
                Span::styled("Keyboard Shortcuts", self.theme.title_style()),
                Span::raw(" "),
                Span::styled(Icons::SPARKLE, Style::default().fg(self.theme.accent)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(Icons::ARROW_RIGHT, Style::default().fg(self.theme.primary)),
                Span::raw(" "),
                Span::styled(
                    "Navigation",
                    Style::default()
                        .fg(self.theme.primary_light)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("j/↓", Style::default().fg(self.theme.accent)),
                Span::raw("     Move down"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("k/↑", Style::default().fg(self.theme.accent)),
                Span::raw("     Move up"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("g", Style::default().fg(self.theme.accent)),
                Span::raw("       Go to top"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("G", Style::default().fg(self.theme.accent)),
                Span::raw("       Go to bottom"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(Icons::ARROW_RIGHT, Style::default().fg(self.theme.primary)),
                Span::raw(" "),
                Span::styled(
                    "Actions",
                    Style::default()
                        .fg(self.theme.primary_light)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("i", Style::default().fg(self.theme.accent)),
                Span::raw("       Insert new todo (add :N for priority)"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("Enter", Style::default().fg(self.theme.accent)),
                Span::raw("   Complete/uncomplete todo"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("d", Style::default().fg(self.theme.accent)),
                Span::raw("       Delete todo"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("e", Style::default().fg(self.theme.accent)),
                Span::raw("       Edit todo title"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("D", Style::default().fg(self.theme.accent)),
                Span::raw("       Edit details/notes"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("u", Style::default().fg(self.theme.accent)),
                Span::raw("       Set/edit due date"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("p", Style::default().fg(self.theme.accent)),
                Span::raw("       Set/change priority (1-5, 0 to clear)"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("v", Style::default().fg(self.theme.accent)),
                Span::raw("       Toggle detail view"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(Icons::ARROW_RIGHT, Style::default().fg(self.theme.primary)),
                Span::raw(" "),
                Span::styled(
                    "Filters",
                    Style::default()
                        .fg(self.theme.primary_light)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("f", Style::default().fg(self.theme.accent)),
                Span::raw("       Cycle through all filters"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("1", Style::default().fg(self.theme.accent)),
                Span::raw("       All tasks"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("2", Style::default().fg(self.theme.accent)),
                Span::raw("       Pending tasks"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("3", Style::default().fg(self.theme.accent)),
                Span::raw("       Completed tasks"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("4-6", Style::default().fg(self.theme.accent)),
                Span::raw("     Priority filters (High/Med/Low)"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("7-0", Style::default().fg(self.theme.accent)),
                Span::raw("     Due date filters"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(Icons::ARROW_RIGHT, Style::default().fg(self.theme.primary)),
                Span::raw(" "),
                Span::styled(
                    "Other",
                    Style::default()
                        .fg(self.theme.primary_light)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("h/?", Style::default().fg(self.theme.accent)),
                Span::raw("     Toggle this help"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("q", Style::default().fg(self.theme.accent)),
                Span::raw("       Save and quit"),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("Esc", Style::default().fg(self.theme.accent)),
                Span::raw("     Cancel/close"),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(vec![
                        Span::raw(" "),
                        Span::styled(Icons::LIGHTNING, Style::default().fg(self.theme.warning)),
                        Span::raw(" Help "),
                        Span::styled(Icons::LIGHTNING, Style::default().fg(self.theme.warning)),
                        Span::raw(" "),
                    ])
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(self.theme.primary))
                    .style(Style::default().bg(self.theme.bg_primary)),
            )
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(self.theme.text_primary));

        // Clear the area and render help with background
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
            KeyCode::Char('D') => self.start_editing_details()?,
            KeyCode::Char('u') => self.prompt_due_date()?,

            // Filters - Basic
            KeyCode::Char('f') => self.cycle_filter(),
            KeyCode::Char('1') => {
                self.filter = TodoFilter::All;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            KeyCode::Char('2') => {
                self.filter = TodoFilter::Pending;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            KeyCode::Char('3') => {
                self.filter = TodoFilter::Completed;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            
            // Filters - Priority (with Alt modifier)
            KeyCode::Char('4') => {
                self.filter = TodoFilter::HighPriority;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            KeyCode::Char('5') => {
                self.filter = TodoFilter::MediumPriority;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            KeyCode::Char('6') => {
                self.filter = TodoFilter::LowPriority;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            
            // Filters - Due dates
            KeyCode::Char('7') => {
                self.filter = TodoFilter::Overdue;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            KeyCode::Char('8') => {
                self.filter = TodoFilter::DueToday;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            KeyCode::Char('9') => {
                self.filter = TodoFilter::DueSoon;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }
            KeyCode::Char('0') => {
                self.filter = TodoFilter::HasDueDate;
                self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
            }

            // Priority
            KeyCode::Char('p') => self.prompt_priority()?,

            // View details toggle
            KeyCode::Char('v') => {
                self.show_details = !self.show_details;
                self.status_message = Some(if self.show_details {
                    "Showing detailed descriptions".to_string()
                } else {
                    "Hiding detailed descriptions".to_string()
                });
            }

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
                    // Check if input contains priority suffix like :1 or :5
                    let (description, priority) = if let Some(pos) = self.input.rfind(':') {
                        let desc = self.input[..pos].trim();
                        let priority_str = self.input[pos + 1..].trim();
                        if let Ok(p) = priority_str.parse::<u8>() {
                            if (1..=5).contains(&p) {
                                (desc.to_string(), Some(p))
                            } else {
                                (self.input.clone(), None)
                            }
                        } else {
                            (self.input.clone(), None)
                        }
                    } else {
                        (self.input.clone(), None)
                    };

                    self.todos.add_todo(description.clone(), priority);
                    save_todos(&self.todos)?;

                    let msg = if let Some(p) = priority {
                        format!("Added: {} (priority {})", description, p)
                    } else {
                        format!("Added: {}", description)
                    };
                    self.status_message = Some(msg);

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
                        self.status_message = Some("Todo title updated".to_string());
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

    /// Handle due date editing mode key events
    fn handle_due_date_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        use chrono::{NaiveDate, Utc, TimeZone};
        
        match key.code {
            KeyCode::Enter => {
                if let Some(idx) = self.selected_index {
                    if idx < self.todos.todos.len() {
                        let input = self.input.trim().to_lowercase();
                        
                        if input.is_empty() {
                            // Clear due date
                            self.todos.todos[idx].due_date = None;
                            self.status_message = Some("Due date cleared".to_string());
                        } else if input == "today" {
                            self.todos.todos[idx].due_date = Some(Utc::now());
                            self.status_message = Some("Due date set to today".to_string());
                        } else if input == "tomorrow" {
                            let tomorrow = Utc::now() + chrono::Duration::days(1);
                            self.todos.todos[idx].due_date = Some(tomorrow);
                            self.status_message = Some("Due date set to tomorrow".to_string());
                        } else {
                            // Try to parse as YYYY-MM-DD
                            match NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                                Ok(date) => {
                                    let datetime = date.and_hms_opt(23, 59, 59)
                                        .map(|dt| Utc.from_utc_datetime(&dt));
                                    if let Some(dt) = datetime {
                                        self.todos.todos[idx].due_date = Some(dt);
                                        self.status_message = Some(format!("Due date set to {}", date));
                                    }
                                }
                                Err(_) => {
                                    self.status_message = Some("Invalid date format. Use YYYY-MM-DD".to_string());
                                    return Ok(());
                                }
                            }
                        }
                        save_todos(&self.todos)?;
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
                self.status_message = Some("Due date edit cancelled".to_string());
            }
            _ => {
                // Reuse insert mode handling for text input
                self.handle_insert_mode(key)?;
            }
        }

        Ok(())
    }

    /// Handle editing details mode key events
    fn handle_editing_details_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if let Some(idx) = self.selected_index {
                    if idx < self.todos.todos.len() {
                        // Set details to the input, or None if empty
                        if self.input.trim().is_empty() {
                            self.todos.todos[idx].details = None;
                            self.status_message = Some("Details cleared".to_string());
                        } else {
                            self.todos.todos[idx].details = Some(self.input.clone());
                            self.status_message = Some("Details updated".to_string());
                        }
                        save_todos(&self.todos)?;
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

    /// Handle priority setting mode key events
    ///
    /// # Key Concepts:
    /// - Direct numeric input for priority
    /// - Immediate feedback on valid/invalid input
    /// - Simple mode for single-key actions
    fn handle_priority_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('0') => {
                // Clear priority
                if let Some(idx) = self.selected_index {
                    if idx < self.todos.todos.len() {
                        self.todos.todos[idx].priority = None;
                        save_todos(&self.todos)?;
                        self.status_message = Some("Priority cleared".to_string());
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) if ('1'..='5').contains(&c) => {
                // Set priority 1-5
                let priority = c.to_digit(10).unwrap() as u8;
                if let Some(idx) = self.selected_index {
                    if idx < self.todos.todos.len() {
                        self.todos.todos[idx].priority = Some(priority);
                        save_todos(&self.todos)?;
                        let priority_name = match priority {
                            1 => "Low",
                            2 => "Normal",
                            3 => "Medium",
                            4 => "High",
                            5 => "Critical",
                            _ => "Unknown",
                        };
                        self.status_message =
                            Some(format!("Priority set to {} ({})", priority, priority_name));
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                // Cancel priority setting
                self.input_mode = InputMode::Normal;
                self.status_message = Some("Priority change cancelled".to_string());
            }
            _ => {
                // Invalid input
                self.status_message = Some(
                    "Invalid priority. Press 1-5 to set, 0 to clear, Esc to cancel".to_string(),
                );
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
                current.saturating_sub(delta.unsigned_abs())
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
                self.status_message = Some("Editing todo title".to_string());
            }
        }
        Ok(())
    }

    /// Cycle through filters
    fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            TodoFilter::All => TodoFilter::Pending,
            TodoFilter::Pending => TodoFilter::Completed,
            TodoFilter::Completed => TodoFilter::HighPriority,
            TodoFilter::HighPriority => TodoFilter::MediumPriority,
            TodoFilter::MediumPriority => TodoFilter::LowPriority,
            TodoFilter::LowPriority => TodoFilter::NoPriority,
            TodoFilter::NoPriority => TodoFilter::Overdue,
            TodoFilter::Overdue => TodoFilter::DueToday,
            TodoFilter::DueToday => TodoFilter::DueSoon,
            TodoFilter::DueSoon => TodoFilter::HasDueDate,
            TodoFilter::HasDueDate => TodoFilter::All,
        };
        self.status_message = Some(format!("Filter: {}", self.get_filter_name()));
    }
    
    /// Get human-readable filter name
    fn get_filter_name(&self) -> &str {
        match self.filter {
            TodoFilter::All => "All Tasks",
            TodoFilter::Completed => "Completed",
            TodoFilter::Pending => "Pending",
            TodoFilter::HighPriority => "High Priority (4-5)",
            TodoFilter::MediumPriority => "Medium Priority (2-3)",
            TodoFilter::LowPriority => "Low Priority (1)",
            TodoFilter::NoPriority => "No Priority",
            TodoFilter::Overdue => "Overdue",
            TodoFilter::DueToday => "Due Today",
            TodoFilter::DueSoon => "Due Soon (7 days)",
            TodoFilter::HasDueDate => "Has Due Date",
        }
    }

    /// Prompt for priority setting
    fn prompt_priority(&mut self) -> Result<()> {
        if self.selected_index.is_some() {
            self.input_mode = InputMode::SettingPriority;
            self.status_message = Some("Enter priority (1-5) or 0 to clear".to_string());
        } else {
            self.status_message = Some("No todo selected".to_string());
        }
        Ok(())
    }

    /// Start editing details for selected todo
    fn start_editing_details(&mut self) -> Result<()> {
        if let Some(idx) = self.selected_index {
            if idx < self.todos.todos.len() {
                // Load existing details or start with empty
                self.input = self.todos.todos[idx]
                    .details
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_default();
                self.cursor_position = self.input.len();
                self.input_mode = InputMode::EditingDetails;
                self.status_message = Some("Editing details (Enter to save, empty to clear)".to_string());
            }
        } else {
            self.status_message = Some("No todo selected".to_string());
        }
        Ok(())
    }

    /// Prompt for due date
    fn prompt_due_date(&mut self) -> Result<()> {
        if let Some(idx) = self.selected_index {
            if idx < self.todos.todos.len() {
                // Load existing due date or start with empty
                self.input = if let Some(due) = self.todos.todos[idx].due_date {
                    due.format("%Y-%m-%d").to_string()
                } else {
                    String::new()
                };
                self.cursor_position = self.input.len();
                self.input_mode = InputMode::EditingDueDate;
                self.status_message = Some(
                    "Enter due date (today, tomorrow, YYYY-MM-DD, or empty to clear)".to_string()
                );
            }
        } else {
            self.status_message = Some("No todo selected".to_string());
        }
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

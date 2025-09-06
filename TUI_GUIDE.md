# TUI Mode Guide 🖥️

## Launching the TUI

```bash
# Build the project
cargo build --release

# Launch TUI mode
./target/release/rust-todo tui

# Or use cargo
cargo run -- tui
```

## Priority Features

### Setting Priority on Existing Todos
1. Navigate to a todo using `j`/`k` or arrow keys
2. Press `p` to enter priority mode
3. Press `1`-`5` to set priority level:
   - `1` = 🔵 Low
   - `2` = 🟢 Normal
   - `3` = 🟡 Medium
   - `4` = 🟠 High
   - `5` = 🔴 Critical
4. Press `0` to clear priority
5. Press `Esc` to cancel

### Adding Todos with Priority
1. Press `i` to enter insert mode
2. Type your todo description
3. Add `:N` at the end for priority (where N is 1-5)
   - Example: `Buy groceries:3` (sets Medium priority)
   - Example: `Fix critical bug:5` (sets Critical priority)
4. Press `Enter` to save

## Complete Keyboard Reference

### Navigation (Normal Mode)
| Key | Action |
|-----|--------|
| `j` or `↓` | Move down |
| `k` or `↑` | Move up |
| `g` | Jump to top |
| `G` | Jump to bottom |

### Actions (Normal Mode)
| Key | Action |
|-----|--------|
| `i` | Insert new todo |
| `Enter` | Toggle complete/incomplete |
| `d` | Delete selected todo |
| `e` | Edit selected todo |
| `p` | Set/change priority |

### Filters (Normal Mode)
| Key | Action |
|-----|--------|
| `f` | Cycle through filters |
| `1` | Show all todos |
| `2` | Show completed only |
| `3` | Show pending only |

### General
| Key | Action |
|-----|--------|
| `h` or `?` | Show/hide help |
| `q` | Quit TUI |
| `Esc` | Cancel current operation |

## Visual Indicators

- **Selection**: `>> ` marks the currently selected todo
- **Completion**: ✅ = completed, ⬜ = pending
- **Priority Colors**:
  - 🔵 Low (Priority 1)
  - 🟢 Normal (Priority 2)
  - 🟡 Medium (Priority 3)
  - 🟠 High (Priority 4)
  - 🔴 Critical (Priority 5)
- **Mode Indicator**: Bottom status bar shows current mode (NORMAL/INSERT/EDITING/PRIORITY)

## Tips

1. **Quick Priority**: When adding todos, append `:3` for medium priority without extra steps
2. **Batch Operations**: Use filters to focus on specific todo types
3. **Visual Feedback**: The status bar always shows what mode you're in
4. **Help Available**: Press `h` anytime to see keyboard shortcuts

## Example Workflow

```
1. Launch TUI: cargo run -- tui
2. Press 'i' to add: "Review PR:4" [Enter]  (adds with High priority)
3. Press 'j' to move down
4. Press 'p' then '2' to set Normal priority
5. Press 'Enter' to mark complete
6. Press 'f' to cycle filters
7. Press 'q' to quit
```

## Troubleshooting

- **Terminal Too Small**: Resize your terminal to at least 80x24
- **Colors Not Showing**: Ensure your terminal supports 256 colors
- **Keys Not Working**: Make sure you're in Normal mode (press Esc)

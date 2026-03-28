# Leuwi Panjang Terminal - Keymaps Reference

All shortcuts are configurable in `~/.config/leuwi-panjang/config.toml` under `[keybindings]`.

## Terminal Navigation

| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| New tab | `Ctrl+Shift+T` | `new_tab` |
| Close tab/pane | `Ctrl+Shift+W` | `close_pane` |
| Next tab | `Ctrl+Tab` | `next_tab` |
| Previous tab | `Ctrl+Shift+Tab` | `prev_tab` |
| Go to tab 1-9 | `Alt+1` - `Alt+9` | `goto_tab_N` |
| Search tabs/panes | `Ctrl+Shift+Space` | `search_tabs` |

## Split Panes

| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Split horizontal (top/bottom) | `Ctrl+Shift+H` | `split_horizontal` |
| Split vertical (left/right) | `Ctrl+Shift+V` | `split_vertical` |
| Navigate up | `Alt+Up` | `navigate_up` |
| Navigate down | `Alt+Down` | `navigate_down` |
| Navigate left | `Alt+Left` | `navigate_left` |
| Navigate right | `Alt+Right` | `navigate_right` |
| Resize up | `Ctrl+Alt+Up` | `resize_up` |
| Resize down | `Ctrl+Alt+Down` | `resize_down` |
| Resize left | `Ctrl+Alt+Left` | `resize_left` |
| Resize right | `Ctrl+Alt+Right` | `resize_right` |
| Maximize/restore pane | `Ctrl+Shift+Enter` | `maximize_pane` |
| Broadcast input toggle | `Ctrl+Shift+B` | `broadcast_input` |

## Clipboard

| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Copy | `Ctrl+Shift+C` | `copy` |
| Paste | `Ctrl+Shift+V` | `paste` |
| Paste special (transform) | `Ctrl+Shift+Alt+V` | `paste_special` |
| Select all | `Ctrl+Shift+A` | `select_all` |

## Search

| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Find in scrollback | `Ctrl+Shift+F` | `search` |
| Find next | `F3` or `Enter` | `search_next` |
| Find previous | `Shift+F3` | `search_prev` |
| Toggle regex | `Alt+R` | `search_regex` |
| Toggle case sensitive | `Alt+C` | `search_case` |
| Close search | `Escape` | - |

## View

| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Zoom in | `Ctrl+=` / `Ctrl++` | `zoom_in` |
| Zoom out | `Ctrl+-` | `zoom_out` |
| Reset zoom | `Ctrl+0` | `zoom_reset` |
| Fullscreen | `F11` | `fullscreen` |
| Toggle status bar | `Ctrl+Shift+S` | `toggle_status_bar` |

## Scrolling

| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Scroll up (page) | `Shift+PageUp` | `scroll_page_up` |
| Scroll down (page) | `Shift+PageDown` | `scroll_page_down` |
| Scroll up (line) | `Shift+Up` | `scroll_line_up` |
| Scroll down (line) | `Shift+Down` | `scroll_line_down` |
| Scroll to top | `Shift+Home` | `scroll_top` |
| Scroll to bottom | `Shift+End` | `scroll_bottom` |
| Previous prompt | `Ctrl+Shift+Up` | `prev_prompt` |
| Next prompt | `Ctrl+Shift+Down` | `next_prompt` |

## Terminal Control

| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Clear screen | `Ctrl+Shift+K` | `clear_screen` |
| Reset terminal | `Ctrl+Shift+R` | `reset_terminal` |
| Select last command output | `Ctrl+Shift+O` | `select_last_output` |

## Application

| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Settings | `Ctrl+Shift+,` | `settings` |
| Command palette | `Ctrl+Shift+P` | `command_palette` |
| Theme browser | `Ctrl+Shift+Alt+T` | `theme_browser` |
| Toggle AI panel | `Ctrl+Shift+A` | `toggle_ai_panel` |
| New window | `Ctrl+Shift+N` | `new_window` |
| Quit | `Ctrl+Shift+Q` | `quit` |

## Command Suggestions

| Action | Default Shortcut |
|--------|-----------------|
| Accept ghost suggestion | `Right Arrow` or `End` |
| Next suggestion | `Tab` or `Down` |
| Previous suggestion | `Shift+Tab` or `Up` |
| Dismiss suggestions | `Escape` |
| Show all suggestions | `Ctrl+Space` |

## Customizing Keybindings

In `~/.config/leuwi-panjang/config.toml`:

```toml
[keybindings]
# Override any default
split_horizontal = "ctrl+shift+minus"
split_vertical = "ctrl+shift+backslash"
new_tab = "ctrl+t"

# Disable a keybinding
broadcast_input = ""

# Custom action
# key = "action:argument"
"ctrl+shift+1" = "run:htop"
"ctrl+shift+2" = "ssh:production-server"
"ctrl+shift+g" = "run:lazygit"
```

### Modifier Keys
- `ctrl` - Control
- `shift` - Shift
- `alt` - Alt / Option (macOS)
- `super` - Super / Cmd (macOS)
- Combine with `+`: `ctrl+shift+t`

### Special Keys
`enter`, `tab`, `escape`, `space`, `backspace`, `delete`, `home`, `end`, `pageup`, `pagedown`, `up`, `down`, `left`, `right`, `f1`-`f12`, `insert`, `minus`, `plus`, `backslash`, `slash`, `comma`, `period`, `semicolon`, `quote`

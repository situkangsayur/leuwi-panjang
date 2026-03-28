# Leuwi Panjang Terminal - Configuration Reference

Configuration is stored in `~/.config/leuwi-panjang/config.toml` and can be edited:
- **Via file**: edit TOML directly (hot-reload on save)
- **Via GUI**: `Ctrl+Shift+,` or hamburger menu -> Settings

## Full Configuration Reference

```toml
# ============================================================
# Leuwi Panjang Terminal - Configuration
# ============================================================

# ─── General ────────────────────────────────────────────────

[general]
# Default shell to launch
default_shell = "/bin/zsh"

# TERM environment variable (xterm-256color recommended for SSH compatibility)
term = "xterm-256color"

# Startup mode: normal, maximized, fullscreen
startup_mode = "normal"

# Confirm before closing terminal with running processes
confirm_close = true

# Working directory for new tabs/panes
# "inherit" = same as current pane, or specify a path
working_directory = "inherit"

# Enable shell integration (prompt marks, CWD tracking)
shell_integration = true

# Startup session file (layout preset)
# startup_session = "~/.config/leuwi-panjang/sessions/default.toml"

# ─── Appearance ─────────────────────────────────────────────

[appearance]
# Theme name (from ~/.config/leuwi-panjang/themes/)
theme = "leuwi-dark"

# Font settings
font_family = "JetBrains Mono"
font_size = 13.0
bold_font = "auto"          # "auto" or specific font name
italic_font = "auto"
bold_italic_font = "auto"

# Font features (OpenType)
# font_features = ["liga", "calt"]  # ligatures, contextual alternates

# Line height multiplier
line_height = 1.0

# Cell width multiplier (character spacing)
cell_width = 1.0

# Background opacity (0.0 = transparent, 1.0 = opaque)
background_opacity = 0.95

# Background blur radius (0 = disabled, requires compositor support)
background_blur = 0

# Background image
# background_image = "~/.config/leuwi-panjang/bg.png"
# background_image_opacity = 0.1

# Window decorations: "none" (chromeless), "titlebar", "full"
window_decorations = "none"

# Corner radius in pixels (0 = sharp corners)
corner_radius = 12

# Tab bar position: "top", "bottom", "hidden"
tab_bar_position = "top"

# Tab bar style: "default", "minimal", "powerline"
tab_bar_style = "default"

# Status bar position: "bottom", "top", "hidden"
status_bar_position = "bottom"

# Status bar components (left to right)
status_bar_components = ["git", "cwd", "spacer", "cpu", "memory", "clock"]

# Cursor style: "block", "beam", "underline"
cursor_style = "beam"

# Cursor blink
cursor_blink = true
cursor_blink_interval_ms = 500

# Inactive pane dimming (0.0 = no dimming, 1.0 = fully dimmed)
inactive_pane_dimming = 0.15

# Padding (pixels)
[appearance.padding]
top = 4
bottom = 4
left = 8
right = 8

# ─── Colors ─────────────────────────────────────────────────
# Override theme colors (or define inline if no theme)

[colors]
foreground = "#e0e0e0"
background = "#1a1a2e"
cursor = "#e94560"
cursor_text = "#1a1a2e"
selection_foreground = "#ffffff"
selection_background = "#0f3460"

[colors.normal]
black = "#1a1a2e"
red = "#e94560"
green = "#0cce6b"
yellow = "#ffc857"
blue = "#0f3460"
magenta = "#c77dff"
cyan = "#00b4d8"
white = "#e0e0e0"

[colors.bright]
black = "#4a4a6a"
red = "#ff6b8a"
green = "#3dff95"
yellow = "#ffe087"
blue = "#3f6490"
magenta = "#e7adff"
cyan = "#30d4f8"
white = "#ffffff"

# ─── Scrollback ─────────────────────────────────────────────

[scrollback]
# Number of lines to keep (0 = disabled)
lines = 10000

# Scroll multiplier (lines per scroll event)
multiplier = 3.0

# Scroll on new output
scroll_on_output = false

# Scroll on keystroke
scroll_on_keystroke = true

# ─── Command Suggestions ────────────────────────────────────

[suggestions]
# Enable command suggestions
enabled = true

# Inline ghost text (like zsh-autosuggestions)
ghost_text = true

# Dropdown suggestion panel
dropdown = true

# History-based suggestions
history_based = true

# Man page flag descriptions
man_page_flags = true

# Fuzzy matching
fuzzy_match = true

# Maximum suggestions to show
max_suggestions = 8

# Minimum characters before showing suggestions
min_chars = 2

# ─── Search ─────────────────────────────────────────────────

[search]
# Wrap around when reaching end/beginning
wrap_around = true

# Default to regex mode
regex_default = false

# Default to case-sensitive
case_sensitive_default = false

# Highlight all matches
highlight_all = true

# ─── Bell ────────────────────────────────────────────────────

[bell]
# Bell mode: "none", "visual", "audible", "both"
mode = "visual"

# Visual bell color (flash)
visual_color = "#ffffff"

# Visual bell duration (ms)
visual_duration_ms = 100

# ─── Mouse ───────────────────────────────────────────────────

[mouse]
# Copy on select
copy_on_select = false

# Click to open URLs
url_click = true

# URL modifier key: "ctrl", "shift", "alt", "none"
url_modifier = "ctrl"

# Double-click word characters (beyond alphanumeric)
word_characters = "-_."

# ─── Keybindings ─────────────────────────────────────────────
# See docs/guides/02-keymaps.md for full reference

[keybindings]
new_tab = "ctrl+shift+t"
close_pane = "ctrl+shift+w"
split_horizontal = "ctrl+shift+h"
split_vertical = "ctrl+shift+v"
navigate_up = "alt+up"
navigate_down = "alt+down"
navigate_left = "alt+left"
navigate_right = "alt+right"
resize_up = "ctrl+alt+up"
resize_down = "ctrl+alt+down"
resize_left = "ctrl+alt+left"
resize_right = "ctrl+alt+right"
maximize_pane = "ctrl+shift+enter"
next_tab = "ctrl+tab"
prev_tab = "ctrl+shift+tab"
search_tabs = "ctrl+shift+space"
copy = "ctrl+shift+c"
paste = "ctrl+shift+v"
search = "ctrl+shift+f"
settings = "ctrl+shift+comma"
command_palette = "ctrl+shift+p"
zoom_in = "ctrl+equal"
zoom_out = "ctrl+minus"
fullscreen = "f11"
broadcast_input = "ctrl+shift+b"

# ─── Profiles ────────────────────────────────────────────────

[profiles.default]
name = "Default"
shell = "/bin/zsh"
working_directory = "~"
# All [appearance] and [colors] keys can be overridden per profile

[profiles.production]
name = "Production"
# Override background for production servers
colors.background = "#2d0000"
# Auto-switch when SSH'd to hosts matching pattern
auto_switch_rules = ["hostname:prod-*", "hostname:*.production.*"]

[profiles.remote]
name = "Remote"
colors.background = "#002d00"
auto_switch_rules = ["ssh:*"]

# ─── Plugins ─────────────────────────────────────────────────

[plugins]
# List of enabled plugins
enabled = ["ai-claude"]

# Plugin-specific config
[plugins.ai-claude]
panel_position = "right"
panel_width = 40
auto_context = true
approval_mode = "always_ask"

[plugins.ai-gemini]
panel_position = "right"
model = "gemini-2.5-pro"

[plugins.ai-ollama]
host = "localhost:11434"
default_model = "codellama:13b"

# ─── Advanced ────────────────────────────────────────────────

[advanced]
# GPU rendering backend preference: "auto", "vulkan", "metal", "dx12", "opengl"
gpu_backend = "auto"

# Enable Kitty Keyboard Protocol (for compatible applications)
kitty_keyboard_protocol = true

# Enable Kitty Graphics Protocol (inline images)
kitty_graphics_protocol = true

# Enable Sixel graphics
sixel = true

# Maximum FPS (0 = unlimited / vsync)
max_fps = 0

# Audit trail logging
audit_trail = true

# Audit trail directory
audit_directory = "~/.local/share/leuwi-panjang/audit/"
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `LEUWI_CONFIG` | Override config file path |
| `LEUWI_THEME` | Override theme |
| `TERM` | Terminal type (set by Leuwi Panjang) |
| `COLORTERM` | Set to `truecolor` by Leuwi Panjang |
| `LEUWI_PANE_ID` | Current pane ID (for scripting) |
| `LEUWI_TAB_ID` | Current tab ID (for scripting) |

## Session Files

Define startup layouts in `~/.config/leuwi-panjang/sessions/`:

```toml
# ~/.config/leuwi-panjang/sessions/dev.toml
[[tabs]]
name = "Code"
layout = "tall"  # tall, fat, grid, splits

[[tabs.panes]]
command = "nvim"
working_directory = "~/project"
focus = true

[[tabs.panes]]
command = "cargo watch -x test"
working_directory = "~/project"

[[tabs]]
name = "Servers"

[[tabs.panes]]
command = "ssh dev@192.168.1.100"

[[tabs.panes]]
command = "ssh dev@192.168.1.101"
```

Load with: `leuwi-panjang --session dev`

## Hot Reload

Changes to `config.toml` are automatically detected and applied without restarting. Some settings (like `gpu_backend`) require a restart.

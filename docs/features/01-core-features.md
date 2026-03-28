# Leuwi Panjang Terminal - Core Features

## 1. Chromeless Window (No Titlebar)

The terminal has NO traditional titlebar. The tab bar IS the header.

```
┌─────────────────────────────────────────────────────────────┐
│ Tab 1 │ Tab 2 │ Tab 3 │                         [+]  │ ≡ │
├────────────────────────────────────────────────────────┼────┤
│ Terminal content...                                        │
└────────────────────────────────────────────────────────────┘
  ↑ Rounded corners
```

- Tabs on top = header bar (Chrome-style)
- Hamburger menu (≡) at far right for settings
- [+] button for new tab
- Drag empty tab area to move window
- Double-click tab area to maximize/restore
- Configurable corner radius (default: 12px)

## 2. Split Panes (Horizontal & Vertical)

Binary tree split model (like iTerm2) for arbitrary nesting.

### Shortcuts (default, configurable)

| Action | Shortcut |
|--------|----------|
| Split horizontal (top/bottom) | `Ctrl+Shift+H` |
| Split vertical (left/right) | `Ctrl+Shift+V` |
| Navigate up | `Alt+Up` |
| Navigate down | `Alt+Down` |
| Navigate left | `Alt+Left` |
| Navigate right | `Alt+Right` |
| Resize pane | `Ctrl+Alt+Arrow` |
| Maximize/restore pane | `Ctrl+Shift+Enter` |
| Close pane | `Ctrl+Shift+W` |
| Swap panes | Drag and drop |
| Broadcast input | `Ctrl+Shift+B` (toggle) |

### Visual
- Active pane has subtle highlight border
- Inactive panes slightly dimmed (configurable)
- Resize by dragging divider

## 3. Tab Management

| Action | Shortcut |
|--------|----------|
| New tab | `Ctrl+Shift+T` |
| Close tab | `Ctrl+Shift+W` |
| Next tab | `Ctrl+Tab` |
| Previous tab | `Ctrl+Shift+Tab` |
| Go to tab N | `Alt+1` through `Alt+9` |
| Move tab | Drag and drop |
| Rename tab | Double-click tab title |

### Tab Search / Window Switcher

Like Zen Browser - quickly search and jump to any tab or split pane.

| Action | Shortcut |
|--------|----------|
| Search tabs/panes | `Ctrl+Shift+P` or `Ctrl+Shift+Space` |

The switcher shows:
```
┌──────────────────────────────────────────┐
│ 🔍 Search tabs and panes...              │
├──────────────────────────────────────────┤
│ Tab 1 > Pane 1: ~/project (zsh)         │
│ Tab 1 > Pane 2: ~/project (cargo build) │
│ Tab 2 > Pane 1: /var/log (tail -f)      │
│ Tab 3 > Pane 1: ssh user@prod           │
│ Tab 3 > Pane 2: ssh user@staging        │
└──────────────────────────────────────────┘
```

Features:
- Fuzzy search by tab name, pane CWD, running command, hostname
- Preview of pane content in search
- Keyboard navigation (arrow keys + Enter)
- Shows which panes are running SSH, with hostname
- Quick switch with no mouse needed

## 4. Command Suggestion / Autocomplete

Built-in command suggestions as you type (like Fig/Warp but native).

### Types of Suggestions

1. **Inline Ghost Text** (zsh-autosuggestions style)
   - Shows most likely completion in dimmed text
   - Press `Right Arrow` or `End` to accept
   - Based on history + frequency

2. **Dropdown Panel** (Fig/Warp style)
   - Shows when typing commands, flags, or paths
   - Max 8 suggestions (configurable)
   - Shows description for each suggestion
   - Tab to cycle, Enter to select, Esc to dismiss

3. **Flag Descriptions**
   - When typing `--` after a command, shows available flags
   - Descriptions from man pages and built-in specs
   - Example: `git commit --` shows `--message`, `--amend`, etc. with descriptions

### Suggestion Providers

| Provider | Source | Example |
|----------|--------|---------|
| History | ~/.zsh_history | Previous commands ranked by frequency/recency |
| Path | Filesystem | Files and directories in current/specified path |
| Command | $PATH | Available executables |
| Flags | Man pages + built-in specs | Command-specific flags with descriptions |
| Git | Git repo state | Branches, tags, remotes, changed files |
| Docker | Docker state | Container names, image names |
| Custom | Plugin-provided | Any plugin can add completion specs |

### Built-in Command Specs
Pre-bundled completion specs for:
- **Dev tools**: git, docker, kubectl, npm, yarn, pnpm, cargo, go, pip, poetry, gradle, maven
- **System**: ssh, scp, rsync, curl, wget, systemctl, journalctl
- **Cloud**: aws, gcloud, az
- **Package managers**: apt, brew, dnf, pacman

## 5. Theming System

### What's Configurable

| Setting | Options |
|---------|---------|
| Font family | Any installed monospace font |
| Font size | Any size |
| Font features | Ligatures, stylistic sets |
| Background color | Any color / gradient |
| Background opacity | 0-100% |
| Background image | Any image with blend mode |
| Background blur | Radius configurable |
| Foreground color | Any color |
| Color palette | 16 ANSI + extended colors |
| Cursor style | Block, beam, underline |
| Cursor color | Any color |
| Cursor blink | On/off, rate |
| Selection colors | FG + BG |
| Tab bar style | Colored tabs, minimal, powerline |
| Corner radius | 0-24px |
| Padding | Top, bottom, left, right |
| Line height | Multiplier |

### Theme Files
Themes stored as TOML files in `~/.config/leuwi-panjang/themes/`:

```toml
# ~/.config/leuwi-panjang/themes/leuwi-dark.toml
[theme]
name = "Leuwi Dark"
author = "situkangsayur"

[colors]
foreground = "#e0e0e0"
background = "#1a1a2e"
cursor = "#e94560"
# ... full palette
```

### Theme Browser
Built-in theme browser with live preview (like Kitty's `themes` kitten).

## 6. Profile System

Multiple named profiles with different settings.

```toml
[profiles.default]
shell = "/bin/zsh"
working_directory = "~"
theme = "leuwi-dark"

[profiles.production]
name = "Production"
theme = "production-red"
auto_switch_rules = ["hostname:prod-*"]

[profiles.remote]
name = "Remote Server"
theme = "leuwi-blue"
auto_switch_rules = ["ssh:*"]
```

### Auto-Switching
Profiles switch automatically based on:
- SSH hostname
- Current working directory
- Running command
- Custom rules

## 7. Search

| Action | Shortcut |
|--------|----------|
| Search in scrollback | `Ctrl+Shift+F` |
| Search next | `Enter` or `F3` |
| Search previous | `Shift+Enter` or `Shift+F3` |
| Regex toggle | `Alt+R` |
| Case-sensitive toggle | `Alt+C` |
| Close search | `Escape` |

Features:
- Regex support
- All matches highlighted
- Match count displayed
- Wrap-around search

## 8. Shell Integration

Lightweight escape sequence protocol (inspired by iTerm2/Kitty).

Features:
- **Prompt marks** - navigate between prompts with `Shift+Ctrl+Up/Down`
- **CWD tracking** - tab title shows current directory
- **Command status** - exit code indicator (green/red)
- **"Select Output of Last Command"** - select just the output
- **Works over SSH** - escape sequences pass through transparently

### Default Shell Prompt (Zsh)

Lightweight, informative prompt (simpler than Powerlevel10k):

```
  user@hostname ~/project/src (git:main ✓)
❯
```

Features:
- OS logo icon (  Ubuntu,  Arch,  macOS, etc.)
- user@hostname
- Current directory (abbreviated)
- Git info: branch + status (✓ clean, ✗ dirty, ↑ ahead, ↓ behind)
- Colored prompt indicator (❯ green if last command succeeded, red if failed)
- Minimal - no heavy dependencies like Powerlevel10k

## 9. Keyboard Shortcuts

All shortcuts are configurable in `config.toml`.

### Full Default Keymap

| Category | Action | Default Shortcut |
|----------|--------|-----------------|
| **Tabs** | New tab | `Ctrl+Shift+T` |
| | Close tab | `Ctrl+Shift+W` |
| | Next tab | `Ctrl+Tab` |
| | Previous tab | `Ctrl+Shift+Tab` |
| | Go to tab N | `Alt+1` - `Alt+9` |
| | Search tabs/panes | `Ctrl+Shift+Space` |
| **Splits** | Split horizontal | `Ctrl+Shift+H` |
| | Split vertical | `Ctrl+Shift+V` |
| | Navigate pane | `Alt+Arrow` |
| | Resize pane | `Ctrl+Alt+Arrow` |
| | Maximize pane | `Ctrl+Shift+Enter` |
| | Broadcast input | `Ctrl+Shift+B` |
| **Clipboard** | Copy | `Ctrl+Shift+C` |
| | Paste | `Ctrl+Shift+V` |
| | Paste special | `Ctrl+Shift+Alt+V` |
| **Search** | Find | `Ctrl+Shift+F` |
| | Find next | `F3` |
| | Find previous | `Shift+F3` |
| **View** | Zoom in | `Ctrl++` |
| | Zoom out | `Ctrl+-` |
| | Reset zoom | `Ctrl+0` |
| | Fullscreen | `F11` |
| | Toggle status bar | `Ctrl+Shift+S` |
| **Terminal** | Scroll up | `Shift+PageUp` |
| | Scroll down | `Shift+PageDown` |
| | Scroll to top | `Shift+Home` |
| | Scroll to bottom | `Shift+End` |
| | Clear | `Ctrl+Shift+K` |
| | Reset | `Ctrl+Shift+R` |
| **Other** | Settings | `Ctrl+Shift+,` |
| | Command palette | `Ctrl+Shift+P` |
| | Theme browser | `Ctrl+Shift+Alt+T` |

## 10. Status Bar

Configurable bottom bar with modular components.

```
[git:main ✓] [cpu:12%] [mem:4.2G/16G] [user@host] [14:32:05]
```

Available components:
- Git branch + status
- CPU usage
- Memory usage
- Hostname
- Clock
- Battery (laptop)
- Active SSH host
- Running command
- Custom (plugin-provided)

Can be hidden (`Ctrl+Shift+S` or config).

## 11. GPU Rendering

- wgpu backend (Vulkan on Linux, Metal on macOS, DX12 on Windows)
- Glyph texture atlas cached on GPU
- Cell grid rendered in single draw call
- Ligature support via rustybuzz (Rust HarfBuzz)
- VSync-aware rendering
- Low input latency (<5ms target)
- Falls back gracefully (wgpu handles backend selection)

## 12. Standard SSH (No Special Commands)

- `TERM=xterm-256color` by default
- SSH just works: `ssh user@host`
- No `kitty +kitten ssh` nonsense
- Optionally copy custom terminfo on first connect (user-initiated, not required)
- Shell integration escape sequences pass through SSH transparently

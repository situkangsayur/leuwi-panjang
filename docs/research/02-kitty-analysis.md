# Kitty Terminal - Technical Analysis

## Overview

Kitty is a GPU-accelerated terminal emulator created by Kovid Goyal. Known for speed, modern features, and extensibility via "kittens".

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Core / Rendering | C (OpenGL 3.3+) |
| UI / Config / Extensions | Python 3 |
| Standalone Tools (SSH, etc.) | Go |
| Text Shaping | HarfBuzz + FreeType |
| Rendering | GPU-accelerated OpenGL |
| Config | Plain text `kitty.conf` |
| Platforms | Linux (X11/Wayland), macOS |

## Architecture

```
Kitty (Multi-process)
├── Main Process (C + Python)
│   ├── OpenGL Rendering Loop
│   │   ├── Glyph texture atlas (GPU)
│   │   ├── Cell grid shader (fragment shader)
│   │   └── VSync-aware rendering
│   ├── Boss (Python) - manages OS windows, tabs, panes
│   │   ├── OS Window
│   │   │   ├── Tab 1
│   │   │   │   ├── Window/Pane A (layout-managed)
│   │   │   │   └── Window/Pane B (layout-managed)
│   │   │   └── Tab 2
│   │   │       └── Window/Pane C
│   │   └── Layout Engine (tall, fat, grid, splits, stack, etc.)
│   ├── VT Parser (C - high-performance)
│   ├── Screen Buffer / Cell Grid (C)
│   └── IPC / Remote Control (JSON over Unix socket)
├── Child Processes (shells via PTY)
└── Kittens (Python/Go subprocesses)
    ├── icat (image display)
    ├── ssh (terminfo + shell integration copy)
    ├── diff (side-by-side viewer)
    ├── themes (interactive browser)
    ├── hints (URL/path picker)
    └── Custom kittens
```

## Features

### Window Management
- Multiple tabs with configurable tab bar (top/bottom)
- Multiple panes per tab with **layout engines**:
  - `tall`, `fat`, `grid`, `horizontal`, `vertical`, `splits`, `stack`
- Layout cycling with keyboard shortcuts
- Move windows between tabs

### Rendering & Display
- GPU-accelerated OpenGL rendering (all drawing in single draw call)
- Ligature support (HarfBuzz)
- Background transparency and blur
- Font features: OpenType toggling, variable fonts, per-codepoint overrides
- Undercurl, colored underlines, strikethrough

### Protocols (Standard + Custom)
- Full xterm/VT220 emulation + true color
- **Kitty Graphics Protocol** - inline images (PNG, JPEG, GIF, animation)
- **Kitty Keyboard Protocol** - unambiguous key events (adopted by WezTerm, foot, Ghostty, Neovim)
- Shell integration (prompt jumping, per-command scrollback)
- OSC 8 hyperlinks, OSC 52 clipboard, OSC 99 notifications
- Synchronized output (mode 2026 - kitty originated)

### Extensibility
- **Remote Control API** - full IPC to control kitty from scripts
- **Kittens** - Python/Go plugin system
- Custom key actions
- Startup sessions (define complex layouts)

## Performance

| Metric | Value |
|--------|-------|
| Throughput | 500+ MB/s raw text rendering |
| Base Memory | ~30-60 MB |
| Startup | Sub-200ms |
| Input Latency | 2-5ms (among lowest) |
| Rendering | GPU (single draw call per screen) |

## Strengths (What to Learn From)

1. **GPU-accelerated rendering** - offloads to GPU, CPU free for parsing
2. **Kitty Graphics Protocol** - most capable image display in terminals
3. **Kitty Keyboard Protocol** - fixes terminal keyboard ambiguities
4. **Kitten extension system** - clean plugin architecture
5. **Layout engines** - multiple built-in layout modes for splits
6. **Remote Control API** - powerful scripting from external tools
7. **Shell integration** - prompt navigation, per-command scrollback
8. **Performance** - consistently fastest in benchmarks
9. **Chromeless mode** - `hide_window_decorations yes` removes titlebar

## Weaknesses (What to Avoid / Improve On)

1. **SSH nightmare** - `TERM=xterm-kitty` breaks on remote servers without terminfo
   - Requires `kitty +kitten ssh` workaround - NOT standard SSH
   - Users hate this friction
2. **No Windows support** - Linux and macOS only
3. **Requires OpenGL 3.3+** - fails in VMs, containers, X-forwarding
4. **No Sixel support** - deliberate omission, ecosystem friction
5. **Maintainer communication** - blunt/dismissive on GitHub issues
6. **Custom protocols create lock-in** - TERM, keyboard, graphics all non-standard
7. **Python startup overhead** - slower than pure-C terminals

## SSH Problem Deep-Dive

The #1 user complaint. Kitty sets `TERM=xterm-kitty` which remote servers don't recognize.

**What `kitty +kitten ssh` does:**
1. Copies xterm-kitty terminfo to remote `~/.terminfo/`
2. Copies shell integration scripts
3. Enables graphics protocol over SSH
4. Enables file transfer

**Problems this creates:**
- Not standard SSH - breaks complex ProxyCommand setups
- Only works inside kitty (muscle memory breaks in other terminals)
- First-connection latency overhead
- Permission issues on locked-down servers
- Ecosystem lock-in

**Leuwi Panjang approach:** Use standard `TERM=xterm-256color` by default. Support custom terminfo optionally but NEVER require special SSH commands. SSH should just work.

## Lessons for Leuwi Panjang

| Aspect | Learn | Avoid |
|--------|-------|-------|
| Rendering | GPU-accelerated (use wgpu instead of OpenGL) | OpenGL-only (use wgpu for Vulkan/Metal/DX12) |
| Extensions | Kitten-style plugin system | |
| Keyboard | Adopt Kitty Keyboard Protocol | |
| Graphics | Support image protocol | Refusing Sixel |
| SSH | | Custom TERM that breaks SSH |
| Layouts | Multiple layout engines | |
| Window | Chromeless mode | |
| Cross-platform | | No Windows support |

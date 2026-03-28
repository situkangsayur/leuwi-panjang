# GNOME Terminal - Technical Analysis

## Overview

GNOME Terminal is the default terminal emulator for the GNOME desktop environment, built as a thin shell around the VTE (Virtual Terminal Emulator) library.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | C |
| GUI Toolkit | GTK 4 (GTK 3 in older versions) |
| Terminal Engine | VTE (libvte-2.91) |
| Text Rendering | Pango + Cairo (CPU-based) |
| Config Storage | dconf / GSettings |
| IPC | D-Bus |
| Build System | Meson |

## Architecture

```
GNOME Terminal (Application - thin shell)
├── GApplication (singleton via D-Bus)
├── gnome-terminal-server (long-lived process)
│   ├── Terminal Window (GtkWindow)
│   │   └── GtkNotebook (Tab Container)
│   │       ├── Terminal Screen 1 (VteTerminal)
│   │       ├── Terminal Screen 2 (VteTerminal)
│   │       └── ...
│   └── Profile System (GSettings schemas per UUID)
└── D-Bus Client (gnome-terminal binary)

VTE Library (does the real work)
├── PTY management
├── Escape sequence parsing (VT220 + xterm extensions)
├── Text rendering (Pango/Cairo)
├── Scrollback ring buffer
├── Selection handling
└── Regex matching (PCRE2 for URL detection)
```

Key architectural trait: **Single-process model via D-Bus**. The `gnome-terminal` binary is a thin D-Bus client; all windows/tabs live in `gnome-terminal-server`.

## Features

- Multiple tabs per window with drag-and-drop reordering
- Robust profile system (per-profile: font, colors, cursor, scrollback, encoding, custom command)
- Configurable color palettes (Tango, Solarized, GNOME, etc.)
- Transparency support (composited desktops)
- In-terminal regex search with highlighting
- Automatic URL detection (Ctrl+click)
- Zoom in/out, fullscreen
- Bidirectional text, CJK/wide character support
- Sixel graphics (VTE 0.72+)
- OSC 8 hyperlinks, OSC 7 CWD tracking, OSC 133 prompt marking
- Read-only mode toggle

## Terminal Emulation

- **TERM**: `xterm-256color`
- VT220-compatible with extensive xterm extensions
- 256-color + 24-bit true color support
- Full mouse protocol support (X10, normal, SGR extended)
- Bracketed paste, alternate screen buffer, focus events
- Synchronized output (mode 2026)
- Underline variants (single, double, curly, dotted, dashed)

## Performance

| Metric | Value |
|--------|-------|
| Idle Memory | ~30-50 MB RSS |
| Per Tab | ~5-15 MB |
| Cold Start | ~300-500ms |
| Warm Start | ~50-150ms (D-Bus server running) |
| Rendering | CPU-based (Cairo/Pango), ~50-100 MB/s throughput |

## Strengths (What to Learn From)

1. **Deep desktop integration** - seamless GNOME look and feel
2. **VTE is excellent** - one of the most compliant terminal emulation libraries
3. **Correct Unicode handling** - leveraging Pango for text shaping
4. **Lightweight for GUI terminal** - low overhead
5. **Robust profile system** - multiple named configs per use case
6. **Reliable scrollback** - efficient ring buffer
7. **Good accessibility** - ATK/AT-SPI screen reader support

## Weaknesses (What to Avoid / Improve On)

1. **Header bar wastes vertical space** - CSD header bar is a top complaint
2. **NO split panes** - must use tmux/screen
3. **NO plugin/extension system** - zero extensibility
4. **Limited config UI** - many settings only via dconf-editor
5. **NO GPU-accelerated rendering** - slower than Alacritty/Kitty for fast output
6. **NO ligature support** - long-standing VTE limitation
7. **NO image protocol** (beyond basic Sixel)
8. **Feature removal tendency** - GNOME philosophy removes features
9. **NO session save/restore**
10. **NO SSH management**

## Lessons for Leuwi Panjang

| Aspect | Learn | Avoid |
|--------|-------|-------|
| Stability | VTE-level terminal emulation quality | |
| UI | | Large header bar eating vertical space |
| Features | Profile system design | Missing splits |
| Config | | dconf-only config (no file-based) |
| Extensions | | No plugin system |
| Rendering | | CPU-only rendering |

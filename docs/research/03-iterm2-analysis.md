# iTerm2 - Technical Analysis

## Overview

iTerm2 is the gold-standard terminal emulator for macOS. Written in Objective-C/Swift with Metal GPU rendering. Known for powerful split panes, shell integration, tmux integration, and Python scripting API.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Objective-C (core), Swift (newer modules) |
| GUI Framework | Cocoa / AppKit (native macOS) |
| Rendering | Metal (GPU), fallback to OpenGL |
| Text Shaping | CoreText |
| Scripting | Python 3 (WebSocket-based API) |
| Config Storage | macOS UserDefaults (plist), Dynamic Profiles (JSON) |
| Platform | macOS ONLY |

## Architecture

```
iTerm2.app
├── Terminal Emulator Core
│   ├── VT100Terminal / VT100Parser (state machine)
│   └── PTYTextView (NSView - renders grid)
├── Session Management
│   ├── PseudoTerminal (PTYWindow) - window controller
│   ├── PTYTab - tab, contains binary tree of splits
│   └── PTYSession - single terminal (PTY, parser, scrollback)
├── Metal Renderer (iTermMetalDriver)
├── Profile System (UserDefaults + Dynamic JSON Profiles)
├── Trigger Engine (regex -> actions per line)
├── Shell Integration (escape sequence protocol)
├── tmux Integration (control-center mode -CC)
├── Scripting Bridge (WebSocket <-> Python 3)
└── Search / Autocomplete subsystem
```

### Split Pane Architecture
Each tab maintains a **binary tree of split views** where leaves are PTYSession instances. This enables arbitrarily nested horizontal/vertical splits.

## Features

### Split Pane System (Best-in-Class)
- **Cmd+D** = split vertical, **Cmd+Shift+D** = split horizontal
- Arbitrarily nested splits
- Navigate: Cmd+Opt+Arrow or Cmd+]/[
- Maximize/restore pane: Cmd+Shift+Enter
- Broadcast input to all panes simultaneously
- Drag to reorder/swap panes
- Dimming of inactive panes

### Profile System
- Named config bundles (~200+ settings each)
- Dynamic Profiles (JSON files, version-controllable)
- **Automatic Profile Switching** based on hostname/username/path
- Profile switching per session mid-session
- Color schemes importable as `.itermcolors`

### Shell Integration
- Lightweight escape sequences injected into shell RC
- Prompt marks (navigate between prompts)
- Command status decorations (blue=running, red=failed, green=success)
- Automatic Profile Switching on SSH
- "Select Output of Last Command"
- Upload/download via `it2dl`/`it2ul`
- **Works over SSH** transparently

### Trigger System
- Regex matched against each line of output
- Actions: highlight, notify, run command, set mark, send text, open URL, run Python script, set profile
- Powerful automation engine

### tmux Integration
- `tmux -CC` control center mode
- tmux windows become native iTerm2 tabs
- tmux panes become native split panes
- Full native scrollback for tmux panes
- Sessions survive SSH disconnection

### Search & Autocomplete
- Regex search across scrollback
- Autocomplete from scrollback content (Cmd+;)
- Command history (Cmd+Shift+;)
- Recent directories (Cmd+Opt+/)
- Paste history (Cmd+Shift+H)

### Other Notable Features
- **Instant Replay** (Cmd+Opt+B) - DVR for terminal, scrub through history
- **Password Manager** - macOS Keychain integration
- **Inline Images** (`imgcat`) + Sixel
- **Badges** - text overlay showing hostname, etc.
- **Marks & Annotations** - bookmarks in scrollback
- **Semantic History** - Cmd+click opens files in editor
- **Hotkey Window** - quake-style dropdown terminal
- **Session Restoration** - reopens tabs/panes/CWD on restart
- **Undo Close** (Cmd+Z after closing session)
- **Status Bar** - configurable components (git, CPU, memory, etc.)
- Font ligature support

## Performance

| Metric | Value |
|--------|-------|
| Base Memory | ~80-150 MB |
| Throughput | ~80-120 MB/s (Metal) |
| Cold Start | ~1.0-1.5s |
| Warm Start | ~0.5-0.8s |
| Rendering | Metal GPU (CPU fallback) |

Memory is the main concern - many tabs with large scrollback can reach 1-2 GB.

## Strengths (What to Learn From)

1. **Split pane UX** - Cmd+D/Cmd+Shift+D is intuitive and memorable
2. **Automatic Profile Switching** - color changes based on SSH host
3. **Shell Integration** - lightweight, works over SSH, enables prompt marks
4. **tmux -CC integration** - native panes backed by persistent tmux
5. **Trigger system** - powerful regex-based automation
6. **Instant Replay** - unique, genuinely useful feature
7. **Search & autocomplete** - regex search + scrollback autocomplete
8. **Password Manager** - Keychain integration for sudo/SSH
9. **Status Bar** - configurable components
10. **Python scripting API** - full programmatic control

## Weaknesses (What to Avoid / Improve On)

1. **macOS only** - biggest limitation
2. **Memory hungry** - can reach gigabytes with many sessions
3. **Slow startup** - 0.5-1.5s vs <100ms for lightweight terminals
4. **Settings UI overwhelming** - hundreds of options, poor discoverability
5. **Config not portable** - plist format, not plain text
6. **Python API has friction** - asyncio-based, high barrier to entry
7. **No built-in multiplexer persistence** - splits gone if app crashes

## Lessons for Leuwi Panjang

| Aspect | Learn | Avoid |
|--------|-------|-------|
| Splits | Binary tree split model, simple shortcuts | |
| Profiles | Automatic switching by host/path | Overwhelming settings UI |
| Shell Integration | Lightweight escape sequence protocol | |
| Triggers | Regex-based automation | |
| Password | Encrypted credential storage | Keychain-only (cross-platform needed) |
| Config | Dynamic Profiles (JSON) concept | plist format |
| Search | Scrollback autocomplete | |
| Status Bar | Configurable components | |
| Instant Replay | Terminal state DVR | |
| Memory | | Unbounded memory growth |
| Startup | | >500ms startup time |
| Platform | | Single-platform lock-in |

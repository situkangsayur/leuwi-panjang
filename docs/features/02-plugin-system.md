# Leuwi Panjang - Plugin System

## Overview

Leuwi Panjang uses a **WASM-based plugin system** for safe, sandboxed, language-agnostic extensibility.

## Why WASM?

| Approach | Pros | Cons |
|----------|------|------|
| Dynamic Libraries (.so/.dll) | Fast, full access | Unsafe, platform-specific, can crash host |
| Lua scripting | Easy, lightweight | Limited performance, single language |
| Python scripting | Rich ecosystem | Heavy (runtime), slow |
| **WASM (wasmtime)** | **Sandboxed, fast, any language, portable** | Slightly more complex to develop |

## Architecture

```
┌──────────────────────────────────────────────────┐
│                 Leuwi Panjang                     │
│                                                   │
│  ┌─────────────────────────────────────────────┐  │
│  │           Plugin Host (wasmtime)            │  │
│  │                                             │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐   │  │
│  │  │ Plugin A │ │ Plugin B │ │ Plugin C │   │  │
│  │  │ (WASM)   │ │ (WASM)   │ │ (WASM)   │   │  │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘   │  │
│  │       │             │             │         │  │
│  │  ┌────v─────────────v─────────────v─────┐   │  │
│  │  │         Host API (Rust traits)       │   │  │
│  │  │                                      │   │  │
│  │  │  TerminalAPI    │  ConfigAPI         │   │  │
│  │  │  CredentialAPI  │  AuditAPI          │   │  │
│  │  │  UIAPI          │  NotificationAPI   │   │  │
│  │  │  NetworkAPI     │  FileAPI           │   │  │
│  │  └──────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────┘  │
│                                                   │
│  ┌─────────────────┐  ┌────────────────────────┐  │
│  │ Credential Vault│  │     Audit Logger       │  │
│  │ (AES-256-GCM)   │  │ (append-only log)      │  │
│  └─────────────────┘  └────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

## Plugin Manifest

Every plugin has a `plugin.toml`:

```toml
[plugin]
name = "ai-claude"
version = "0.1.0"
description = "Claude CLI Integration for Leuwi Panjang Terminal"
author = "situkangsayur"
license = "MIT"
homepage = "https://github.com/situkangsayur/leuwi-panjang-plugins"
wasm_file = "ai_claude.wasm"

[capabilities]
# What the plugin can access
terminal_read = true       # Read terminal output
terminal_write = true      # Write to terminal
credential_access = true   # Access encrypted vault
notification = true        # Show desktop notifications
ui_components = true       # Add status bar items, menu items
network_access = false     # No direct network access
file_read = false          # No filesystem read
file_write = false         # No filesystem write

[permissions]
requires_user_approval = true   # Always ask before actions
audit_all_actions = true        # Log everything to audit trail

[ui]
# UI elements the plugin adds
status_bar_components = ["ai-status"]
menu_items = ["Start AI Session", "AI Settings"]
keybindings = [
    { key = "ctrl+shift+a", action = "toggle_ai_panel" }
]
```

## Host API

### TerminalAPI
```rust
trait TerminalAPI {
    /// Read current terminal screen content
    fn read_screen(&self) -> Screen;

    /// Read scrollback buffer (limited range)
    fn read_scrollback(&self, lines: usize) -> Vec<String>;

    /// Write text to terminal (requires user approval if AI-initiated)
    fn write_text(&self, text: &str) -> Result<(), PermissionDenied>;

    /// Send keystrokes to terminal
    fn send_keys(&self, keys: &str) -> Result<(), PermissionDenied>;

    /// Get current working directory
    fn get_cwd(&self) -> PathBuf;

    /// Get running command
    fn get_running_command(&self) -> Option<String>;

    /// Subscribe to terminal output events
    fn on_output(&self, callback: fn(String));

    /// Subscribe to command completion events
    fn on_command_complete(&self, callback: fn(command: String, exit_code: i32));
}
```

### CredentialAPI
```rust
trait CredentialAPI {
    /// Store a credential (encrypted)
    fn store(&self, key: &str, value: &str) -> Result<()>;

    /// Retrieve a credential (requires user approval)
    fn retrieve(&self, key: &str) -> Result<String, PermissionDenied>;

    /// Delete a credential
    fn delete(&self, key: &str) -> Result<()>;

    /// List credential keys (not values)
    fn list_keys(&self) -> Vec<String>;
}
```

### AuditAPI
```rust
trait AuditAPI {
    /// Log an action to the audit trail
    fn log_action(&self, action: AuditEntry);

    /// Query audit trail
    fn query(&self, filter: AuditFilter) -> Vec<AuditEntry>;
}

struct AuditEntry {
    timestamp: DateTime,
    actor: Actor,           // Human, AI(name), System
    action: String,         // What was done
    target: String,         // What was affected
    result: ActionResult,   // Success, Failure, Denied
    approved_by: Option<String>,  // Who approved (if AI action)
    details: Option<String>,
}
```

### UIAPI
```rust
trait UIAPI {
    /// Add a status bar component
    fn add_status_component(&self, component: StatusComponent);

    /// Update status bar component
    fn update_status(&self, id: &str, content: &str);

    /// Show a notification
    fn notify(&self, title: &str, body: &str, level: NotifyLevel);

    /// Add menu item to hamburger menu
    fn add_menu_item(&self, item: MenuItem);

    /// Show a side panel
    fn show_panel(&self, side: PanelSide, content: PanelContent);

    /// Show dialog (confirmation, input, etc.)
    fn show_dialog(&self, dialog: Dialog) -> DialogResult;
}
```

## Permission System

Every plugin action that affects the system goes through a permission gate:

```
Plugin requests action
        │
        v
┌─────────────────────────────┐
│   Permission Manager         │
│                              │
│   Check: Does plugin have    │
│   capability for this?       │
│          │                   │
│          v                   │
│   Check: Is user approval    │
│   required?                  │
│          │                   │
│   ┌──────┴──────┐           │
│   │ Auto-allow  │ No ───►Execute
│   │ required?   │           │
│   └──────┬──────┘           │
│          │ Yes              │
│          v                   │
│   ┌──────────────────┐      │
│   │ Show Dialog:     │      │
│   │ "AI wants to run:│      │
│   │  sudo apt..."    │      │
│   │ [Allow] [Deny]   │      │
│   │ [Always Allow]   │      │
│   └──────┬───────────┘      │
│          │                   │
│          v                   │
│   Log to Audit Trail         │
│          │                   │
│          v                   │
│   Execute or Deny            │
└─────────────────────────────┘
```

### Permission Levels
1. **Always Ask** - prompt every time
2. **Remember for Session** - auto-allow until terminal restart
3. **Always Allow** - never ask again (can be revoked)
4. **Always Deny** - block this action type

## Credential Vault

Encrypted storage for sensitive data (passwords, API keys, SSH keys).

### Encryption
- **Algorithm**: AES-256-GCM
- **Key Derivation**: Argon2id (from master password)
- **Storage**: `~/.config/leuwi-panjang/vault.encrypted`
- Master password required to unlock (can be cached for session)
- Auto-lock after configurable timeout

### What Gets Stored
- sudo / root passwords (for AI-initiated commands)
- SSH passphrases
- API keys (for AI plugins)
- Custom secrets

## Audit Trail

Append-only log of all significant actions.

### Storage
- `~/.local/share/leuwi-panjang/audit/YYYY-MM-DD.log`
- JSON Lines format for easy parsing
- Rotated daily, compressed after 7 days

### Example Entries
```jsonl
{"ts":"2026-03-26T14:32:01Z","actor":"ai:claude","action":"terminal_write","target":"pane:1","content":"sudo apt install nginx","result":"pending_approval"}
{"ts":"2026-03-26T14:32:05Z","actor":"human","action":"approve","target":"ai:claude:terminal_write","result":"approved"}
{"ts":"2026-03-26T14:32:06Z","actor":"ai:claude","action":"terminal_write","target":"pane:1","content":"sudo apt install nginx","result":"executed","approved_by":"human"}
{"ts":"2026-03-26T14:32:10Z","actor":"ai:claude","action":"credential_access","target":"vault:sudo_password","result":"executed","approved_by":"human"}
```

### Audit Viewer
- Built-in viewer: hamburger menu -> Audit Trail
- Filter by: actor, action type, date range, result
- Export to CSV/JSON

## Plugin Development

### Writing a Plugin in Rust

```rust
// src/lib.rs
use leuwi_panjang_plugin_sdk::*;

#[plugin_init]
fn init(api: &PluginAPI) -> Result<()> {
    // Register status bar component
    api.ui().add_status_component(StatusComponent {
        id: "ai-status",
        content: "AI: Ready",
        position: StatusPosition::Right,
    });

    // Register keybinding
    api.register_keybinding("ctrl+shift+a", |api| {
        api.ui().show_panel(PanelSide::Right, PanelContent::Custom {
            title: "Claude AI",
            render: render_ai_panel,
        });
    });

    Ok(())
}

fn render_ai_panel(api: &PluginAPI) -> PanelContent {
    // AI chat interface
    // ...
}
```

Build: `cargo build --target wasm32-wasi --release`

### Writing a Plugin in Go, AssemblyScript, etc.
Any language that compiles to WASM can write plugins using the same API (via WASM interface types).

## Desktop vs Mobile Plugins

| Feature | Desktop Plugins | Mobile Plugins |
|---------|----------------|----------------|
| Runtime | wasmtime | wasmtime (via Rust FFI) |
| Same WASM binary | Yes | Yes |
| Network access | Via capability | Via capability |
| UI panels | Side panels, dialogs | Bottom sheets, dialogs |
| Credential vault | OS-level encryption | OS-level keychain |

## Plugin Distribution

Plugins distributed via the `leuwi-panjang-plugins` repository:
- git@github.com:situkangsayur/leuwi-panjang-plugins.git

### Plugin Registry (future)
- Central registry for community plugins
- `leuwi install plugin-name`
- Verified/signed plugins
- Automatic updates

# Leuwi Panjang - AI Integration

## Overview

AI integration in Leuwi Panjang is implemented as **separate plugins per AI provider**. This modular approach means:
- Install only the AI you use
- No bloated dependencies
- Each plugin is independently maintained
- Easy to add new AI providers

## Available AI Plugins

| Plugin | AI Provider | Type |
|--------|------------|------|
| `plugin-claude` | Anthropic Claude (CLI) | Cloud AI |
| `plugin-gemini` | Google Gemini (CLI) | Cloud AI |
| `plugin-ollama` | Ollama (local models) | Local AI |
| `plugin-wireguard-ai` | Remote AI via WireGuard | Network AI |

## Common AI Features (All Plugins)

### 1. AI Side Panel
```
┌────────────────────────────┬──────────────────┐
│ Terminal                    │ AI Assistant     │
│                            │                  │
│ user@host ~/project        │ You: Fix the     │
│ $ cargo build              │ build error      │
│ error[E0308]: mismatched   │                  │
│ types                      │ Claude: The error│
│                            │ is in line 42... │
│                            │                  │
│                            │ [Send Selection] │
│                            │ [Run Suggestion] │
│                            │ [Clear Chat]     │
└────────────────────────────┴──────────────────┘
```

### 2. Permission Gate
Every AI action requires explicit user approval:

```
┌────────────────────────────────────────┐
│  AI wants to execute:                  │
│                                        │
│  $ sudo systemctl restart nginx        │
│                                        │
│  [Allow Once] [Allow Session] [Deny]   │
│                                        │
│  Actor: Claude CLI                     │
│  Risk: HIGH (sudo)                     │
└────────────────────────────────────────┘
```

### 3. Credential Vault
Securely store credentials that AI may need:
- sudo/admin passwords (encrypted AES-256-GCM)
- SSH passphrases
- Service tokens
- Never stored in plain text
- Master password to unlock vault
- Auto-lock after inactivity

### 4. Audit Trail
Every AI action is logged:
```
[2026-03-26 14:32:01] ACTOR: ai:claude  ACTION: read_terminal  RESULT: allowed
[2026-03-26 14:32:03] ACTOR: ai:claude  ACTION: suggest_command "cargo fix"  RESULT: pending
[2026-03-26 14:32:05] ACTOR: human     ACTION: approve ai:claude suggestion  RESULT: approved
[2026-03-26 14:32:06] ACTOR: ai:claude  ACTION: execute "cargo fix"  RESULT: success
```

### 5. Context Awareness
AI plugins can read:
- Current terminal output (with permission)
- Current working directory
- Running command
- Git status
- Shell history (limited)

## Plugin: Claude CLI (`plugin-claude`)

### Features
- Integrates Anthropic's Claude CLI (`claude`) into Leuwi Panjang
- Side panel for Claude conversation
- Send terminal selection to Claude
- Claude can suggest and execute commands (with approval)
- Persistent conversation across sessions
- Multiple Claude profiles (different system prompts)

### Shortcuts
| Action | Shortcut |
|--------|----------|
| Toggle Claude panel | `Ctrl+Shift+A` |
| Send selection to Claude | `Ctrl+Shift+Alt+A` |
| Quick ask (inline) | `Ctrl+Shift+?` |

### Configuration
```toml
[plugins.ai-claude]
enabled = true
panel_position = "right"
panel_width = 40  # percentage
auto_context = true  # send CWD, git status as context
max_history = 100  # conversation messages to keep
approval_mode = "always_ask"  # always_ask, session, auto
```

## Plugin: Gemini CLI (`plugin-gemini`)

Same architecture as Claude plugin but for Google's Gemini CLI.

### Configuration
```toml
[plugins.ai-gemini]
enabled = true
panel_position = "right"
model = "gemini-2.5-pro"
```

## Plugin: Ollama (`plugin-ollama`)

### Features
- Local AI - no internet required
- Privacy-first (nothing leaves your machine)
- Support multiple models (llama, codellama, mistral, etc.)
- Model management (pull, list, delete)
- Lower latency than cloud AI

### Configuration
```toml
[plugins.ai-ollama]
enabled = true
host = "localhost:11434"
default_model = "codellama:13b"
context_window = 4096
```

## Plugin: WireGuard Remote AI (`plugin-wireguard-ai`)

### Architecture
```
┌──────────────┐   WireGuard    ┌─────────────────────────┐
│ Leuwi Panjang│◄──Tunnel─────►│ Leuwi Panjang Server    │
│ (Client)     │               │                          │
│              │               │ ┌──────────────────────┐ │
│ plugin-      │               │ │    AI Router          │ │
│ wireguard-ai │               │ │                       │ │
│              │               │ │ Claude API ─► Claude  │ │
│              │               │ │ Gemini API ─► Gemini  │ │
│              │               │ │ Ollama ────► Local AI │ │
│              │               │ └──────────────────────┘ │
└──────────────┘               │                          │
                               │ Auth + Audit + Rate Limit│
                               └─────────────────────────┘
```

### Benefits
- **No API keys on client** - server manages all API authentication
- **Centralized billing** - one API key for team/org
- **Audit everything** - server logs all AI requests
- **Rate limiting** - prevent abuse
- **Model routing** - server decides which AI handles which request
- **Works from mobile** - mobile app connects via WireGuard

### Configuration
```toml
[plugins.ai-wireguard]
enabled = true
server = "10.0.0.1:8443"  # WireGuard IP
auth_token_vault_key = "wireguard_auth"  # stored in credential vault
preferred_ai = "claude"
fallback_ai = "ollama"
```

## Security Model

### Principle of Least Privilege
1. AI plugins can ONLY do what their capabilities allow
2. Every action goes through the permission gate
3. Users can revoke permissions at any time
4. High-risk actions (sudo, rm, network) always require approval

### Risk Classification
| Risk Level | Actions | Default Approval |
|------------|---------|-----------------|
| LOW | Read CWD, read terminal (visible) | Auto-allow |
| MEDIUM | Execute safe commands (ls, cat, git status) | Ask once per session |
| HIGH | sudo, rm, network access, file write | Always ask |
| CRITICAL | rm -rf, format, credential access | Always ask + confirm |

### Data Protection
- AI plugins CANNOT read credential vault without explicit approval
- Credentials are NEVER sent to AI providers
- AI only receives the password AFTER user approves a specific command
- Passwords are passed via PTY stdin, never in command arguments

## Usage Flow Example

```
1. User opens Leuwi Panjang
2. User presses Ctrl+Shift+A to open Claude panel
3. User types: "Help me fix the compilation error"
4. Claude reads terminal output (auto-allowed, LOW risk)
5. Claude suggests: "The error is in src/main.rs:42. Try: cargo fix"
6. User clicks [Run Suggestion]
7. Permission dialog appears: "Claude wants to run: cargo fix" [Allow] [Deny]
8. User clicks [Allow]
9. Command executes in terminal
10. Audit trail logs: Claude suggested, User approved, Command executed
```

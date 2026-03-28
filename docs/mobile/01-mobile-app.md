# Leuwi Panjang Mobile - Architecture & Features

## Overview

Leuwi Panjang Mobile is the **same Rust codebase** as desktop, compiled for Android and iOS using **Makepad**. No Flutter, no Dart, no FFI bridge — pure Rust everywhere.

## Tech Stack (Same as Desktop)

| Layer | Technology | Notes |
|-------|-----------|-------|
| UI Framework | Makepad | Same framework, same code, mobile targets |
| Terminal Core | Shared Rust crates | VT parser, screen buffer, config — 100% shared |
| Rendering | Makepad GPU renderer | Metal (iOS), OpenGL ES (Android) |
| SSH Client | russh (pure Rust) | Primary use case on mobile |
| Credential Storage | OS Keychain (via Rust) | Android Keystore / iOS Keychain |
| WireGuard | boringtun (embedded) | Zero-config pairing to desktop |
| Plugin Runtime | wasmtime | Same WASM plugins as desktop |

## Architecture

```
┌──────────────────────────────────────────┐
│          Makepad UI Layer                │
│  (same framework as desktop,             │
│   compiled for mobile targets)           │
│                                          │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │ Terminal  │ │Connection│ │Settings │ │
│  │ View     │ │ Manager  │ │ Screen  │ │
│  └────┬─────┘ └────┬─────┘ └────┬────┘ │
└───────┼─────────────┼─────────────┼──────┘
        │             │             │
┌───────v─────────────v─────────────v──────┐
│          Shared Rust Core                │
│  (identical code as desktop)             │
│                                          │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │ VT Parser│ │ SSH      │ │ Plugin  │ │
│  │ (shared) │ │ Client   │ │ Host    │ │
│  └──────────┘ └──────────┘ └─────────┘ │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │ Crypto   │ │ WireGuard│ │ Config  │ │
│  │ Vault    │ │(boringtun│ │ Manager │ │
│  └──────────┘ └──────────┘ └─────────┘ │
└──────────────────────────────────────────┘
```

### Code Sharing

| Component | Desktop | Mobile | Shared? |
|-----------|---------|--------|---------|
| VT Parser | Yes | Yes | **100%** |
| Config System | Yes | Yes | **100%** |
| Plugin System (WASM) | Yes | Yes | **100%** |
| SSH Client | - | Yes | Mobile-primary |
| Credential Vault | Yes | Yes | **100%** (storage backend differs) |
| WireGuard | Server mode | Client mode | **Shared core**, different mode |
| UI Components | Desktop layout | Mobile layout | **Shared widgets**, different layout |
| Terminal Renderer | Yes | Yes | **100%** (Makepad shader) |

With Makepad, platform-specific code is minimal — mostly just:
- Touch gestures vs mouse input
- Mobile keyboard handling
- OS-level keychain access
- Screen size/orientation handling

## Mobile-Specific Features

### 1. SSH Connection Manager

The primary use case for mobile — quick SSH access to servers.

```
┌─────────────────────────────────────┐
│ ← Connections                    ⚙️ │
├─────────────────────────────────────┤
│                                     │
│ 📁 Production                       │
│   ├── 🟢 Web Server 1              │
│   │   user@web1.prod.com:22        │
│   │   Last: 2h ago                 │
│   ├── 🟢 Web Server 2              │
│   │   user@web2.prod.com:22        │
│   │   Last: 5h ago                 │
│   └── 🔴 Database                   │
│       dba@db.prod.com:22           │
│       Last: 3d ago                 │
│                                     │
│ 📁 Development                      │
│   ├── 🟢 Dev Box                    │
│   │   dev@192.168.1.100:22         │
│   └── 🟢 Home Lab                   │
│       admin@homelab.local:2222     │
│                                     │
│        [+ New Connection]           │
└─────────────────────────────────────┘
```

Features:
- **Save connections** with name, host, port, user, identity key
- **Connection groups** / folders
- **One-tap connect** to saved connections
- **Connection status** (online/offline indicator)
- **SSH key management** (generate, import, export)
- **Port forwarding** per connection
- **Jump host / bastion** support (ProxyJump)
- **Mosh support** for unstable mobile connections
- **Quick reconnect** after connection drop
- **Connection history** with timestamps

### 2. Connection Editor

```
┌─────────────────────────────────────┐
│ ← Edit Connection              💾  │
├─────────────────────────────────────┤
│                                     │
│ Name: [Production Web Server 1  ]  │
│ Group: [Production            ▼ ]  │
│                                     │
│ ─── Connection ───                  │
│ Host: [web1.prod.com           ]   │
│ Port: [22                      ]   │
│ User: [deploy                  ]   │
│                                     │
│ ─── Authentication ───              │
│ Method: [SSH Key             ▼ ]   │
│ Key:    [id_ed25519_prod     ▼ ]   │
│                                     │
│ ─── Advanced ───                    │
│ Jump Host: [bastion.prod.com   ]   │
│ Port Fwd:  [8080:localhost:80  ]   │
│ Startup:   [cd /app && htop   ]    │
│ Use Mosh:  [  ] (toggle)           │
│                                     │
│ ─── Appearance ───                  │
│ Profile: [Production Red     ▼ ]   │
│ Font Size: [14                 ]   │
│                                     │
│ [Test Connection]  [Save]  [Delete] │
└─────────────────────────────────────┘
```

### 3. Mobile Terminal UI

```
┌─────────────────────────────────────┐
│ 🟢 Web Server 1           [≡] [×] │
├─────────────────────────────────────┤
│                                     │
│ deploy@web1:~$ systemctl status     │
│ ● nginx.service - A high...        │
│    Active: active (running)         │
│                                     │
│                                     │
│                                     │
├─────────────────────────────────────┤
│ [Tab] [Ctrl] [Alt] [Esc] [↑↓←→]   │  <- Extra key row
│                                     │
│ ┌─────────────────────────────────┐ │
│ │      On-Screen Keyboard         │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

Mobile-optimized features:
- **Extra key row** above keyboard (Tab, Ctrl, Alt, Esc, Arrows, Fn keys)
- **Swipe gestures**:
  - Swipe left/right: switch tabs
  - Two-finger swipe down: show connection list
  - Pinch to zoom (font size)
  - Long press: select text, copy
- **Hardware keyboard support** (Bluetooth, Samsung DeX)
- **Split screen** support (Android multi-window)

### 4. Desktop Pairing (Zero-Config WireGuard)

```
MOBILE                              DESKTOP
──────                              ───────
1. Open "Pair Desktop"
2. Camera opens
3. Scan QR code shown  ◄────────── QR code on desktop screen
4. Connected!                       "Phone connected ✓"
```

Once paired:
- Use AI (Claude/Gemini/Ollama) running on desktop
- View/control desktop terminal sessions
- Transfer files via desktop
- Use desktop as SSH jump host

### 5. Mobile AI Integration

Connect to AI running on desktop via embedded WireGuard:

```
Mobile App
    │
    │ Embedded WireGuard (boringtun)
    │ (auto-paired, zero config)
    │
    v
Desktop Terminal (Leuwi Panjang)
    │
    ├── Claude CLI (installed on desktop)
    ├── Gemini CLI (installed on desktop)
    └── Ollama (running on desktop)
```

- No API keys on mobile device
- AI runs on desktop hardware (faster)
- Mobile just sends requests via tunnel
- Same audit trail and permission system

## Platform-Specific Notes

### Android
- Min SDK: 26 (Android 8.0)
- Makepad compiles via Android NDK (aarch64, armv7, x86_64)
- Supports Samsung DeX for desktop-like experience
- Widget: quick-connect to saved connections (planned)

### iOS
- Min: iOS 15.0
- Makepad compiles via cargo-lipo (aarch64-apple-ios)
- No local terminal (Apple restriction) — SSH only
- Supports External Keyboard shortcuts
- App Store compatible

## Build Targets

```bash
# Desktop (same code)
cargo build --release                          # Linux
cargo build --release --target x86_64-apple-darwin   # macOS
cargo build --release --target x86_64-pc-windows-msvc # Windows

# Mobile (same code, Makepad handles platform layer)
cargo makepad android build --release          # Android APK
cargo makepad ios build --release              # iOS IPA
```

All from the **same codebase** — no separate mobile project.

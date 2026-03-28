/// Menu items for the hamburger menu (≡)
/// Each item has an id, label, optional shortcut hint, and optional submenu

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: &'static str,
    pub label: String,
    pub shortcut: Option<String>,
    pub submenu: Vec<MenuItem>,
    pub separator: bool,
    pub enabled: bool,
}

impl MenuItem {
    pub fn action(id: &'static str, label: &str, shortcut: Option<&str>) -> Self {
        Self {
            id,
            label: label.to_string(),
            shortcut: shortcut.map(|s| s.to_string()),
            submenu: Vec::new(),
            separator: false,
            enabled: true,
        }
    }

    pub fn submenu(id: &'static str, label: &str, items: Vec<MenuItem>) -> Self {
        Self {
            id,
            label: label.to_string(),
            shortcut: None,
            submenu: items,
            separator: false,
            enabled: true,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: "sep",
            label: String::new(),
            shortcut: None,
            submenu: Vec::new(),
            separator: true,
            enabled: true,
        }
    }
}

/// Build the main hamburger menu structure
pub fn build_main_menu() -> Vec<MenuItem> {
    vec![
        // Terminal actions
        MenuItem::action("new_tab", "New Tab", Some("Ctrl+Shift+T")),
        MenuItem::action("new_window", "New Window", Some("Ctrl+Shift+N")),
        MenuItem::separator(),

        // Split
        MenuItem::submenu("split", "Split Pane", vec![
            MenuItem::action("split_vertical", "Split Vertical", Some("Ctrl+Shift+D")),
            MenuItem::action("split_horizontal", "Split Horizontal", Some("Ctrl+Shift+E")),
            MenuItem::separator(),
            MenuItem::action("close_pane", "Close Pane", Some("Ctrl+Shift+W")),
            MenuItem::action("pane_to_tab", "Move Pane to New Tab", None),
            MenuItem::action("pane_to_window", "Move Pane to New Window", None),
        ]),
        MenuItem::separator(),

        // Edit
        MenuItem::action("copy", "Copy", Some("Ctrl+Shift+C")),
        MenuItem::action("paste", "Paste", Some("Ctrl+Shift+V")),
        MenuItem::action("select_all", "Select All", Some("Ctrl+Shift+A")),
        MenuItem::action("find", "Find", Some("Ctrl+Shift+F")),
        MenuItem::separator(),

        // Appearance
        MenuItem::submenu("appearance", "Appearance", vec![
            MenuItem::action("theme_settings", "Theme Settings...", None),
            MenuItem::separator(),
            MenuItem::action("theme_dark_green", "Dark Green (Default)", None),
            MenuItem::action("theme_dark_blue", "Dark Blue", None),
            MenuItem::action("theme_dark_purple", "Dark Purple", None),
            MenuItem::action("theme_monokai", "Monokai", None),
            MenuItem::action("theme_solarized", "Solarized Dark", None),
            MenuItem::separator(),
            MenuItem::action("font_increase", "Increase Font Size", Some("Ctrl++")),
            MenuItem::action("font_decrease", "Decrease Font Size", Some("Ctrl+-")),
            MenuItem::action("font_reset", "Reset Font Size", Some("Ctrl+0")),
        ]),

        // Shell
        MenuItem::submenu("shell", "Shell", vec![
            MenuItem::action("shell_zsh", "Zsh", None),
            MenuItem::action("shell_bash", "Bash", None),
            MenuItem::action("shell_fish", "Fish", None),
            MenuItem::separator(),
            MenuItem::action("shell_custom", "Custom Shell...", None),
        ]),
        MenuItem::separator(),

        // Network & Connections
        MenuItem::submenu("network", "Network", vec![
            MenuItem::action("ssh_connect", "SSH Connect...", None),
            MenuItem::action("ssh_saved", "Saved Connections", None),
            MenuItem::separator(),
            MenuItem::action("wireguard_settings", "WireGuard Settings...", None),
            MenuItem::action("wireguard_pair", "Pair Device (QR)", None),
            MenuItem::action("wireguard_devices", "Connected Devices", None),
        ]),
        MenuItem::separator(),

        // Settings
        MenuItem::submenu("settings", "Settings", vec![
            MenuItem::action("general_settings", "General", None),
            MenuItem::action("keybinding_settings", "Keybindings", None),
            MenuItem::action("cell_settings", "Cell / Font", None),
            MenuItem::action("profile_settings", "Profiles", None),
            MenuItem::separator(),
            MenuItem::action("config_file", "Open Config File", None),
            MenuItem::action("config_reload", "Reload Config", None),
        ]),

        // Plugins
        MenuItem::submenu("plugins", "Plugins", vec![
            MenuItem::action("plugin_manager", "Plugin Manager...", None),
            MenuItem::separator(),
            MenuItem::action("plugin_ai_claude", "AI: Claude", None),
            MenuItem::action("plugin_ai_gemini", "AI: Gemini", None),
            MenuItem::action("plugin_ai_ollama", "AI: Ollama (Local)", None),
        ]),
        MenuItem::separator(),

        // Help
        MenuItem::action("fullscreen", "Fullscreen", Some("F11")),
        MenuItem::action("about", "About Leuwi Panjang", None),
        MenuItem::action("quit", "Quit", Some("Ctrl+Shift+Q")),
    ]
}

/// Format menu for display as simple text (temporary until proper popup)
pub fn menu_to_text(items: &[MenuItem], indent: usize) -> String {
    let mut text = String::new();
    let pad = " ".repeat(indent);

    for item in items {
        if item.separator {
            text.push_str(&pad);
            text.push_str("────────────────────────────────\n");
            continue;
        }

        text.push_str(&pad);

        if !item.submenu.is_empty() {
            text.push_str(&format!("▸ {}\n", item.label));
            text.push_str(&menu_to_text(&item.submenu, indent + 4));
        } else {
            if let Some(ref shortcut) = item.shortcut {
                let spacing = 32_usize.saturating_sub(item.label.len() + indent);
                text.push_str(&format!(
                    "{}{}{}",
                    item.label,
                    " ".repeat(spacing),
                    shortcut
                ));
            } else {
                text.push_str(&item.label);
            }
            text.push('\n');
        }
    }

    text
}

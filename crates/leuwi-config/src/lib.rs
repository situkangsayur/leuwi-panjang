use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub scrollback: ScrollbackConfig,
    #[serde(default)]
    pub suggestions: SuggestionsConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_shell")]
    pub default_shell: String,
    #[serde(default = "default_term")]
    pub term: String,
    #[serde(default = "default_startup_mode")]
    pub startup_mode: String,
    #[serde(default = "default_true")]
    pub confirm_close: bool,
    #[serde(default = "default_true")]
    pub shell_integration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_opacity")]
    pub background_opacity: f32,
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,
    #[serde(default = "default_window_decorations")]
    pub window_decorations: String,
    #[serde(default = "default_tab_bar_position")]
    pub tab_bar_position: String,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    #[serde(default = "default_true")]
    pub cursor_blink: bool,
    #[serde(default)]
    pub padding: PaddingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaddingConfig {
    #[serde(default = "default_padding_v")]
    pub top: u32,
    #[serde(default = "default_padding_v")]
    pub bottom: u32,
    #[serde(default = "default_padding_h")]
    pub left: u32,
    #[serde(default = "default_padding_h")]
    pub right: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    #[serde(default = "default_fg")]
    pub foreground: String,
    #[serde(default = "default_bg")]
    pub background: String,
    #[serde(default = "default_cursor_color")]
    pub cursor: String,
    #[serde(default)]
    pub normal: AnsiColors,
    #[serde(default)]
    pub bright: AnsiColors,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnsiColors {
    #[serde(default = "default_black")]
    pub black: String,
    #[serde(default = "default_red")]
    pub red: String,
    #[serde(default = "default_green")]
    pub green: String,
    #[serde(default = "default_yellow")]
    pub yellow: String,
    #[serde(default = "default_blue")]
    pub blue: String,
    #[serde(default = "default_magenta")]
    pub magenta: String,
    #[serde(default = "default_cyan")]
    pub cyan: String,
    #[serde(default = "default_white")]
    pub white: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollbackConfig {
    #[serde(default = "default_scrollback_lines")]
    pub lines: u32,
    #[serde(default = "default_scroll_multiplier")]
    pub multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub ghost_text: bool,
    #[serde(default = "default_true")]
    pub history_based: bool,
    #[serde(default = "default_true")]
    pub fuzzy_match: bool,
    #[serde(default = "default_max_suggestions")]
    pub max_suggestions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    #[serde(default = "default_kb_new_tab")]
    pub new_tab: String,
    #[serde(default = "default_kb_close_pane")]
    pub close_pane: String,
    #[serde(default = "default_kb_split_h")]
    pub split_horizontal: String,
    #[serde(default = "default_kb_split_v")]
    pub split_vertical: String,
    #[serde(default = "default_kb_copy")]
    pub copy: String,
    #[serde(default = "default_kb_paste")]
    pub paste: String,
    #[serde(default = "default_kb_search")]
    pub search: String,
    #[serde(default = "default_kb_search_tabs")]
    pub search_tabs: String,
    #[serde(default = "default_kb_settings")]
    pub settings: String,
}

// Default value functions
fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
}
fn default_term() -> String { "xterm-256color".to_string() }
fn default_startup_mode() -> String { "normal".to_string() }
fn default_true() -> bool { true }
fn default_theme() -> String { "leuwi-dark".to_string() }
fn default_font_family() -> String { "JetBrains Mono".to_string() }
fn default_font_size() -> f32 { 13.0 }
fn default_opacity() -> f32 { 0.85 }
fn default_corner_radius() -> f32 { 12.0 }
fn default_window_decorations() -> String { "none".to_string() }
fn default_tab_bar_position() -> String { "top".to_string() }
fn default_cursor_style() -> String { "beam".to_string() }
fn default_padding_v() -> u32 { 4 }
fn default_padding_h() -> u32 { 8 }
fn default_fg() -> String { "#e0e0e0".to_string() }
fn default_bg() -> String { "#1a1a2e".to_string() }
fn default_cursor_color() -> String { "#e94560".to_string() }
fn default_black() -> String { "#1a1a2e".to_string() }
fn default_red() -> String { "#e94560".to_string() }
fn default_green() -> String { "#0cce6b".to_string() }
fn default_yellow() -> String { "#ffc857".to_string() }
fn default_blue() -> String { "#0f3460".to_string() }
fn default_magenta() -> String { "#c77dff".to_string() }
fn default_cyan() -> String { "#00b4d8".to_string() }
fn default_white() -> String { "#e0e0e0".to_string() }
fn default_scrollback_lines() -> u32 { 10000 }
fn default_scroll_multiplier() -> f32 { 3.0 }
fn default_max_suggestions() -> u32 { 8 }
fn default_kb_new_tab() -> String { "ctrl+shift+t".to_string() }
fn default_kb_close_pane() -> String { "ctrl+shift+w".to_string() }
fn default_kb_split_h() -> String { "ctrl+shift+h".to_string() }
fn default_kb_split_v() -> String { "ctrl+shift+v".to_string() }
fn default_kb_copy() -> String { "ctrl+shift+c".to_string() }
fn default_kb_paste() -> String { "ctrl+shift+v".to_string() }
fn default_kb_search() -> String { "ctrl+shift+f".to_string() }
fn default_kb_search_tabs() -> String { "ctrl+shift+space".to_string() }
fn default_kb_settings() -> String { "ctrl+shift+comma".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            colors: ColorConfig::default(),
            scrollback: ScrollbackConfig::default(),
            suggestions: SuggestionsConfig::default(),
            keybindings: KeybindingsConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_shell: default_shell(),
            term: default_term(),
            startup_mode: default_startup_mode(),
            confirm_close: true,
            shell_integration: true,
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font_family: default_font_family(),
            font_size: default_font_size(),
            background_opacity: default_opacity(),
            corner_radius: default_corner_radius(),
            window_decorations: default_window_decorations(),
            tab_bar_position: default_tab_bar_position(),
            cursor_style: default_cursor_style(),
            cursor_blink: true,
            padding: PaddingConfig::default(),
        }
    }
}

impl Default for PaddingConfig {
    fn default() -> Self {
        Self { top: 4, bottom: 4, left: 8, right: 8 }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            foreground: default_fg(),
            background: default_bg(),
            cursor: default_cursor_color(),
            normal: AnsiColors::default(),
            bright: AnsiColors::default(),
        }
    }
}

impl Default for ScrollbackConfig {
    fn default() -> Self {
        Self { lines: 10000, multiplier: 3.0 }
    }
}

impl Default for SuggestionsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ghost_text: true,
            history_based: true,
            fuzzy_match: true,
            max_suggestions: 8,
        }
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            new_tab: default_kb_new_tab(),
            close_pane: default_kb_close_pane(),
            split_horizontal: default_kb_split_h(),
            split_vertical: default_kb_split_v(),
            copy: default_kb_copy(),
            paste: default_kb_paste(),
            search: default_kb_search(),
            search_tabs: default_kb_search_tabs(),
            settings: default_kb_settings(),
        }
    }
}

/// Get config directory path
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("leuwi-panjang")
}

/// Load config from file, or return default
pub fn load_config() -> Config {
    let config_path = config_dir().join("config.toml");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => return config,
                Err(e) => tracing::warn!("Failed to parse config: {e}, using defaults"),
            },
            Err(e) => tracing::warn!("Failed to read config: {e}, using defaults"),
        }
    }
    Config::default()
}

/// Save config to file
pub fn save_config(config: &Config) -> anyhow::Result<()> {
    let config_path = config_dir().join("config.toml");
    std::fs::create_dir_all(config_dir())?;
    let content = toml::to_string_pretty(config)?;
    std::fs::write(config_path, content)?;
    Ok(())
}

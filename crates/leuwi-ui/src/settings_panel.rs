/// Settings panel structure — rendered as text in menu panel for now.
/// Will become a proper Makepad GUI panel later.

pub fn render_general_settings() -> String {
    let config = leuwi_config::load_config();
    format!(
        "═══ General Settings ═══\n\
         \n\
         Shell:         {}\n\
         TERM:          {}\n\
         Startup Mode:  {}\n\
         Confirm Close: {}\n\
         \n\
         ═══ Appearance ═══\n\
         \n\
         Theme:         {}\n\
         Font:          {}\n\
         Font Size:     {}\n\
         Opacity:       {}%\n\
         Corner Radius: {}\n\
         Decorations:   {}\n\
         Tab Position:  {}\n\
         Cursor:        {}\n\
         \n\
         ═══ Scrollback ═══\n\
         \n\
         Lines:         {}\n\
         Multiplier:    {}\n\
         \n\
         ═══ Suggestions ═══\n\
         \n\
         Enabled:       {}\n\
         Ghost Text:    {}\n\
         History:       {}\n\
         Fuzzy Match:   {}\n\
         Max Items:     {}\n\
         \n\
         Config file: ~/.config/leuwi-panjang/config.toml",
        config.general.default_shell,
        config.general.term,
        config.general.startup_mode,
        config.general.confirm_close,
        config.appearance.theme,
        config.appearance.font_family,
        config.appearance.font_size,
        (config.appearance.background_opacity * 100.0) as u32,
        config.appearance.corner_radius,
        config.appearance.window_decorations,
        config.appearance.tab_bar_position,
        config.appearance.cursor_style,
        config.scrollback.lines,
        config.scrollback.multiplier,
        config.suggestions.enabled,
        config.suggestions.ghost_text,
        config.suggestions.history_based,
        config.suggestions.fuzzy_match,
        config.suggestions.max_suggestions,
    )
}

pub fn render_keybindings() -> String {
    "═══ Keybindings ═══\n\
     \n\
     Ctrl+Shift+T     New Tab\n\
     Ctrl+Shift+W     Close Pane/Tab\n\
     Ctrl+Tab         Next Tab\n\
     Ctrl+Shift+Tab   Previous Tab\n\
     Alt+1-5          Go to Tab N\n\
     \n\
     Ctrl+Shift+D     Split Vertical\n\
     Ctrl+Shift+E     Split Horizontal\n\
     Alt+Left/Right   Switch Pane\n\
     \n\
     Ctrl+Shift+C     Copy\n\
     Ctrl+Shift+V     Paste\n\
     Ctrl+Shift+A     Select All\n\
     \n\
     Shift+PageUp     Scroll Up\n\
     Shift+PageDown   Scroll Down\n\
     Shift+Home       Scroll to Top\n\
     Shift+End        Scroll to Bottom\n\
     \n\
     Escape           Close Menu\n\
     F11              Fullscreen\n"
        .to_string()
}

pub fn render_about() -> String {
    "═══ About Leuwi Panjang ═══\n\
     \n\
     Terminal Leuwi Panjang v0.1.0\n\
     \n\
     A lightweight, modern, GPU-accelerated\n\
     terminal emulator built in Rust.\n\
     \n\
     Built with Makepad UI framework.\n\
     \n\
     github.com/situkangsayur/leuwi-panjang\n\
     \n\
     © 2026 situkangsayur\n\
     Licensed under MIT"
        .to_string()
}

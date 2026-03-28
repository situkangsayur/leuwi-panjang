use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    #[serde(default)]
    pub author: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    #[serde(default)]
    pub selection_foreground: Option<String>,
    #[serde(default)]
    pub selection_background: Option<String>,
    #[serde(default)]
    pub normal: Option<super::AnsiColors>,
    #[serde(default)]
    pub bright: Option<super::AnsiColors>,
}

/// Built-in Leuwi Dark theme
pub fn leuwi_dark() -> Theme {
    Theme {
        name: "Leuwi Dark".to_string(),
        author: "situkangsayur".to_string(),
        colors: ThemeColors {
            foreground: "#e0e0e0".to_string(),
            background: "#1a1a2e".to_string(),
            cursor: "#e94560".to_string(),
            selection_foreground: Some("#ffffff".to_string()),
            selection_background: Some("#0f3460".to_string()),
            normal: None,
            bright: None,
        },
    }
}

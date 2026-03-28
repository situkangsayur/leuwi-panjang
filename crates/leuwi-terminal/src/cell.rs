/// A single cell in the terminal grid.
#[derive(Debug, Clone, Default)]
pub struct Cell {
    /// The character displayed in this cell (may be multi-byte for wide/emoji chars)
    pub c: char,
    /// Visual attributes
    pub attrs: CellAttributes,
    /// Whether this cell has been modified since last render
    pub dirty: bool,
}

/// Visual attributes for a cell.
#[derive(Debug, Clone)]
pub struct CellAttributes {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: UnderlineStyle,
    pub strikethrough: bool,
    pub inverse: bool,
    pub dim: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnderlineStyle {
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

/// RGBA color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a hex color string like "#e0e0e0"
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self::rgb(r, g, b))
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::rgb(224, 224, 224) // #e0e0e0
    }
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self {
            fg: Color::rgb(224, 224, 224),
            bg: Color::rgb(26, 26, 46), // #1a1a2e
            bold: false,
            italic: false,
            underline: UnderlineStyle::None,
            strikethrough: false,
            inverse: false,
            dim: false,
            hidden: false,
        }
    }
}

impl Cell {
    pub fn new(c: char) -> Self {
        Self {
            c,
            attrs: CellAttributes::default(),
            dirty: true,
        }
    }

    pub fn blank() -> Self {
        Self {
            c: ' ',
            attrs: CellAttributes::default(),
            dirty: true,
        }
    }
}

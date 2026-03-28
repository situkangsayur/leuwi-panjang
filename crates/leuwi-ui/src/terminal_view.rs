// Terminal grid rendering - uses colored spans
// For now, we serialize colored text to a single string for Label rendering.
// Per-cell GPU color rendering will come when we have Makepad custom draw pipeline.

use crate::terminal_pane::ColoredLine;

/// Convert colored lines to plain text for Label display.
/// Colors are stripped — Label can only show one color.
pub fn colored_lines_to_text(lines: &[ColoredLine]) -> String {
    let mut output = String::with_capacity(lines.len() * 80);
    for (i, line) in lines.iter().enumerate() {
        for span in &line.spans {
            output.push_str(&span.text);
        }
        if i < lines.len() - 1 {
            output.push('\n');
        }
    }
    output
}

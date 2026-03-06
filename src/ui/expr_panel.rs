//! Expression input panel

use crate::common::Color;

/// An entry in the expression list
#[derive(Debug, Clone)]
pub struct ExpressionEntry {
    /// Expression source text
    pub source: String,
    /// Display color
    pub color: Color,
    /// Is the expression visible
    pub visible: bool,
    /// Is there an error
    pub error: Option<String>,
    /// Is currently being edited
    pub editing: bool,
}

impl ExpressionEntry {
    pub fn new(source: String, color: Color) -> Self {
        Self {
            source,
            color,
            visible: true,
            error: None,
            editing: false,
        }
    }
}

/// Expression panel state
pub struct ExpressionPanel {
    /// Input buffer for new expression
    pub input_buffer: String,
    /// Expression entries
    pub entries: Vec<ExpressionEntry>,
    /// Currently selected entry index
    pub selected: Option<usize>,
}

impl ExpressionPanel {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            entries: Vec::new(),
            selected: None,
        }
    }

    /// Add a new expression entry
    pub fn add_entry(&mut self, entry: ExpressionEntry) {
        self.entries.push(entry);
    }

    /// Remove an entry by index
    pub fn remove_entry(&mut self, index: usize) -> Option<ExpressionEntry> {
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    /// Get entry by index
    pub fn get_entry(&self, index: usize) -> Option<&ExpressionEntry> {
        self.entries.get(index)
    }

    /// Get mutable entry by index
    pub fn get_entry_mut(&mut self, index: usize) -> Option<&mut ExpressionEntry> {
        self.entries.get_mut(index)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected = None;
    }

    /// Toggle visibility of an entry
    pub fn toggle_visibility(&mut self, index: usize) {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.visible = !entry.visible;
        }
    }
}

impl Default for ExpressionPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_panel() {
        let mut panel = ExpressionPanel::new();

        panel.add_entry(ExpressionEntry::new("y = x".to_string(), Color::BLUE));
        panel.add_entry(ExpressionEntry::new("y = x^2".to_string(), Color::RED));

        assert_eq!(panel.entries.len(), 2);

        panel.toggle_visibility(0);
        assert!(!panel.entries[0].visible);

        panel.remove_entry(0);
        assert_eq!(panel.entries.len(), 1);
        assert_eq!(panel.entries[0].source, "y = x^2");
    }
}

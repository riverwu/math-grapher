//! Toolbar widget

/// Tool mode for interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolMode {
    /// Pan and zoom mode
    #[default]
    Pan,
    /// Point placement mode
    Point,
    /// Measurement mode
    Measure,
    /// Selection mode
    Select,
}

/// Toolbar state
pub struct Toolbar {
    /// Current tool mode
    pub mode: ToolMode,
    /// Show help tooltip
    pub show_help: bool,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            mode: ToolMode::default(),
            show_help: false,
        }
    }

    /// Set the current tool mode
    pub fn set_mode(&mut self, mode: ToolMode) {
        self.mode = mode;
    }

    /// Get the current tool mode
    pub fn current_mode(&self) -> ToolMode {
        self.mode
    }

    /// Check if in pan mode
    pub fn is_pan_mode(&self) -> bool {
        self.mode == ToolMode::Pan
    }

    /// Check if in point mode
    pub fn is_point_mode(&self) -> bool {
        self.mode == ToolMode::Point
    }

    /// Toggle help display
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar() {
        let mut toolbar = Toolbar::new();
        assert!(toolbar.is_pan_mode());

        toolbar.set_mode(ToolMode::Point);
        assert!(toolbar.is_point_mode());
        assert!(!toolbar.is_pan_mode());
    }
}

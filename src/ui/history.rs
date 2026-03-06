//! Undo/Redo history management

use crate::common::Color;
use crate::parser::ExpressionType;

/// Maximum number of actions to keep in history
const MAX_HISTORY_SIZE: usize = 100;

/// An action that can be undone/redone
#[derive(Debug, Clone)]
pub enum Action {
    /// Added an expression
    AddExpression {
        index: usize,
        source: String,
        color: Color,
        expr_type: ExpressionType,
    },
    /// Removed an expression
    RemoveExpression {
        index: usize,
        source: String,
        color: Color,
        expr_type: ExpressionType,
    },
    /// Changed expression visibility
    ToggleVisibility {
        index: usize,
        was_visible: bool,
    },
    /// Added a data point for curve fitting
    AddDataPoint {
        index: usize,
        x: f64,
        y: f64,
    },
    /// Cleared all data points
    ClearDataPoints {
        points: Vec<(f64, f64)>,
    },
}

/// History manager for undo/redo operations
#[derive(Debug, Clone)]
pub struct History {
    /// Stack of actions that can be undone
    undo_stack: Vec<Action>,
    /// Stack of actions that can be redone
    redo_stack: Vec<Action>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Record a new action (clears redo stack)
    pub fn record(&mut self, action: Action) {
        self.undo_stack.push(action);
        self.redo_stack.clear();

        // Limit history size
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.remove(0);
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the action to undo (moves it to redo stack)
    pub fn undo(&mut self) -> Option<Action> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    /// Get the action to redo (moves it to undo stack)
    pub fn redo(&mut self) -> Option<Action> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get the number of actions that can be undone
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of actions that can be redone
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_record() {
        let mut history = History::new();
        assert!(!history.can_undo());

        history.record(Action::AddExpression {
            index: 0,
            source: "y = x".to_string(),
            color: Color::RED,
            expr_type: ExpressionType::Explicit,
        });

        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_undo_redo() {
        let mut history = History::new();

        history.record(Action::AddExpression {
            index: 0,
            source: "y = x".to_string(),
            color: Color::RED,
            expr_type: ExpressionType::Explicit,
        });

        // Undo
        let action = history.undo();
        assert!(action.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        // Redo
        let action = history.redo();
        assert!(action.is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_redo_cleared_on_new_action() {
        let mut history = History::new();

        history.record(Action::AddExpression {
            index: 0,
            source: "y = x".to_string(),
            color: Color::RED,
            expr_type: ExpressionType::Explicit,
        });

        history.undo();
        assert!(history.can_redo());

        // New action clears redo
        history.record(Action::AddExpression {
            index: 0,
            source: "y = sin(x)".to_string(),
            color: Color::BLUE,
            expr_type: ExpressionType::Explicit,
        });

        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_max_size() {
        let mut history = History::new();

        for i in 0..150 {
            history.record(Action::ToggleVisibility {
                index: i,
                was_visible: true,
            });
        }

        assert!(history.undo_count() <= MAX_HISTORY_SIZE);
    }
}

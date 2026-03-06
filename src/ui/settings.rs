//! Settings panel

use crate::common::Color;
use serde::{Deserialize, Serialize};

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Background color
    pub background_color: Color,
    /// Default curve width
    pub default_curve_width: f32,
    /// Show grid
    pub show_grid: bool,
    /// Show minor grid
    pub show_minor_grid: bool,
    /// Show axis labels
    pub show_labels: bool,
    /// Number of samples for curve rendering
    pub sample_count: usize,
    /// Enable anti-aliasing
    pub anti_aliasing: bool,
    /// Enable adaptive sampling
    pub adaptive_sampling: bool,
    /// Dark mode
    pub dark_mode: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            background_color: Color::WHITE,
            default_curve_width: 2.0,
            show_grid: true,
            show_minor_grid: true,
            show_labels: true,
            sample_count: 500,
            anti_aliasing: true,
            adaptive_sampling: true,
            dark_mode: false,
        }
    }
}

impl AppSettings {
    /// Create dark mode settings
    pub fn dark() -> Self {
        Self {
            background_color: Color::rgb(0.1, 0.1, 0.1),
            dark_mode: true,
            ..Default::default()
        }
    }

    /// Toggle dark mode
    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
        if self.dark_mode {
            self.background_color = Color::rgb(0.1, 0.1, 0.1);
        } else {
            self.background_color = Color::WHITE;
        }
    }
}

/// Settings panel state
pub struct SettingsPanel {
    /// Current settings
    pub settings: AppSettings,
    /// Has unsaved changes
    pub dirty: bool,
}

impl SettingsPanel {
    pub fn new() -> Self {
        Self {
            settings: AppSettings::default(),
            dirty: false,
        }
    }

    pub fn with_settings(settings: AppSettings) -> Self {
        Self {
            settings,
            dirty: false,
        }
    }

    /// Mark settings as changed
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Save settings and clear dirty flag
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        // In a real app, this would save to a config file
        self.dirty = false;
        Ok(())
    }

    /// Reset to default settings
    pub fn reset(&mut self) {
        self.settings = AppSettings::default();
        self.dirty = true;
    }

    /// Apply dark mode preset
    pub fn apply_dark_mode(&mut self) {
        self.settings.toggle_dark_mode();
        self.dirty = true;
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.show_grid);
        assert!(!settings.dark_mode);
    }

    #[test]
    fn test_settings_dark_mode() {
        let mut settings = AppSettings::default();
        settings.toggle_dark_mode();
        assert!(settings.dark_mode);

        settings.toggle_dark_mode();
        assert!(!settings.dark_mode);
    }

    #[test]
    fn test_settings_panel() {
        let mut panel = SettingsPanel::new();
        assert!(!panel.dirty);

        panel.mark_dirty();
        assert!(panel.dirty);

        panel.save().unwrap();
        assert!(!panel.dirty);
    }
}

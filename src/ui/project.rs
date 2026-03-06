//! Project save/load functionality
//!
//! Allows saving and loading the current graph state (expressions, viewport, settings) to JSON files.

use serde::{Deserialize, Serialize};
use crate::common::{Color, Rect};
use std::path::Path;

/// Serializable project state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    /// Version for forward compatibility
    pub version: u32,
    /// List of expression source strings
    pub expressions: Vec<ExpressionData>,
    /// Viewport bounds
    pub viewport: Rect,
    /// Parameter values
    pub parameters: Vec<ParameterData>,
}

/// Serializable expression data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionData {
    /// Original expression string
    pub source: String,
    /// Display color
    pub color: Color,
    /// Is visible
    pub visible: bool,
}

/// Serializable parameter data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterData {
    /// Parameter name
    pub name: String,
    /// Current value
    pub value: f64,
}

impl ProjectData {
    pub fn new() -> Self {
        Self {
            version: 1,
            expressions: Vec::new(),
            viewport: Rect::default(),
            parameters: Vec::new(),
        }
    }

    /// Save project to a JSON file
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    /// Load project from a JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        let data: ProjectData = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse project file: {}", e))?;
        Ok(data)
    }
}

impl Default for ProjectData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_project_data_new() {
        let data = ProjectData::new();
        assert_eq!(data.version, 1);
        assert!(data.expressions.is_empty());
        assert!(data.parameters.is_empty());
    }

    #[test]
    fn test_project_data_serialize_deserialize() {
        let data = ProjectData {
            version: 1,
            expressions: vec![
                ExpressionData {
                    source: "y = sin(x)".to_string(),
                    color: Color::rgb(0.2, 0.4, 0.8),
                    visible: true,
                },
                ExpressionData {
                    source: "y = x^2".to_string(),
                    color: Color::rgb(0.8, 0.2, 0.2),
                    visible: false,
                },
            ],
            viewport: Rect::new(-5.0, 5.0, -3.0, 3.0),
            parameters: vec![
                ParameterData {
                    name: "a".to_string(),
                    value: 2.5,
                },
            ],
        };

        let json = serde_json::to_string_pretty(&data).unwrap();
        let loaded: ProjectData = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.expressions.len(), 2);
        assert_eq!(loaded.expressions[0].source, "y = sin(x)");
        assert_eq!(loaded.expressions[1].visible, false);
        assert_eq!(loaded.viewport.x_min, -5.0);
        assert_eq!(loaded.parameters[0].name, "a");
        assert!((loaded.parameters[0].value - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_project_save_load_file() {
        let data = ProjectData {
            version: 1,
            expressions: vec![
                ExpressionData {
                    source: "y = cos(x)".to_string(),
                    color: Color::BLUE,
                    visible: true,
                },
            ],
            viewport: Rect::default(),
            parameters: Vec::new(),
        };

        // Create a temp file
        let dir = std::env::temp_dir();
        let path = dir.join("test_math_grapher_project.json");

        // Save
        data.save_to_file(&path).unwrap();

        // Load
        let loaded = ProjectData::load_from_file(&path).unwrap();
        assert_eq!(loaded.expressions.len(), 1);
        assert_eq!(loaded.expressions[0].source, "y = cos(x)");

        // Cleanup
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_project_load_invalid_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_invalid_project.json");

        // Write invalid JSON
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"not json").unwrap();

        let result = ProjectData::load_from_file(&path);
        assert!(result.is_err());

        // Cleanup
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_project_load_nonexistent_file() {
        let path = Path::new("/tmp/nonexistent_math_grapher_file.json");
        let result = ProjectData::load_from_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_expression_data_color_preserved() {
        let expr = ExpressionData {
            source: "y = x".to_string(),
            color: Color::new(0.1, 0.2, 0.3, 0.9),
            visible: true,
        };

        let json = serde_json::to_string(&expr).unwrap();
        let loaded: ExpressionData = serde_json::from_str(&json).unwrap();

        assert!((loaded.color.r - 0.1).abs() < f32::EPSILON);
        assert!((loaded.color.g - 0.2).abs() < f32::EPSILON);
        assert!((loaded.color.b - 0.3).abs() < f32::EPSILON);
        assert!((loaded.color.a - 0.9).abs() < f32::EPSILON);
    }
}

//! Math Grapher Library
//!
//! Core modules for the mathematical graphing calculator.

pub mod parser;
pub mod evaluator;
pub mod algebra;
pub mod render;
pub mod ui;

/// Common types used across modules
pub mod common {
    use serde::{Deserialize, Serialize};

    /// A 2D point
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }

    impl Point {
        pub fn new(x: f64, y: f64) -> Self {
            Self { x, y }
        }
    }

    /// A rectangular region
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Rect {
        pub x_min: f64,
        pub x_max: f64,
        pub y_min: f64,
        pub y_max: f64,
    }

    impl Rect {
        pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
            Self { x_min, x_max, y_min, y_max }
        }

        pub fn width(&self) -> f64 {
            self.x_max - self.x_min
        }

        pub fn height(&self) -> f64 {
            self.y_max - self.y_min
        }

        pub fn center(&self) -> Point {
            Point::new(
                (self.x_min + self.x_max) / 2.0,
                (self.y_min + self.y_max) / 2.0,
            )
        }

        /// Zoom in/out by a factor (factor > 1 zooms out, factor < 1 zooms in)
        pub fn zoom(&self, factor: f64, center: Point) -> Self {
            let new_half_width = self.width() * factor / 2.0;
            let new_half_height = self.height() * factor / 2.0;
            Self {
                x_min: center.x - new_half_width,
                x_max: center.x + new_half_width,
                y_min: center.y - new_half_height,
                y_max: center.y + new_half_height,
            }
        }

        /// Pan by delta
        pub fn pan(&self, dx: f64, dy: f64) -> Self {
            Self {
                x_min: self.x_min + dx,
                x_max: self.x_max + dx,
                y_min: self.y_min + dy,
                y_max: self.y_max + dy,
            }
        }
    }

    impl Default for Rect {
        fn default() -> Self {
            Self {
                x_min: -10.0,
                x_max: 10.0,
                y_min: -10.0,
                y_max: 10.0,
            }
        }
    }

    /// A line segment defined by two points
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct LineSegment {
        pub start: Point,
        pub end: Point,
    }

    impl LineSegment {
        pub fn new(start: Point, end: Point) -> Self {
            Self { start, end }
        }
    }

    /// Color representation (RGBA)
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    impl Color {
        pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
            Self { r, g, b, a }
        }

        pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
            Self { r, g, b, a: 1.0 }
        }

        // Predefined colors
        pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
        pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
        pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
        pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
        pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
        pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
        pub const ORANGE: Color = Color::rgb(1.0, 0.65, 0.0);
        pub const PURPLE: Color = Color::rgb(0.5, 0.0, 0.5);
        pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
        pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    }

    impl Default for Color {
        fn default() -> Self {
            Self::BLUE
        }
    }

    impl From<Color> for egui::Color32 {
        fn from(c: Color) -> Self {
            egui::Color32::from_rgba_unmultiplied(
                (c.r * 255.0) as u8,
                (c.g * 255.0) as u8,
                (c.b * 255.0) as u8,
                (c.a * 255.0) as u8,
            )
        }
    }

    /// Predefined color palette for curves
    pub const CURVE_COLORS: [Color; 10] = [
        Color::rgb(0.2, 0.4, 0.8),   // Blue
        Color::rgb(0.8, 0.2, 0.2),   // Red
        Color::rgb(0.2, 0.7, 0.3),   // Green
        Color::rgb(0.8, 0.5, 0.0),   // Orange
        Color::rgb(0.6, 0.2, 0.8),   // Purple
        Color::rgb(0.0, 0.7, 0.7),   // Cyan
        Color::rgb(0.8, 0.0, 0.5),   // Magenta
        Color::rgb(0.5, 0.5, 0.0),   // Olive
        Color::rgb(0.0, 0.5, 0.5),   // Teal
        Color::rgb(0.5, 0.0, 0.0),   // Maroon
    ];
}

pub use common::{Color, LineSegment, Point, Rect, CURVE_COLORS};

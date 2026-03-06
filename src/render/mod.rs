//! Rendering Module
//!
//! Provides rendering capabilities for the graph canvas, grid, axes, and curves.

mod canvas;
mod grid;
mod curve;
mod markers;
mod region;

pub use canvas::{GraphCanvas, CanvasInteraction};
pub use grid::{GridRenderer, GridStyle};
pub use curve::{CurveRenderer, CurveStyle, LineStyle};
pub use markers::{MarkerRenderer, Marker, MarkerType};
pub use region::{RegionRenderer, RegionStyle};

use crate::common::{Color, Point, Rect};

/// Coordinate transformer between world and screen coordinates
#[derive(Debug, Clone, Copy)]
pub struct CoordinateTransform {
    /// Viewport bounds in world coordinates
    pub viewport: Rect,
    /// Screen size in pixels
    pub screen_width: f32,
    pub screen_height: f32,
}

impl CoordinateTransform {
    pub fn new(viewport: Rect, screen_width: f32, screen_height: f32) -> Self {
        Self {
            viewport,
            screen_width,
            screen_height,
        }
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, point: Point) -> (f32, f32) {
        let x = ((point.x - self.viewport.x_min) / self.viewport.width()) as f32 * self.screen_width;
        // Y is inverted (screen Y increases downward)
        let y = ((self.viewport.y_max - point.y) / self.viewport.height()) as f32 * self.screen_height;
        (x, y)
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, x: f32, y: f32) -> Point {
        let world_x = self.viewport.x_min + (x as f64 / self.screen_width as f64) * self.viewport.width();
        let world_y = self.viewport.y_max - (y as f64 / self.screen_height as f64) * self.viewport.height();
        Point::new(world_x, world_y)
    }

    /// Convert world distance to screen distance (horizontal)
    pub fn world_to_screen_dx(&self, dx: f64) -> f32 {
        (dx / self.viewport.width()) as f32 * self.screen_width
    }

    /// Convert world distance to screen distance (vertical)
    pub fn world_to_screen_dy(&self, dy: f64) -> f32 {
        (dy / self.viewport.height()) as f32 * self.screen_height
    }

    /// Convert screen distance to world distance (horizontal)
    pub fn screen_to_world_dx(&self, dx: f32) -> f64 {
        (dx as f64 / self.screen_width as f64) * self.viewport.width()
    }

    /// Convert screen distance to world distance (vertical)
    pub fn screen_to_world_dy(&self, dy: f32) -> f64 {
        (dy as f64 / self.screen_height as f64) * self.viewport.height()
    }

    /// Get the scale factor (pixels per world unit, average of x and y)
    pub fn scale(&self) -> f32 {
        let scale_x = self.screen_width / self.viewport.width() as f32;
        let scale_y = self.screen_height / self.viewport.height() as f32;
        (scale_x + scale_y) / 2.0
    }
}

/// Render state passed to all renderers
pub struct RenderContext<'a> {
    pub transform: CoordinateTransform,
    pub painter: &'a egui::Painter,
    pub clip_rect: egui::Rect,
}

impl<'a> RenderContext<'a> {
    pub fn new(transform: CoordinateTransform, painter: &'a egui::Painter, clip_rect: egui::Rect) -> Self {
        Self {
            transform,
            painter,
            clip_rect,
        }
    }

    /// Convert world coordinates to screen coordinates (with clip_rect offset)
    fn to_screen_pos(&self, point: Point) -> egui::Pos2 {
        let (x, y) = self.transform.world_to_screen(point);
        // Add the clip_rect offset so drawing happens at the correct position
        egui::pos2(x + self.clip_rect.left(), y + self.clip_rect.top())
    }

    /// Draw a line in world coordinates
    pub fn draw_line(&self, p1: Point, p2: Point, color: Color, width: f32) {
        let pos1 = self.to_screen_pos(p1);
        let pos2 = self.to_screen_pos(p2);
        let color32: egui::Color32 = color.into();

        self.painter.line_segment(
            [pos1, pos2],
            egui::Stroke::new(width, color32),
        );
    }

    /// Draw a circle in world coordinates
    pub fn draw_circle(&self, center: Point, radius: f32, fill: Color, stroke: Option<(Color, f32)>) {
        let pos = self.to_screen_pos(center);
        let fill32: egui::Color32 = fill.into();

        self.painter.circle(
            pos,
            radius,
            fill32,
            stroke.map_or(egui::Stroke::NONE, |(c, w)| egui::Stroke::new(w, egui::Color32::from(c))),
        );
    }

    /// Draw text at world coordinates
    pub fn draw_text(&self, pos: Point, text: &str, color: Color, anchor: egui::Align2) {
        let screen_pos = self.to_screen_pos(pos);

        self.painter.text(
            screen_pos,
            anchor,
            text,
            egui::FontId::proportional(12.0),
            color.into(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_transform() {
        let transform = CoordinateTransform::new(
            Rect::new(-10.0, 10.0, -10.0, 10.0),
            800.0,
            600.0,
        );

        // Origin should map to center of screen
        let (x, y) = transform.world_to_screen(Point::new(0.0, 0.0));
        assert!((x - 400.0).abs() < 0.01);
        assert!((y - 300.0).abs() < 0.01);

        // Test inverse
        let p = transform.screen_to_world(400.0, 300.0);
        assert!((p.x - 0.0).abs() < 0.01);
        assert!((p.y - 0.0).abs() < 0.01);
    }
}

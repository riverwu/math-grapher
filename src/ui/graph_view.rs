//! Graph view widget

use crate::common::{Point, Rect};

/// Graph view state
pub struct GraphView {
    /// Current viewport bounds
    pub viewport: Rect,
    /// Minimum zoom level (maximum viewport size)
    pub min_zoom: f64,
    /// Maximum zoom level (minimum viewport size)
    pub max_zoom: f64,
    /// Pan velocity for smooth panning
    pub pan_velocity: (f64, f64),
    /// Enable smooth animations
    pub smooth_animations: bool,
}

impl GraphView {
    pub fn new() -> Self {
        Self {
            viewport: Rect::default(),
            min_zoom: 1e-6,  // Very zoomed in
            max_zoom: 1e6,   // Very zoomed out
            pan_velocity: (0.0, 0.0),
            smooth_animations: true,
        }
    }

    /// Set the viewport bounds
    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = self.clamp_viewport(viewport);
    }

    /// Zoom by a factor, centered on a point
    pub fn zoom(&mut self, factor: f64, center: Point) {
        let new_viewport = self.viewport.zoom(factor, center);
        self.viewport = self.clamp_viewport(new_viewport);
    }

    /// Pan by a delta in world coordinates
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.viewport = self.viewport.pan(dx, dy);
    }

    /// Reset to default view
    pub fn reset(&mut self) {
        self.viewport = Rect::default();
        self.pan_velocity = (0.0, 0.0);
    }

    /// Fit the view to show specific bounds with some padding
    pub fn fit_to_bounds(&mut self, bounds: Rect, padding: f64) {
        let padded = Rect::new(
            bounds.x_min - bounds.width() * padding,
            bounds.x_max + bounds.width() * padding,
            bounds.y_min - bounds.height() * padding,
            bounds.y_max + bounds.height() * padding,
        );
        self.viewport = self.clamp_viewport(padded);
    }

    /// Clamp viewport to valid zoom range
    fn clamp_viewport(&self, viewport: Rect) -> Rect {
        let width = viewport.width().clamp(self.min_zoom, self.max_zoom);
        let height = viewport.height().clamp(self.min_zoom, self.max_zoom);

        let center = viewport.center();

        Rect::new(
            center.x - width / 2.0,
            center.x + width / 2.0,
            center.y - height / 2.0,
            center.y + height / 2.0,
        )
    }

    /// Update for smooth panning (call each frame)
    pub fn update(&mut self, dt: f64) {
        if !self.smooth_animations {
            return;
        }

        // Apply velocity
        if self.pan_velocity.0.abs() > 0.001 || self.pan_velocity.1.abs() > 0.001 {
            self.pan(self.pan_velocity.0 * dt, self.pan_velocity.1 * dt);

            // Friction
            let friction = 0.95;
            self.pan_velocity.0 *= friction;
            self.pan_velocity.1 *= friction;
        }
    }

    /// Set pan velocity for momentum scrolling
    pub fn set_pan_velocity(&mut self, vx: f64, vy: f64) {
        self.pan_velocity = (vx, vy);
    }
}

impl Default for GraphView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_view_zoom() {
        let mut view = GraphView::new();
        let center = view.viewport.center();

        view.zoom(0.5, center);

        // Viewport should be smaller (zoomed in)
        assert!(view.viewport.width() < 20.0);
    }

    #[test]
    fn test_graph_view_pan() {
        let mut view = GraphView::new();
        let orig_center = view.viewport.center();

        view.pan(5.0, 3.0);

        let new_center = view.viewport.center();
        assert!((new_center.x - orig_center.x - 5.0).abs() < 0.001);
        assert!((new_center.y - orig_center.y - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_graph_view_fit_bounds() {
        let mut view = GraphView::new();
        let bounds = Rect::new(-2.0, 2.0, -1.0, 1.0);

        view.fit_to_bounds(bounds, 0.1);

        // Viewport should contain the bounds
        assert!(view.viewport.x_min <= -2.0);
        assert!(view.viewport.x_max >= 2.0);
    }
}

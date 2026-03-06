//! Graph canvas management

use crate::common::{Color, Point, Rect};
use super::{CoordinateTransform, GridRenderer, CurveRenderer, MarkerRenderer, RenderContext};

/// The main graph canvas that manages rendering
pub struct GraphCanvas {
    /// Current viewport in world coordinates
    pub viewport: Rect,
    /// Background color
    pub background_color: Color,
    /// Grid renderer
    grid_renderer: GridRenderer,
    /// Curve renderer
    curve_renderer: CurveRenderer,
    /// Marker renderer
    marker_renderer: MarkerRenderer,
}

impl GraphCanvas {
    pub fn new() -> Self {
        Self {
            viewport: Rect::default(),
            background_color: Color::WHITE,
            grid_renderer: GridRenderer::new(),
            curve_renderer: CurveRenderer::new(),
            marker_renderer: MarkerRenderer::new(),
        }
    }

    /// Set the viewport bounds
    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    /// Zoom the viewport by a factor, centered on a point
    pub fn zoom(&mut self, factor: f64, center: Point) {
        self.viewport = self.viewport.zoom(factor, center);
    }

    /// Pan the viewport by a delta
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.viewport = self.viewport.pan(dx, dy);
    }

    /// Reset viewport to default
    pub fn reset_viewport(&mut self) {
        self.viewport = Rect::default();
    }

    /// Render the canvas
    pub fn render(&self, painter: &egui::Painter, rect: egui::Rect) {
        let transform = CoordinateTransform::new(
            self.viewport,
            rect.width(),
            rect.height(),
        );

        // Fill background
        painter.rect_filled(rect, 0.0, self.background_color);

        // Create render context
        let ctx = RenderContext::new(transform, painter, rect);

        // Render grid and axes
        self.grid_renderer.render(&ctx);
    }

    /// Get the grid renderer for configuration
    pub fn grid_mut(&mut self) -> &mut GridRenderer {
        &mut self.grid_renderer
    }

    /// Get the curve renderer
    pub fn curve_renderer(&self) -> &CurveRenderer {
        &self.curve_renderer
    }

    /// Get the marker renderer
    pub fn marker_renderer(&self) -> &MarkerRenderer {
        &self.marker_renderer
    }
}

impl Default for GraphCanvas {
    fn default() -> Self {
        Self::new()
    }
}

/// Handles mouse interaction with the canvas
pub struct CanvasInteraction {
    /// Is the user currently dragging?
    pub is_dragging: bool,
    /// Last mouse position during drag
    pub drag_start: Option<egui::Pos2>,
    /// Current mouse position in world coordinates
    pub mouse_world_pos: Option<Point>,
}

impl CanvasInteraction {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            drag_start: None,
            mouse_world_pos: None,
        }
    }

    /// Handle input events for a canvas
    pub fn handle_input(
        &mut self,
        response: &egui::Response,
        canvas: &mut GraphCanvas,
        transform: &CoordinateTransform,
    ) {
        // Update mouse world position
        if let Some(pos) = response.hover_pos() {
            self.mouse_world_pos = Some(transform.screen_to_world(
                pos.x - response.rect.left(),
                pos.y - response.rect.top(),
            ));
        } else {
            self.mouse_world_pos = None;
        }

        // Handle dragging (panning)
        if response.dragged() {
            let delta = response.drag_delta();
            let dx = -transform.screen_to_world_dx(delta.x);
            let dy = transform.screen_to_world_dy(delta.y);
            canvas.pan(dx, dy);
            self.is_dragging = true;
        } else {
            self.is_dragging = false;
        }

        // Handle scrolling (zooming)
        if response.hovered() {
            let scroll = response.ctx.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                // Zoom factor: scroll up = zoom in, scroll down = zoom out
                let factor = 1.0 - scroll as f64 * 0.001;

                // Zoom centered on mouse position
                if let Some(pos) = self.mouse_world_pos {
                    canvas.zoom(factor, pos);
                } else {
                    canvas.zoom(factor, canvas.viewport.center());
                }
            }
        }

        // Handle keyboard shortcuts
        response.ctx.input(|i| {
            // Reset view with 'R' or '0'
            if i.key_pressed(egui::Key::R) || i.key_pressed(egui::Key::Num0) {
                canvas.reset_viewport();
            }

            // Zoom with +/- keys
            if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                canvas.zoom(0.8, canvas.viewport.center());
            }
            if i.key_pressed(egui::Key::Minus) {
                canvas.zoom(1.25, canvas.viewport.center());
            }

            // Arrow key panning
            let pan_amount = canvas.viewport.width() * 0.1;
            if i.key_pressed(egui::Key::ArrowLeft) {
                canvas.pan(-pan_amount, 0.0);
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                canvas.pan(pan_amount, 0.0);
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                canvas.pan(0.0, pan_amount);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                canvas.pan(0.0, -pan_amount);
            }
        });
    }
}

impl Default for CanvasInteraction {
    fn default() -> Self {
        Self::new()
    }
}

//! Grid and axes rendering

use crate::common::{Color, Point};
use super::RenderContext;

/// Grid visual style configuration
#[derive(Debug, Clone)]
pub struct GridStyle {
    /// Color of the main axes
    pub axis_color: Color,
    /// Width of the main axes
    pub axis_width: f32,
    /// Color of major grid lines
    pub major_grid_color: Color,
    /// Width of major grid lines
    pub major_grid_width: f32,
    /// Color of minor grid lines
    pub minor_grid_color: Color,
    /// Width of minor grid lines
    pub minor_grid_width: f32,
    /// Show minor grid lines
    pub show_minor_grid: bool,
    /// Show grid labels
    pub show_labels: bool,
    /// Label color
    pub label_color: Color,
}

impl Default for GridStyle {
    fn default() -> Self {
        Self {
            axis_color: Color::rgb(0.2, 0.2, 0.2),
            axis_width: 2.0,
            major_grid_color: Color::rgb(0.8, 0.8, 0.8),
            major_grid_width: 1.0,
            minor_grid_color: Color::rgb(0.9, 0.9, 0.9),
            minor_grid_width: 0.5,
            show_minor_grid: true,
            show_labels: true,
            label_color: Color::rgb(0.3, 0.3, 0.3),
        }
    }
}

/// Grid renderer
pub struct GridRenderer {
    pub style: GridStyle,
}

impl GridRenderer {
    pub fn new() -> Self {
        Self {
            style: GridStyle::default(),
        }
    }

    pub fn with_style(style: GridStyle) -> Self {
        Self { style }
    }

    /// Compute nice grid spacing for a given range
    fn compute_grid_spacing(range: f64) -> (f64, f64) {
        // Target ~5-10 major divisions
        let raw_step = range / 8.0;
        let magnitude = 10_f64.powf(raw_step.log10().floor());

        let normalized = raw_step / magnitude;

        let major_step = if normalized < 1.5 {
            magnitude
        } else if normalized < 3.5 {
            2.0 * magnitude
        } else if normalized < 7.5 {
            5.0 * magnitude
        } else {
            10.0 * magnitude
        };

        // Minor step is 1/5 of major step (or 1/4 for 2x steps)
        let minor_step = if (major_step / magnitude - 2.0).abs() < 0.01 {
            major_step / 4.0
        } else {
            major_step / 5.0
        };

        (major_step, minor_step)
    }

    /// Render the grid
    pub fn render(&self, ctx: &RenderContext) {
        let viewport = ctx.transform.viewport;

        // Compute grid spacing
        let (major_x, minor_x) = Self::compute_grid_spacing(viewport.width());
        let (major_y, minor_y) = Self::compute_grid_spacing(viewport.height());

        // Draw minor grid lines
        if self.style.show_minor_grid {
            self.draw_vertical_lines(ctx, minor_x, self.style.minor_grid_color, self.style.minor_grid_width, false);
            self.draw_horizontal_lines(ctx, minor_y, self.style.minor_grid_color, self.style.minor_grid_width, false);
        }

        // Draw major grid lines
        self.draw_vertical_lines(ctx, major_x, self.style.major_grid_color, self.style.major_grid_width, self.style.show_labels);
        self.draw_horizontal_lines(ctx, major_y, self.style.major_grid_color, self.style.major_grid_width, self.style.show_labels);

        // Draw axes
        self.draw_axes(ctx);
    }

    fn draw_vertical_lines(&self, ctx: &RenderContext, step: f64, color: Color, width: f32, labels: bool) {
        let viewport = ctx.transform.viewport;

        // Find first line position
        let start_x = (viewport.x_min / step).floor() * step;

        let mut x = start_x;
        while x <= viewport.x_max {
            // Skip axis (will be drawn separately)
            if x.abs() > step * 0.1 {
                ctx.draw_line(
                    Point::new(x, viewport.y_min),
                    Point::new(x, viewport.y_max),
                    color,
                    width,
                );

                if labels {
                    self.draw_x_label(ctx, x);
                }
            }
            x += step;
        }
    }

    fn draw_horizontal_lines(&self, ctx: &RenderContext, step: f64, color: Color, width: f32, labels: bool) {
        let viewport = ctx.transform.viewport;

        let start_y = (viewport.y_min / step).floor() * step;

        let mut y = start_y;
        while y <= viewport.y_max {
            // Skip axis
            if y.abs() > step * 0.1 {
                ctx.draw_line(
                    Point::new(viewport.x_min, y),
                    Point::new(viewport.x_max, y),
                    color,
                    width,
                );

                if labels {
                    self.draw_y_label(ctx, y);
                }
            }
            y += step;
        }
    }

    fn draw_axes(&self, ctx: &RenderContext) {
        let viewport = ctx.transform.viewport;

        // X axis (y = 0)
        if viewport.y_min <= 0.0 && viewport.y_max >= 0.0 {
            ctx.draw_line(
                Point::new(viewport.x_min, 0.0),
                Point::new(viewport.x_max, 0.0),
                self.style.axis_color,
                self.style.axis_width,
            );
        }

        // Y axis (x = 0)
        if viewport.x_min <= 0.0 && viewport.x_max >= 0.0 {
            ctx.draw_line(
                Point::new(0.0, viewport.y_min),
                Point::new(0.0, viewport.y_max),
                self.style.axis_color,
                self.style.axis_width,
            );
        }
    }

    fn draw_x_label(&self, ctx: &RenderContext, x: f64) {
        let label = format_number(x);
        // Position label slightly below x-axis (or at bottom if axis not visible)
        let y_pos = if ctx.transform.viewport.y_min <= 0.0 && ctx.transform.viewport.y_max >= 0.0 {
            0.0
        } else {
            ctx.transform.viewport.y_min
        };

        let (screen_x, screen_y) = ctx.transform.world_to_screen(Point::new(x, y_pos));
        // Add clip_rect offset for proper positioning
        let offset_x = ctx.clip_rect.left();
        let offset_y = ctx.clip_rect.top();

        ctx.painter.text(
            egui::pos2(screen_x + offset_x, screen_y + offset_y + 15.0),
            egui::Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(11.0),
            self.style.label_color.into(),
        );
    }

    fn draw_y_label(&self, ctx: &RenderContext, y: f64) {
        let label = format_number(y);
        // Position label slightly left of y-axis (or at left edge if axis not visible)
        let x_pos = if ctx.transform.viewport.x_min <= 0.0 && ctx.transform.viewport.x_max >= 0.0 {
            0.0
        } else {
            ctx.transform.viewport.x_min
        };

        let (screen_x, screen_y) = ctx.transform.world_to_screen(Point::new(x_pos, y));
        // Add clip_rect offset for proper positioning
        let offset_x = ctx.clip_rect.left();
        let offset_y = ctx.clip_rect.top();

        ctx.painter.text(
            egui::pos2(screen_x + offset_x - 5.0, screen_y + offset_y),
            egui::Align2::RIGHT_CENTER,
            label,
            egui::FontId::proportional(11.0),
            self.style.label_color.into(),
        );
    }
}

impl Default for GridRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a number for display (remove trailing zeros, handle special cases)
fn format_number(n: f64) -> String {
    if n.abs() < 1e-10 {
        return "0".to_string();
    }

    let abs = n.abs();

    if abs >= 1e6 || abs < 1e-4 {
        // Scientific notation
        format!("{:.2e}", n)
    } else if abs >= 1.0 {
        // Integer or 1-2 decimal places
        if (n - n.round()).abs() < 1e-10 {
            format!("{}", n.round() as i64)
        } else {
            format!("{:.2}", n).trim_end_matches('0').trim_end_matches('.').to_string()
        }
    } else {
        // Small decimal
        format!("{:.4}", n).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(1.5), "1.5");
        assert_eq!(format_number(0.001), "0.001");
        assert_eq!(format_number(1000000.0), "1.00e6");
    }

    #[test]
    fn test_grid_spacing() {
        let (major, minor) = GridRenderer::compute_grid_spacing(10.0);
        assert!((major - 2.0).abs() < 0.01 || (major - 1.0).abs() < 0.01);
        assert!(minor < major);
    }
}

//! Region rendering for inequalities

use crate::common::{Color, Point};
use crate::evaluator::InequalityRegion;
use super::RenderContext;

/// Style for rendering inequality regions
#[derive(Debug, Clone)]
pub struct RegionStyle {
    /// Fill color (with alpha for transparency)
    pub fill_color: Color,
    /// Boundary line color
    pub boundary_color: Color,
    /// Boundary line width
    pub boundary_width: f32,
    /// Whether to show the boundary
    pub show_boundary: bool,
    /// Whether to show the fill
    pub show_fill: bool,
}

impl Default for RegionStyle {
    fn default() -> Self {
        Self {
            fill_color: Color::new(0.3, 0.6, 0.9, 0.3), // Semi-transparent blue
            boundary_color: Color::new(0.2, 0.4, 0.8, 1.0), // Solid blue
            boundary_width: 2.0,
            show_boundary: true,
            show_fill: true,
        }
    }
}

impl RegionStyle {
    /// Create a style from a base color
    pub fn from_color(color: Color) -> Self {
        Self {
            fill_color: Color::new(color.r, color.g, color.b, 0.3),
            boundary_color: color,
            boundary_width: 2.0,
            show_boundary: true,
            show_fill: true,
        }
    }

    pub fn with_fill_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn with_boundary_color(mut self, color: Color) -> Self {
        self.boundary_color = color;
        self
    }

    pub fn with_boundary_width(mut self, width: f32) -> Self {
        self.boundary_width = width;
        self
    }

    pub fn without_boundary(mut self) -> Self {
        self.show_boundary = false;
        self
    }

    pub fn without_fill(mut self) -> Self {
        self.show_fill = false;
        self
    }
}

/// Region renderer for inequality regions
pub struct RegionRenderer;

impl RegionRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render an inequality region
    pub fn render(&self, ctx: &RenderContext, region: &InequalityRegion, style: &RegionStyle) {
        // Render the fill first (so boundary is on top)
        if style.show_fill {
            self.render_fill(ctx, region, style);
        }

        // Then render the boundary
        if style.show_boundary {
            self.render_boundary(ctx, region, style);
        }
    }

    /// Render the filled region using small rectangles for each grid cell
    fn render_fill(&self, ctx: &RenderContext, region: &InequalityRegion, style: &RegionStyle) {
        let offset_x = ctx.clip_rect.left();
        let offset_y = ctx.clip_rect.top();
        let fill_color: egui::Color32 = style.fill_color.into();

        let resolution = region.resolution;
        let step_x = region.bounds.width() / resolution as f64;
        let step_y = region.bounds.height() / resolution as f64;

        // For each cell in the grid, if at least one corner satisfies,
        // draw a semi-transparent quad
        for i in 0..resolution {
            for j in 0..resolution {
                // Count how many corners satisfy the inequality
                let corners = [
                    region.grid[i][j],
                    region.grid[i + 1][j],
                    region.grid[i][j + 1],
                    region.grid[i + 1][j + 1],
                ];

                let satisfying_count = corners.iter().filter(|&&c| c).count();

                // Only fill if at least one corner satisfies
                // For partial cells (on boundary), use lower alpha
                if satisfying_count > 0 {
                    let x0 = region.bounds.x_min + i as f64 * step_x;
                    let y0 = region.bounds.y_min + j as f64 * step_y;
                    let x1 = x0 + step_x;
                    let y1 = y0 + step_y;

                    // Convert to screen coordinates
                    let (sx0, sy0) = ctx.transform.world_to_screen(Point::new(x0, y1));
                    let (sx1, sy1) = ctx.transform.world_to_screen(Point::new(x1, y0));

                    let rect = egui::Rect::from_min_max(
                        egui::pos2(sx0 + offset_x, sy0 + offset_y),
                        egui::pos2(sx1 + offset_x, sy1 + offset_y),
                    );

                    // Adjust alpha based on how many corners satisfy
                    let cell_color = if satisfying_count == 4 {
                        fill_color
                    } else {
                        // Partial fill - reduce alpha proportionally
                        let alpha = fill_color.a() as f32 * (satisfying_count as f32 / 4.0);
                        egui::Color32::from_rgba_unmultiplied(
                            fill_color.r(),
                            fill_color.g(),
                            fill_color.b(),
                            alpha as u8,
                        )
                    };

                    ctx.painter.rect_filled(rect, 0.0, cell_color);
                }
            }
        }
    }

    /// Render the boundary line segments
    fn render_boundary(&self, ctx: &RenderContext, region: &InequalityRegion, style: &RegionStyle) {
        for (p1, p2) in &region.boundary_segments {
            if !p1.x.is_finite() || !p1.y.is_finite() || !p2.x.is_finite() || !p2.y.is_finite() {
                continue;
            }

            ctx.draw_line(*p1, *p2, style.boundary_color, style.boundary_width);
        }
    }
}

impl Default for RegionRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_style_builder() {
        let style = RegionStyle::from_color(Color::RED)
            .with_boundary_width(3.0)
            .without_fill();

        assert_eq!(style.boundary_color, Color::RED);
        assert_eq!(style.boundary_width, 3.0);
        assert!(!style.show_fill);
    }
}

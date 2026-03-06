//! Curve rendering

use crate::common::{Color, Point};
use crate::evaluator::CurveData;
use super::RenderContext;

/// Line style for curves
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Curve visual style configuration
#[derive(Debug, Clone)]
pub struct CurveStyle {
    /// Line color
    pub color: Color,
    /// Line width in pixels
    pub width: f32,
    /// Line style
    pub style: LineStyle,
    /// Dash length for dashed lines
    pub dash_length: f32,
    /// Gap length for dashed lines
    pub gap_length: f32,
    /// Show points on the curve
    pub show_points: bool,
    /// Point radius if showing points
    pub point_radius: f32,
}

impl Default for CurveStyle {
    fn default() -> Self {
        Self {
            color: Color::BLUE,
            width: 2.0,
            style: LineStyle::Solid,
            dash_length: 8.0,
            gap_length: 4.0,
            show_points: false,
            point_radius: 3.0,
        }
    }
}

impl CurveStyle {
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    pub fn dashed() -> Self {
        Self {
            style: LineStyle::Dashed,
            ..Default::default()
        }
    }

    pub fn dotted() -> Self {
        Self {
            style: LineStyle::Dotted,
            dash_length: 2.0,
            gap_length: 4.0,
            ..Default::default()
        }
    }
}

/// Curve renderer
pub struct CurveRenderer;

impl CurveRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render curve data with the given style
    pub fn render(&self, ctx: &RenderContext, curve: &CurveData, style: &CurveStyle) {
        if curve.points.is_empty() {
            return;
        }

        match style.style {
            LineStyle::Solid => self.render_solid(ctx, curve, style),
            LineStyle::Dashed => self.render_dashed(ctx, curve, style),
            LineStyle::Dotted => self.render_dotted(ctx, curve, style),
        }

        if style.show_points {
            self.render_points(ctx, curve, style);
        }
    }

    fn render_solid(&self, ctx: &RenderContext, curve: &CurveData, style: &CurveStyle) {
        let mut i = 0;
        while i < curve.points.len() {
            // Build a continuous segment
            let mut segment_points = Vec::new();

            while i < curve.points.len() {
                let p = &curve.points[i];

                if !p.x.is_finite() || !p.y.is_finite() {
                    // Break on NaN
                    i += 1;
                    break;
                }

                segment_points.push(p);

                // Check continuity
                if i > 0 && i - 1 < curve.continuous.len() && !curve.continuous[i - 1] {
                    i += 1;
                    break;
                }

                i += 1;
            }

            // Draw the segment
            if segment_points.len() >= 2 {
                self.draw_polyline(ctx, &segment_points, style);
            }
        }
    }

    fn render_dashed(&self, ctx: &RenderContext, curve: &CurveData, style: &CurveStyle) {
        // For dashed, we need to measure distance along the curve
        let dash = style.dash_length;
        let gap = style.gap_length;
        let offset_x = ctx.clip_rect.left();
        let offset_y = ctx.clip_rect.top();

        let mut i = 0;
        while i < curve.points.len() - 1 {
            let p1 = &curve.points[i];
            let p2 = &curve.points[i + 1];

            if !p1.x.is_finite() || !p1.y.is_finite() || !p2.x.is_finite() || !p2.y.is_finite() {
                i += 1;
                continue;
            }

            // Check continuity
            if i < curve.continuous.len() && !curve.continuous[i] {
                i += 1;
                continue;
            }

            let (sx1, sy1) = ctx.transform.world_to_screen(*p1);
            let (sx2, sy2) = ctx.transform.world_to_screen(*p2);

            let dx = sx2 - sx1;
            let dy = sy2 - sy1;
            let len = (dx * dx + dy * dy).sqrt();

            if len < 1.0 {
                i += 1;
                continue;
            }

            // Draw dashes along this segment
            let ux = dx / len;
            let uy = dy / len;

            let mut t = 0.0;
            let mut drawing = true;
            while t < len {
                let segment_len = if drawing { dash } else { gap };
                let end_t = (t + segment_len).min(len);

                if drawing {
                    let x1 = sx1 + ux * t + offset_x;
                    let y1 = sy1 + uy * t + offset_y;
                    let x2 = sx1 + ux * end_t + offset_x;
                    let y2 = sy1 + uy * end_t + offset_y;

                    let color32: egui::Color32 = style.color.into();
                    ctx.painter.line_segment(
                        [egui::pos2(x1, y1), egui::pos2(x2, y2)],
                        egui::Stroke::new(style.width, color32),
                    );
                }

                t = end_t;
                drawing = !drawing;
            }

            i += 1;
        }
    }

    fn render_dotted(&self, ctx: &RenderContext, curve: &CurveData, style: &CurveStyle) {
        let spacing = style.dash_length + style.gap_length;
        let offset_x = ctx.clip_rect.left();
        let offset_y = ctx.clip_rect.top();

        let mut accumulated = 0.0;

        for i in 0..curve.points.len() - 1 {
            let p1 = &curve.points[i];
            let p2 = &curve.points[i + 1];

            if !p1.x.is_finite() || !p1.y.is_finite() || !p2.x.is_finite() || !p2.y.is_finite() {
                accumulated = 0.0;
                continue;
            }

            if i < curve.continuous.len() && !curve.continuous[i] {
                accumulated = 0.0;
                continue;
            }

            let (sx1, sy1) = ctx.transform.world_to_screen(*p1);
            let (sx2, sy2) = ctx.transform.world_to_screen(*p2);

            let dx = sx2 - sx1;
            let dy = sy2 - sy1;
            let len = (dx * dx + dy * dy).sqrt();

            let ux = dx / len;
            let uy = dy / len;

            let mut t = 0.0;
            while t < len {
                let pos = accumulated + t;
                if (pos % spacing) < style.dash_length {
                    let x = sx1 + ux * t + offset_x;
                    let y = sy1 + uy * t + offset_y;
                    let color32: egui::Color32 = style.color.into();
                    ctx.painter.circle_filled(
                        egui::pos2(x, y),
                        style.width / 2.0,
                        color32,
                    );
                }
                t += 1.0;
            }

            accumulated += len;
        }
    }

    fn render_points(&self, ctx: &RenderContext, curve: &CurveData, style: &CurveStyle) {
        let color32: egui::Color32 = style.color.into();
        let offset_x = ctx.clip_rect.left();
        let offset_y = ctx.clip_rect.top();

        for p in &curve.points {
            if !p.x.is_finite() || !p.y.is_finite() {
                continue;
            }

            let (sx, sy) = ctx.transform.world_to_screen(*p);
            ctx.painter.circle_filled(
                egui::pos2(sx + offset_x, sy + offset_y),
                style.point_radius,
                color32,
            );
        }
    }

    fn draw_polyline(&self, ctx: &RenderContext, points: &[&Point], style: &CurveStyle) {
        if points.len() < 2 {
            return;
        }

        let offset_x = ctx.clip_rect.left();
        let offset_y = ctx.clip_rect.top();

        let screen_points: Vec<egui::Pos2> = points
            .iter()
            .map(|p| {
                let (x, y) = ctx.transform.world_to_screen(**p);
                egui::pos2(x + offset_x, y + offset_y)
            })
            .collect();

        let color32: egui::Color32 = style.color.into();
        let stroke = egui::Stroke::new(style.width, color32);

        // Draw as connected line segments
        for i in 0..screen_points.len() - 1 {
            ctx.painter.line_segment(
                [screen_points[i], screen_points[i + 1]],
                stroke,
            );
        }
    }

    /// Render implicit function segments
    pub fn render_implicit(
        &self,
        ctx: &RenderContext,
        segments: &[(Point, Point)],
        style: &CurveStyle,
    ) {
        for (p1, p2) in segments {
            if !p1.x.is_finite() || !p1.y.is_finite() || !p2.x.is_finite() || !p2.y.is_finite() {
                continue;
            }

            ctx.draw_line(*p1, *p2, style.color, style.width);
        }
    }
}

impl Default for CurveRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_style_builder() {
        let style = CurveStyle::default()
            .with_color(Color::RED)
            .with_width(3.0);

        assert_eq!(style.color, Color::RED);
        assert_eq!(style.width, 3.0);
    }
}

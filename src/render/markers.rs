//! Marker rendering for special points (intersections, extrema, etc.)

use crate::common::{Color, Point};
use super::RenderContext;

/// Type of marker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkerType {
    /// Intersection point
    Intersection,
    /// Local maximum
    Maximum,
    /// Local minimum
    Minimum,
    /// Zero crossing (root)
    Root,
    /// User-placed point
    UserPoint,
    /// Data point for curve fitting
    DataPoint,
}

/// A marker at a specific point
#[derive(Debug, Clone)]
pub struct Marker {
    /// Position of the marker
    pub position: Point,
    /// Type of marker
    pub marker_type: MarkerType,
    /// Optional label
    pub label: Option<String>,
    /// Color override
    pub color: Option<Color>,
}

impl Marker {
    pub fn new(position: Point, marker_type: MarkerType) -> Self {
        Self {
            position,
            marker_type,
            label: None,
            color: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Create an intersection marker with coordinate label
    pub fn intersection(position: Point) -> Self {
        let label = format!("({:.3}, {:.3})", position.x, position.y);
        Self::new(position, MarkerType::Intersection).with_label(label)
    }

    /// Create a root marker
    pub fn root(x: f64) -> Self {
        let label = format!("x = {:.3}", x);
        Self::new(Point::new(x, 0.0), MarkerType::Root).with_label(label)
    }

    /// Create an extremum marker
    pub fn extremum(position: Point, is_max: bool) -> Self {
        let label = format!("({:.3}, {:.3})", position.x, position.y);
        let marker_type = if is_max { MarkerType::Maximum } else { MarkerType::Minimum };
        Self::new(position, marker_type).with_label(label)
    }
}

/// Marker style configuration
#[derive(Debug, Clone)]
pub struct MarkerStyle {
    /// Radius of the marker in pixels
    pub radius: f32,
    /// Fill color
    pub fill_color: Color,
    /// Stroke color
    pub stroke_color: Color,
    /// Stroke width
    pub stroke_width: f32,
    /// Show label
    pub show_label: bool,
    /// Label color
    pub label_color: Color,
    /// Label offset from marker
    pub label_offset: (f32, f32),
}

impl Default for MarkerStyle {
    fn default() -> Self {
        Self {
            radius: 5.0,
            fill_color: Color::WHITE,
            stroke_color: Color::rgb(0.2, 0.2, 0.8),
            stroke_width: 2.0,
            show_label: true,
            label_color: Color::rgb(0.2, 0.2, 0.2),
            label_offset: (8.0, -8.0),
        }
    }
}

impl MarkerStyle {
    /// Get default style for a marker type
    pub fn for_type(marker_type: MarkerType) -> Self {
        match marker_type {
            MarkerType::Intersection => Self {
                fill_color: Color::WHITE,
                stroke_color: Color::rgb(0.8, 0.2, 0.2),
                ..Default::default()
            },
            MarkerType::Maximum => Self {
                fill_color: Color::rgb(0.2, 0.8, 0.2),
                stroke_color: Color::rgb(0.1, 0.5, 0.1),
                ..Default::default()
            },
            MarkerType::Minimum => Self {
                fill_color: Color::rgb(0.8, 0.5, 0.0),
                stroke_color: Color::rgb(0.5, 0.3, 0.0),
                ..Default::default()
            },
            MarkerType::Root => Self {
                fill_color: Color::rgb(0.5, 0.2, 0.8),
                stroke_color: Color::rgb(0.3, 0.1, 0.5),
                ..Default::default()
            },
            MarkerType::UserPoint => Self {
                fill_color: Color::rgb(0.2, 0.6, 0.9),
                stroke_color: Color::rgb(0.1, 0.4, 0.7),
                ..Default::default()
            },
            MarkerType::DataPoint => Self {
                radius: 4.0,
                fill_color: Color::rgb(0.9, 0.3, 0.3),
                stroke_color: Color::rgb(0.7, 0.1, 0.1),
                stroke_width: 1.5,
                show_label: false,
                ..Default::default()
            },
        }
    }
}

/// Marker renderer
pub struct MarkerRenderer {
    default_styles: std::collections::HashMap<MarkerType, MarkerStyle>,
}

impl MarkerRenderer {
    pub fn new() -> Self {
        use std::collections::HashMap;
        let mut styles = HashMap::new();
        styles.insert(MarkerType::Intersection, MarkerStyle::for_type(MarkerType::Intersection));
        styles.insert(MarkerType::Maximum, MarkerStyle::for_type(MarkerType::Maximum));
        styles.insert(MarkerType::Minimum, MarkerStyle::for_type(MarkerType::Minimum));
        styles.insert(MarkerType::Root, MarkerStyle::for_type(MarkerType::Root));
        styles.insert(MarkerType::UserPoint, MarkerStyle::for_type(MarkerType::UserPoint));
        styles.insert(MarkerType::DataPoint, MarkerStyle::for_type(MarkerType::DataPoint));

        Self {
            default_styles: styles,
        }
    }

    /// Render a single marker
    pub fn render(&self, ctx: &RenderContext, marker: &Marker) {
        let style = self.default_styles
            .get(&marker.marker_type)
            .cloned()
            .unwrap_or_default();

        self.render_with_style(ctx, marker, &style);
    }

    /// Render a marker with custom style
    pub fn render_with_style(&self, ctx: &RenderContext, marker: &Marker, style: &MarkerStyle) {
        let (sx, sy) = ctx.transform.world_to_screen(marker.position);
        // Add clip_rect offset for proper positioning
        let sx = sx + ctx.clip_rect.left();
        let sy = sy + ctx.clip_rect.top();

        // Determine fill color
        let fill_color = marker.color.unwrap_or(style.fill_color);
        let fill32: egui::Color32 = fill_color.into();
        let stroke32: egui::Color32 = style.stroke_color.into();

        // Draw the marker circle
        ctx.painter.circle(
            egui::pos2(sx, sy),
            style.radius,
            fill32,
            egui::Stroke::new(style.stroke_width, stroke32),
        );

        // Draw label
        if style.show_label {
            if let Some(label) = &marker.label {
                ctx.painter.text(
                    egui::pos2(sx + style.label_offset.0, sy + style.label_offset.1),
                    egui::Align2::LEFT_BOTTOM,
                    label,
                    egui::FontId::proportional(11.0),
                    style.label_color.into(),
                );
            }
        }
    }

    /// Render multiple markers
    pub fn render_all(&self, ctx: &RenderContext, markers: &[Marker]) {
        for marker in markers {
            self.render(ctx, marker);
        }
    }

    /// Render data points for curve fitting
    pub fn render_data_points(&self, ctx: &RenderContext, points: &[Point]) {
        let style = MarkerStyle::for_type(MarkerType::DataPoint);

        for point in points {
            let marker = Marker::new(*point, MarkerType::DataPoint);
            self.render_with_style(ctx, &marker, &style);
        }
    }
}

impl Default for MarkerRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_creation() {
        let marker = Marker::intersection(Point::new(1.0, 2.0));
        assert_eq!(marker.marker_type, MarkerType::Intersection);
        assert!(marker.label.is_some());
    }

    #[test]
    fn test_marker_builder() {
        let marker = Marker::new(Point::new(0.0, 0.0), MarkerType::Root)
            .with_label("origin")
            .with_color(Color::RED);

        assert_eq!(marker.label, Some("origin".to_string()));
        assert_eq!(marker.color, Some(Color::RED));
    }
}

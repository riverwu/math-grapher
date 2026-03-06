//! Expression Evaluator Module
//!
//! Provides CPU-based expression evaluation with support for
//! adaptive sampling and interval arithmetic.

mod cpu_eval;
mod interval;
mod adaptive;

pub use cpu_eval::{Evaluator, EvalContext, EvalError};
pub use interval::{Interval, IntervalEvaluator};
pub use adaptive::{AdaptiveSampler, SamplePoint};

use crate::parser::{AstNode, ComparisonOp};
use crate::common::{Point, Rect};

/// Result of evaluating a curve over a range
#[derive(Debug, Clone)]
pub struct CurveData {
    /// Points on the curve
    pub points: Vec<Point>,
    /// Whether each segment is continuous (no discontinuity/undefined)
    pub continuous: Vec<bool>,
}

impl CurveData {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            continuous: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            points: Vec::with_capacity(capacity),
            continuous: Vec::with_capacity(capacity.saturating_sub(1)),
        }
    }
}

impl Default for CurveData {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of evaluating an inequality region
#[derive(Debug, Clone)]
pub struct InequalityRegion {
    /// Grid of points that satisfy the inequality
    /// grid[i][j] is true if the point at (x_min + i*dx, y_min + j*dy) satisfies the condition
    pub grid: Vec<Vec<bool>>,
    /// Boundary line segments (for rendering the boundary curve)
    pub boundary_segments: Vec<(Point, Point)>,
    /// Resolution of the grid
    pub resolution: usize,
    /// Bounds of the region
    pub bounds: Rect,
}

impl InequalityRegion {
    pub fn new(resolution: usize, bounds: Rect) -> Self {
        Self {
            grid: vec![vec![false; resolution + 1]; resolution + 1],
            boundary_segments: Vec::new(),
            resolution,
            bounds,  // Rect is Copy
        }
    }

    /// Get the world coordinates for grid index (i, j)
    pub fn grid_to_world(&self, i: usize, j: usize) -> Point {
        let dx = self.bounds.width() / self.resolution as f64;
        let dy = self.bounds.height() / self.resolution as f64;
        Point::new(
            self.bounds.x_min + i as f64 * dx,
            self.bounds.y_min + j as f64 * dy,
        )
    }
}

/// Evaluate an explicit function y = f(x) over a viewport
pub fn evaluate_explicit(
    ast: &AstNode,
    bounds: &Rect,
    num_samples: usize,
) -> Result<CurveData, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let mut curve = CurveData::with_capacity(num_samples);
    let step = bounds.width() / (num_samples - 1) as f64;

    let mut prev_y: Option<f64> = None;

    for i in 0..num_samples {
        let x = bounds.x_min + i as f64 * step;
        ctx.set("x", x);

        match evaluator.eval(ast, &ctx) {
            Ok(y) if y.is_finite() => {
                curve.points.push(Point::new(x, y));

                // Check for continuity
                if let Some(py) = prev_y {
                    // Consider discontinuous if jump is too large relative to step
                    let jump = (y - py).abs();
                    let threshold = bounds.height() * 0.5;
                    curve.continuous.push(jump < threshold);
                }
                prev_y = Some(y);
            }
            _ => {
                // Undefined point - mark discontinuity
                if prev_y.is_some() {
                    prev_y = None;
                }
                // Insert NaN point to indicate break
                curve.points.push(Point::new(x, f64::NAN));
                if !curve.continuous.is_empty() || !curve.points.is_empty() {
                    curve.continuous.push(false);
                }
            }
        }
    }

    Ok(curve)
}

/// Evaluate an implicit function F(x, y) = 0 using marching squares
pub fn evaluate_implicit(
    ast: &AstNode,
    bounds: &Rect,
    resolution: usize,
) -> Result<Vec<(Point, Point)>, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let step_x = bounds.width() / resolution as f64;
    let step_y = bounds.height() / resolution as f64;

    // Evaluate on grid
    let mut grid = vec![vec![0.0; resolution + 1]; resolution + 1];

    for i in 0..=resolution {
        for j in 0..=resolution {
            let x = bounds.x_min + i as f64 * step_x;
            let y = bounds.y_min + j as f64 * step_y;
            ctx.set("x", x);
            ctx.set("y", y);

            grid[i][j] = evaluator.eval(ast, &ctx).unwrap_or(f64::NAN);
        }
    }

    // Marching squares
    let mut segments = Vec::new();

    for i in 0..resolution {
        for j in 0..resolution {
            let x0 = bounds.x_min + i as f64 * step_x;
            let y0 = bounds.y_min + j as f64 * step_y;
            let x1 = x0 + step_x;
            let y1 = y0 + step_y;

            let v00 = grid[i][j];
            let v10 = grid[i + 1][j];
            let v01 = grid[i][j + 1];
            let v11 = grid[i + 1][j + 1];

            // Skip if any NaN
            if v00.is_nan() || v10.is_nan() || v01.is_nan() || v11.is_nan() {
                continue;
            }

            // Compute case index (4-bit)
            let mut case = 0;
            if v00 > 0.0 { case |= 1; }
            if v10 > 0.0 { case |= 2; }
            if v01 > 0.0 { case |= 4; }
            if v11 > 0.0 { case |= 8; }

            // For saddle points (cases 6 and 9), sample the center to disambiguate
            let center_value = if case == 6 || case == 9 {
                let xc = (x0 + x1) / 2.0;
                let yc = (y0 + y1) / 2.0;
                ctx.set("x", xc);
                ctx.set("y", yc);
                evaluator.eval(ast, &ctx).unwrap_or(0.0)
            } else {
                0.0
            };

            // Process marching squares cases
            let segs = march_square_case(case, x0, y0, x1, y1, v00, v10, v01, v11, center_value);
            segments.extend(segs);
        }
    }

    Ok(segments)
}

/// Linear interpolation for zero crossing
fn lerp_zero(v0: f64, v1: f64, t0: f64, t1: f64) -> f64 {
    if (v1 - v0).abs() < 1e-10 {
        (t0 + t1) / 2.0
    } else {
        t0 + (t1 - t0) * (-v0) / (v1 - v0)
    }
}

/// Process a single marching squares cell
///
/// Corner layout and bit positions:
/// ```text
/// v01 (bit 2) --- v11 (bit 3)
///     |               |
/// v00 (bit 0) --- v10 (bit 1)
/// ```
fn march_square_case(
    case: u8,
    x0: f64, y0: f64,
    x1: f64, y1: f64,
    v00: f64, v10: f64, v01: f64, v11: f64,
    center_value: f64,
) -> Vec<(Point, Point)> {
    // Edge crossing points using linear interpolation
    let left = || Point::new(x0, lerp_zero(v00, v01, y0, y1));
    let right = || Point::new(x1, lerp_zero(v10, v11, y0, y1));
    let bottom = || Point::new(lerp_zero(v00, v10, x0, x1), y0);
    let top = || Point::new(lerp_zero(v01, v11, x0, x1), y1);

    match case {
        // All same sign - no contour
        0 | 15 => vec![],

        // Single corner different - one line segment
        1 | 14 => vec![(left(), bottom())],      // Only v00 different
        2 | 13 => vec![(bottom(), right())],     // Only v10 different
        4 | 11 => vec![(left(), top())],         // Only v01 different
        8 | 7 => vec![(right(), top())],         // Only v11 different

        // Two adjacent corners same sign - one line segment
        3 | 12 => vec![(left(), right())],       // Bottom row same (v00, v10)
        5 | 10 => vec![(bottom(), top())],       // Left column same (v00, v01) or right column same (v10, v11)

        // Saddle points: diagonal corners same sign - need disambiguation
        // Case 6: v10 > 0, v01 > 0 (diagonal positive), v00 <= 0, v11 <= 0
        6 => {
            if center_value > 0.0 {
                // Center positive - connects the positive diagonal
                vec![(left(), bottom()), (right(), top())]
            } else {
                // Center negative - separates the positive diagonal
                vec![(left(), top()), (bottom(), right())]
            }
        }
        // Case 9: v00 > 0, v11 > 0 (diagonal positive), v10 <= 0, v01 <= 0
        9 => {
            if center_value > 0.0 {
                // Center positive - connects the positive diagonal
                vec![(left(), top()), (bottom(), right())]
            } else {
                // Center negative - separates the positive diagonal
                vec![(left(), bottom()), (right(), top())]
            }
        }

        _ => vec![],
    }
}

/// Evaluate a parametric curve (x(t), y(t))
pub fn evaluate_parametric(
    x_ast: &AstNode,
    y_ast: &AstNode,
    t_range: (f64, f64),
    num_samples: usize,
) -> Result<CurveData, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let mut curve = CurveData::with_capacity(num_samples);
    let step = (t_range.1 - t_range.0) / (num_samples - 1) as f64;

    let mut prev_valid = false;

    for i in 0..num_samples {
        let t = t_range.0 + i as f64 * step;
        ctx.set("t", t);

        let x_result = evaluator.eval(x_ast, &ctx);
        let y_result = evaluator.eval(y_ast, &ctx);

        match (x_result, y_result) {
            (Ok(x), Ok(y)) if x.is_finite() && y.is_finite() => {
                curve.points.push(Point::new(x, y));
                if prev_valid && curve.points.len() > 1 {
                    curve.continuous.push(true);
                } else if curve.points.len() > 1 {
                    curve.continuous.push(false);
                }
                prev_valid = true;
            }
            _ => {
                curve.points.push(Point::new(f64::NAN, f64::NAN));
                if !curve.continuous.is_empty() || curve.points.len() > 1 {
                    curve.continuous.push(false);
                }
                prev_valid = false;
            }
        }
    }

    Ok(curve)
}

/// Evaluate a polar curve r = f(theta)
pub fn evaluate_polar(
    ast: &AstNode,
    theta_range: (f64, f64),
    num_samples: usize,
) -> Result<CurveData, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let mut curve = CurveData::with_capacity(num_samples);
    let step = (theta_range.1 - theta_range.0) / (num_samples - 1) as f64;

    let mut prev_valid = false;

    for i in 0..num_samples {
        let theta = theta_range.0 + i as f64 * step;
        ctx.set("theta", theta);
        ctx.set("t", theta); // Allow 't' as alias for theta

        match evaluator.eval(ast, &ctx) {
            Ok(r) if r.is_finite() => {
                let x = r * theta.cos();
                let y = r * theta.sin();
                curve.points.push(Point::new(x, y));
                if prev_valid && curve.points.len() > 1 {
                    curve.continuous.push(true);
                } else if curve.points.len() > 1 {
                    curve.continuous.push(false);
                }
                prev_valid = true;
            }
            _ => {
                curve.points.push(Point::new(f64::NAN, f64::NAN));
                if curve.points.len() > 1 {
                    curve.continuous.push(false);
                }
                prev_valid = false;
            }
        }
    }

    Ok(curve)
}

/// Evaluate an inequality region
///
/// The expression `ast` represents (left - right) where the original inequality was
/// `left op right`. For example, `y > x^2` becomes `y - x^2` with op = Greater.
///
/// The region is satisfied when:
/// - `op = Greater`: ast > 0
/// - `op = GreaterEq`: ast >= 0
/// - `op = Less`: ast < 0
/// - `op = LessEq`: ast <= 0
pub fn evaluate_inequality(
    ast: &AstNode,
    op: ComparisonOp,
    bounds: &Rect,
    resolution: usize,
) -> Result<InequalityRegion, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let step_x = bounds.width() / resolution as f64;
    let step_y = bounds.height() / resolution as f64;

    let mut region = InequalityRegion::new(resolution, *bounds);

    // Evaluate the function on a grid
    let mut values = vec![vec![0.0f64; resolution + 1]; resolution + 1];

    for i in 0..=resolution {
        for j in 0..=resolution {
            let x = bounds.x_min + i as f64 * step_x;
            let y = bounds.y_min + j as f64 * step_y;
            ctx.set("x", x);
            ctx.set("y", y);

            let value = evaluator.eval(ast, &ctx).unwrap_or(f64::NAN);
            values[i][j] = value;

            // Check if point satisfies the inequality
            if value.is_finite() {
                let satisfies = match op {
                    ComparisonOp::Greater => value > 0.0,
                    ComparisonOp::GreaterEq => value >= 0.0,
                    ComparisonOp::Less => value < 0.0,
                    ComparisonOp::LessEq => value <= 0.0,
                };
                region.grid[i][j] = satisfies;
            }
        }
    }

    // Find boundary using marching squares (looking for zero crossings)
    // The boundary is where the expression equals zero
    for i in 0..resolution {
        for j in 0..resolution {
            let x0 = bounds.x_min + i as f64 * step_x;
            let y0 = bounds.y_min + j as f64 * step_y;
            let x1 = x0 + step_x;
            let y1 = y0 + step_y;

            let v00 = values[i][j];
            let v10 = values[i + 1][j];
            let v01 = values[i][j + 1];
            let v11 = values[i + 1][j + 1];

            // Skip if any NaN
            if v00.is_nan() || v10.is_nan() || v01.is_nan() || v11.is_nan() {
                continue;
            }

            // Compute case index (4-bit) based on sign
            let mut case = 0u8;
            if v00 > 0.0 { case |= 1; }
            if v10 > 0.0 { case |= 2; }
            if v01 > 0.0 { case |= 4; }
            if v11 > 0.0 { case |= 8; }

            // For saddle points, sample the center
            let center_value = if case == 6 || case == 9 {
                let xc = (x0 + x1) / 2.0;
                let yc = (y0 + y1) / 2.0;
                ctx.set("x", xc);
                ctx.set("y", yc);
                evaluator.eval(ast, &ctx).unwrap_or(0.0)
            } else {
                0.0
            };

            // Get boundary segments using the same marching squares logic
            let segs = march_square_case(case, x0, y0, x1, y1, v00, v10, v01, v11, center_value);
            region.boundary_segments.extend(segs);
        }
    }

    Ok(region)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_evaluate_explicit() {
        let ast = parse("x^2").unwrap();
        let bounds = Rect::new(-2.0, 2.0, -1.0, 5.0);
        let result = evaluate_explicit(&ast, &bounds, 5).unwrap();

        assert_eq!(result.points.len(), 5);
        // x = 0 should give y = 0
        let mid = &result.points[2];
        assert!((mid.x - 0.0).abs() < 0.01);
        assert!((mid.y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_evaluate_inequality_greater() {
        // Test y > x^2, which becomes y - x^2 > 0
        let ast = parse("y - x^2").unwrap();
        let bounds = Rect::new(-2.0, 2.0, -1.0, 4.0);
        let result = evaluate_inequality(&ast, ComparisonOp::Greater, &bounds, 20).unwrap();

        // At (0, 1): y - x^2 = 1 - 0 = 1 > 0, should satisfy
        // At (0, -0.5): y - x^2 = -0.5 - 0 = -0.5 < 0, should not satisfy

        // Find grid indices for approximate positions
        let mid_i = 10; // x = 0
        let high_j = 10; // y = 1 (approximately)
        let low_j = 0;   // y = -1 (approximately)

        // Point above parabola should satisfy
        assert!(result.grid[mid_i][high_j], "Point at y=1, x=0 should satisfy y > x^2");

        // Point below parabola should not satisfy
        assert!(!result.grid[mid_i][low_j], "Point at y=-1, x=0 should not satisfy y > x^2");

        // Should have boundary segments
        assert!(!result.boundary_segments.is_empty(), "Should have boundary segments");
    }

    #[test]
    fn test_evaluate_inequality_less() {
        // Test y < x, which becomes y - x < 0
        let ast = parse("y - x").unwrap();
        let bounds = Rect::new(-2.0, 2.0, -2.0, 2.0);
        let result = evaluate_inequality(&ast, ComparisonOp::Less, &bounds, 20).unwrap();

        // At (1, 0): y - x = 0 - 1 = -1 < 0, should satisfy
        // At (0, 1): y - x = 1 - 0 = 1 > 0, should not satisfy

        // The line y = x divides the plane
        // Points below the line (where y < x) should satisfy

        // Check some grid points
        let resolution = 20;
        let step = 4.0 / resolution as f64; // bounds width and height are 4

        // Point at (1, 0) - should be below line y=x
        let i = ((1.0 - (-2.0)) / step).round() as usize;
        let j = ((0.0 - (-2.0)) / step).round() as usize;
        assert!(result.grid[i][j], "Point (1, 0) should satisfy y < x");

        // Should have boundary segments
        assert!(!result.boundary_segments.is_empty(), "Should have boundary segments");
    }

    #[test]
    fn test_evaluate_implicit_circle() {
        let ast = parse("x^2 + y^2 - 1").unwrap();
        let bounds = Rect::new(-2.0, 2.0, -2.0, 2.0);
        let result = evaluate_implicit(&ast, &bounds, 20).unwrap();

        // Should produce segments forming a circle
        assert!(!result.is_empty());
    }

    #[test]
    fn test_evaluate_implicit_circle_no_extra_lines() {
        // Test that a circle doesn't produce diagonal artifacts
        let ast = parse("x^2 + y^2 - 4").unwrap();
        let bounds = Rect::new(-3.0, 3.0, -3.0, 3.0);
        let result = evaluate_implicit(&ast, &bounds, 50).unwrap();

        // Check that all segments are approximately on the circle (radius 2)
        for (p1, p2) in &result {
            let r1 = (p1.x * p1.x + p1.y * p1.y).sqrt();
            let r2 = (p2.x * p2.x + p2.y * p2.y).sqrt();
            // Allow some tolerance due to linear interpolation
            assert!((r1 - 2.0).abs() < 0.3, "Point ({}, {}) has radius {} instead of 2", p1.x, p1.y, r1);
            assert!((r2 - 2.0).abs() < 0.3, "Point ({}, {}) has radius {} instead of 2", p2.x, p2.y, r2);
        }
    }

    #[test]
    fn test_evaluate_implicit_ellipse() {
        // Test an ellipse: x^2/4 + y^2 - 1 = 0 (semi-major axis 2, semi-minor axis 1)
        let ast = parse("x^2/4 + y^2 - 1").unwrap();
        let bounds = Rect::new(-3.0, 3.0, -2.0, 2.0);
        let result = evaluate_implicit(&ast, &bounds, 40).unwrap();

        // Should produce segments forming an ellipse
        assert!(!result.is_empty());

        // Check that all segments are approximately on the ellipse
        for (p1, p2) in &result {
            let e1 = p1.x * p1.x / 4.0 + p1.y * p1.y;
            let e2 = p2.x * p2.x / 4.0 + p2.y * p2.y;
            // Should be close to 1
            assert!((e1 - 1.0).abs() < 0.2, "Point not on ellipse: ({}, {})", p1.x, p1.y);
            assert!((e2 - 1.0).abs() < 0.2, "Point not on ellipse: ({}, {})", p2.x, p2.y);
        }
    }

    #[test]
    fn test_marching_squares_saddle_disambiguation() {
        // Create a scenario that should trigger saddle points
        // Use a hyperbola: x*y = 1, which has saddle points at the origin
        let ast = parse("x*y - 1").unwrap();
        let bounds = Rect::new(-3.0, 3.0, -3.0, 3.0);
        let result = evaluate_implicit(&ast, &bounds, 30).unwrap();

        // Should produce two hyperbola branches
        assert!(!result.is_empty());

        // Verify points are approximately on the hyperbola
        for (p1, p2) in &result {
            if p1.x.abs() > 0.3 && p1.y.abs() > 0.3 {
                let product1 = p1.x * p1.y;
                assert!((product1 - 1.0).abs() < 0.3, "Point not on hyperbola: ({}, {})", p1.x, p1.y);
            }
            if p2.x.abs() > 0.3 && p2.y.abs() > 0.3 {
                let product2 = p2.x * p2.y;
                assert!((product2 - 1.0).abs() < 0.3, "Point not on hyperbola: ({}, {})", p2.x, p2.y);
            }
        }
    }
}

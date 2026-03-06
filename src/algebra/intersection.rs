//! Intersection finding between curves

use crate::common::Point;
use crate::parser::AstNode;
use crate::evaluator::{Evaluator, EvalContext, EvalError};

/// An intersection point between two curves
#[derive(Debug, Clone)]
pub struct Intersection {
    /// The intersection point
    pub point: Point,
    /// Confidence score (0 to 1)
    pub confidence: f64,
    /// Indices of the intersecting curves
    pub curves: (usize, usize),
}

impl Intersection {
    pub fn new(point: Point, curves: (usize, usize)) -> Self {
        Self {
            point,
            confidence: 1.0,
            curves,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Find intersections between two explicit functions y = f(x) and y = g(x)
pub fn find_intersections(
    f: &AstNode,
    g: &AstNode,
    x_range: (f64, f64),
    tolerance: f64,
) -> Result<Vec<Point>, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    // Number of initial samples
    let num_samples = 100;
    let step = (x_range.1 - x_range.0) / num_samples as f64;

    let mut intersections = Vec::new();

    // Sample both functions
    let mut prev_diff: Option<f64> = None;
    let mut prev_x: Option<f64> = None;

    for i in 0..=num_samples {
        let x = x_range.0 + i as f64 * step;
        ctx.set("x", x);

        let y_f = evaluator.eval(f, &ctx)?;
        let y_g = evaluator.eval(g, &ctx)?;

        if !y_f.is_finite() || !y_g.is_finite() {
            prev_diff = None;
            prev_x = None;
            continue;
        }

        let diff = y_f - y_g;

        // Check for exact intersection
        if diff.abs() < tolerance {
            intersections.push(Point::new(x, y_f));
            prev_diff = Some(diff);
            prev_x = Some(x);
            continue;
        }

        // Check for sign change (zero crossing)
        if let (Some(pd), Some(px)) = (prev_diff, prev_x) {
            if pd * diff < 0.0 {
                // Sign change detected, refine with bisection
                if let Ok(root_x) = bisection_refine(f, g, px, x, tolerance) {
                    ctx.set("x", root_x);
                    let y = evaluator.eval(f, &ctx)?;
                    if y.is_finite() {
                        intersections.push(Point::new(root_x, y));
                    }
                }
            }
        }

        prev_diff = Some(diff);
        prev_x = Some(x);
    }

    // Remove duplicates (intersections too close together)
    remove_duplicate_points(&mut intersections, tolerance * 10.0);

    Ok(intersections)
}

/// Refine an intersection using bisection method
fn bisection_refine(
    f: &AstNode,
    g: &AstNode,
    mut a: f64,
    mut b: f64,
    tolerance: f64,
) -> Result<f64, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let max_iterations = 50;

    for _ in 0..max_iterations {
        let mid = (a + b) / 2.0;

        if (b - a) / 2.0 < tolerance {
            return Ok(mid);
        }

        ctx.set("x", a);
        let fa = evaluator.eval(f, &ctx)? - evaluator.eval(g, &ctx)?;

        ctx.set("x", mid);
        let fmid = evaluator.eval(f, &ctx)? - evaluator.eval(g, &ctx)?;

        if !fa.is_finite() || !fmid.is_finite() {
            return Ok(mid);
        }

        if fa * fmid < 0.0 {
            b = mid;
        } else {
            a = mid;
        }
    }

    Ok((a + b) / 2.0)
}

/// Remove duplicate points that are within a certain distance
fn remove_duplicate_points(points: &mut Vec<Point>, min_distance: f64) {
    let min_dist_sq = min_distance * min_distance;

    let mut i = 0;
    while i < points.len() {
        let mut j = i + 1;
        while j < points.len() {
            let dx = points[i].x - points[j].x;
            let dy = points[i].y - points[j].y;
            if dx * dx + dy * dy < min_dist_sq {
                points.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}

/// Find intersections between an explicit function and an implicit function
#[allow(dead_code)]
pub fn find_explicit_implicit_intersections(
    explicit: &AstNode,  // y = f(x)
    implicit: &AstNode,  // F(x, y) = 0
    x_range: (f64, f64),
    tolerance: f64,
) -> Result<Vec<Point>, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let num_samples = 200;
    let step = (x_range.1 - x_range.0) / num_samples as f64;

    let mut intersections = Vec::new();
    let mut prev_val: Option<f64> = None;
    let mut prev_x: Option<f64> = None;
    let mut prev_y: Option<f64> = None;

    for i in 0..=num_samples {
        let x = x_range.0 + i as f64 * step;
        ctx.set("x", x);

        // Evaluate explicit function to get y
        let y = match evaluator.eval(explicit, &ctx) {
            Ok(v) if v.is_finite() => v,
            _ => {
                prev_val = None;
                prev_x = None;
                prev_y = None;
                continue;
            }
        };

        ctx.set("y", y);

        // Evaluate implicit function
        let val = match evaluator.eval(implicit, &ctx) {
            Ok(v) if v.is_finite() => v,
            _ => {
                prev_val = None;
                prev_x = None;
                prev_y = None;
                continue;
            }
        };

        // Check for sign change
        if let (Some(pv), Some(px), Some(py)) = (prev_val, prev_x, prev_y) {
            if pv * val < 0.0 {
                // Linear interpolation to find approximate crossing
                let t = pv.abs() / (pv.abs() + val.abs());
                let int_x = px + t * (x - px);
                let int_y = py + t * (y - py);
                intersections.push(Point::new(int_x, int_y));
            }
        }

        prev_val = Some(val);
        prev_x = Some(x);
        prev_y = Some(y);
    }

    remove_duplicate_points(&mut intersections, tolerance * 10.0);

    Ok(intersections)
}

/// Find all intersections in a set of curves
#[allow(dead_code)]
pub fn find_all_intersections(
    curves: &[(AstNode, bool)],  // (ast, is_implicit)
    x_range: (f64, f64),
    tolerance: f64,
) -> Vec<Intersection> {
    let mut all_intersections = Vec::new();

    for i in 0..curves.len() {
        for j in (i + 1)..curves.len() {
            let result = match (curves[i].1, curves[j].1) {
                (false, false) => {
                    // Both explicit
                    find_intersections(&curves[i].0, &curves[j].0, x_range, tolerance)
                }
                (false, true) => {
                    // First explicit, second implicit
                    find_explicit_implicit_intersections(&curves[i].0, &curves[j].0, x_range, tolerance)
                }
                (true, false) => {
                    // First implicit, second explicit
                    find_explicit_implicit_intersections(&curves[j].0, &curves[i].0, x_range, tolerance)
                }
                (true, true) => {
                    // Both implicit - more complex, skip for now
                    Ok(vec![])
                }
            };

            if let Ok(points) = result {
                for point in points {
                    all_intersections.push(Intersection::new(point, (i, j)));
                }
            }
        }
    }

    all_intersections
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_find_intersections_lines() {
        let f = parse("x").unwrap();       // y = x
        let g = parse("-x + 2").unwrap();  // y = -x + 2

        let intersections = find_intersections(&f, &g, (-5.0, 5.0), 1e-6).unwrap();

        assert_eq!(intersections.len(), 1);
        let p = &intersections[0];
        assert!((p.x - 1.0).abs() < 0.001);
        assert!((p.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_find_intersections_parabola_line() {
        let f = parse("x^2").unwrap();    // y = x^2
        let g = parse("x + 2").unwrap();  // y = x + 2

        let intersections = find_intersections(&f, &g, (-5.0, 5.0), 1e-6).unwrap();

        // x^2 = x + 2 => x^2 - x - 2 = 0 => (x-2)(x+1) = 0 => x = 2 or x = -1
        assert_eq!(intersections.len(), 2);

        let x_values: Vec<f64> = intersections.iter().map(|p| p.x).collect();
        assert!(x_values.iter().any(|&x| (x - 2.0).abs() < 0.01));
        assert!(x_values.iter().any(|&x| (x + 1.0).abs() < 0.01));
    }

    #[test]
    fn test_no_intersections() {
        let f = parse("x^2 + 1").unwrap();  // y = x^2 + 1
        let g = parse("0").unwrap();        // y = 0

        let intersections = find_intersections(&f, &g, (-5.0, 5.0), 1e-6).unwrap();

        assert_eq!(intersections.len(), 0);
    }
}

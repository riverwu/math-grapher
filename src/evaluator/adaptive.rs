//! Adaptive sampling for curve rendering
//!
//! Uses adaptive subdivision to place more samples where the curve changes rapidly.

use crate::parser::AstNode;
use crate::common::Point;
use super::{Evaluator, EvalContext, EvalError};

/// A sample point with derivative information
#[derive(Debug, Clone, Copy)]
pub struct SamplePoint {
    pub x: f64,
    pub y: f64,
    pub dy: f64,  // Estimated derivative
}

impl SamplePoint {
    pub fn new(x: f64, y: f64, dy: f64) -> Self {
        Self { x, y, dy }
    }
}

/// Adaptive sampler configuration
#[derive(Debug, Clone)]
pub struct AdaptiveSamplerConfig {
    /// Minimum number of samples
    pub min_samples: usize,
    /// Maximum number of samples
    pub max_samples: usize,
    /// Maximum recursion depth
    pub max_depth: usize,
    /// Flatness threshold (radians)
    pub flatness_threshold: f64,
    /// Maximum allowed y-change between samples (relative to viewport)
    pub max_y_change: f64,
}

impl Default for AdaptiveSamplerConfig {
    fn default() -> Self {
        Self {
            min_samples: 100,
            max_samples: 5000,
            max_depth: 10,
            flatness_threshold: 0.1,
            max_y_change: 0.2,
        }
    }
}

/// Adaptive sampler for explicit functions
pub struct AdaptiveSampler {
    evaluator: Evaluator,
    config: AdaptiveSamplerConfig,
}

impl AdaptiveSampler {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
            config: AdaptiveSamplerConfig::default(),
        }
    }

    pub fn with_config(config: AdaptiveSamplerConfig) -> Self {
        Self {
            evaluator: Evaluator::new(),
            config,
        }
    }

    /// Sample an explicit function y = f(x) adaptively
    pub fn sample(
        &self,
        ast: &AstNode,
        x_min: f64,
        x_max: f64,
        viewport_height: f64,
    ) -> Result<Vec<Point>, EvalError> {
        let mut ctx = EvalContext::new();
        let mut points = Vec::with_capacity(self.config.min_samples);

        // Initial uniform sampling
        let initial_samples = self.config.min_samples.min(50);
        let step = (x_max - x_min) / (initial_samples - 1) as f64;

        let mut initial_points = Vec::with_capacity(initial_samples);
        for i in 0..initial_samples {
            let x = x_min + i as f64 * step;
            ctx.set("x", x);
            let y = self.evaluator.eval(ast, &ctx).ok();
            initial_points.push((x, y));
        }

        // Adaptively subdivide segments
        for i in 0..initial_points.len() - 1 {
            let (x0, y0) = initial_points[i];
            let (x1, y1) = initial_points[i + 1];

            self.subdivide(
                ast,
                x0, y0,
                x1, y1,
                0,
                viewport_height,
                &mut points,
            )?;
        }

        // Add final point
        if let Some((x, Some(y))) = initial_points.last() {
            if y.is_finite() {
                points.push(Point::new(*x, *y));
            }
        }

        Ok(points)
    }

    fn subdivide(
        &self,
        ast: &AstNode,
        x0: f64,
        y0: Option<f64>,
        x1: f64,
        y1: Option<f64>,
        depth: usize,
        viewport_height: f64,
        points: &mut Vec<Point>,
    ) -> Result<(), EvalError> {
        // Add first point if valid
        if let Some(y) = y0 {
            if y.is_finite() {
                points.push(Point::new(x0, y));
            }
        }

        // Check if we've reached maximum depth or samples
        if depth >= self.config.max_depth || points.len() >= self.config.max_samples {
            return Ok(());
        }

        // Check if subdivision is needed
        let needs_subdivision = self.needs_subdivision(y0, y1, viewport_height);

        if needs_subdivision {
            // Evaluate midpoint
            let x_mid = (x0 + x1) / 2.0;
            let mut ctx = EvalContext::new();
            ctx.set("x", x_mid);
            let y_mid = self.evaluator.eval(ast, &ctx).ok();

            // Check flatness
            if let (Some(y0_val), Some(y_mid_val), Some(y1_val)) = (y0, y_mid, y1) {
                if y0_val.is_finite() && y_mid_val.is_finite() && y1_val.is_finite() {
                    // Check if midpoint is close to linear interpolation
                    let interpolated = (y0_val + y1_val) / 2.0;
                    let error = (y_mid_val - interpolated).abs();
                    let threshold = self.config.flatness_threshold * viewport_height;

                    if error > threshold && depth < self.config.max_depth {
                        // Recurse on both halves
                        self.subdivide(ast, x0, y0, x_mid, y_mid, depth + 1, viewport_height, points)?;
                        self.subdivide(ast, x_mid, y_mid, x1, y1, depth + 1, viewport_height, points)?;
                        return Ok(());
                    }
                }
            }

            // Check for sign change (possible discontinuity)
            if let (Some(y0_val), Some(y_mid_val)) = (y0, y_mid) {
                if y0_val.is_finite() && y_mid_val.is_finite() {
                    if (y0_val > 0.0) != (y_mid_val > 0.0) && depth < self.config.max_depth {
                        // Possible zero crossing, subdivide
                        self.subdivide(ast, x0, y0, x_mid, y_mid, depth + 1, viewport_height, points)?;
                        self.subdivide(ast, x_mid, y_mid, x1, y1, depth + 1, viewport_height, points)?;
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    fn needs_subdivision(&self, y0: Option<f64>, y1: Option<f64>, viewport_height: f64) -> bool {
        match (y0, y1) {
            (Some(a), Some(b)) if a.is_finite() && b.is_finite() => {
                let change = (b - a).abs();
                change > self.config.max_y_change * viewport_height
            }
            // One or both undefined - might need subdivision to find boundary
            (Some(_), None) | (None, Some(_)) => true,
            _ => false,
        }
    }
}

impl Default for AdaptiveSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute numerical derivative using central difference
pub fn numerical_derivative(
    ast: &AstNode,
    x: f64,
    h: f64,
) -> Result<f64, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    ctx.set("x", x + h);
    let y_plus = evaluator.eval(ast, &ctx)?;

    ctx.set("x", x - h);
    let y_minus = evaluator.eval(ast, &ctx)?;

    Ok((y_plus - y_minus) / (2.0 * h))
}

/// Compute numerical second derivative
pub fn numerical_second_derivative(
    ast: &AstNode,
    x: f64,
    h: f64,
) -> Result<f64, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    ctx.set("x", x + h);
    let y_plus = evaluator.eval(ast, &ctx)?;

    ctx.set("x", x);
    let y = evaluator.eval(ast, &ctx)?;

    ctx.set("x", x - h);
    let y_minus = evaluator.eval(ast, &ctx)?;

    Ok((y_plus - 2.0 * y + y_minus) / (h * h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_adaptive_sampling_linear() {
        let ast = parse("2 * x + 1").unwrap();
        let sampler = AdaptiveSampler::new();
        let points = sampler.sample(&ast, -5.0, 5.0, 10.0).unwrap();

        // Linear function should not need many subdivisions
        assert!(points.len() >= 10);
        assert!(points.len() < 200);
    }

    #[test]
    fn test_adaptive_sampling_sin() {
        let ast = parse("sin(x)").unwrap();
        let sampler = AdaptiveSampler::new();
        let points = sampler.sample(&ast, -6.28, 6.28, 2.0).unwrap();

        // Sin function should sample appropriately
        assert!(points.len() >= 50);
    }

    #[test]
    fn test_numerical_derivative() {
        let ast = parse("x^2").unwrap();

        // Derivative of x^2 at x=2 should be 4
        let dy = numerical_derivative(&ast, 2.0, 0.0001).unwrap();
        assert!((dy - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_numerical_second_derivative() {
        let ast = parse("x^3").unwrap();

        // Second derivative of x^3 at x=1 should be 6
        let d2y = numerical_second_derivative(&ast, 1.0, 0.001).unwrap();
        assert!((d2y - 6.0).abs() < 0.1);
    }
}

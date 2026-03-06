//! Root finding algorithms

use crate::parser::AstNode;
use crate::evaluator::{Evaluator, EvalContext, EvalError};

/// Root finder configuration
#[derive(Debug, Clone)]
pub struct RootFinderConfig {
    /// Tolerance for convergence
    pub tolerance: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Number of initial samples for bracketing
    pub num_samples: usize,
}

impl Default for RootFinderConfig {
    fn default() -> Self {
        Self {
            tolerance: 1e-10,
            max_iterations: 100,
            num_samples: 100,
        }
    }
}

/// Root finder for expressions
pub struct RootFinder {
    config: RootFinderConfig,
    evaluator: Evaluator,
}

impl RootFinder {
    pub fn new() -> Self {
        Self {
            config: RootFinderConfig::default(),
            evaluator: Evaluator::new(),
        }
    }

    pub fn with_config(config: RootFinderConfig) -> Self {
        Self {
            config,
            evaluator: Evaluator::new(),
        }
    }

    /// Find all roots of f(x) = 0 in the given range
    pub fn find_roots(&self, ast: &AstNode, x_range: (f64, f64)) -> Result<Vec<f64>, EvalError> {
        let mut roots = Vec::new();
        let step = (x_range.1 - x_range.0) / self.config.num_samples as f64;

        let mut ctx = EvalContext::new();
        let mut prev_y: Option<f64> = None;
        let mut prev_x: Option<f64> = None;

        for i in 0..=self.config.num_samples {
            let x = x_range.0 + i as f64 * step;
            ctx.set("x", x);

            let y = match self.evaluator.eval(ast, &ctx) {
                Ok(v) if v.is_finite() => v,
                _ => {
                    prev_y = None;
                    prev_x = None;
                    continue;
                }
            };

            // Check for exact zero
            if y.abs() < self.config.tolerance {
                if roots.last().map_or(true, |&last: &f64| (x - last).abs() > step) {
                    roots.push(x);
                }
            }
            // Check for sign change
            else if let (Some(py), Some(px)) = (prev_y, prev_x) {
                if py * y < 0.0 {
                    // Refine root using Brent's method
                    match self.brent_method(ast, px, x) {
                        Ok(root) => {
                            if roots.last().map_or(true, |&last| (root - last).abs() > self.config.tolerance * 10.0) {
                                roots.push(root);
                            }
                        }
                        Err(_) => {}
                    }
                }
            }

            prev_y = Some(y);
            prev_x = Some(x);
        }

        Ok(roots)
    }

    /// Brent's method for root finding (combines bisection, secant, and inverse quadratic interpolation)
    fn brent_method(&self, ast: &AstNode, mut a: f64, mut b: f64) -> Result<f64, EvalError> {
        let mut ctx = EvalContext::new();

        ctx.set("x", a);
        let mut fa = self.evaluator.eval(ast, &ctx)?;

        ctx.set("x", b);
        let mut fb = self.evaluator.eval(ast, &ctx)?;

        if fa * fb > 0.0 {
            return Err(EvalError::InvalidOperation("No sign change in interval".to_string()));
        }

        // Ensure |f(a)| >= |f(b)|
        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }

        let mut c = a;
        let mut fc = fa;
        let mut d = b - a;
        let mut e = d;

        for _ in 0..self.config.max_iterations {
            if fb.abs() < self.config.tolerance {
                return Ok(b);
            }

            if (b - a).abs() < self.config.tolerance {
                return Ok(b);
            }

            let mut s;

            if (fa - fc).abs() > self.config.tolerance && (fb - fc).abs() > self.config.tolerance {
                // Inverse quadratic interpolation
                s = a * fb * fc / ((fa - fb) * (fa - fc))
                    + b * fa * fc / ((fb - fa) * (fb - fc))
                    + c * fa * fb / ((fc - fa) * (fc - fb));
            } else {
                // Secant method
                s = b - fb * (b - a) / (fb - fa);
            }

            // Conditions for accepting s
            let cond1 = (s - (3.0 * a + b) / 4.0) * (s - b) >= 0.0;
            let cond2 = e.abs() >= self.config.tolerance && (s - b).abs() >= e.abs() / 2.0;
            let cond3 = e.abs() < self.config.tolerance && (s - b).abs() >= (b - c).abs() / 2.0;

            if cond1 || cond2 || cond3 {
                // Bisection
                s = (a + b) / 2.0;
                e = b - a;
                d = e;
            } else {
                e = d;
                d = s - b;
            }

            c = b;
            fc = fb;

            ctx.set("x", s);
            let fs = self.evaluator.eval(ast, &ctx)?;

            if fa * fs < 0.0 {
                b = s;
                fb = fs;
            } else {
                a = s;
                fa = fs;
            }

            // Ensure |f(a)| >= |f(b)|
            if fa.abs() < fb.abs() {
                std::mem::swap(&mut a, &mut b);
                std::mem::swap(&mut fa, &mut fb);
            }
        }

        Ok(b)
    }

    /// Newton-Raphson method (requires derivative)
    pub fn newton_method(
        &self,
        ast: &AstNode,
        initial_guess: f64,
    ) -> Result<f64, EvalError> {
        let mut ctx = EvalContext::new();
        let mut x = initial_guess;
        let h = 1e-8;

        for _ in 0..self.config.max_iterations {
            ctx.set("x", x);
            let fx = self.evaluator.eval(ast, &ctx)?;

            if fx.abs() < self.config.tolerance {
                return Ok(x);
            }

            // Numerical derivative
            ctx.set("x", x + h);
            let fx_plus = self.evaluator.eval(ast, &ctx)?;
            ctx.set("x", x - h);
            let fx_minus = self.evaluator.eval(ast, &ctx)?;

            let fpx = (fx_plus - fx_minus) / (2.0 * h);

            if fpx.abs() < 1e-15 {
                // Derivative too small, method fails
                return Err(EvalError::InvalidOperation("Zero derivative".to_string()));
            }

            let x_new = x - fx / fpx;

            if (x_new - x).abs() < self.config.tolerance {
                return Ok(x_new);
            }

            x = x_new;
        }

        Ok(x)
    }
}

impl Default for RootFinder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to find roots
pub fn find_roots(ast: &AstNode, x_range: (f64, f64)) -> Result<Vec<f64>, EvalError> {
    RootFinder::new().find_roots(ast, x_range)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_find_roots_quadratic() {
        let ast = parse("x^2 - 4").unwrap();  // x^2 = 4 => x = ±2
        let roots = find_roots(&ast, (-5.0, 5.0)).unwrap();

        assert_eq!(roots.len(), 2);
        assert!(roots.iter().any(|&r| (r - 2.0).abs() < 0.001));
        assert!(roots.iter().any(|&r| (r + 2.0).abs() < 0.001));
    }

    #[test]
    fn test_find_roots_sin() {
        let ast = parse("sin(x)").unwrap();
        let roots = find_roots(&ast, (-3.5, 3.5)).unwrap();

        // Should find roots at -π, 0, π
        assert!(roots.len() >= 2);
        assert!(roots.iter().any(|&r| r.abs() < 0.001));
    }

    #[test]
    fn test_newton_method() {
        let ast = parse("x^2 - 2").unwrap();  // √2 ≈ 1.414
        let finder = RootFinder::new();

        let root = finder.newton_method(&ast, 1.5).unwrap();
        assert!((root - std::f64::consts::SQRT_2).abs() < 0.001);
    }

    #[test]
    fn test_no_roots() {
        let ast = parse("x^2 + 1").unwrap();  // No real roots
        let roots = find_roots(&ast, (-10.0, 10.0)).unwrap();

        assert_eq!(roots.len(), 0);
    }

    #[test]
    fn test_cubic_roots() {
        let ast = parse("x^3 - x").unwrap();  // x(x^2 - 1) = x(x-1)(x+1) => x = -1, 0, 1
        let roots = find_roots(&ast, (-2.0, 2.0)).unwrap();

        assert_eq!(roots.len(), 3);
        assert!(roots.iter().any(|&r| (r + 1.0).abs() < 0.001));
        assert!(roots.iter().any(|&r| r.abs() < 0.001));
        assert!(roots.iter().any(|&r| (r - 1.0).abs() < 0.001));
    }
}

//! Numerical differentiation

use crate::parser::AstNode;
use crate::evaluator::{Evaluator, EvalContext, EvalError};

/// Compute numerical derivative using central difference
///
/// f'(x) ≈ (f(x+h) - f(x-h)) / (2h)
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

    if !y_plus.is_finite() || !y_minus.is_finite() {
        return Err(EvalError::DomainError("Cannot compute derivative at this point".to_string()));
    }

    Ok((y_plus - y_minus) / (2.0 * h))
}

/// Compute numerical second derivative using central difference
///
/// f''(x) ≈ (f(x+h) - 2f(x) + f(x-h)) / h²
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

    if !y_plus.is_finite() || !y.is_finite() || !y_minus.is_finite() {
        return Err(EvalError::DomainError("Cannot compute second derivative at this point".to_string()));
    }

    Ok((y_plus - 2.0 * y + y_minus) / (h * h))
}

/// Compute numerical gradient for an implicit function F(x, y)
///
/// Returns (∂F/∂x, ∂F/∂y)
pub fn numerical_gradient(
    ast: &AstNode,
    x: f64,
    y: f64,
    h: f64,
) -> Result<(f64, f64), EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    // ∂F/∂x
    ctx.set("x", x + h);
    ctx.set("y", y);
    let fx_plus = evaluator.eval(ast, &ctx)?;

    ctx.set("x", x - h);
    let fx_minus = evaluator.eval(ast, &ctx)?;

    let df_dx = (fx_plus - fx_minus) / (2.0 * h);

    // ∂F/∂y
    ctx.set("x", x);
    ctx.set("y", y + h);
    let fy_plus = evaluator.eval(ast, &ctx)?;

    ctx.set("y", y - h);
    let fy_minus = evaluator.eval(ast, &ctx)?;

    let df_dy = (fy_plus - fy_minus) / (2.0 * h);

    if !df_dx.is_finite() || !df_dy.is_finite() {
        return Err(EvalError::DomainError("Cannot compute gradient at this point".to_string()));
    }

    Ok((df_dx, df_dy))
}

/// Compute higher-order derivative using Richardson extrapolation
pub fn richardson_derivative(
    ast: &AstNode,
    x: f64,
    h: f64,
    order: usize,
) -> Result<f64, EvalError> {
    if order == 0 {
        let evaluator = Evaluator::new();
        let mut ctx = EvalContext::new();
        ctx.set("x", x);
        return evaluator.eval(ast, &ctx);
    }

    // Use Richardson extrapolation for better accuracy
    let d1 = numerical_derivative(ast, x, h)?;
    let d2 = numerical_derivative(ast, x, h / 2.0)?;

    // Richardson extrapolation: (4*D(h/2) - D(h)) / 3
    let improved = (4.0 * d2 - d1) / 3.0;

    if order == 1 {
        Ok(improved)
    } else {
        // For higher orders, recursively apply
        let h_new = h / 2.0;
        let d1_new = richardson_derivative(ast, x, h_new, order - 1)?;
        let d2_new = richardson_derivative(ast, x, h_new / 2.0, order - 1)?;
        Ok((4.0 * d2_new - d1_new) / 3.0)
    }
}

/// Find local extrema (maxima and minima) of a function
pub fn find_extrema(
    ast: &AstNode,
    x_range: (f64, f64),
    tolerance: f64,
) -> Result<Vec<(f64, f64, bool)>, EvalError> {
    // Returns (x, y, is_maximum)
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let h = 1e-6;
    let num_samples = 200;
    let step = (x_range.1 - x_range.0) / num_samples as f64;

    let mut extrema = Vec::new();
    let mut prev_deriv: Option<f64> = None;
    let mut prev_x: Option<f64> = None;

    for i in 0..=num_samples {
        let x = x_range.0 + i as f64 * step;

        let deriv = match numerical_derivative(ast, x, h) {
            Ok(d) if d.is_finite() => d,
            _ => {
                prev_deriv = None;
                prev_x = None;
                continue;
            }
        };

        // Check for derivative near zero (exact extremum)
        if deriv.abs() < 1e-8 {
            ctx.set("x", x);
            let extremum_y = evaluator.eval(ast, &ctx)?;
            if extremum_y.is_finite() {
                let second_deriv = numerical_second_derivative(ast, x, h)?;
                let is_max = second_deriv < 0.0;
                extrema.push((x, extremum_y, is_max));
            }
            prev_deriv = Some(deriv);
            prev_x = Some(x);
            continue;
        }

        // Check for sign change in derivative
        if let (Some(pd), Some(px)) = (prev_deriv, prev_x) {
            if pd * deriv < 0.0 {
                // Sign change - potential extremum
                // Refine with bisection on derivative
                let extremum_x = refine_extremum(ast, px, x, 1e-8)?;

                ctx.set("x", extremum_x);
                let extremum_y = evaluator.eval(ast, &ctx)?;

                if extremum_y.is_finite() {
                    // Determine if max or min based on second derivative
                    let second_deriv = numerical_second_derivative(ast, extremum_x, h)?;
                    let is_max = second_deriv < 0.0;

                    extrema.push((extremum_x, extremum_y, is_max));
                }
            }
        }

        prev_deriv = Some(deriv);
        prev_x = Some(x);
    }

    Ok(extrema)
}

/// Refine extremum location using bisection on derivative
fn refine_extremum(
    ast: &AstNode,
    mut a: f64,
    mut b: f64,
    tolerance: f64,
) -> Result<f64, EvalError> {
    let h = 1e-8;
    let max_iterations = 50;

    for _ in 0..max_iterations {
        let mid = (a + b) / 2.0;

        if (b - a) < tolerance {
            return Ok(mid);
        }

        let da = numerical_derivative(ast, a, h)?;
        let dmid = numerical_derivative(ast, mid, h)?;

        if da * dmid < 0.0 {
            b = mid;
        } else {
            a = mid;
        }
    }

    Ok((a + b) / 2.0)
}

/// Find inflection points (where second derivative changes sign)
pub fn find_inflection_points(
    ast: &AstNode,
    x_range: (f64, f64),
    tolerance: f64,
) -> Result<Vec<(f64, f64)>, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    let h = 1e-5;
    let num_samples = 200;
    let step = (x_range.1 - x_range.0) / num_samples as f64;

    let mut inflections = Vec::new();
    let mut prev_d2: Option<f64> = None;
    let mut prev_x: Option<f64> = None;

    for i in 0..=num_samples {
        let x = x_range.0 + i as f64 * step;

        let d2 = match numerical_second_derivative(ast, x, h) {
            Ok(d) if d.is_finite() => d,
            _ => {
                prev_d2 = None;
                prev_x = None;
                continue;
            }
        };

        // Check for second derivative near zero (exact inflection)
        if d2.abs() < 1e-8 {
            ctx.set("x", x);
            let inflection_y = evaluator.eval(ast, &ctx)?;
            if inflection_y.is_finite() {
                inflections.push((x, inflection_y));
            }
            prev_d2 = Some(d2);
            prev_x = Some(x);
            continue;
        }

        if let (Some(pd2), Some(_px)) = (prev_d2, prev_x) {
            if pd2 * d2 < 0.0 {
                // Sign change in second derivative - refine
                ctx.set("x", x);
                let inflection_y = evaluator.eval(ast, &ctx)?;

                if inflection_y.is_finite() {
                    inflections.push((x, inflection_y));
                }
            }
        }

        prev_d2 = Some(d2);
        prev_x = Some(x);
    }

    Ok(inflections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_numerical_derivative() {
        let ast = parse("x^2").unwrap();

        // Derivative of x^2 at x=3 should be 6
        let deriv = numerical_derivative(&ast, 3.0, 0.0001).unwrap();
        assert!((deriv - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_second_derivative() {
        let ast = parse("x^3").unwrap();

        // Second derivative of x^3 at x=2 should be 12
        let d2 = numerical_second_derivative(&ast, 2.0, 0.001).unwrap();
        assert!((d2 - 12.0).abs() < 0.1);
    }

    #[test]
    fn test_gradient() {
        let ast = parse("x^2 + y^2").unwrap();

        // Gradient of x^2 + y^2 at (1, 1) should be (2, 2)
        let (dx, dy) = numerical_gradient(&ast, 1.0, 1.0, 0.0001).unwrap();
        assert!((dx - 2.0).abs() < 0.001);
        assert!((dy - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_find_extrema() {
        let ast = parse("x^2").unwrap();

        let extrema = find_extrema(&ast, (-2.0, 2.0), 1e-6).unwrap();

        // Should find minimum at x=0
        assert!(!extrema.is_empty());
        let (x, y, is_max) = extrema[0];
        assert!(x.abs() < 0.01);
        assert!(y.abs() < 0.01);
        assert!(!is_max);  // It's a minimum
    }

    #[test]
    fn test_find_inflection() {
        let ast = parse("x^3").unwrap();

        let inflections = find_inflection_points(&ast, (-2.0, 2.0), 1e-4).unwrap();

        // Should find inflection at x=0
        assert!(!inflections.is_empty());
        assert!(inflections[0].0.abs() < 0.1);
    }
}

//! Curve fitting algorithms

use crate::common::Point;
use nalgebra::{DMatrix, DVector};

/// Fitting model type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitModel {
    /// y = a₀ + a₁x + a₂x² + ... + aₙxⁿ
    Polynomial(usize),
    /// y = a + bx
    Linear,
    /// y = a * e^(bx)
    Exponential,
    /// y = a * ln(x) + b
    Logarithmic,
    /// y = a * x^b
    Power,
}

/// Result of curve fitting
#[derive(Debug, Clone)]
pub struct FitResult {
    /// Fitted coefficients
    pub coefficients: Vec<f64>,
    /// R² (coefficient of determination)
    pub r_squared: f64,
    /// Residual sum of squares
    pub residual_sum: f64,
    /// Model used
    pub model: FitModel,
}

impl FitResult {
    /// Evaluate the fitted function at x
    pub fn evaluate(&self, x: f64) -> f64 {
        match self.model {
            FitModel::Linear | FitModel::Polynomial(_) => {
                // y = a₀ + a₁x + a₂x² + ...
                self.coefficients.iter()
                    .enumerate()
                    .map(|(i, &c)| c * x.powi(i as i32))
                    .sum()
            }
            FitModel::Exponential => {
                // y = a * e^(bx), stored as [a, b]
                self.coefficients[0] * (self.coefficients[1] * x).exp()
            }
            FitModel::Logarithmic => {
                // y = a * ln(x) + b, stored as [a, b]
                if x > 0.0 {
                    self.coefficients[0] * x.ln() + self.coefficients[1]
                } else {
                    f64::NAN
                }
            }
            FitModel::Power => {
                // y = a * x^b, stored as [a, b]
                if x > 0.0 {
                    self.coefficients[0] * x.powf(self.coefficients[1])
                } else {
                    f64::NAN
                }
            }
        }
    }

    /// Get the expression as a string
    pub fn to_expression(&self) -> String {
        match self.model {
            FitModel::Linear => {
                format!("{:.4} + {:.4}*x", self.coefficients[0], self.coefficients[1])
            }
            FitModel::Polynomial(degree) => {
                let mut terms = Vec::new();
                for (i, &c) in self.coefficients.iter().enumerate() {
                    if c.abs() < 1e-10 {
                        continue;
                    }
                    match i {
                        0 => terms.push(format!("{:.4}", c)),
                        1 => terms.push(format!("{:.4}*x", c)),
                        _ => terms.push(format!("{:.4}*x^{}", c, i)),
                    }
                }
                if terms.is_empty() {
                    "0".to_string()
                } else {
                    terms.join(" + ")
                }
            }
            FitModel::Exponential => {
                format!("{:.4} * e^({:.4}*x)", self.coefficients[0], self.coefficients[1])
            }
            FitModel::Logarithmic => {
                format!("{:.4} * ln(x) + {:.4}", self.coefficients[0], self.coefficients[1])
            }
            FitModel::Power => {
                format!("{:.4} * x^{:.4}", self.coefficients[0], self.coefficients[1])
            }
        }
    }
}

/// Curve fitter
pub struct CurveFitter;

impl CurveFitter {
    pub fn new() -> Self {
        Self
    }

    /// Fit data points to a model
    pub fn fit(&self, points: &[Point], model: FitModel) -> Option<FitResult> {
        if points.len() < 2 {
            return None;
        }

        match model {
            FitModel::Linear => self.fit_linear(points),
            FitModel::Polynomial(degree) => self.fit_polynomial(points, degree),
            FitModel::Exponential => self.fit_exponential(points),
            FitModel::Logarithmic => self.fit_logarithmic(points),
            FitModel::Power => self.fit_power(points),
        }
    }

    /// Linear least squares fit: y = a + bx
    fn fit_linear(&self, points: &[Point]) -> Option<FitResult> {
        self.fit_polynomial(points, 1)
    }

    /// Polynomial least squares fit
    fn fit_polynomial(&self, points: &[Point], degree: usize) -> Option<FitResult> {
        let n = points.len();
        let m = degree + 1;

        if n < m {
            return None;
        }

        // Build Vandermonde matrix
        let mut a_data = Vec::with_capacity(n * m);
        let mut b_data = Vec::with_capacity(n);

        for p in points {
            let mut x_pow = 1.0;
            for _ in 0..m {
                a_data.push(x_pow);
                x_pow *= p.x;
            }
            b_data.push(p.y);
        }

        let a = DMatrix::from_row_slice(n, m, &a_data);
        let b = DVector::from_vec(b_data);

        // Solve normal equations: (AᵀA)x = Aᵀb
        let ata = a.transpose() * &a;
        let atb = a.transpose() * &b;

        // Use LU decomposition to solve
        let lu = ata.lu();
        let coefficients: Vec<f64> = lu.solve(&atb)?.iter().cloned().collect();

        // Compute R²
        let y_mean: f64 = points.iter().map(|p| p.y).sum::<f64>() / n as f64;
        let ss_tot: f64 = points.iter().map(|p| (p.y - y_mean).powi(2)).sum();
        let ss_res: f64 = points.iter()
            .map(|p| {
                let y_pred: f64 = coefficients.iter()
                    .enumerate()
                    .map(|(i, &c)| c * p.x.powi(i as i32))
                    .sum();
                (p.y - y_pred).powi(2)
            })
            .sum();

        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 1.0 };

        Some(FitResult {
            coefficients,
            r_squared,
            residual_sum: ss_res,
            model: if degree == 1 { FitModel::Linear } else { FitModel::Polynomial(degree) },
        })
    }

    /// Exponential fit: y = a * e^(bx)
    /// Using linearization: ln(y) = ln(a) + bx
    fn fit_exponential(&self, points: &[Point]) -> Option<FitResult> {
        // Filter positive y values for log transformation
        let log_points: Vec<Point> = points.iter()
            .filter(|p| p.y > 0.0)
            .map(|p| Point::new(p.x, p.y.ln()))
            .collect();

        if log_points.len() < 2 {
            return None;
        }

        // Fit linear to transformed data
        let linear_fit = self.fit_linear(&log_points)?;

        // Transform coefficients back
        let ln_a = linear_fit.coefficients[0];
        let b = linear_fit.coefficients[1];
        let a = ln_a.exp();

        // Recompute R² on original scale
        let y_mean: f64 = points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64;
        let ss_tot: f64 = points.iter().map(|p| (p.y - y_mean).powi(2)).sum();
        let ss_res: f64 = points.iter()
            .map(|p| {
                let y_pred = a * (b * p.x).exp();
                (p.y - y_pred).powi(2)
            })
            .sum();

        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 1.0 };

        Some(FitResult {
            coefficients: vec![a, b],
            r_squared,
            residual_sum: ss_res,
            model: FitModel::Exponential,
        })
    }

    /// Logarithmic fit: y = a * ln(x) + b
    fn fit_logarithmic(&self, points: &[Point]) -> Option<FitResult> {
        // Filter positive x values for log transformation
        let log_points: Vec<Point> = points.iter()
            .filter(|p| p.x > 0.0)
            .map(|p| Point::new(p.x.ln(), p.y))
            .collect();

        if log_points.len() < 2 {
            return None;
        }

        // Fit linear to transformed data: y = a * z + b where z = ln(x)
        let linear_fit = self.fit_linear(&log_points)?;

        let a = linear_fit.coefficients[1];  // Slope becomes coefficient of ln(x)
        let b = linear_fit.coefficients[0];  // Intercept

        // Recompute R² on original scale
        let valid_points: Vec<&Point> = points.iter().filter(|p| p.x > 0.0).collect();
        let y_mean: f64 = valid_points.iter().map(|p| p.y).sum::<f64>() / valid_points.len() as f64;
        let ss_tot: f64 = valid_points.iter().map(|p| (p.y - y_mean).powi(2)).sum();
        let ss_res: f64 = valid_points.iter()
            .map(|p| {
                let y_pred = a * p.x.ln() + b;
                (p.y - y_pred).powi(2)
            })
            .sum();

        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 1.0 };

        Some(FitResult {
            coefficients: vec![a, b],
            r_squared,
            residual_sum: ss_res,
            model: FitModel::Logarithmic,
        })
    }

    /// Power fit: y = a * x^b
    /// Using linearization: ln(y) = ln(a) + b*ln(x)
    fn fit_power(&self, points: &[Point]) -> Option<FitResult> {
        // Filter positive x and y values for log transformation
        let log_points: Vec<Point> = points.iter()
            .filter(|p| p.x > 0.0 && p.y > 0.0)
            .map(|p| Point::new(p.x.ln(), p.y.ln()))
            .collect();

        if log_points.len() < 2 {
            return None;
        }

        // Fit linear to transformed data
        let linear_fit = self.fit_linear(&log_points)?;

        // Transform coefficients back
        let ln_a = linear_fit.coefficients[0];
        let b = linear_fit.coefficients[1];
        let a = ln_a.exp();

        // Recompute R² on original scale
        let valid_points: Vec<&Point> = points.iter().filter(|p| p.x > 0.0 && p.y > 0.0).collect();
        let y_mean: f64 = valid_points.iter().map(|p| p.y).sum::<f64>() / valid_points.len() as f64;
        let ss_tot: f64 = valid_points.iter().map(|p| (p.y - y_mean).powi(2)).sum();
        let ss_res: f64 = valid_points.iter()
            .map(|p| {
                let y_pred = a * p.x.powf(b);
                (p.y - y_pred).powi(2)
            })
            .sum();

        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 1.0 };

        Some(FitResult {
            coefficients: vec![a, b],
            r_squared,
            residual_sum: ss_res,
            model: FitModel::Power,
        })
    }
}

impl Default for CurveFitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for polynomial fit
pub fn polynomial_fit(points: &[Point], degree: usize) -> Option<FitResult> {
    CurveFitter::new().fit(points, FitModel::Polynomial(degree))
}

/// Convenience function for linear fit
pub fn linear_fit(points: &[Point]) -> Option<FitResult> {
    CurveFitter::new().fit(points, FitModel::Linear)
}

/// Convenience function for exponential fit
pub fn exponential_fit(points: &[Point]) -> Option<FitResult> {
    CurveFitter::new().fit(points, FitModel::Exponential)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_fit() {
        // y = 2x + 1
        let points = vec![
            Point::new(0.0, 1.0),
            Point::new(1.0, 3.0),
            Point::new(2.0, 5.0),
            Point::new(3.0, 7.0),
        ];

        let result = linear_fit(&points).unwrap();

        assert!((result.coefficients[0] - 1.0).abs() < 0.01);  // intercept
        assert!((result.coefficients[1] - 2.0).abs() < 0.01);  // slope
        assert!(result.r_squared > 0.99);
    }

    #[test]
    fn test_quadratic_fit() {
        // y = x²
        let points = vec![
            Point::new(-2.0, 4.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 4.0),
        ];

        let result = polynomial_fit(&points, 2).unwrap();

        assert!(result.coefficients[0].abs() < 0.01);  // constant term ≈ 0
        assert!(result.coefficients[1].abs() < 0.01);  // linear term ≈ 0
        assert!((result.coefficients[2] - 1.0).abs() < 0.01);  // quadratic term ≈ 1
        assert!(result.r_squared > 0.99);
    }

    #[test]
    fn test_exponential_fit() {
        // y = 2 * e^(0.5x)
        let points: Vec<Point> = (0..5)
            .map(|i| {
                let x = i as f64;
                let y = 2.0 * (0.5 * x).exp();
                Point::new(x, y)
            })
            .collect();

        let result = exponential_fit(&points).unwrap();

        assert!((result.coefficients[0] - 2.0).abs() < 0.1);  // a ≈ 2
        assert!((result.coefficients[1] - 0.5).abs() < 0.1);  // b ≈ 0.5
        assert!(result.r_squared > 0.99);
    }

    #[test]
    fn test_fit_evaluate() {
        let points = vec![
            Point::new(0.0, 1.0),
            Point::new(1.0, 3.0),
            Point::new(2.0, 5.0),
        ];

        let result = linear_fit(&points).unwrap();

        assert!((result.evaluate(0.0) - 1.0).abs() < 0.01);
        assert!((result.evaluate(1.0) - 3.0).abs() < 0.01);
        assert!((result.evaluate(1.5) - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_fit_expression() {
        let points = vec![
            Point::new(0.0, 1.0),
            Point::new(1.0, 3.0),
        ];

        let result = linear_fit(&points).unwrap();
        let expr = result.to_expression();

        assert!(expr.contains("+"));
        assert!(expr.contains("x"));
    }
}

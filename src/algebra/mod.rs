//! Algebra Module
//!
//! Provides algebraic operations including:
//! - Intersection finding
//! - Root finding
//! - Numerical differentiation
//! - Curve fitting

mod intersection;
mod roots;
mod derivative;
mod fitting;

pub use intersection::{find_intersections, Intersection};
pub use roots::{find_roots, RootFinder, RootFinderConfig};
pub use derivative::{
    numerical_derivative,
    numerical_second_derivative,
    numerical_gradient,
};
pub use fitting::{
    CurveFitter,
    FitResult,
    FitModel,
    polynomial_fit,
    linear_fit,
    exponential_fit,
};

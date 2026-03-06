//! Algebra integration tests

use math_grapher::parser::parse;
use math_grapher::algebra::{find_intersections, find_roots, linear_fit, polynomial_fit};
use math_grapher::common::Point;

#[test]
fn test_find_line_intersection() {
    let f = parse("x").unwrap();         // y = x
    let g = parse("-x + 4").unwrap();    // y = -x + 4

    let intersections = find_intersections(&f, &g, (-5.0, 5.0), 1e-6).unwrap();

    assert_eq!(intersections.len(), 1);
    let p = &intersections[0];
    assert!((p.x - 2.0).abs() < 0.01);
    assert!((p.y - 2.0).abs() < 0.01);
}

#[test]
fn test_find_roots_polynomial() {
    let ast = parse("x^2 - 1").unwrap();  // (x-1)(x+1) = 0

    let roots = find_roots(&ast, (-5.0, 5.0)).unwrap();

    assert_eq!(roots.len(), 2);
    assert!(roots.iter().any(|&r| (r - 1.0).abs() < 0.01));
    assert!(roots.iter().any(|&r| (r + 1.0).abs() < 0.01));
}

#[test]
fn test_linear_regression() {
    // Perfect linear data: y = 3x + 2
    let points = vec![
        Point::new(0.0, 2.0),
        Point::new(1.0, 5.0),
        Point::new(2.0, 8.0),
        Point::new(3.0, 11.0),
    ];

    let result = linear_fit(&points).unwrap();

    assert!((result.coefficients[0] - 2.0).abs() < 0.01);  // intercept
    assert!((result.coefficients[1] - 3.0).abs() < 0.01);  // slope
    assert!(result.r_squared > 0.99);
}

#[test]
fn test_polynomial_regression() {
    // Quadratic data: y = x² - 2x + 1 = (x-1)²
    let points = vec![
        Point::new(-1.0, 4.0),
        Point::new(0.0, 1.0),
        Point::new(1.0, 0.0),
        Point::new(2.0, 1.0),
        Point::new(3.0, 4.0),
    ];

    let result = polynomial_fit(&points, 2).unwrap();

    assert!((result.coefficients[0] - 1.0).abs() < 0.01);   // constant
    assert!((result.coefficients[1] - (-2.0)).abs() < 0.01); // linear
    assert!((result.coefficients[2] - 1.0).abs() < 0.01);   // quadratic
    assert!(result.r_squared > 0.99);
}

#[test]
fn test_fit_evaluate() {
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(2.0, 4.0),
    ];

    let result = polynomial_fit(&points, 2).unwrap();

    // Should predict well for training data
    assert!((result.evaluate(0.0) - 0.0).abs() < 0.1);
    assert!((result.evaluate(1.0) - 1.0).abs() < 0.1);
    assert!((result.evaluate(2.0) - 4.0).abs() < 0.1);
}

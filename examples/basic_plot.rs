//! Basic plotting example
//!
//! Demonstrates how to use the math-grapher library programmatically.

use math_grapher::parser::{parse, parse_equation};
use math_grapher::evaluator::{evaluate_explicit, evaluate_implicit, Evaluator, EvalContext};
use math_grapher::algebra::{find_roots, find_intersections, linear_fit};
use math_grapher::common::{Rect, Point};

fn main() {
    println!("=== Math Grapher Examples ===\n");

    // Example 1: Parse and evaluate an expression
    println!("1. Parsing and Evaluation");
    let ast = parse("sin(x)^2 + cos(x)^2").unwrap();
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    ctx.set("x", std::f64::consts::PI / 4.0);
    let result = evaluator.eval(&ast, &ctx).unwrap();
    println!("   sin²(π/4) + cos²(π/4) = {:.6}", result);
    println!("   (Should equal 1.0)\n");

    // Example 2: Evaluate a function over a range
    println!("2. Function Sampling");
    let parabola = parse("x^2").unwrap();
    let bounds = Rect::new(-2.0, 2.0, 0.0, 4.0);
    let curve = evaluate_explicit(&parabola, &bounds, 5).unwrap();

    println!("   y = x² sampled at 5 points:");
    for point in &curve.points {
        println!("   ({:.2}, {:.2})", point.x, point.y);
    }
    println!();

    // Example 3: Equation parsing
    println!("3. Equation Parsing");
    let (expr_type, _ast) = parse_equation("y = x^2 + 1").unwrap();
    println!("   'y = x^2 + 1' is type: {:?}", expr_type);

    let (expr_type, _ast) = parse_equation("x^2 + y^2 = 4").unwrap();
    println!("   'x^2 + y^2 = 4' is type: {:?}", expr_type);
    println!();

    // Example 4: Root finding
    println!("4. Root Finding");
    let polynomial = parse("x^3 - 6*x^2 + 11*x - 6").unwrap();  // (x-1)(x-2)(x-3)
    let roots = find_roots(&polynomial, (-1.0, 5.0)).unwrap();

    println!("   Roots of x³ - 6x² + 11x - 6:");
    for root in &roots {
        println!("   x = {:.4}", root);
    }
    println!("   (Should be 1, 2, and 3)\n");

    // Example 5: Intersection finding
    println!("5. Finding Intersections");
    let line = parse("x + 1").unwrap();           // y = x + 1
    let parabola = parse("x^2").unwrap();         // y = x²

    let intersections = find_intersections(&line, &parabola, (-3.0, 3.0), 1e-8).unwrap();

    println!("   Intersections of y=x+1 and y=x²:");
    for point in &intersections {
        println!("   ({:.4}, {:.4})", point.x, point.y);
    }
    println!();

    // Example 6: Curve fitting
    println!("6. Curve Fitting");
    let data = vec![
        Point::new(0.0, 1.0),
        Point::new(1.0, 2.1),
        Point::new(2.0, 2.9),
        Point::new(3.0, 4.2),
        Point::new(4.0, 4.8),
    ];

    let fit = linear_fit(&data).unwrap();
    println!("   Linear fit to noisy data:");
    println!("   y = {:.4} + {:.4}x", fit.coefficients[0], fit.coefficients[1]);
    println!("   R² = {:.4}", fit.r_squared);
    println!();

    // Example 7: Implicit function (circle)
    println!("7. Implicit Function");
    let circle = parse("x^2 + y^2 - 4").unwrap();  // x² + y² = 4 (circle radius 2)
    let bounds = Rect::new(-3.0, 3.0, -3.0, 3.0);
    let segments = evaluate_implicit(&circle, &bounds, 50).unwrap();

    println!("   Circle x² + y² = 4:");
    println!("   Generated {} line segments for rendering", segments.len());
    println!();

    println!("=== Examples Complete ===");
}

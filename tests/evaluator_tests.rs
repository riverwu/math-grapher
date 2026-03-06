//! Evaluator integration tests

use math_grapher::parser::parse;
use math_grapher::evaluator::{Evaluator, EvalContext, evaluate_explicit};
use math_grapher::common::Rect;

#[test]
fn test_basic_evaluation() {
    let ast = parse("x^2 + 1").unwrap();
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    ctx.set("x", 3.0);
    let result = evaluator.eval(&ast, &ctx).unwrap();
    assert!((result - 10.0).abs() < 1e-10);
}

#[test]
fn test_trigonometric_identity() {
    let ast = parse("sin(x)^2 + cos(x)^2").unwrap();
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    for x in [0.0, 1.0, 2.0, 3.14159] {
        ctx.set("x", x);
        let result = evaluator.eval(&ast, &ctx).unwrap();
        assert!((result - 1.0).abs() < 1e-10, "sin²(x) + cos²(x) should equal 1");
    }
}

#[test]
fn test_constants() {
    let ast = parse("pi").unwrap();
    let evaluator = Evaluator::new();
    let ctx = EvalContext::new();

    let result = evaluator.eval(&ast, &ctx).unwrap();
    assert!((result - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_evaluate_explicit_curve() {
    let ast = parse("x^2").unwrap();
    let bounds = Rect::new(-2.0, 2.0, 0.0, 4.0);

    let curve = evaluate_explicit(&ast, &bounds, 5).unwrap();

    assert_eq!(curve.points.len(), 5);

    // Check endpoints
    let first = curve.points.first().unwrap();
    assert!((first.x - (-2.0)).abs() < 0.01);
    assert!((first.y - 4.0).abs() < 0.01);

    let last = curve.points.last().unwrap();
    assert!((last.x - 2.0).abs() < 0.01);
    assert!((last.y - 4.0).abs() < 0.01);
}

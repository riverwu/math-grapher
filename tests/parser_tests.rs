//! Parser integration tests

use math_grapher::parser::{parse, parse_equation, ExpressionType};

#[test]
fn test_basic_expressions() {
    assert!(parse("x").is_ok());
    assert!(parse("42").is_ok());
    assert!(parse("x + y").is_ok());
    assert!(parse("2 * x + 3").is_ok());
}

#[test]
fn test_function_expressions() {
    assert!(parse("sin(x)").is_ok());
    assert!(parse("cos(x) + sin(x)").is_ok());
    assert!(parse("exp(x)").is_ok());
    assert!(parse("ln(x)").is_ok());
    assert!(parse("sqrt(x)").is_ok());
}

#[test]
fn test_complex_expressions() {
    assert!(parse("sin(x)^2 + cos(x)^2").is_ok());
    assert!(parse("(x + 1) * (x - 1)").is_ok());
    assert!(parse("1 / (1 + x^2)").is_ok());
}

#[test]
fn test_equation_parsing() {
    let (expr_type, _) = parse_equation("y = x^2").unwrap();
    assert_eq!(expr_type, ExpressionType::Explicit);

    let (expr_type, _) = parse_equation("x^2 + y^2 = 1").unwrap();
    assert_eq!(expr_type, ExpressionType::Implicit);
}

#[test]
fn test_error_handling() {
    assert!(parse("sin(").is_err());
    assert!(parse("1 +").is_err());
    assert!(parse("unknown_func(x)").is_err());
}

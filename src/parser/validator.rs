//! Expression validation

use super::ast::{AstNode, ExpressionType};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Unknown variable: {0}")]
    UnknownVariable(String),

    #[error("Expression type mismatch: expected {expected:?}, found {found:?}")]
    TypeMismatch {
        expected: ExpressionType,
        found: ExpressionType,
    },

    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
}

/// Allowed variables for each expression type
pub fn allowed_variables(expr_type: ExpressionType) -> Vec<&'static str> {
    match expr_type {
        ExpressionType::Explicit => vec!["x"],
        ExpressionType::Implicit => vec!["x", "y"],
        ExpressionType::Parametric => vec!["t"],
        ExpressionType::Polar => vec!["theta", "θ", "t"],
        ExpressionType::Inequality => vec!["x", "y"],
    }
}

/// Mathematical constants that are allowed
pub fn allowed_constants() -> Vec<&'static str> {
    vec!["pi", "π", "e", "phi", "φ", "tau", "τ"]
}

/// Validate an expression AST
pub fn validate_expression(
    ast: &AstNode,
    expr_type: ExpressionType,
) -> Result<(), ValidationError> {
    let allowed_vars = allowed_variables(expr_type);
    let constants = allowed_constants();

    validate_variables(ast, &allowed_vars, &constants)
}

fn validate_variables(
    ast: &AstNode,
    allowed_vars: &[&str],
    constants: &[&str],
) -> Result<(), ValidationError> {
    match ast {
        AstNode::Number(_) => Ok(()),

        AstNode::Variable(name) => {
            if allowed_vars.contains(&name.as_str()) || constants.contains(&name.as_str()) {
                Ok(())
            } else {
                Err(ValidationError::UnknownVariable(name.clone()))
            }
        }

        AstNode::Constant(name) => {
            if constants.contains(&name.as_str()) {
                Ok(())
            } else {
                Err(ValidationError::UnknownVariable(name.clone()))
            }
        }

        AstNode::BinaryOp { left, right, .. } => {
            validate_variables(left, allowed_vars, constants)?;
            validate_variables(right, allowed_vars, constants)
        }

        AstNode::UnaryOp { operand, .. } => validate_variables(operand, allowed_vars, constants),

        AstNode::Function { args, .. } => {
            for arg in args {
                validate_variables(arg, allowed_vars, constants)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_valid_explicit_function() {
        let ast = parse("sin(x) + x^2").unwrap();
        assert!(validate_expression(&ast, ExpressionType::Explicit).is_ok());
    }

    #[test]
    fn test_invalid_variable() {
        let ast = parse("y + x").unwrap();
        assert!(validate_expression(&ast, ExpressionType::Explicit).is_err());
    }

    #[test]
    fn test_implicit_function() {
        let ast = parse("x^2 + y^2 - 1").unwrap();
        assert!(validate_expression(&ast, ExpressionType::Implicit).is_ok());
    }

    #[test]
    fn test_constants() {
        let ast = parse("sin(pi * x)").unwrap();
        assert!(validate_expression(&ast, ExpressionType::Explicit).is_ok());
    }
}

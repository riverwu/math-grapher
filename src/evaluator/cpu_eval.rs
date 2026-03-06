//! CPU-based expression evaluator

use crate::parser::{AstNode, BinaryOp, UnaryOp, Function};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Math domain error: {0}")]
    DomainError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Evaluation context holding variable values
#[derive(Debug, Clone, Default)]
pub struct EvalContext {
    variables: HashMap<String, f64>,
}

impl EvalContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            variables: HashMap::new(),
        };
        // Initialize mathematical constants
        ctx.set("pi", std::f64::consts::PI);
        ctx.set("π", std::f64::consts::PI);
        ctx.set("e", std::f64::consts::E);
        ctx.set("tau", std::f64::consts::TAU);
        ctx.set("τ", std::f64::consts::TAU);
        ctx.set("phi", 1.618033988749895); // Golden ratio
        ctx.set("φ", 1.618033988749895);
        ctx
    }

    pub fn set(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }

    pub fn clear_variables(&mut self) {
        // Keep constants, clear user variables
        let constants = ["pi", "π", "e", "tau", "τ", "phi", "φ"];
        self.variables.retain(|k, _| constants.contains(&k.as_str()));
    }
}

/// Expression evaluator
#[derive(Debug, Clone, Default)]
pub struct Evaluator;

impl Evaluator {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate an AST node with the given context
    pub fn eval(&self, node: &AstNode, ctx: &EvalContext) -> Result<f64, EvalError> {
        match node {
            AstNode::Number(n) => Ok(*n),

            AstNode::Variable(name) => ctx
                .get(name)
                .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

            AstNode::Constant(name) => ctx
                .get(name)
                .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

            AstNode::BinaryOp { op, left, right } => {
                let l = self.eval(left, ctx)?;
                let r = self.eval(right, ctx)?;
                self.eval_binary_op(*op, l, r)
            }

            AstNode::UnaryOp { op, operand } => {
                let v = self.eval(operand, ctx)?;
                self.eval_unary_op(*op, v)
            }

            AstNode::Function { func, args } => {
                let evaluated_args: Result<Vec<f64>, _> =
                    args.iter().map(|arg| self.eval(arg, ctx)).collect();
                self.eval_function(*func, &evaluated_args?)
            }
        }
    }

    fn eval_binary_op(&self, op: BinaryOp, left: f64, right: f64) -> Result<f64, EvalError> {
        match op {
            BinaryOp::Add => Ok(left + right),
            BinaryOp::Sub => Ok(left - right),
            BinaryOp::Mul => Ok(left * right),
            BinaryOp::Div => {
                if right == 0.0 {
                    // Return infinity for division by zero to handle asymptotes
                    Ok(if left >= 0.0 { f64::INFINITY } else { f64::NEG_INFINITY })
                } else {
                    Ok(left / right)
                }
            }
            BinaryOp::Pow => Ok(left.powf(right)),
            BinaryOp::Mod => {
                if right == 0.0 {
                    Err(EvalError::DivisionByZero)
                } else {
                    Ok(left % right)
                }
            }
        }
    }

    fn eval_unary_op(&self, op: UnaryOp, value: f64) -> Result<f64, EvalError> {
        match op {
            UnaryOp::Neg => Ok(-value),
            UnaryOp::Pos => Ok(value),
        }
    }

    fn eval_function(&self, func: Function, args: &[f64]) -> Result<f64, EvalError> {
        match func {
            // Trigonometric
            Function::Sin => Ok(args[0].sin()),
            Function::Cos => Ok(args[0].cos()),
            Function::Tan => Ok(args[0].tan()),
            Function::Asin => {
                if args[0] < -1.0 || args[0] > 1.0 {
                    Err(EvalError::DomainError("asin requires -1 <= x <= 1".to_string()))
                } else {
                    Ok(args[0].asin())
                }
            }
            Function::Acos => {
                if args[0] < -1.0 || args[0] > 1.0 {
                    Err(EvalError::DomainError("acos requires -1 <= x <= 1".to_string()))
                } else {
                    Ok(args[0].acos())
                }
            }
            Function::Atan => Ok(args[0].atan()),
            Function::Atan2 => Ok(args[0].atan2(args[1])),
            Function::Sinh => Ok(args[0].sinh()),
            Function::Cosh => Ok(args[0].cosh()),
            Function::Tanh => Ok(args[0].tanh()),

            // Exponential and logarithmic
            Function::Exp => Ok(args[0].exp()),
            Function::Ln => {
                if args[0] <= 0.0 {
                    Ok(f64::NEG_INFINITY)
                } else {
                    Ok(args[0].ln())
                }
            }
            Function::Log => {
                if args[0] <= 0.0 {
                    Ok(f64::NEG_INFINITY)
                } else {
                    Ok(args[0].log10())
                }
            }
            Function::Log2 => {
                if args[0] <= 0.0 {
                    Ok(f64::NEG_INFINITY)
                } else {
                    Ok(args[0].log2())
                }
            }

            // Power and roots
            Function::Sqrt => {
                if args[0] < 0.0 {
                    Ok(f64::NAN) // Return NaN for negative input
                } else {
                    Ok(args[0].sqrt())
                }
            }
            Function::Cbrt => Ok(args[0].cbrt()),
            Function::Pow => Ok(args[0].powf(args[1])),

            // Absolute value and sign
            Function::Abs => Ok(args[0].abs()),
            Function::Sign => Ok(args[0].signum()),

            // Rounding
            Function::Floor => Ok(args[0].floor()),
            Function::Ceil => Ok(args[0].ceil()),
            Function::Round => Ok(args[0].round()),

            // Min/Max
            Function::Min => Ok(args[0].min(args[1])),
            Function::Max => Ok(args[0].max(args[1])),

            // Special
            Function::Factorial => {
                if args[0] < 0.0 || args[0].fract() != 0.0 {
                    Err(EvalError::DomainError(
                        "factorial requires non-negative integer".to_string(),
                    ))
                } else {
                    Ok(factorial(args[0] as u64))
                }
            }
        }
    }
}

/// Compute factorial
fn factorial(n: u64) -> f64 {
    if n > 170 {
        return f64::INFINITY; // Overflow
    }
    (1..=n).fold(1.0, |acc, x| acc * x as f64)
}

/// Convenience function to evaluate an expression with a single variable
pub fn eval_at(ast: &AstNode, var: &str, value: f64) -> Result<f64, EvalError> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();
    ctx.set(var, value);
    evaluator.eval(ast, &ctx)
}

/// Evaluate expression at multiple x values (batch evaluation)
pub fn eval_batch(ast: &AstNode, x_values: &[f64]) -> Vec<Result<f64, EvalError>> {
    let evaluator = Evaluator::new();
    let mut ctx = EvalContext::new();

    x_values
        .iter()
        .map(|&x| {
            ctx.set("x", x);
            evaluator.eval(ast, &ctx)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_simple_number() {
        let ast = parse("42").unwrap();
        let evaluator = Evaluator::new();
        let ctx = EvalContext::new();
        assert_eq!(evaluator.eval(&ast, &ctx).unwrap(), 42.0);
    }

    #[test]
    fn test_variable() {
        let ast = parse("x").unwrap();
        let evaluator = Evaluator::new();
        let mut ctx = EvalContext::new();
        ctx.set("x", 5.0);
        assert_eq!(evaluator.eval(&ast, &ctx).unwrap(), 5.0);
    }

    #[test]
    fn test_arithmetic() {
        let ast = parse("2 + 3 * 4").unwrap();
        let evaluator = Evaluator::new();
        let ctx = EvalContext::new();
        assert_eq!(evaluator.eval(&ast, &ctx).unwrap(), 14.0);
    }

    #[test]
    fn test_power() {
        let ast = parse("2^3").unwrap();
        let evaluator = Evaluator::new();
        let ctx = EvalContext::new();
        assert_eq!(evaluator.eval(&ast, &ctx).unwrap(), 8.0);
    }

    #[test]
    fn test_functions() {
        let evaluator = Evaluator::new();
        let mut ctx = EvalContext::new();
        ctx.set("x", 0.0);

        let ast = parse("sin(x)").unwrap();
        assert!((evaluator.eval(&ast, &ctx).unwrap() - 0.0).abs() < 1e-10);

        let ast = parse("cos(x)").unwrap();
        assert!((evaluator.eval(&ast, &ctx).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_constants() {
        let ast = parse("pi").unwrap();
        let evaluator = Evaluator::new();
        let ctx = EvalContext::new();
        assert!((evaluator.eval(&ast, &ctx).unwrap() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_complex_expression() {
        let ast = parse("sin(x)^2 + cos(x)^2").unwrap();
        let evaluator = Evaluator::new();
        let mut ctx = EvalContext::new();

        // Should equal 1 for any x
        for x in [-1.0, 0.0, 1.0, 2.0, 3.14159] {
            ctx.set("x", x);
            let result = evaluator.eval(&ast, &ctx).unwrap();
            assert!((result - 1.0).abs() < 1e-10, "sin²(x) + cos²(x) should equal 1");
        }
    }

    #[test]
    fn test_eval_batch() {
        let ast = parse("x^2").unwrap();
        let results = eval_batch(&ast, &[-2.0, -1.0, 0.0, 1.0, 2.0]);

        let expected = [4.0, 1.0, 0.0, 1.0, 4.0];
        for (result, &exp) in results.iter().zip(expected.iter()) {
            assert!((result.as_ref().unwrap() - exp).abs() < 1e-10);
        }
    }
}

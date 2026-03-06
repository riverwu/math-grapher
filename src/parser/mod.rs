//! Expression Parser Module
//!
//! Parses mathematical expressions into an AST for evaluation.

mod ast;
mod lexer;
mod validator;

pub use ast::{AstNode, BinaryOp, UnaryOp, Function, ExpressionType, ComparisonOp, ParsedEquation};
pub use lexer::{Token, Lexer, LexerError};
pub use validator::{validate_expression, ValidationError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] LexerError),

    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },

    #[error("Unexpected end of expression")]
    UnexpectedEnd,

    #[error("Unknown function: {0}")]
    UnknownFunction(String),

    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    #[error("Invalid parametric equation: expected [x(t), y(t)]")]
    InvalidParametric,
}

/// Parser for mathematical expressions
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Ok(Self { tokens, pos: 0 })
    }

    /// Parse the expression and return the AST
    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        let result = self.parse_expression()?;
        if self.pos < self.tokens.len() {
            return Err(ParseError::InvalidExpression(format!(
                "Unexpected tokens after expression: {:?}",
                &self.tokens[self.pos..]
            )));
        }
        Ok(result)
    }

    /// Parse an equation (left = right) and determine expression type
    pub fn parse_equation(&mut self) -> Result<(ExpressionType, AstNode), ParseError> {
        let parsed = self.parse_full_equation()?;

        // Convert ParsedEquation to legacy (ExpressionType, AstNode) format
        match parsed {
            ParsedEquation::Explicit(ast) => Ok((ExpressionType::Explicit, ast)),
            ParsedEquation::Implicit(ast) => Ok((ExpressionType::Implicit, ast)),
            ParsedEquation::Polar(ast) => Ok((ExpressionType::Polar, ast)),
            ParsedEquation::Parametric { .. } => {
                // For now, return a dummy AST - the full equation is stored separately
                Err(ParseError::InvalidExpression(
                    "Use parse_full_equation for parametric equations".to_string()
                ))
            }
            ParsedEquation::Inequality { expr, .. } => Ok((ExpressionType::Inequality, expr)),
        }
    }

    /// Parse a full equation and return detailed ParsedEquation
    pub fn parse_full_equation(&mut self) -> Result<ParsedEquation, ParseError> {
        // Check for parametric equation: [x(t), y(t)]
        if self.peek() == Some(&Token::LBracket) {
            return self.parse_parametric();
        }

        let left = self.parse_expression()?;

        // Check for comparison operators (inequalities)
        if let Some(comp_op) = self.try_parse_comparison_op() {
            let right = self.parse_expression()?;

            // Convert y > f(x) to y - f(x) > 0, i.e., expr = left - right
            let expr = AstNode::BinaryOp {
                op: BinaryOp::Sub,
                left: Box::new(left),
                right: Box::new(right),
            };

            return Ok(ParsedEquation::Inequality { expr, op: comp_op });
        }

        // Check for equals sign
        if self.peek() == Some(&Token::Equals) {
            self.advance();
            let right = self.parse_expression()?;

            // Check for polar: r = f(theta)
            if let AstNode::Variable(name) = &left {
                if name == "r" {
                    return Ok(ParsedEquation::Polar(right));
                }
            }

            // Check for explicit: y = f(x)
            if let AstNode::Variable(name) = &left {
                if name == "y" {
                    return Ok(ParsedEquation::Explicit(right));
                }
            }

            // Otherwise it's implicit: F(x, y) = 0
            let combined = AstNode::BinaryOp {
                op: BinaryOp::Sub,
                left: Box::new(left),
                right: Box::new(right),
            };
            return Ok(ParsedEquation::Implicit(combined));
        }

        // No operator, assume it's an expression f(x)
        Ok(ParsedEquation::Explicit(left))
    }

    /// Try to parse a comparison operator, returning None if not found
    fn try_parse_comparison_op(&mut self) -> Option<ComparisonOp> {
        match self.peek() {
            Some(Token::Less) => {
                self.advance();
                Some(ComparisonOp::Less)
            }
            Some(Token::LessEq) => {
                self.advance();
                Some(ComparisonOp::LessEq)
            }
            Some(Token::Greater) => {
                self.advance();
                Some(ComparisonOp::Greater)
            }
            Some(Token::GreaterEq) => {
                self.advance();
                Some(ComparisonOp::GreaterEq)
            }
            _ => None,
        }
    }

    /// Parse a parametric equation: [x(t), y(t)]
    fn parse_parametric(&mut self) -> Result<ParsedEquation, ParseError> {
        // Consume '['
        self.advance();

        // Parse x(t)
        let x_ast = self.parse_expression()?;

        // Expect comma
        if self.peek() != Some(&Token::Comma) {
            return Err(ParseError::InvalidParametric);
        }
        self.advance();

        // Parse y(t)
        let y_ast = self.parse_expression()?;

        // Expect ']'
        if self.peek() != Some(&Token::RBracket) {
            return Err(ParseError::InvalidParametric);
        }
        self.advance();

        Ok(ParsedEquation::Parametric { x_ast, y_ast })
    }

    fn determine_expression_type(&self, left: &AstNode, _right: &AstNode) -> ExpressionType {
        // Check if left side is just "y"
        if let AstNode::Variable(name) = left {
            if name == "y" {
                return ExpressionType::Explicit;
            }
            if name == "r" {
                return ExpressionType::Polar;
            }
        }

        // Otherwise it's implicit
        ExpressionType::Implicit
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn parse_expression(&mut self) -> Result<AstNode, ParseError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_multiplicative()?;

        while let Some(token) = self.peek() {
            let op = match token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = AstNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_power()?;

        while let Some(token) = self.peek() {
            let op = match token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = AstNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<AstNode, ParseError> {
        let base = self.parse_unary()?;

        if let Some(Token::Caret) = self.peek() {
            self.advance();
            let exponent = self.parse_power()?; // Right associative
            Ok(AstNode::BinaryOp {
                op: BinaryOp::Pow,
                left: Box::new(base),
                right: Box::new(exponent),
            })
        } else {
            Ok(base)
        }
    }

    fn parse_unary(&mut self) -> Result<AstNode, ParseError> {
        if let Some(Token::Minus) = self.peek() {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(AstNode::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            });
        }

        if let Some(Token::Plus) = self.peek() {
            self.advance();
            return self.parse_unary();
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<AstNode, ParseError> {
        let token = self.advance().ok_or(ParseError::UnexpectedEnd)?.clone();

        match token {
            Token::Number(n) => Ok(AstNode::Number(n)),

            Token::Identifier(name) => {
                // Check if it's a function call
                if self.peek() == Some(&Token::LParen) {
                    self.parse_function_call(&name)
                } else {
                    // It's a variable
                    Ok(AstNode::Variable(name))
                }
            }

            Token::LParen => {
                let expr = self.parse_expression()?;
                if self.peek() != Some(&Token::RParen) {
                    return Err(ParseError::UnexpectedToken {
                        expected: ")".to_string(),
                        found: format!("{:?}", self.peek()),
                    });
                }
                self.advance();
                Ok(expr)
            }

            Token::Pipe => {
                // Absolute value |x|
                let expr = self.parse_expression()?;
                if self.peek() != Some(&Token::Pipe) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "|".to_string(),
                        found: format!("{:?}", self.peek()),
                    });
                }
                self.advance();
                Ok(AstNode::Function {
                    func: Function::Abs,
                    args: vec![expr],
                })
            }

            _ => Err(ParseError::UnexpectedToken {
                expected: "number, variable, or (".to_string(),
                found: format!("{:?}", token),
            }),
        }
    }

    fn parse_function_call(&mut self, name: &str) -> Result<AstNode, ParseError> {
        // Consume '('
        self.advance();

        let func = Function::from_name(name)
            .ok_or_else(|| ParseError::UnknownFunction(name.to_string()))?;

        let mut args = Vec::new();

        // Check for empty argument list
        if self.peek() != Some(&Token::RParen) {
            args.push(self.parse_expression()?);

            while self.peek() == Some(&Token::Comma) {
                self.advance();
                args.push(self.parse_expression()?);
            }
        }

        if self.peek() != Some(&Token::RParen) {
            return Err(ParseError::UnexpectedToken {
                expected: ")".to_string(),
                found: format!("{:?}", self.peek()),
            });
        }
        self.advance();

        // Validate argument count
        let expected_args = func.arg_count();
        if args.len() != expected_args {
            return Err(ParseError::InvalidExpression(format!(
                "Function {} expects {} arguments, got {}",
                name, expected_args, args.len()
            )));
        }

        Ok(AstNode::Function { func, args })
    }
}

/// Convenience function to parse an expression string
pub fn parse(input: &str) -> Result<AstNode, ParseError> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

/// Parse an equation and return expression type and AST
pub fn parse_equation(input: &str) -> Result<(ExpressionType, AstNode), ParseError> {
    let mut parser = Parser::new(input)?;
    parser.parse_equation()
}

/// Parse a full equation with detailed type information
pub fn parse_full_equation(input: &str) -> Result<ParsedEquation, ParseError> {
    let mut parser = Parser::new(input)?;
    parser.parse_full_equation()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_number() {
        let ast = parse("42").unwrap();
        assert_eq!(ast, AstNode::Number(42.0));
    }

    #[test]
    fn test_simple_variable() {
        let ast = parse("x").unwrap();
        assert_eq!(ast, AstNode::Variable("x".to_string()));
    }

    #[test]
    fn test_binary_operations() {
        let ast = parse("x + 2").unwrap();
        match ast {
            AstNode::BinaryOp { op: BinaryOp::Add, .. } => {}
            _ => panic!("Expected binary add operation"),
        }
    }

    #[test]
    fn test_function_call() {
        let ast = parse("sin(x)").unwrap();
        match ast {
            AstNode::Function { func: Function::Sin, args } => {
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_complex_expression() {
        let ast = parse("sin(x)^2 + cos(x)^2").unwrap();
        assert!(matches!(ast, AstNode::BinaryOp { op: BinaryOp::Add, .. }));
    }

    #[test]
    fn test_equation_parsing() {
        let (expr_type, _ast) = parse_equation("y = x^2").unwrap();
        assert_eq!(expr_type, ExpressionType::Explicit);
    }

    #[test]
    fn test_parametric_parsing() {
        let result = parse_full_equation("[cos(t), sin(t)]").unwrap();
        match result {
            ParsedEquation::Parametric { x_ast, y_ast } => {
                // x_ast should be cos(t)
                assert!(matches!(x_ast, AstNode::Function { func: Function::Cos, .. }));
                // y_ast should be sin(t)
                assert!(matches!(y_ast, AstNode::Function { func: Function::Sin, .. }));
            }
            _ => panic!("Expected parametric equation"),
        }
    }

    #[test]
    fn test_polar_parsing() {
        let result = parse_full_equation("r = sin(3*theta)").unwrap();
        match result {
            ParsedEquation::Polar(ast) => {
                // Should parse the right side
                assert!(matches!(ast, AstNode::Function { func: Function::Sin, .. }));
            }
            _ => panic!("Expected polar equation"),
        }
    }

    #[test]
    fn test_inequality_parsing() {
        let result = parse_full_equation("y > x^2").unwrap();
        match result {
            ParsedEquation::Inequality { op, .. } => {
                assert_eq!(op, ComparisonOp::Greater);
            }
            _ => panic!("Expected inequality"),
        }
    }

    #[test]
    fn test_inequality_less_than() {
        let result = parse_full_equation("y < sin(x)").unwrap();
        match result {
            ParsedEquation::Inequality { op, .. } => {
                assert_eq!(op, ComparisonOp::Less);
            }
            _ => panic!("Expected inequality"),
        }
    }

    #[test]
    fn test_inequality_less_eq() {
        let result = parse_full_equation("y <= x").unwrap();
        match result {
            ParsedEquation::Inequality { op, .. } => {
                assert_eq!(op, ComparisonOp::LessEq);
            }
            _ => panic!("Expected inequality"),
        }
    }

    #[test]
    fn test_inequality_greater_eq() {
        let result = parse_full_equation("y >= 0").unwrap();
        match result {
            ParsedEquation::Inequality { op, .. } => {
                assert_eq!(op, ComparisonOp::GreaterEq);
            }
            _ => panic!("Expected inequality"),
        }
    }
}

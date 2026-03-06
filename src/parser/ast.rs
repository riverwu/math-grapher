//! Abstract Syntax Tree definitions for mathematical expressions

use serde::{Deserialize, Serialize};

/// Type of mathematical expression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpressionType {
    /// Explicit function: y = f(x)
    Explicit,
    /// Implicit function: F(x, y) = 0
    Implicit,
    /// Parametric: (x(t), y(t))
    Parametric,
    /// Polar: r = f(θ)
    Polar,
    /// Inequality: y > f(x), etc.
    Inequality,
}

/// Comparison operators for inequalities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Less,      // <
    LessEq,    // <=
    Greater,   // >
    GreaterEq, // >=
}

impl ComparisonOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            ComparisonOp::Less => "<",
            ComparisonOp::LessEq => "<=",
            ComparisonOp::Greater => ">",
            ComparisonOp::GreaterEq => ">=",
        }
    }

    /// Returns true if the comparison is strict (< or >)
    pub fn is_strict(&self) -> bool {
        matches!(self, ComparisonOp::Less | ComparisonOp::Greater)
    }
}

/// Parsed equation with full type information
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedEquation {
    /// Explicit function: y = f(x)
    Explicit(AstNode),
    /// Implicit function: F(x, y) = 0
    Implicit(AstNode),
    /// Polar function: r = f(theta)
    Polar(AstNode),
    /// Parametric equations: [x(t), y(t)]
    Parametric { x_ast: AstNode, y_ast: AstNode },
    /// Inequality: y > f(x), y < f(x), etc.
    Inequality {
        /// The expression (left - right for comparison)
        expr: AstNode,
        /// The comparison operator
        op: ComparisonOp,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
}

impl BinaryOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Pow => "^",
            BinaryOp::Mod => "%",
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Pos,
}

/// Mathematical functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Function {
    // Trigonometric
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Sinh,
    Cosh,
    Tanh,

    // Exponential and logarithmic
    Exp,
    Ln,
    Log,    // log base 10
    Log2,   // log base 2

    // Power and roots
    Sqrt,
    Cbrt,
    Pow,

    // Absolute value and sign
    Abs,
    Sign,

    // Rounding
    Floor,
    Ceil,
    Round,

    // Min/Max
    Min,
    Max,

    // Special
    Factorial,
}

impl Function {
    /// Parse function name to Function enum
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        match lower.as_str() {
            "sin" => Some(Function::Sin),
            "cos" => Some(Function::Cos),
            "tan" => Some(Function::Tan),
            "asin" | "arcsin" => Some(Function::Asin),
            "acos" | "arccos" => Some(Function::Acos),
            "atan" | "arctan" => Some(Function::Atan),
            "atan2" => Some(Function::Atan2),
            "sinh" => Some(Function::Sinh),
            "cosh" => Some(Function::Cosh),
            "tanh" => Some(Function::Tanh),
            "exp" => Some(Function::Exp),
            "ln" => Some(Function::Ln),
            "log" => Some(Function::Log),
            "log2" => Some(Function::Log2),
            "sqrt" => Some(Function::Sqrt),
            "cbrt" => Some(Function::Cbrt),
            "pow" => Some(Function::Pow),
            "abs" => Some(Function::Abs),
            "sign" | "sgn" => Some(Function::Sign),
            "floor" => Some(Function::Floor),
            "ceil" | "ceiling" => Some(Function::Ceil),
            "round" => Some(Function::Round),
            "min" => Some(Function::Min),
            "max" => Some(Function::Max),
            "factorial" | "fact" => Some(Function::Factorial),
            _ => None,
        }
    }

    /// Get the expected number of arguments for this function
    pub fn arg_count(&self) -> usize {
        match self {
            Function::Atan2 | Function::Pow | Function::Min | Function::Max => 2,
            _ => 1,
        }
    }
}

/// AST Node representing a mathematical expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    /// A numeric literal
    Number(f64),

    /// A variable (x, y, t, etc.)
    Variable(String),

    /// A named constant (pi, e, etc.)
    Constant(String),

    /// Binary operation
    BinaryOp {
        op: BinaryOp,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<AstNode>,
    },

    /// Function call
    Function {
        func: Function,
        args: Vec<AstNode>,
    },
}

impl AstNode {
    /// Check if this AST contains a specific variable
    pub fn contains_variable(&self, var: &str) -> bool {
        match self {
            AstNode::Number(_) => false,
            AstNode::Variable(name) => name == var,
            AstNode::Constant(_) => false,
            AstNode::BinaryOp { left, right, .. } => {
                left.contains_variable(var) || right.contains_variable(var)
            }
            AstNode::UnaryOp { operand, .. } => operand.contains_variable(var),
            AstNode::Function { args, .. } => args.iter().any(|arg| arg.contains_variable(var)),
        }
    }

    /// Get all variables used in this AST
    pub fn get_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables(&mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    fn collect_variables(&self, vars: &mut Vec<String>) {
        match self {
            AstNode::Number(_) | AstNode::Constant(_) => {}
            AstNode::Variable(name) => vars.push(name.clone()),
            AstNode::BinaryOp { left, right, .. } => {
                left.collect_variables(vars);
                right.collect_variables(vars);
            }
            AstNode::UnaryOp { operand, .. } => operand.collect_variables(vars),
            AstNode::Function { args, .. } => {
                for arg in args {
                    arg.collect_variables(vars);
                }
            }
        }
    }

    /// Convert the AST back to a string representation
    pub fn to_string_expr(&self) -> String {
        match self {
            AstNode::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            AstNode::Variable(name) => name.clone(),
            AstNode::Constant(name) => name.clone(),
            AstNode::BinaryOp { op, left, right } => {
                format!("({} {} {})", left.to_string_expr(), op.symbol(), right.to_string_expr())
            }
            AstNode::UnaryOp { op, operand } => {
                match op {
                    UnaryOp::Neg => format!("(-{})", operand.to_string_expr()),
                    UnaryOp::Pos => operand.to_string_expr(),
                }
            }
            AstNode::Function { func, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_string_expr()).collect();
                format!("{:?}({})", func, args_str.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_from_name() {
        assert_eq!(Function::from_name("sin"), Some(Function::Sin));
        assert_eq!(Function::from_name("SIN"), Some(Function::Sin));
        assert_eq!(Function::from_name("arctan"), Some(Function::Atan));
        assert_eq!(Function::from_name("unknown"), None);
    }

    #[test]
    fn test_contains_variable() {
        let ast = AstNode::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(AstNode::Variable("x".to_string())),
            right: Box::new(AstNode::Number(1.0)),
        };
        assert!(ast.contains_variable("x"));
        assert!(!ast.contains_variable("y"));
    }

    #[test]
    fn test_get_variables() {
        let ast = AstNode::BinaryOp {
            op: BinaryOp::Mul,
            left: Box::new(AstNode::Variable("x".to_string())),
            right: Box::new(AstNode::Variable("y".to_string())),
        };
        let vars = ast.get_variables();
        assert_eq!(vars, vec!["x".to_string(), "y".to_string()]);
    }
}

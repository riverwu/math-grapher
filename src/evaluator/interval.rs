//! Interval arithmetic for robust evaluation

use crate::parser::{AstNode, BinaryOp, UnaryOp, Function};
use std::collections::HashMap;

/// An interval [lo, hi] representing a range of possible values
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

impl Interval {
    pub fn new(lo: f64, hi: f64) -> Self {
        debug_assert!(lo <= hi || lo.is_nan() || hi.is_nan());
        Self { lo, hi }
    }

    pub fn point(value: f64) -> Self {
        Self { lo: value, hi: value }
    }

    pub fn entire() -> Self {
        Self {
            lo: f64::NEG_INFINITY,
            hi: f64::INFINITY,
        }
    }

    pub fn empty() -> Self {
        Self {
            lo: f64::NAN,
            hi: f64::NAN,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lo.is_nan() || self.hi.is_nan()
    }

    pub fn contains(&self, value: f64) -> bool {
        !self.is_empty() && self.lo <= value && value <= self.hi
    }

    pub fn contains_zero(&self) -> bool {
        self.contains(0.0)
    }

    pub fn width(&self) -> f64 {
        if self.is_empty() {
            f64::NAN
        } else {
            self.hi - self.lo
        }
    }

    pub fn midpoint(&self) -> f64 {
        if self.is_empty() {
            f64::NAN
        } else {
            (self.lo + self.hi) / 2.0
        }
    }

    /// Check if interval crosses zero (useful for finding roots)
    pub fn straddles_zero(&self) -> bool {
        !self.is_empty() && self.lo <= 0.0 && self.hi >= 0.0
    }

    /// Union of two intervals
    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            other
        } else if other.is_empty() {
            self
        } else {
            Self::new(self.lo.min(other.lo), self.hi.max(other.hi))
        }
    }

    /// Intersection of two intervals
    pub fn intersection(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            Self::empty()
        } else {
            let lo = self.lo.max(other.lo);
            let hi = self.hi.min(other.hi);
            if lo > hi {
                Self::empty()
            } else {
                Self::new(lo, hi)
            }
        }
    }
}

// Arithmetic operations on intervals
impl std::ops::Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if self.is_empty() || rhs.is_empty() {
            Self::empty()
        } else {
            Self::new(self.lo + rhs.lo, self.hi + rhs.hi)
        }
    }
}

impl std::ops::Sub for Interval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        if self.is_empty() || rhs.is_empty() {
            Self::empty()
        } else {
            Self::new(self.lo - rhs.hi, self.hi - rhs.lo)
        }
    }
}

impl std::ops::Mul for Interval {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        if self.is_empty() || rhs.is_empty() {
            return Self::empty();
        }

        let products = [
            self.lo * rhs.lo,
            self.lo * rhs.hi,
            self.hi * rhs.lo,
            self.hi * rhs.hi,
        ];

        let lo = products.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = products.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Self::new(lo, hi)
    }
}

impl std::ops::Div for Interval {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        if self.is_empty() || rhs.is_empty() {
            return Self::empty();
        }

        // Handle division by interval containing zero
        if rhs.contains_zero() {
            return Self::entire(); // Returns [-inf, inf]
        }

        let quotients = [
            self.lo / rhs.lo,
            self.lo / rhs.hi,
            self.hi / rhs.lo,
            self.hi / rhs.hi,
        ];

        let lo = quotients.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = quotients.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Self::new(lo, hi)
    }
}

impl std::ops::Neg for Interval {
    type Output = Self;

    fn neg(self) -> Self {
        if self.is_empty() {
            Self::empty()
        } else {
            Self::new(-self.hi, -self.lo)
        }
    }
}

impl Interval {
    /// Power operation
    pub fn pow(self, exp: Self) -> Self {
        if self.is_empty() || exp.is_empty() {
            return Self::empty();
        }

        // For simplicity, evaluate at corners (this is not tight for all cases)
        let values = [
            self.lo.powf(exp.lo),
            self.lo.powf(exp.hi),
            self.hi.powf(exp.lo),
            self.hi.powf(exp.hi),
        ];

        let valid: Vec<f64> = values.iter().cloned().filter(|v| v.is_finite()).collect();

        if valid.is_empty() {
            Self::empty()
        } else {
            let lo = valid.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            // Special case: even power of interval containing negative numbers
            if exp.lo == exp.hi && exp.lo.fract() == 0.0 && exp.lo as i64 % 2 == 0 {
                if self.lo < 0.0 && self.hi > 0.0 {
                    return Self::new(0.0, hi);
                }
            }

            Self::new(lo, hi)
        }
    }

    /// Square root
    pub fn sqrt(self) -> Self {
        if self.is_empty() || self.hi < 0.0 {
            Self::empty()
        } else {
            Self::new(self.lo.max(0.0).sqrt(), self.hi.sqrt())
        }
    }

    /// Sine
    pub fn sin(self) -> Self {
        if self.is_empty() {
            return Self::empty();
        }

        // Conservative bound: could be tightened
        if self.width() >= 2.0 * std::f64::consts::PI {
            return Self::new(-1.0, 1.0);
        }

        // Sample several points for approximation
        let samples = 20;
        let step = self.width() / samples as f64;
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;

        for i in 0..=samples {
            let x = self.lo + i as f64 * step;
            let y = x.sin();
            lo = lo.min(y);
            hi = hi.max(y);
        }

        Self::new(lo.max(-1.0), hi.min(1.0))
    }

    /// Cosine
    pub fn cos(self) -> Self {
        if self.is_empty() {
            return Self::empty();
        }

        if self.width() >= 2.0 * std::f64::consts::PI {
            return Self::new(-1.0, 1.0);
        }

        let samples = 20;
        let step = self.width() / samples as f64;
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;

        for i in 0..=samples {
            let x = self.lo + i as f64 * step;
            let y = x.cos();
            lo = lo.min(y);
            hi = hi.max(y);
        }

        Self::new(lo.max(-1.0), hi.min(1.0))
    }

    /// Exponential
    pub fn exp(self) -> Self {
        if self.is_empty() {
            Self::empty()
        } else {
            Self::new(self.lo.exp(), self.hi.exp())
        }
    }

    /// Natural logarithm
    pub fn ln(self) -> Self {
        if self.is_empty() || self.hi <= 0.0 {
            Self::empty()
        } else if self.lo <= 0.0 {
            Self::new(f64::NEG_INFINITY, self.hi.ln())
        } else {
            Self::new(self.lo.ln(), self.hi.ln())
        }
    }

    /// Absolute value
    pub fn abs(self) -> Self {
        if self.is_empty() {
            Self::empty()
        } else if self.lo >= 0.0 {
            self
        } else if self.hi <= 0.0 {
            -self
        } else {
            Self::new(0.0, self.lo.abs().max(self.hi.abs()))
        }
    }
}

/// Interval evaluator for AST nodes
pub struct IntervalEvaluator {
    variables: HashMap<String, Interval>,
}

impl IntervalEvaluator {
    pub fn new() -> Self {
        let mut vars = HashMap::new();
        vars.insert("pi".to_string(), Interval::point(std::f64::consts::PI));
        vars.insert("π".to_string(), Interval::point(std::f64::consts::PI));
        vars.insert("e".to_string(), Interval::point(std::f64::consts::E));
        Self { variables: vars }
    }

    pub fn set(&mut self, name: &str, interval: Interval) {
        self.variables.insert(name.to_string(), interval);
    }

    pub fn eval(&self, node: &AstNode) -> Interval {
        match node {
            AstNode::Number(n) => Interval::point(*n),

            AstNode::Variable(name) | AstNode::Constant(name) => {
                self.variables.get(name).copied().unwrap_or(Interval::empty())
            }

            AstNode::BinaryOp { op, left, right } => {
                let l = self.eval(left);
                let r = self.eval(right);
                match op {
                    BinaryOp::Add => l + r,
                    BinaryOp::Sub => l - r,
                    BinaryOp::Mul => l * r,
                    BinaryOp::Div => l / r,
                    BinaryOp::Pow => l.pow(r),
                    BinaryOp::Mod => {
                        // Modulo is complex for intervals, use conservative bound
                        if r.contains_zero() {
                            Interval::entire()
                        } else {
                            Interval::new(-r.hi.abs(), r.hi.abs())
                        }
                    }
                }
            }

            AstNode::UnaryOp { op, operand } => {
                let v = self.eval(operand);
                match op {
                    UnaryOp::Neg => -v,
                    UnaryOp::Pos => v,
                }
            }

            AstNode::Function { func, args } => {
                let arg_intervals: Vec<Interval> = args.iter().map(|a| self.eval(a)).collect();
                self.eval_function(*func, &arg_intervals)
            }
        }
    }

    fn eval_function(&self, func: Function, args: &[Interval]) -> Interval {
        if args.iter().any(|a| a.is_empty()) {
            return Interval::empty();
        }

        match func {
            Function::Sin => args[0].sin(),
            Function::Cos => args[0].cos(),
            Function::Tan => {
                // Tan is complex due to asymptotes
                // Use conservative sampling
                let samples = 50;
                let step = args[0].width() / samples as f64;
                let mut lo = f64::INFINITY;
                let mut hi = f64::NEG_INFINITY;

                for i in 0..=samples {
                    let x = args[0].lo + i as f64 * step;
                    let y = x.tan();
                    if y.is_finite() {
                        lo = lo.min(y);
                        hi = hi.max(y);
                    }
                }

                if lo.is_infinite() || hi.is_infinite() {
                    Interval::entire()
                } else {
                    Interval::new(lo, hi)
                }
            }
            Function::Asin | Function::Acos => {
                let clamped = args[0].intersection(Interval::new(-1.0, 1.0));
                if clamped.is_empty() {
                    Interval::empty()
                } else if matches!(func, Function::Asin) {
                    Interval::new(clamped.lo.asin(), clamped.hi.asin())
                } else {
                    Interval::new(clamped.hi.acos(), clamped.lo.acos())
                }
            }
            Function::Atan => Interval::new(args[0].lo.atan(), args[0].hi.atan()),
            Function::Atan2 => {
                // Conservative bound
                Interval::new(-std::f64::consts::PI, std::f64::consts::PI)
            }
            Function::Sinh => {
                Interval::new(args[0].lo.sinh(), args[0].hi.sinh())
            }
            Function::Cosh => {
                let a = args[0];
                if a.lo >= 0.0 {
                    Interval::new(a.lo.cosh(), a.hi.cosh())
                } else if a.hi <= 0.0 {
                    Interval::new(a.hi.cosh(), a.lo.cosh())
                } else {
                    Interval::new(1.0, a.lo.cosh().max(a.hi.cosh()))
                }
            }
            Function::Tanh => Interval::new(args[0].lo.tanh(), args[0].hi.tanh()),
            Function::Exp => args[0].exp(),
            Function::Ln => args[0].ln(),
            Function::Log => {
                let ln_10 = 10.0_f64.ln();
                args[0].ln() / Interval::point(ln_10)
            }
            Function::Log2 => {
                let ln_2 = 2.0_f64.ln();
                args[0].ln() / Interval::point(ln_2)
            }
            Function::Sqrt => args[0].sqrt(),
            Function::Cbrt => {
                Interval::new(args[0].lo.cbrt(), args[0].hi.cbrt())
            }
            Function::Pow => args[0].pow(args[1]),
            Function::Abs => args[0].abs(),
            Function::Sign => {
                let a = args[0];
                if a.hi < 0.0 {
                    Interval::point(-1.0)
                } else if a.lo > 0.0 {
                    Interval::point(1.0)
                } else {
                    Interval::new(-1.0, 1.0)
                }
            }
            Function::Floor => Interval::new(args[0].lo.floor(), args[0].hi.floor()),
            Function::Ceil => Interval::new(args[0].lo.ceil(), args[0].hi.ceil()),
            Function::Round => Interval::new(args[0].lo.round(), args[0].hi.round()),
            Function::Min => {
                let (a, b) = (args[0], args[1]);
                Interval::new(a.lo.min(b.lo), a.hi.min(b.hi))
            }
            Function::Max => {
                let (a, b) = (args[0], args[1]);
                Interval::new(a.lo.max(b.lo), a.hi.max(b.hi))
            }
            Function::Factorial => {
                // Factorial is complex for intervals, return conservative bound
                Interval::new(1.0, f64::INFINITY)
            }
        }
    }
}

impl Default for IntervalEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_arithmetic() {
        let a = Interval::new(1.0, 2.0);
        let b = Interval::new(3.0, 4.0);

        let sum = a + b;
        assert_eq!(sum.lo, 4.0);
        assert_eq!(sum.hi, 6.0);

        let diff = a - b;
        assert_eq!(diff.lo, -3.0);
        assert_eq!(diff.hi, -1.0);

        let prod = a * b;
        assert_eq!(prod.lo, 3.0);
        assert_eq!(prod.hi, 8.0);
    }

    #[test]
    fn test_interval_contains_zero() {
        assert!(Interval::new(-1.0, 1.0).straddles_zero());
        assert!(!Interval::new(1.0, 2.0).straddles_zero());
        assert!(!Interval::new(-2.0, -1.0).straddles_zero());
    }

    #[test]
    fn test_interval_sin() {
        let i = Interval::new(0.0, std::f64::consts::PI);
        let sin_i = i.sin();
        assert!(sin_i.lo >= 0.0);
        assert!(sin_i.hi <= 1.0);
    }
}

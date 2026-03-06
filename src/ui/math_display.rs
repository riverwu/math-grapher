//! Math expression display formatting
//!
//! Converts mathematical expressions to Unicode symbols for better readability.

/// Formats mathematical expressions for display using Unicode symbols
pub struct MathFormatter;

impl MathFormatter {
    /// Convert an expression string to a prettier Unicode format
    pub fn format(expr: &str) -> String {
        let mut result = expr.to_string();

        // Superscript numbers (must be done in order from longest to shortest patterns)
        // Handle multi-digit exponents first
        result = Self::replace_exponents(&result);

        // Greek letters
        result = result.replace("theta", "\u{03B8}"); // θ
        result = result.replace("Theta", "\u{0398}"); // Θ
        result = result.replace("alpha", "\u{03B1}"); // α
        result = result.replace("Alpha", "\u{0391}"); // Α
        result = result.replace("beta", "\u{03B2}");  // β
        result = result.replace("Beta", "\u{0392}");  // Β
        result = result.replace("gamma", "\u{03B3}"); // γ
        result = result.replace("Gamma", "\u{0393}"); // Γ
        result = result.replace("delta", "\u{03B4}"); // δ
        result = result.replace("Delta", "\u{0394}"); // Δ
        result = result.replace("epsilon", "\u{03B5}"); // ε
        result = result.replace("lambda", "\u{03BB}"); // λ
        result = result.replace("sigma", "\u{03C3}"); // σ
        result = result.replace("omega", "\u{03C9}"); // ω
        result = result.replace("Omega", "\u{03A9}"); // Ω
        result = result.replace("phi", "\u{03C6}");   // φ
        result = result.replace("Phi", "\u{03A6}");   // Φ
        result = result.replace("psi", "\u{03C8}");   // ψ
        result = result.replace("tau", "\u{03C4}");   // τ
        result = result.replace("mu", "\u{03BC}");    // μ

        // Mathematical constants
        result = result.replace("pi", "\u{03C0}");    // π
        result = result.replace("PI", "\u{03C0}");    // π

        // Mathematical symbols and functions
        result = result.replace("sqrt", "\u{221A}");  // √
        result = result.replace("infinity", "\u{221E}"); // ∞ (must come before "inf")
        result = result.replace("inf", "\u{221E}");   // ∞

        // Comparison operators
        result = result.replace("<=", "\u{2264}");    // ≤
        result = result.replace(">=", "\u{2265}");    // ≥
        result = result.replace("!=", "\u{2260}");    // ≠

        // Multiplication (careful not to replace ** or *)
        // Only replace standalone * between operands
        result = Self::replace_multiplication(&result);

        result
    }

    /// Replace ^n patterns with superscript Unicode characters
    fn replace_exponents(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '^' {
                // Check if next char is a digit or minus sign
                if let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '-' || next == '(' {
                        // Handle parenthesized exponents like ^(2) or ^(-1)
                        if next == '(' {
                            chars.next(); // consume '('
                            let mut exp = String::new();
                            while let Some(&inner) = chars.peek() {
                                if inner == ')' {
                                    chars.next(); // consume ')'
                                    break;
                                }
                                exp.push(chars.next().unwrap());
                            }
                            result.push_str(&Self::to_superscript(&exp));
                        } else {
                            // Collect consecutive digits (and leading minus)
                            let mut exp = String::new();
                            if next == '-' {
                                exp.push(chars.next().unwrap());
                            }
                            while let Some(&digit) = chars.peek() {
                                if digit.is_ascii_digit() {
                                    exp.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            result.push_str(&Self::to_superscript(&exp));
                        }
                    } else {
                        // Not a number exponent, keep the ^
                        result.push(c);
                    }
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Convert a string of digits to superscript Unicode characters
    fn to_superscript(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '0' => '\u{2070}', // ⁰
                '1' => '\u{00B9}', // ¹
                '2' => '\u{00B2}', // ²
                '3' => '\u{00B3}', // ³
                '4' => '\u{2074}', // ⁴
                '5' => '\u{2075}', // ⁵
                '6' => '\u{2076}', // ⁶
                '7' => '\u{2077}', // ⁷
                '8' => '\u{2078}', // ⁸
                '9' => '\u{2079}', // ⁹
                '-' => '\u{207B}', // ⁻
                '+' => '\u{207A}', // ⁺
                _ => c,
            })
            .collect()
    }

    /// Replace * with · for multiplication, but be smart about it
    fn replace_multiplication(s: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();

        for (i, &c) in chars.iter().enumerate() {
            if c == '*' {
                // Check if it's part of ** (power operator)
                let prev_is_star = i > 0 && chars[i - 1] == '*';
                let next_is_star = i + 1 < chars.len() && chars[i + 1] == '*';

                if prev_is_star || next_is_star {
                    result.push(c);
                } else {
                    result.push('\u{00B7}'); // ·
                }
            } else {
                result.push(c);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superscript_conversion() {
        assert_eq!(MathFormatter::format("x^2"), "x²");
        assert_eq!(MathFormatter::format("x^3"), "x³");
        assert_eq!(MathFormatter::format("x^10"), "x¹⁰");
        assert_eq!(MathFormatter::format("x^(-1)"), "x⁻¹");
    }

    #[test]
    fn test_superscript_all_digits() {
        assert_eq!(MathFormatter::format("x^0"), "x⁰");
        assert_eq!(MathFormatter::format("x^1"), "x¹");
        assert_eq!(MathFormatter::format("x^4"), "x⁴");
        assert_eq!(MathFormatter::format("x^5"), "x⁵");
        assert_eq!(MathFormatter::format("x^6"), "x⁶");
        assert_eq!(MathFormatter::format("x^7"), "x⁷");
        assert_eq!(MathFormatter::format("x^8"), "x⁸");
        assert_eq!(MathFormatter::format("x^9"), "x⁹");
    }

    #[test]
    fn test_superscript_negative() {
        assert_eq!(MathFormatter::format("x^-1"), "x⁻¹");
        assert_eq!(MathFormatter::format("x^-2"), "x⁻²");
        assert_eq!(MathFormatter::format("x^(-3)"), "x⁻³");
    }

    #[test]
    fn test_superscript_multi_digit() {
        assert_eq!(MathFormatter::format("x^12"), "x¹²");
        assert_eq!(MathFormatter::format("x^123"), "x¹²³");
        assert_eq!(MathFormatter::format("2^10"), "2¹⁰");
    }

    #[test]
    fn test_superscript_with_variable() {
        // ^x should remain as ^x (not converted)
        assert_eq!(MathFormatter::format("e^x"), "e^x");
        assert_eq!(MathFormatter::format("2^n"), "2^n");
    }

    #[test]
    fn test_greek_letters() {
        assert_eq!(MathFormatter::format("sin(theta)"), "sin(θ)");
        assert_eq!(MathFormatter::format("alpha + beta"), "α + β");
    }

    #[test]
    fn test_greek_letters_all() {
        assert_eq!(MathFormatter::format("gamma"), "γ");
        assert_eq!(MathFormatter::format("delta"), "δ");
        assert_eq!(MathFormatter::format("epsilon"), "ε");
        assert_eq!(MathFormatter::format("lambda"), "λ");
        assert_eq!(MathFormatter::format("sigma"), "σ");
        assert_eq!(MathFormatter::format("omega"), "ω");
        assert_eq!(MathFormatter::format("phi"), "φ");
        assert_eq!(MathFormatter::format("psi"), "ψ");
        assert_eq!(MathFormatter::format("tau"), "τ");
        assert_eq!(MathFormatter::format("mu"), "μ");
    }

    #[test]
    fn test_greek_letters_uppercase() {
        assert_eq!(MathFormatter::format("Theta"), "Θ");
        assert_eq!(MathFormatter::format("Alpha"), "Α");
        assert_eq!(MathFormatter::format("Beta"), "Β");
        assert_eq!(MathFormatter::format("Gamma"), "Γ");
        assert_eq!(MathFormatter::format("Delta"), "Δ");
        assert_eq!(MathFormatter::format("Omega"), "Ω");
        assert_eq!(MathFormatter::format("Phi"), "Φ");
    }

    #[test]
    fn test_math_constants() {
        assert_eq!(MathFormatter::format("2*pi"), "2·π");
        assert_eq!(MathFormatter::format("sqrt(x)"), "√(x)");
    }

    #[test]
    fn test_math_constants_pi() {
        assert_eq!(MathFormatter::format("pi"), "π");
        assert_eq!(MathFormatter::format("PI"), "π");
        assert_eq!(MathFormatter::format("2*pi*r"), "2·π·r");
    }

    #[test]
    fn test_infinity() {
        assert_eq!(MathFormatter::format("inf"), "∞");
        assert_eq!(MathFormatter::format("infinity"), "∞");
    }

    #[test]
    fn test_comparison_operators() {
        assert_eq!(MathFormatter::format("y <= x^2"), "y ≤ x²");
        assert_eq!(MathFormatter::format("y >= x"), "y ≥ x");
    }

    #[test]
    fn test_comparison_not_equal() {
        assert_eq!(MathFormatter::format("x != 0"), "x ≠ 0");
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(MathFormatter::format("2*x"), "2·x");
        assert_eq!(MathFormatter::format("a*sin(x)"), "a·sin(x)");
    }

    #[test]
    fn test_multiplication_preserves_double_star() {
        // ** should remain as ** (for power in some contexts)
        assert_eq!(MathFormatter::format("x**2"), "x**2");
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(
            MathFormatter::format("y = a*x^2 + b*x + c"),
            "y = a·x² + b·x + c"
        );
    }

    #[test]
    fn test_polar_expression() {
        assert_eq!(
            MathFormatter::format("r = sin(3*theta)"),
            "r = sin(3·θ)"
        );
    }

    #[test]
    fn test_inequality_expression() {
        assert_eq!(
            MathFormatter::format("y <= x^2 + 1"),
            "y ≤ x² + 1"
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(MathFormatter::format(""), "");
    }

    #[test]
    fn test_no_changes_needed() {
        assert_eq!(MathFormatter::format("x + y"), "x + y");
        assert_eq!(MathFormatter::format("sin(x)"), "sin(x)");
    }

    #[test]
    fn test_to_superscript_directly() {
        assert_eq!(MathFormatter::to_superscript("0123456789"), "⁰¹²³⁴⁵⁶⁷⁸⁹");
        assert_eq!(MathFormatter::to_superscript("-1"), "⁻¹");
        assert_eq!(MathFormatter::to_superscript("+2"), "⁺²");
    }
}

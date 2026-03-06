//! LaTeX input conversion
//!
//! Converts LaTeX-style mathematical notation to the internal format.

use std::collections::HashMap;

/// LaTeX to internal format converter
pub struct LatexConverter {
    /// Function name mappings
    functions: HashMap<&'static str, &'static str>,
    /// Constant mappings
    constants: HashMap<&'static str, &'static str>,
}

impl LatexConverter {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        // Trigonometric
        functions.insert("sin", "sin");
        functions.insert("cos", "cos");
        functions.insert("tan", "tan");
        functions.insert("arcsin", "asin");
        functions.insert("arccos", "acos");
        functions.insert("arctan", "atan");
        functions.insert("sinh", "sinh");
        functions.insert("cosh", "cosh");
        functions.insert("tanh", "tanh");
        // Exponential/Log
        functions.insert("exp", "exp");
        functions.insert("ln", "ln");
        functions.insert("log", "log");
        // Other
        functions.insert("sqrt", "sqrt");
        functions.insert("abs", "abs");
        functions.insert("floor", "floor");
        functions.insert("ceil", "ceil");
        functions.insert("min", "min");
        functions.insert("max", "max");

        let mut constants = HashMap::new();
        constants.insert("pi", "pi");
        constants.insert("Pi", "pi");
        constants.insert("theta", "theta");
        constants.insert("Theta", "theta");
        constants.insert("alpha", "a");
        constants.insert("beta", "b");
        constants.insert("gamma", "c");
        constants.insert("lambda", "l");
        constants.insert("mu", "m");
        constants.insert("sigma", "s");
        constants.insert("omega", "w");
        constants.insert("phi", "phi");
        constants.insert("psi", "psi");
        constants.insert("infty", "inf");

        Self { functions, constants }
    }

    /// Convert LaTeX input to internal format
    pub fn convert(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Handle \frac{a}{b} -> (a)/(b)
        result = self.convert_fractions(&result);

        // Handle \sqrt{x} and \sqrt[n]{x}
        result = self.convert_sqrt(&result);

        // Handle function calls: \sin x -> sin(x), \sin{x} -> sin(x)
        result = self.convert_functions(&result);

        // Handle constants: \pi -> pi, \theta -> theta
        result = self.convert_constants(&result);

        // Handle superscripts: x^{2} -> x^(2)
        result = self.convert_superscripts(&result);

        // Handle subscripts (remove them for now): x_{1} -> x1
        result = self.convert_subscripts(&result);

        // Handle operators
        result = result.replace("\\cdot", "*");
        result = result.replace("\\times", "*");
        result = result.replace("\\div", "/");
        result = result.replace("\\pm", "+");
        result = result.replace("\\mp", "-");
        result = result.replace("\\le", "<=");
        result = result.replace("\\leq", "<=");
        result = result.replace("\\ge", ">=");
        result = result.replace("\\geq", ">=");
        result = result.replace("\\ne", "!=");
        result = result.replace("\\neq", "!=");

        // Handle parentheses variations
        result = result.replace("\\left(", "(");
        result = result.replace("\\right)", ")");
        result = result.replace("\\left[", "[");
        result = result.replace("\\right]", "]");
        result = result.replace("\\left\\{", "(");
        result = result.replace("\\right\\}", ")");
        result = result.replace("\\{", "(");
        result = result.replace("\\}", ")");

        // Remove remaining backslashes for unrecognized commands
        result = result.replace("\\,", " ");
        result = result.replace("\\ ", " ");

        // Clean up whitespace
        result = result.split_whitespace().collect::<Vec<_>>().join(" ");

        result
    }

    fn convert_fractions(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Find \frac{...}{...} patterns
        while let Some(pos) = result.find("\\frac") {
            if let Some((num, den, end)) = self.extract_frac_args(&result[pos..]) {
                let replacement = format!("({})/({})", num, den);
                result = format!("{}{}{}", &result[..pos], replacement, &result[pos + end..]);
            } else {
                break;
            }
        }

        result
    }

    fn extract_frac_args(&self, s: &str) -> Option<(String, String, usize)> {
        // Skip \frac
        let s = &s[5..];

        // Extract first argument
        let (num, consumed1) = self.extract_brace_content(s)?;
        let s = &s[consumed1..];

        // Extract second argument
        let (den, consumed2) = self.extract_brace_content(s)?;

        Some((num, den, 5 + consumed1 + consumed2))
    }

    fn extract_brace_content(&self, s: &str) -> Option<(String, usize)> {
        let s = s.trim_start();
        let skip_ws = s.len() - s.trim_start().len();

        if !s.starts_with('{') {
            // Single character or word without braces
            let content: String = s.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '\\')
                .collect();
            if content.is_empty() {
                return None;
            }
            return Some((content.clone(), skip_ws + content.len()));
        }

        let mut depth = 0;
        let mut end = 0;

        for (i, c) in s.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth != 0 || end == 0 {
            return None;
        }

        let content = s[1..end].to_string();
        Some((content, skip_ws + end + 1))
    }

    fn convert_sqrt(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Handle \sqrt[n]{x} (nth root)
        while let Some(pos) = result.find("\\sqrt[") {
            if let Some((n, content, end)) = self.extract_sqrt_n_args(&result[pos..]) {
                let replacement = format!("(({})^(1/{}))", content, n);
                result = format!("{}{}{}", &result[..pos], replacement, &result[pos + end..]);
            } else {
                break;
            }
        }

        // Handle \sqrt{x}
        while let Some(pos) = result.find("\\sqrt") {
            if result[pos..].starts_with("\\sqrt[") {
                // Already handled above, skip
                break;
            }
            if let Some((content, end)) = self.extract_brace_content(&result[pos + 5..]) {
                let replacement = format!("sqrt({})", content);
                result = format!("{}{}{}", &result[..pos], replacement, &result[pos + 5 + end..]);
            } else {
                break;
            }
        }

        result
    }

    fn extract_sqrt_n_args(&self, s: &str) -> Option<(String, String, usize)> {
        // Skip \sqrt[
        let s = &s[6..];

        // Find closing bracket
        let bracket_end = s.find(']')?;
        let n = s[..bracket_end].to_string();
        let s = &s[bracket_end + 1..];

        // Extract content
        let (content, consumed) = self.extract_brace_content(s)?;

        Some((n, content, 6 + bracket_end + 1 + consumed))
    }

    fn convert_functions(&self, input: &str) -> String {
        let mut result = input.to_string();

        for (latex_name, internal_name) in &self.functions {
            let latex_cmd = format!("\\{}", latex_name);

            // Handle \func{arg}
            while let Some(pos) = result.find(&latex_cmd) {
                let after = &result[pos + latex_cmd.len()..];

                if let Some((arg, consumed)) = self.extract_brace_content(after) {
                    let replacement = format!("{}({})", internal_name, arg);
                    result = format!("{}{}{}", &result[..pos], replacement, &result[pos + latex_cmd.len() + consumed..]);
                } else if let Some(first_char) = after.trim_start().chars().next() {
                    if first_char.is_alphanumeric() {
                        // \sin x -> sin(x)
                        let ws = after.len() - after.trim_start().len();
                        let arg: String = after.trim_start().chars()
                            .take_while(|c| c.is_alphanumeric())
                            .collect();
                        let replacement = format!("{}({})", internal_name, arg);
                        result = format!("{}{}{}", &result[..pos], replacement, &result[pos + latex_cmd.len() + ws + arg.len()..]);
                    } else {
                        // Just replace the command name
                        result = format!("{}{}{}", &result[..pos], internal_name, &result[pos + latex_cmd.len()..]);
                    }
                } else {
                    result = format!("{}{}{}", &result[..pos], internal_name, &result[pos + latex_cmd.len()..]);
                }
            }
        }

        result
    }

    fn convert_constants(&self, input: &str) -> String {
        let mut result = input.to_string();

        for (latex_name, internal_name) in &self.constants {
            let latex_cmd = format!("\\{}", latex_name);
            result = result.replace(&latex_cmd, internal_name);
        }

        result
    }

    fn convert_superscripts(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Handle x^{...} -> x^(...)
        while let Some(pos) = result.find("^{") {
            if let Some(end) = self.find_matching_brace(&result[pos + 1..]) {
                let content = &result[pos + 2..pos + 1 + end];
                let replacement = format!("^({})", content);
                result = format!("{}{}{}", &result[..pos], replacement, &result[pos + 2 + end..]);
            } else {
                break;
            }
        }

        result
    }

    fn convert_subscripts(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Handle x_{...} -> x...
        while let Some(pos) = result.find("_{") {
            if let Some(end) = self.find_matching_brace(&result[pos + 1..]) {
                let content = &result[pos + 2..pos + 1 + end];
                // Simply append the subscript content (for variable names like x_1)
                result = format!("{}{}{}", &result[..pos], content, &result[pos + 2 + end..]);
            } else {
                break;
            }
        }

        // Handle x_n (single character subscript)
        let mut chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '_' && i + 1 < chars.len() && chars[i + 1].is_alphanumeric() && (i + 2 >= chars.len() || chars[i + 2] != '{') {
                // Remove the underscore, keep the subscript character
                chars.remove(i);
            } else {
                i += 1;
            }
        }
        result = chars.into_iter().collect();

        result
    }

    fn find_matching_brace(&self, s: &str) -> Option<usize> {
        if !s.starts_with('{') {
            return None;
        }

        let mut depth = 0;
        for (i, c) in s.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }
}

impl Default for LatexConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if input contains LaTeX-style commands
pub fn is_latex_input(input: &str) -> bool {
    input.contains('\\') || input.contains('^') && input.contains('{')
}

/// Convert LaTeX input to internal format
pub fn convert_latex(input: &str) -> String {
    let converter = LatexConverter::new();
    converter.convert(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        assert_eq!(convert_latex("\\sin{x}"), "sin(x)");
        assert_eq!(convert_latex("\\cos{x}"), "cos(x)");
        assert_eq!(convert_latex("\\tan{x}"), "tan(x)");
    }

    #[test]
    fn test_function_without_braces() {
        // When followed by a space and identifier, wraps it in parens
        let result = convert_latex("\\sin x");
        assert!(result.contains("sin") && result.contains("x"));
    }

    #[test]
    fn test_fraction() {
        assert_eq!(convert_latex("\\frac{1}{x}"), "(1)/(x)");
        assert_eq!(convert_latex("\\frac{x+1}{x-1}"), "(x+1)/(x-1)");
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(convert_latex("\\sqrt{x}"), "sqrt(x)");
        assert_eq!(convert_latex("\\sqrt{x+1}"), "sqrt(x+1)");
    }

    #[test]
    fn test_nth_root() {
        assert_eq!(convert_latex("\\sqrt[3]{x}"), "((x)^(1/3))");
    }

    #[test]
    fn test_superscript() {
        assert_eq!(convert_latex("x^{2}"), "x^(2)");
        assert_eq!(convert_latex("x^{n+1}"), "x^(n+1)");
    }

    #[test]
    fn test_constants() {
        assert_eq!(convert_latex("\\pi"), "pi");
        assert_eq!(convert_latex("\\theta"), "theta");
    }

    #[test]
    fn test_operators() {
        assert_eq!(convert_latex("2 \\cdot x"), "2 * x");
        assert_eq!(convert_latex("2 \\times x"), "2 * x");
    }

    #[test]
    fn test_complex_expression() {
        let result = convert_latex("y = \\frac{\\sin{x}}{x}");
        assert!(result.contains("sin(x)"));
        assert!(result.contains("/"));
    }

    #[test]
    fn test_is_latex() {
        assert!(is_latex_input("\\sin{x}"));
        assert!(is_latex_input("x^{2}"));
        assert!(!is_latex_input("sin(x)"));
        assert!(!is_latex_input("x^2"));
    }
}

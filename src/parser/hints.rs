//! Error hints and suggestions for parse errors

/// Common mathematical functions
const KNOWN_FUNCTIONS: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "atan2",
    "sinh", "cosh", "tanh",
    "exp", "ln", "log", "log2",
    "sqrt", "cbrt", "pow",
    "abs", "sign", "sgn",
    "floor", "ceil", "ceiling", "round",
    "min", "max", "factorial", "fact",
    "arcsin", "arccos", "arctan",
];

/// Common constants
const KNOWN_CONSTANTS: &[&str] = &["pi", "e", "tau"];

/// Standard variables
const STANDARD_VARS: &[&str] = &["x", "y", "t", "theta", "r"];

/// Error hint with suggestion
#[derive(Debug, Clone)]
pub struct ErrorHint {
    /// The original error message
    pub message: String,
    /// Suggestions for fixing the error
    pub suggestions: Vec<String>,
    /// Did-you-mean suggestions (typo corrections)
    pub did_you_mean: Vec<String>,
}

impl ErrorHint {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            suggestions: Vec::new(),
            did_you_mean: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_did_you_mean(mut self, suggestion: impl Into<String>) -> Self {
        self.did_you_mean.push(suggestion.into());
        self
    }

    /// Format the hint for display
    pub fn format(&self) -> String {
        let mut result = self.message.clone();

        if !self.did_you_mean.is_empty() {
            result.push_str("\nDid you mean: ");
            result.push_str(&self.did_you_mean.join(", "));
            result.push('?');
        }

        if !self.suggestions.is_empty() {
            result.push_str("\nSuggestions:");
            for s in &self.suggestions {
                result.push_str(&format!("\n  - {}", s));
            }
        }

        result
    }
}

/// Generate error hints based on the error and input
pub fn generate_hint(error: &str, input: &str) -> ErrorHint {
    let error_lower = error.to_lowercase();
    let input_lower = input.to_lowercase();

    // Check for unknown function errors
    if error_lower.contains("unknown function") {
        if let Some(func_name) = extract_unknown_function(&error_lower) {
            let mut hint = ErrorHint::new(error);

            // Find similar functions
            let similar = find_similar_functions(&func_name);
            for s in similar {
                hint = hint.with_did_you_mean(s);
            }

            // Add general suggestions
            hint = hint.with_suggestion("Available functions: sin, cos, tan, exp, ln, sqrt, abs, etc.");
            return hint;
        }
    }

    // Check for missing operator between terms
    if error_lower.contains("expected") && error_lower.contains("operator") {
        return ErrorHint::new(error)
            .with_suggestion("Add * for multiplication: 2x -> 2*x")
            .with_suggestion("Add parentheses for function calls: sinx -> sin(x)");
    }

    // Check for unbalanced parentheses
    let open_parens = input.matches('(').count();
    let close_parens = input.matches(')').count();
    if open_parens != close_parens {
        return ErrorHint::new(error)
            .with_suggestion(format!(
                "Unbalanced parentheses: {} opening, {} closing",
                open_parens, close_parens
            ));
    }

    // Check for unbalanced brackets
    let open_brackets = input.matches('[').count();
    let close_brackets = input.matches(']').count();
    if open_brackets != close_brackets {
        return ErrorHint::new(error)
            .with_suggestion(format!(
                "Unbalanced brackets: {} opening, {} closing",
                open_brackets, close_brackets
            ));
    }

    // Check for common syntax issues
    if input.contains("**") {
        return ErrorHint::new(error)
            .with_suggestion("Use ^ for exponentiation, not **: x**2 -> x^2");
    }

    if input.contains("//") {
        return ErrorHint::new(error)
            .with_suggestion("Use / for division, not //");
    }

    // Check for assignment instead of equality
    if input.contains("==") {
        return ErrorHint::new(error)
            .with_suggestion("Use = for equality, not ==: y == x -> y = x");
    }

    // Check for implicit multiplication issues
    if let Some(issue) = check_implicit_multiplication(input) {
        return ErrorHint::new(error)
            .with_suggestion(issue);
    }

    // Check for theta variants
    if input_lower.contains("θ") {
        return ErrorHint::new(error)
            .with_suggestion("Use 'theta' instead of θ symbol");
    }

    // Check for π variants
    if input.contains('π') {
        return ErrorHint::new(error)
            .with_suggestion("Use 'pi' instead of π symbol");
    }

    // Default hint
    ErrorHint::new(error)
        .with_suggestion("Check for missing operators or parentheses")
        .with_suggestion("Function calls need parentheses: sin(x), not sinx")
}

/// Extract the unknown function name from an error message
fn extract_unknown_function(error: &str) -> Option<String> {
    // Try to find a word after "unknown function"
    if let Some(pos) = error.find("unknown function") {
        let rest = &error[pos + 16..];
        let func_name: String = rest
            .chars()
            .skip_while(|c| !c.is_alphabetic())
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !func_name.is_empty() {
            return Some(func_name);
        }
    }
    None
}

/// Find similar function names using Levenshtein distance
fn find_similar_functions(name: &str) -> Vec<String> {
    let name_lower = name.to_lowercase();
    let mut similar: Vec<(String, usize)> = Vec::new();

    for &func in KNOWN_FUNCTIONS {
        let distance = levenshtein_distance(&name_lower, func);
        if distance <= 2 {
            similar.push((func.to_string(), distance));
        }
    }

    // Sort by distance
    similar.sort_by_key(|(_, d)| *d);

    // Return up to 3 suggestions
    similar.into_iter()
        .take(3)
        .map(|(s, _)| s)
        .collect()
}

/// Simple Levenshtein distance implementation
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 { return n; }
    if n == 0 { return m; }

    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

/// Check for implicit multiplication issues
fn check_implicit_multiplication(input: &str) -> Option<String> {
    let chars: Vec<char> = input.chars().collect();

    for i in 0..chars.len().saturating_sub(1) {
        let c1 = chars[i];
        let c2 = chars[i + 1];

        // Number followed by letter (2x -> 2*x)
        if c1.is_ascii_digit() && c2.is_ascii_alphabetic() {
            return Some(format!("Add * for implicit multiplication: {}{}  -> {}*{}", c1, c2, c1, c2));
        }

        // Letter followed by parenthesis without being a function
        if c2 == '(' && c1.is_ascii_alphabetic() {
            // Check if it's not a known function
            let mut func_start = i;
            while func_start > 0 && chars[func_start - 1].is_ascii_alphabetic() {
                func_start -= 1;
            }
            let func_name: String = chars[func_start..=i].iter().collect();
            let func_lower = func_name.to_lowercase();

            if !KNOWN_FUNCTIONS.contains(&func_lower.as_str())
                && !KNOWN_CONSTANTS.contains(&func_lower.as_str())
                && !STANDARD_VARS.contains(&func_lower.as_str()) {
                return Some(format!("Unknown function '{}'. Did you mean to multiply?", func_name));
            }
        }
    }

    None
}

/// Validate expression and return hints
pub fn validate_with_hints(input: &str) -> Option<ErrorHint> {
    // Quick validation checks

    // Empty input
    if input.trim().is_empty() {
        return Some(ErrorHint::new("Empty expression")
            .with_suggestion("Try: y = sin(x)")
            .with_suggestion("Try: x^2 + y^2 = 4"));
    }

    // Check for common mistakes
    if input.contains(")(") {
        return Some(ErrorHint::new("Adjacent parentheses need an operator")
            .with_suggestion("Add * between: )(  -> )*("));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("sin", "sin"), 0);
        assert_eq!(levenshtein_distance("sin", "son"), 1);
        assert_eq!(levenshtein_distance("sin", "cos"), 3); // All three chars are different
        assert_eq!(levenshtein_distance("sin", "sinh"), 1);
        assert_eq!(levenshtein_distance("sqrt", "sqr"), 1);
    }

    #[test]
    fn test_find_similar_functions() {
        let similar = find_similar_functions("sn");
        assert!(similar.contains(&"sin".to_string()));

        let similar = find_similar_functions("sqr");
        assert!(similar.contains(&"sqrt".to_string()));
    }

    #[test]
    fn test_generate_hint_unknown_function() {
        let hint = generate_hint("Unknown function: sn", "y = sn(x)");
        assert!(!hint.did_you_mean.is_empty());
    }

    #[test]
    fn test_generate_hint_unbalanced_parens() {
        let hint = generate_hint("Parse error", "sin(x");
        assert!(hint.suggestions.iter().any(|s| s.contains("parentheses")));
    }

    #[test]
    fn test_generate_hint_power_operator() {
        let hint = generate_hint("Parse error", "x**2");
        assert!(hint.suggestions.iter().any(|s| s.contains("^")));
    }

    #[test]
    fn test_validate_empty() {
        let hint = validate_with_hints("");
        assert!(hint.is_some());
    }

    #[test]
    fn test_check_implicit_multiplication() {
        let issue = check_implicit_multiplication("2x");
        assert!(issue.is_some());
        assert!(issue.unwrap().contains("*"));
    }
}

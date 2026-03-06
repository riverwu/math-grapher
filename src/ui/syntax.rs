//! Syntax highlighting for mathematical expressions

use eframe::egui;

/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Numbers (integers and floats)
    Number,
    /// Standard variables (x, y, t, theta, r)
    Variable,
    /// Parameters (a, b, c, etc.)
    Parameter,
    /// Functions (sin, cos, sqrt, etc.)
    Function,
    /// Operators (+, -, *, /, ^, %)
    Operator,
    /// Comparison operators (<, >, <=, >=, =)
    Comparison,
    /// Brackets and parentheses
    Bracket,
    /// Constants (pi, e)
    Constant,
    /// Commas and other punctuation
    Punctuation,
    /// Whitespace
    Whitespace,
    /// Unknown/invalid characters
    Unknown,
}

/// A token with position information
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Known mathematical functions
const FUNCTIONS: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "atan2",
    "sinh", "cosh", "tanh",
    "exp", "ln", "log", "log2",
    "sqrt", "cbrt", "pow",
    "abs", "sign", "sgn",
    "floor", "ceil", "ceiling", "round",
    "min", "max", "factorial", "fact",
    "arcsin", "arccos", "arctan",
];

/// Known constants
const CONSTANTS: &[&str] = &["pi", "e", "tau"];

/// Standard variables
const STANDARD_VARS: &[&str] = &["x", "y", "t", "theta", "r"];

/// Simple lexer for syntax highlighting
pub struct SyntaxHighlighter;

impl SyntaxHighlighter {
    /// Tokenize an expression for syntax highlighting
    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let start = i;
            let c = chars[i];

            if c.is_whitespace() {
                // Whitespace
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                tokens.push(Token {
                    token_type: TokenType::Whitespace,
                    start,
                    end: i,
                    text: input[start..i].to_string(),
                });
            } else if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
                // Number
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                // Handle scientific notation
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    i += 1;
                    if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                        i += 1;
                    }
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                tokens.push(Token {
                    token_type: TokenType::Number,
                    start,
                    end: i,
                    text: input[start..i].to_string(),
                });
            } else if c.is_ascii_alphabetic() || c == '_' {
                // Identifier (function, variable, constant, or parameter)
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text = &input[start..i];
                let lower = text.to_lowercase();

                let token_type = if FUNCTIONS.contains(&lower.as_str()) {
                    TokenType::Function
                } else if CONSTANTS.contains(&lower.as_str()) {
                    TokenType::Constant
                } else if STANDARD_VARS.contains(&lower.as_str()) {
                    TokenType::Variable
                } else {
                    TokenType::Parameter
                };

                tokens.push(Token {
                    token_type,
                    start,
                    end: i,
                    text: text.to_string(),
                });
            } else if c == '+' || c == '-' || c == '*' || c == '/' || c == '^' || c == '%' {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    start,
                    end: i + 1,
                    text: c.to_string(),
                });
                i += 1;
            } else if c == '<' || c == '>' || c == '=' {
                // Handle <= and >=
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        token_type: TokenType::Comparison,
                        start,
                        end: i + 2,
                        text: input[start..i + 2].to_string(),
                    });
                    i += 2;
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Comparison,
                        start,
                        end: i + 1,
                        text: c.to_string(),
                    });
                    i += 1;
                }
            } else if c == '(' || c == ')' || c == '[' || c == ']' {
                tokens.push(Token {
                    token_type: TokenType::Bracket,
                    start,
                    end: i + 1,
                    text: c.to_string(),
                });
                i += 1;
            } else if c == ',' {
                tokens.push(Token {
                    token_type: TokenType::Punctuation,
                    start,
                    end: i + 1,
                    text: c.to_string(),
                });
                i += 1;
            } else {
                // Unknown character
                tokens.push(Token {
                    token_type: TokenType::Unknown,
                    start,
                    end: i + 1,
                    text: c.to_string(),
                });
                i += 1;
            }
        }

        tokens
    }

    /// Get the color for a token type (light theme)
    pub fn color_for_type(token_type: TokenType) -> egui::Color32 {
        match token_type {
            TokenType::Number => egui::Color32::from_rgb(0, 100, 200),      // Blue
            TokenType::Variable => egui::Color32::from_rgb(0, 130, 0),      // Green
            TokenType::Parameter => egui::Color32::from_rgb(180, 80, 0),    // Orange
            TokenType::Function => egui::Color32::from_rgb(128, 0, 128),    // Purple
            TokenType::Operator => egui::Color32::from_rgb(80, 80, 80),     // Dark gray
            TokenType::Comparison => egui::Color32::from_rgb(200, 0, 0),    // Red
            TokenType::Bracket => egui::Color32::from_rgb(100, 100, 100),   // Gray
            TokenType::Constant => egui::Color32::from_rgb(0, 100, 150),    // Teal
            TokenType::Punctuation => egui::Color32::from_rgb(80, 80, 80),  // Dark gray
            TokenType::Whitespace => egui::Color32::TRANSPARENT,
            TokenType::Unknown => egui::Color32::from_rgb(200, 50, 50),     // Red-ish
        }
    }

    /// Create a LayoutJob with syntax highlighting for an expression
    pub fn highlight(text: &str) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        let tokens = Self::tokenize(text);

        for token in tokens {
            let format = egui::TextFormat {
                color: Self::color_for_type(token.token_type),
                ..Default::default()
            };
            job.append(&token.text, 0.0, format);
        }

        // If empty, add a placeholder format
        if text.is_empty() {
            job.append("", 0.0, egui::TextFormat::default());
        }

        job
    }
}

/// Widget for syntax-highlighted text input
pub fn syntax_highlighted_text_edit(ui: &mut egui::Ui, text: &mut String) -> egui::Response {
    let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
        let mut layout_job = SyntaxHighlighter::highlight(text);
        layout_job.wrap.max_width = f32::INFINITY;
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.add(
        egui::TextEdit::singleline(text)
            .layouter(&mut layouter)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_number() {
        let tokens = SyntaxHighlighter::tokenize("123.45");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].text, "123.45");
    }

    #[test]
    fn test_tokenize_scientific_notation() {
        let tokens = SyntaxHighlighter::tokenize("1.5e-10");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Number);
    }

    #[test]
    fn test_tokenize_function() {
        let tokens = SyntaxHighlighter::tokenize("sin(x)");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_type, TokenType::Function);
        assert_eq!(tokens[1].token_type, TokenType::Bracket);
        assert_eq!(tokens[2].token_type, TokenType::Variable);
        assert_eq!(tokens[3].token_type, TokenType::Bracket);
    }

    #[test]
    fn test_tokenize_expression() {
        let tokens = SyntaxHighlighter::tokenize("y = a*sin(x)");
        // Filter out whitespace for easier testing
        let non_ws: Vec<_> = tokens.iter().filter(|t| t.token_type != TokenType::Whitespace).collect();
        assert_eq!(non_ws.len(), 8);
        assert_eq!(non_ws[0].token_type, TokenType::Variable);  // y
        assert_eq!(non_ws[1].token_type, TokenType::Comparison); // =
        assert_eq!(non_ws[2].token_type, TokenType::Parameter);  // a
        assert_eq!(non_ws[3].token_type, TokenType::Operator);   // *
        assert_eq!(non_ws[4].token_type, TokenType::Function);   // sin
    }

    #[test]
    fn test_tokenize_comparison() {
        let tokens = SyntaxHighlighter::tokenize("y >= x");
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Comparison && t.text == ">="));
    }

    #[test]
    fn test_tokenize_constants() {
        let tokens = SyntaxHighlighter::tokenize("2*pi");
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Constant && t.text == "pi"));
    }

    #[test]
    fn test_tokenize_parametric() {
        let tokens = SyntaxHighlighter::tokenize("[cos(t), sin(t)]");
        let brackets: Vec<_> = tokens.iter().filter(|t| t.token_type == TokenType::Bracket).collect();
        assert_eq!(brackets.len(), 6); // [ ( ) , ( ) ]
    }
}

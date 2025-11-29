use crate::{LispExpr, Location, SourceExpr};

#[derive(Debug, Clone)]
struct Token {
    text: String,
    line: usize,
    column: usize,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    file: String,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self::new_with_file(input, "<input>".to_string())
    }

    pub fn new_with_file(input: &str, file: String) -> Self {
        let tokens = tokenize(input);
        Parser { tokens, pos: 0, file }
    }

    pub fn parse_all(&mut self) -> Result<Vec<SourceExpr>, String> {
        let mut exprs = Vec::new();
        while self.pos < self.tokens.len() {
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }

    fn parse_expr(&mut self) -> Result<SourceExpr, String> {
        if self.pos >= self.tokens.len() {
            return Err("Unexpected end of input".to_string());
        }

        let token = &self.tokens[self.pos];
        let location = Location::new(token.line, token.column, self.file.clone());

        if token.text == "(" {
            self.parse_list()
        } else if token.text == ")" {
            Err("Unexpected closing parenthesis".to_string())
        } else if token.text == "'" {
            // Quote syntax: 'expr → (quote expr)
            self.pos += 1;
            let quoted_expr = self.parse_expr()?;
            let quote_symbol = SourceExpr::new(LispExpr::Symbol("quote".to_string()), location.clone());
            let quoted_list = vec![quote_symbol, quoted_expr];
            Ok(SourceExpr::new(LispExpr::List(quoted_list), location))
        } else if token.text == "`" {
            // Quasiquote syntax: `expr → (quasiquote expr)
            self.pos += 1;
            let quoted_expr = self.parse_expr()?;
            let quasiquote_symbol = SourceExpr::new(LispExpr::Symbol("quasiquote".to_string()), location.clone());
            let quoted_list = vec![quasiquote_symbol, quoted_expr];
            Ok(SourceExpr::new(LispExpr::List(quoted_list), location))
        } else if token.text == "," {
            // Unquote or unquote-splicing
            self.pos += 1;

            // Check if next token is @ for unquote-splicing
            if self.pos < self.tokens.len() && self.tokens[self.pos].text == "@" {
                // Unquote-splicing: ,@expr → (unquote-splicing expr)
                self.pos += 1;
                let unquoted_expr = self.parse_expr()?;
                let unquote_splicing_symbol = SourceExpr::new(
                    LispExpr::Symbol("unquote-splicing".to_string()),
                    location.clone()
                );
                let unquoted_list = vec![unquote_splicing_symbol, unquoted_expr];
                Ok(SourceExpr::new(LispExpr::List(unquoted_list), location))
            } else {
                // Unquote: ,expr → (unquote expr)
                let unquoted_expr = self.parse_expr()?;
                let unquote_symbol = SourceExpr::new(LispExpr::Symbol("unquote".to_string()), location.clone());
                let unquoted_list = vec![unquote_symbol, unquoted_expr];
                Ok(SourceExpr::new(LispExpr::List(unquoted_list), location))
            }
        } else if token.text == "#" {
            // Reader macro dispatch character
            self.pos += 1;

            if self.pos >= self.tokens.len() {
                return Err("Unexpected end of input after '#'".to_string());
            }

            let dispatch_char = &self.tokens[self.pos].text;

            if dispatch_char == "(" {
                // Vector literal: #(1 2 3) → (vector 1 2 3)
                let vector_list = self.parse_list()?;

                match vector_list.expr {
                    LispExpr::List(elements) => {
                        // Transform to (vector element1 element2 ...)
                        let mut vector_call = vec![
                            SourceExpr::new(LispExpr::Symbol("vector".to_string()), location.clone())
                        ];
                        vector_call.extend(elements);
                        Ok(SourceExpr::new(LispExpr::List(vector_call), location))
                    }
                    _ => Err("Expected list after #(".to_string()),
                }
            } else if dispatch_char == "t" {
                // Boolean true: #t → true
                self.pos += 1;
                Ok(SourceExpr::new(LispExpr::Boolean(true), location))
            } else if dispatch_char == "f" {
                // Boolean false: #f → false
                self.pos += 1;
                Ok(SourceExpr::new(LispExpr::Boolean(false), location))
            } else if dispatch_char == ";" {
                // Comment out next expression: #;expr → (nothing)
                self.pos += 1; // consume ';'

                // Parse and discard the next expression
                let _discarded = self.parse_expr()?;

                // Now parse and return the expression after the discarded one
                self.parse_expr()
            } else if dispatch_char == "'" {
                // Function quote: #'symbol → symbol
                // In our Lisp, function names are already first-class values
                self.pos += 1; // consume '
                self.parse_expr()
            } else {
                Err(format!("Unknown reader macro: #{}", dispatch_char))
            }
        } else if token.text == "true" {
            self.pos += 1;
            Ok(SourceExpr::new(LispExpr::Boolean(true), location))
        } else if token.text == "false" {
            self.pos += 1;
            Ok(SourceExpr::new(LispExpr::Boolean(false), location))
        } else if token.text.starts_with('"') && token.text.ends_with('"') {
            // String literal
            self.pos += 1;
            let string_content = token.text[1..token.text.len()-1].to_string();
            // @TODO: for now, represents strings as symbols prefixed with "str:"
            // This is a temporary hack, there should be a String variant to LispExpr
            // for simplicity, just a special symbol so the compiler can recognise
            Ok(SourceExpr::new(LispExpr::Symbol(format!("__STRING__{}", string_content)), location))
        } else if token.text.contains('.') || token.text.contains('e') || token.text.contains('E') {
            // Try parsing as float (contains decimal point or scientific notation)
            if let Ok(f) = token.text.parse::<f64>() {
                self.pos += 1;
                Ok(SourceExpr::new(LispExpr::Float(f), location))
            } else if let Ok(n) = token.text.parse::<i64>() {
                // Fallback to integer if float parsing fails
                self.pos += 1;
                Ok(SourceExpr::new(LispExpr::Number(n), location))
            } else {
                let symbol = token.text.clone();
                self.pos += 1;
                Ok(SourceExpr::new(LispExpr::Symbol(symbol), location))
            }
        } else if let Ok(n) = token.text.parse::<i64>() {
            self.pos += 1;
            Ok(SourceExpr::new(LispExpr::Number(n), location))
        } else {
            let symbol = token.text.clone();
            self.pos += 1;
            Ok(SourceExpr::new(LispExpr::Symbol(symbol), location))
        }
    }

    fn parse_list(&mut self) -> Result<SourceExpr, String> {
        let start_token = &self.tokens[self.pos];
        let location = Location::new(start_token.line, start_token.column, self.file.clone());

        self.pos += 1; // consume '('

        let mut items = Vec::new();

        while self.pos < self.tokens.len() {
            if self.tokens[self.pos].text == ")" {
                self.pos += 1; // consume ')'
                return Ok(SourceExpr::new(LispExpr::List(items), location));
            }

            items.push(self.parse_expr()?);

            // Check for dot syntax: (a b . rest)
            if self.pos < self.tokens.len() && self.tokens[self.pos].text == "." {
                self.pos += 1; // consume '.'

                // Parse the rest expression
                let rest = self.parse_expr()?;

                // Expect closing paren
                if self.pos >= self.tokens.len() || self.tokens[self.pos].text != ")" {
                    return Err("Expected ')' after dotted pair".to_string());
                }
                self.pos += 1; // consume ')'

                return Ok(SourceExpr::new(
                    LispExpr::DottedList(items, Box::new(rest)),
                    location,
                ));
            }
        }

        Err("Unclosed list - missing closing parenthesis".to_string())
    }
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut line = 1;
    let mut column = 1;
    let mut token_start_column = 1;
    let mut in_string = false;
    let mut string_content = String::new();
    let mut string_start_line = 1;
    let mut string_start_column = 1;
    let mut in_comment = false;

    for ch in input.chars() {
        if in_comment {
            // Skip everything until newline
            if ch == '\n' {
                in_comment = false;
                line += 1;
                column = 1;
                token_start_column = 1;
            } else {
                column += 1;
            }
        } else if in_string {
            if ch == '"' {
                // End of string
                tokens.push(Token {
                    text: format!("\"{}\"", string_content),
                    line: string_start_line,
                    column: string_start_column,
                });
                string_content.clear();
                in_string = false;
                column += 1;
                token_start_column = column;
            } else if ch == '\\' {
                // Handle escape sequences (basic support)
                string_content.push(ch);
                column += 1;
            } else {
                string_content.push(ch);
                if ch == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
            }
        } else {
            match ch {
                '"' => {
                    // Start of string
                    if !current.is_empty() {
                        tokens.push(Token {
                            text: current.clone(),
                            line,
                            column: token_start_column,
                        });
                        current.clear();
                    }
                    in_string = true;
                    string_start_line = line;
                    string_start_column = column;
                    column += 1;
                }
                ';' => {
                    // Check if previous token was '#' - if so, this is part of #; reader macro
                    let is_reader_macro = !tokens.is_empty() && tokens.last().unwrap().text == "#";

                    if is_reader_macro {
                        // Treat ';' as a special token (part of #; reader macro)
                        if !current.is_empty() {
                            tokens.push(Token {
                                text: current.clone(),
                                line,
                                column: token_start_column,
                            });
                            current.clear();
                        }
                        tokens.push(Token {
                            text: ";".to_string(),
                            line,
                            column,
                        });
                        column += 1;
                        token_start_column = column;
                    } else {
                        // Start of comment - skip until end of line
                        if !current.is_empty() {
                            tokens.push(Token {
                                text: current.clone(),
                                line,
                                column: token_start_column,
                            });
                            current.clear();
                        }
                        in_comment = true;
                        column += 1;
                    }
                }
                '(' | ')' | '\'' | '`' | ',' | '@' | '#' => {
                    if !current.is_empty() {
                        tokens.push(Token {
                            text: current.clone(),
                            line,
                            column: token_start_column,
                        });
                        current.clear();
                    }
                    tokens.push(Token {
                        text: ch.to_string(),
                        line,
                        column,
                    });
                    column += 1;
                    token_start_column = column;
                }
                '\n' => {
                    if !current.is_empty() {
                        tokens.push(Token {
                            text: current.clone(),
                            line,
                            column: token_start_column,
                        });
                        current.clear();
                    }
                    line += 1;
                    column = 1;
                    token_start_column = 1;
                }
                ' ' | '\t' | '\r' => {
                    if !current.is_empty() {
                        tokens.push(Token {
                            text: current.clone(),
                            line,
                            column: token_start_column,
                        });
                        current.clear();
                    }
                    column += 1;
                    token_start_column = column;
                }
                _ => {
                    if current.is_empty() {
                        token_start_column = column;
                    }
                    current.push(ch);
                    column += 1;
                }
            }
        }
    }

    if !current.is_empty() {
        tokens.push(Token {
            text: current,
            line,
            column: token_start_column,
        });
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let mut parser = Parser::new("42");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].expr, LispExpr::Number(42));
    }

    #[test]
    fn test_parse_symbol() {
        let mut parser = Parser::new("fib");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].expr, LispExpr::Symbol("fib".to_string()));
    }

    #[test]
    fn test_parse_simple_list() {
        let mut parser = Parser::new("(+ 1 2)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].expr, LispExpr::Symbol("+".to_string()));
                assert_eq!(items[1].expr, LispExpr::Number(1));
                assert_eq!(items[2].expr, LispExpr::Number(2));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_nested_list() {
        let mut parser = Parser::new("(+ (* 5 2) 3)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].expr, LispExpr::Symbol("+".to_string()));
                match &items[1].expr {
                    LispExpr::List(inner) => {
                        assert_eq!(inner.len(), 3);
                        assert_eq!(inner[0].expr, LispExpr::Symbol("*".to_string()));
                        assert_eq!(inner[1].expr, LispExpr::Number(5));
                        assert_eq!(inner[2].expr, LispExpr::Number(2));
                    }
                    _ => panic!("Expected nested List"),
                }
                assert_eq!(items[2].expr, LispExpr::Number(3));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_multiple_expressions() {
        let mut parser = Parser::new("(+ 1 2) (* 3 4)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 2);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].expr, LispExpr::Symbol("+".to_string()));
                assert_eq!(items[1].expr, LispExpr::Number(1));
                assert_eq!(items[2].expr, LispExpr::Number(2));
            }
            _ => panic!("Expected List"),
        }

        match &exprs[1].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].expr, LispExpr::Symbol("*".to_string()));
                assert_eq!(items[1].expr, LispExpr::Number(3));
                assert_eq!(items[2].expr, LispExpr::Number(4));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_boolean_true() {
        let mut parser = Parser::new("true");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].expr, LispExpr::Boolean(true));
    }

    #[test]
    fn test_parse_boolean_false() {
        let mut parser = Parser::new("false");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].expr, LispExpr::Boolean(false));
    }

    #[test]
    fn test_parse_negative_number() {
        let mut parser = Parser::new("-42");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].expr, LispExpr::Number(-42));
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let mut parser = Parser::new("  ( +   1    2  )  ");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].expr, LispExpr::Symbol("+".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_multiline() {
        let source = r#"
            (defun double (x)
                (* x 2))
        "#;
        let mut parser = Parser::new(source);
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
    }

    #[test]
    fn test_parse_empty_list_error() {
        let mut parser = Parser::new("()");
        let exprs = parser.parse_all().unwrap();
        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 0);
            }
            _ => panic!("Expected empty list"),
        }
    }

    #[test]
    fn test_parse_unclosed_paren_error() {
        let mut parser = Parser::new("(+ 1 2");
        let result = parser.parse_all();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing closing parenthesis"));
    }

    #[test]
    fn test_parse_location_tracking() {
        let mut parser = Parser::new_with_file("42", "test.lisp".to_string());
        let exprs = parser.parse_all().unwrap();

        assert_eq!(exprs[0].location.file, "test.lisp");
        assert_eq!(exprs[0].location.line, 1);
        assert_eq!(exprs[0].location.column, 1);
    }

    #[test]
    fn test_parse_multiline_location() {
        let source = "10\n20\n30";
        let mut parser = Parser::new(source);
        let exprs = parser.parse_all().unwrap();

        assert_eq!(exprs.len(), 3);
        assert_eq!(exprs[0].location.line, 1);
        assert_eq!(exprs[1].location.line, 2);
        assert_eq!(exprs[2].location.line, 3);
    }

    #[test]
    fn test_parse_operators() {
        let operators = vec!["+", "-", "*", "/", "%", "<=", "<", ">", ">=", "==", "!=", "neg"];

        for op in operators {
            let mut parser = Parser::new(op);
            let exprs = parser.parse_all().unwrap();
            assert_eq!(exprs.len(), 1);
            assert_eq!(exprs[0].expr, LispExpr::Symbol(op.to_string()));
        }
    }

    #[test]
    fn test_parse_defun() {
        let source = "(defun add (a b) (+ a b))";
        let mut parser = Parser::new(source);
        let exprs = parser.parse_all().unwrap();

        assert_eq!(exprs.len(), 1);
        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items[0].expr, LispExpr::Symbol("defun".to_string()));
                assert_eq!(items[1].expr, LispExpr::Symbol("add".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_vector_literal() {
        let mut parser = Parser::new("#(1 2 3)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items[0].expr, LispExpr::Symbol("vector".to_string()));
                assert_eq!(items[1].expr, LispExpr::Number(1));
                assert_eq!(items[2].expr, LispExpr::Number(2));
                assert_eq!(items[3].expr, LispExpr::Number(3));
            }
            _ => panic!("Expected List (vector ...)"),
        }
    }

    #[test]
    fn test_parse_empty_vector_literal() {
        let mut parser = Parser::new("#()");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].expr, LispExpr::Symbol("vector".to_string()));
            }
            _ => panic!("Expected List (vector)"),
        }
    }

    #[test]
    fn test_parse_nested_vector_literal() {
        let mut parser = Parser::new("#(1 #(2 3) 4)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items[0].expr, LispExpr::Symbol("vector".to_string()));
                assert_eq!(items[1].expr, LispExpr::Number(1));

                // Check nested vector
                match &items[2].expr {
                    LispExpr::List(nested) => {
                        assert_eq!(nested.len(), 3);
                        assert_eq!(nested[0].expr, LispExpr::Symbol("vector".to_string()));
                        assert_eq!(nested[1].expr, LispExpr::Number(2));
                        assert_eq!(nested[2].expr, LispExpr::Number(3));
                    }
                    _ => panic!("Expected nested vector"),
                }

                assert_eq!(items[3].expr, LispExpr::Number(4));
            }
            _ => panic!("Expected List (vector ...)"),
        }
    }

    #[test]
    fn test_parse_vector_with_expressions() {
        let mut parser = Parser::new("#((+ 1 2) (* 3 4))");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].expr, LispExpr::Symbol("vector".to_string()));

                // Check first element is (+ 1 2)
                match &items[1].expr {
                    LispExpr::List(add_expr) => {
                        assert_eq!(add_expr.len(), 3);
                        assert_eq!(add_expr[0].expr, LispExpr::Symbol("+".to_string()));
                    }
                    _ => panic!("Expected list expression"),
                }
            }
            _ => panic!("Expected List (vector ...)"),
        }
    }

    #[test]
    fn test_parse_boolean_true_reader_macro() {
        let mut parser = Parser::new("#t");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].expr, LispExpr::Boolean(true));
    }

    #[test]
    fn test_parse_boolean_false_reader_macro() {
        let mut parser = Parser::new("#f");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].expr, LispExpr::Boolean(false));
    }

    #[test]
    fn test_parse_boolean_reader_macros_in_list() {
        let mut parser = Parser::new("(if #t #f true)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items[0].expr, LispExpr::Symbol("if".to_string()));
                assert_eq!(items[1].expr, LispExpr::Boolean(true));
                assert_eq!(items[2].expr, LispExpr::Boolean(false));
                assert_eq!(items[3].expr, LispExpr::Boolean(true));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_expression_comment() {
        let mut parser = Parser::new("(+ 1 #;(* 2 3) 4)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3); // +, 1, 4 (the (* 2 3) is commented out)
                assert_eq!(items[0].expr, LispExpr::Symbol("+".to_string()));
                assert_eq!(items[1].expr, LispExpr::Number(1));
                assert_eq!(items[2].expr, LispExpr::Number(4));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_expression_comment_at_top_level() {
        let mut parser = Parser::new("#;(this is ignored) 42");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].expr, LispExpr::Number(42));
    }

    #[test]
    fn test_parse_nested_expression_comment() {
        let mut parser = Parser::new("(+ 1 #;#;2 3 4)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3); // +, 1, 4 (both 2 and 3 are commented)
                assert_eq!(items[0].expr, LispExpr::Symbol("+".to_string()));
                assert_eq!(items[1].expr, LispExpr::Number(1));
                assert_eq!(items[2].expr, LispExpr::Number(4));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_function_quote() {
        let mut parser = Parser::new("#'add");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        // #'add → add (since function names are first-class)
        assert_eq!(exprs[0].expr, LispExpr::Symbol("add".to_string()));
    }

    #[test]
    fn test_parse_function_quote_in_expression() {
        let mut parser = Parser::new("(apply #'+ '(1 2 3))");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);

        match &exprs[0].expr {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].expr, LispExpr::Symbol("apply".to_string()));
                // Check #'+ became +
                assert_eq!(items[1].expr, LispExpr::Symbol("+".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }
}

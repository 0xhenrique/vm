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
        } else if token.text == "true" {
            self.pos += 1;
            Ok(SourceExpr::new(LispExpr::Boolean(true), location))
        } else if token.text == "false" {
            self.pos += 1;
            Ok(SourceExpr::new(LispExpr::Boolean(false), location))
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

    for ch in input.chars() {
        match ch {
            '(' | ')' => {
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
        assert_eq!(exprs[0], LispExpr::Number(42));
    }

    #[test]
    fn test_parse_symbol() {
        let mut parser = Parser::new("fib");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0], LispExpr::Symbol("fib".to_string()));
    }

    #[test]
    fn test_parse_simple_list() {
        let mut parser = Parser::new("(+ 1 2)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(
            exprs[0],
            LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                LispExpr::Number(1),
                LispExpr::Number(2),
            ])
        );
    }

    #[test]
    fn test_parse_nested_list() {
        let mut parser = Parser::new("(+ (* 5 2) 3)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(
            exprs[0],
            LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                LispExpr::List(vec![
                    LispExpr::Symbol("*".to_string()),
                    LispExpr::Number(5),
                    LispExpr::Number(2),
                ]),
                LispExpr::Number(3),
            ])
        );
    }

    #[test]
    fn test_parse_multiple_expressions() {
        let mut parser = Parser::new("(+ 1 2) (* 3 4)");
        let exprs = parser.parse_all().unwrap();
        assert_eq!(exprs.len(), 2);
        assert_eq!(
            exprs[0],
            LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                LispExpr::Number(1),
                LispExpr::Number(2),
            ])
        );
        assert_eq!(
            exprs[1],
            LispExpr::List(vec![
                LispExpr::Symbol("*".to_string()),
                LispExpr::Number(3),
                LispExpr::Number(4),
            ])
        );
    }
}

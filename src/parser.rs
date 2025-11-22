use crate::LispExpr;

pub struct Parser {
    tokens: Vec<String>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = tokenize(input);
        Parser { tokens, pos: 0 }
    }

    pub fn parse_all(&mut self) -> Result<Vec<LispExpr>, String> {
        let mut exprs = Vec::new();
        while self.pos < self.tokens.len() {
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }

    fn parse_expr(&mut self) -> Result<LispExpr, String> {
        if self.pos >= self.tokens.len() {
            return Err("Unexpected end of input".to_string());
        }

        let token = &self.tokens[self.pos];

        if token == "(" {
            self.parse_list()
        } else if token == ")" {
            Err("Unexpected closing parenthesis".to_string())
        } else if token == "true" {
            self.pos += 1;
            Ok(LispExpr::Boolean(true))
        } else if token == "false" {
            self.pos += 1;
            Ok(LispExpr::Boolean(false))
        } else if let Ok(n) = token.parse::<i64>() {
            self.pos += 1;
            Ok(LispExpr::Number(n))
        } else {
            self.pos += 1;
            Ok(LispExpr::Symbol(token.clone()))
        }
    }

    fn parse_list(&mut self) -> Result<LispExpr, String> {
        self.pos += 1; // consume '('

        let mut items = Vec::new();

        while self.pos < self.tokens.len() {
            if &self.tokens[self.pos] == ")" {
                self.pos += 1; // consume ')'
                return Ok(LispExpr::List(items));
            }

            items.push(self.parse_expr()?);
        }

        Err("Unclosed list - missing closing parenthesis".to_string())
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '(' | ')' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(ch.to_string());
            }
            ' ' | '\n' | '\t' | '\r' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
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

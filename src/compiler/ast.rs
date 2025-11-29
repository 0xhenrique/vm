use crate::vm::errors::Location;

#[derive(Debug, Clone, PartialEq)]
pub enum LispExpr {
    Number(i64),
    Float(f64),
    Boolean(bool),
    Symbol(String),
    List(Vec<SourceExpr>),
    DottedList(Vec<SourceExpr>, Box<SourceExpr>), // (a b . rest) - for cons patterns
}

// Wrapper that includes source location
#[derive(Debug, Clone, PartialEq)]
pub struct SourceExpr {
    pub expr: LispExpr,
    pub location: Location,
}

impl SourceExpr {
    pub fn new(expr: LispExpr, location: Location) -> Self {
        SourceExpr { expr, location }
    }

    pub fn unknown(expr: LispExpr) -> Self {
        SourceExpr {
            expr,
            location: Location::unknown(),
        }
    }
}

// Helper functions for creating AST nodes (used in tests)
#[allow(dead_code)]
pub fn number(n: i64) -> SourceExpr {
    SourceExpr::unknown(LispExpr::Number(n))
}

#[allow(dead_code)]
pub fn float(f: f64) -> SourceExpr {
    SourceExpr::unknown(LispExpr::Float(f))
}

#[allow(dead_code)]
pub fn boolean(b: bool) -> SourceExpr {
    SourceExpr::unknown(LispExpr::Boolean(b))
}

#[allow(dead_code)]
pub fn symbol(s: &str) -> SourceExpr {
    SourceExpr::unknown(LispExpr::Symbol(s.to_string()))
}

#[allow(dead_code)]
pub fn list(items: Vec<SourceExpr>) -> SourceExpr {
    SourceExpr::unknown(LispExpr::List(items))
}

use super::instructions::Instruction;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    List(Vec<Value>),
    Symbol(String),
    String(String),
    Closure {
        params: Vec<String>,
        body: Vec<Instruction>,
        captured: Vec<(String, Value)>, // Captured environment as ordered pairs
    },
}

impl Value {
    pub fn is_int(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_))
    }

    pub fn is_closure(&self) -> bool {
        matches!(self, Value::Closure { .. })
    }

    pub fn as_int(&self) -> Option<i64> {
        if let Value::Integer(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let Value::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_symbol(&self) -> Option<&str> {
        if let Value::Symbol(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        if let Value::List(lst) = self {
            Some(lst)
        } else {
            None
        }
    }
}

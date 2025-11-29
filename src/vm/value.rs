use super::instructions::Instruction;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    List(Vec<Value>),
    Symbol(String),
    String(String),
    Function(String), // Reference to a named function
    Closure {
        params: Vec<String>,              // Required parameters
        rest_param: Option<String>,       // Optional rest parameter for variadic functions
        body: Vec<Instruction>,
        captured: Vec<(String, Value)>,   // Captured environment as ordered pairs
    },
    HashMap(HashMap<String, Value>), // Hash map with string keys
    Vector(Vec<Value>), // Efficient array with O(1) indexed access
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

    pub fn is_function(&self) -> bool {
        matches!(self, Value::Function(_))
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

    pub fn as_function(&self) -> Option<&str> {
        if let Value::Function(name) = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn is_hashmap(&self) -> bool {
        matches!(self, Value::HashMap(_))
    }

    pub fn as_hashmap(&self) -> Option<&HashMap<String, Value>> {
        if let Value::HashMap(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn is_vector(&self) -> bool {
        matches!(self, Value::Vector(_))
    }

    pub fn as_vector(&self) -> Option<&Vec<Value>> {
        if let Value::Vector(vec) = self {
            Some(vec)
        } else {
            None
        }
    }
}

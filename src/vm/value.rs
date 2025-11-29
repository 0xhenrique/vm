use super::instructions::Instruction;
use std::collections::HashMap;
use std::sync::Arc;

/// Cons-cell based list structure for O(1) cons/car/cdr operations.
/// Uses Arc for structural sharing - cdr returns a reference to existing tail.
#[derive(Debug, Clone)]
pub struct ConsCell {
    pub head: Value,
    pub tail: List,
}

/// List type using cons cells with Arc for efficient structural sharing.
/// - Nil represents empty list '()
/// - Cons(Arc<ConsCell>) wraps a cons cell in Arc for O(1) sharing
#[derive(Debug, Clone)]
pub enum List {
    Nil,
    Cons(Arc<ConsCell>),
}

impl List {
    /// Create an empty list
    pub fn nil() -> Self {
        List::Nil
    }

    /// Create a cons cell (prepend element to list)
    pub fn cons(head: Value, tail: List) -> Self {
        List::Cons(Arc::new(ConsCell { head, tail }))
    }

    /// Check if list is empty
    pub fn is_nil(&self) -> bool {
        matches!(self, List::Nil)
    }

    /// Get the head (car) of the list
    pub fn car(&self) -> Option<&Value> {
        match self {
            List::Nil => None,
            List::Cons(cell) => Some(&cell.head),
        }
    }

    /// Get the tail (cdr) of the list - O(1) operation via Arc clone
    pub fn cdr(&self) -> Option<List> {
        match self {
            List::Nil => None,
            List::Cons(cell) => Some(cell.tail.clone()),
        }
    }

    /// Get the length of the list
    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut current = self;
        while let List::Cons(cell) = current {
            count += 1;
            current = &cell.tail;
        }
        count
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.is_nil()
    }

    /// Convert from Vec<Value> to List (builds from end to preserve order)
    pub fn from_vec(items: Vec<Value>) -> Self {
        let mut list = List::Nil;
        for item in items.into_iter().rev() {
            list = List::cons(item, list);
        }
        list
    }

    /// Convert to Vec<Value>
    pub fn to_vec(&self) -> Vec<Value> {
        let mut result = Vec::new();
        let mut current = self;
        while let List::Cons(cell) = current {
            result.push(cell.head.clone());
            current = &cell.tail;
        }
        result
    }

    /// Iterate over the list
    pub fn iter(&self) -> ListIter {
        ListIter { current: self }
    }
}

/// Iterator over List elements
pub struct ListIter<'a> {
    current: &'a List,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            List::Nil => None,
            List::Cons(cell) => {
                let head = &cell.head;
                self.current = &cell.tail;
                Some(head)
            }
        }
    }
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        let mut a = self;
        let mut b = other;
        loop {
            match (a, b) {
                (List::Nil, List::Nil) => return true,
                (List::Cons(cell_a), List::Cons(cell_b)) => {
                    if cell_a.head != cell_b.head {
                        return false;
                    }
                    a = &cell_a.tail;
                    b = &cell_b.tail;
                }
                _ => return false,
            }
        }
    }
}

/// Closure data stored behind Arc for efficient sharing
#[derive(Debug, Clone, PartialEq)]
pub struct ClosureData {
    pub params: Vec<String>,
    pub rest_param: Option<String>,
    pub body: Vec<Instruction>,
    pub captured: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(List),
    Symbol(Arc<String>),
    String(Arc<String>),
    Function(Arc<String>), // Reference to a named function
    Closure(Arc<ClosureData>),
    HashMap(Arc<HashMap<String, Value>>), // Hash map with string keys
    Vector(Arc<Vec<Value>>), // Efficient array with O(1) indexed access
}

// Custom PartialEq to handle NaN in floats
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => {
                // NaN != NaN, but we treat them as equal for Value comparison
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::HashMap(a), Value::HashMap(b)) => a == b,
            (Value::Vector(a), Value::Vector(b)) => a == b,
            (Value::Closure(a), Value::Closure(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    pub fn is_int(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Float(_))
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
        matches!(self, Value::Closure(_))
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

    pub fn as_float(&self) -> Option<f64> {
        if let Value::Float(f) = self {
            Some(*f)
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

    pub fn as_list(&self) -> Option<&List> {
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

    /// Helper to create a Symbol from a string
    pub fn symbol(s: impl Into<String>) -> Self {
        Value::Symbol(Arc::new(s.into()))
    }

    /// Helper to create a String value
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(Arc::new(s.into()))
    }

    /// Helper to create a Function reference
    pub fn function(name: impl Into<String>) -> Self {
        Value::Function(Arc::new(name.into()))
    }

    /// Helper to create an empty list
    pub fn empty_list() -> Self {
        Value::List(List::Nil)
    }

    /// Helper to create a list from a vector
    pub fn list_from_vec(items: Vec<Value>) -> Self {
        Value::List(List::from_vec(items))
    }
}

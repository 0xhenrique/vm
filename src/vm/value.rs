use super::instructions::Instruction;
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use std::net::{TcpListener, TcpStream};

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
    pub fn iter(&self) -> ListIter<'_> {
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

/// Custom Drop implementation for List to avoid stack overflow with large lists.
///
/// Rust's default recursive Drop would create a call stack as deep as the list,
/// causing stack overflow for lists with 500k+ items. This implementation uses
/// an iterative approach to walk the list chain without growing the call stack.
///
/// Note: If `cell.head` contains a nested large List (Value::List), dropping it
/// will still use recursion. This is acceptable for most use cases since deeply
/// nested list structures are uncommon. The primary goal is handling flat lists
/// with millions of items.
impl Drop for List {
    fn drop(&mut self) {
        // Replace self with Nil, taking ownership of the original value
        let mut current = std::mem::replace(self, List::Nil);

        loop {
            // Get pointer to the Arc inside the Cons cell (if it's Cons)
            let arc_ptr = match &current {
                List::Nil => {
                    // Nil - nothing to drop. Forget to prevent recursive drop call.
                    std::mem::forget(current);
                    return;
                }
                List::Cons(arc) => arc as *const Arc<ConsCell>,
            };

            // SAFETY: We read the Arc out of current. ptr::read does a bitwise copy
            // without affecting refcount. We then forget current to prevent it from
            // dropping the Arc again (which would double-decrement the refcount).
            let arc = unsafe { std::ptr::read(arc_ptr) };
            std::mem::forget(current);

            // Try to take sole ownership of the ConsCell
            match Arc::try_unwrap(arc) {
                Ok(cell) => {
                    // We're the only owner - drop head, continue with tail
                    drop(cell.head);
                    current = cell.tail;
                }
                Err(_arc) => {
                    // Shared - _arc is dropped here, decrementing refcount
                    return;
                }
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
    TcpListener(Rc<RefCell<TcpListener>>), // TCP listener for HTTP server
    TcpStream(Rc<RefCell<TcpStream>>), // TCP stream for HTTP connections
    SharedTcpListener(Arc<std::net::TcpListener>), // Thread-safe TCP listener for parallel serving
    Pointer(i64), // Raw pointer for FFI (null = 0)
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
            (Value::Pointer(a), Value::Pointer(b)) => a == b,
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

    pub fn is_pointer(&self) -> bool {
        matches!(self, Value::Pointer(_))
    }

    pub fn as_pointer(&self) -> Option<i64> {
        if let Value::Pointer(p) = self {
            Some(*p)
        } else {
            None
        }
    }

    /// Helper to create a null pointer
    pub fn null_pointer() -> Self {
        Value::Pointer(0)
    }

    /// Helper to create a pointer from an address
    pub fn pointer(addr: i64) -> Self {
        Value::Pointer(addr)
    }
}

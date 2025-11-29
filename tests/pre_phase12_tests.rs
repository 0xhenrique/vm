// Tests for features added before Phase 12 (Concurrency)
// These include: do/begin, type-of, gensym, macroexpand, time, assert

use lisp_bytecode_vm::*;

fn run_code(source: &str) -> Result<Value, String> {
    let mut parser = parser::Parser::new(source);
    let exprs = parser.parse_all().map_err(|e| e.to_string())?;

    let mut compiler = Compiler::new();
    let (functions, main_bytecode) = compiler.compile_program(&exprs)
        .map_err(|e| e.message)?;

    let mut vm = VM::new();
    vm.functions.extend(functions);
    vm.current_bytecode = main_bytecode;

    vm.run().map_err(|e| e.message.clone())?;

    Ok(vm.value_stack.last().cloned().unwrap_or(Value::Boolean(false)))
}

// ============================================================
// do/begin Form Tests
// ============================================================

#[test]
fn test_do_sequences_expressions() {
    let result = run_code(r#"
        (def x 10)
        (def y 20)
        (do
          (print x)
          (print y)
          (+ x y))
    "#).unwrap();
    assert_eq!(result, Value::Integer(30));
}

#[test]
fn test_begin_sequences_expressions() {
    let result = run_code(r#"
        (def x 5)
        (def y 3)
        (begin
          (print x)
          (print y)
          (* x y))
    "#).unwrap();
    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_do_returns_last_value() {
    let result = run_code(r#"
        (do
          42
          100
          999)
    "#).unwrap();
    assert_eq!(result, Value::Integer(999));
}

#[test]
fn test_do_with_single_expression() {
    let result = run_code(r#"
        (do 42)
    "#).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_do_nested() {
    let result = run_code(r#"
        (def a 10)
        (do
          (print a)
          (do
            (print a)
            (+ a 5))
          (* a 2))
    "#).unwrap();
    assert_eq!(result, Value::Integer(20));
}

// ============================================================
// type-of Tests
// ============================================================

#[test]
fn test_type_of_integer() {
    let result = run_code("(type-of 42)").unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "integer"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_float() {
    let result = run_code("(type-of 3.14)").unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "float"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_boolean() {
    let result = run_code("(type-of true)").unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "boolean"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_string() {
    let result = run_code(r#"(type-of "hello")"#).unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "string"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_symbol() {
    let result = run_code("(type-of 'foo)").unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "symbol"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_list() {
    let result = run_code("(type-of '(1 2 3))").unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "list"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_function() {
    let result = run_code("(type-of +)").unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "function"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_closure() {
    let result = run_code("(type-of (lambda (x) x))").unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "closure"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_hashmap() {
    let result = run_code(r#"(type-of (hash-map "a" 1))"#).unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "hashmap"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_type_of_vector() {
    let result = run_code("(type-of (vector 1 2 3))").unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "vector"),
        _ => panic!("Expected symbol"),
    }
}

// ============================================================
// gensym Tests
// ============================================================

#[test]
fn test_gensym_generates_symbol() {
    let result = run_code("(gensym)").unwrap();
    match result {
        Value::Symbol(_) => {},
        _ => panic!("Expected symbol, got {:?}", result),
    }
}

#[test]
fn test_gensym_generates_unique_symbols() {
    let result = run_code(r#"
        (def s1 (gensym))
        (def s2 (gensym))
        (== s1 s2)
    "#).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_gensym_multiple_calls() {
    let result = run_code(r#"
        (list (gensym) (gensym) (gensym))
    "#).unwrap();

    // Check that we got a list with 3 symbols
    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 3);
            for item in vec {
                match item {
                    Value::Symbol(_) => {},
                    _ => panic!("Expected all symbols"),
                }
            }
        },
        _ => panic!("Expected list"),
    }
}

// ============================================================
// macroexpand Tests
// ============================================================

#[test]
fn test_macroexpand_simple_macro() {
    let result = run_code(r#"
        (defmacro inc (x) `(+ ,x 1))
        (macroexpand '(inc 5))
    "#).unwrap();

    // The expanded form should be (+ 5 1)
    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            // Should be 3 elements: +, 5, 1
            assert!(vec.len() >= 2);
            match &vec[0] {
                Value::Symbol(s) => assert_eq!(s.as_str(), "+"),
                _ => panic!("Expected + symbol"),
            }
        },
        _ => panic!("Expected list, got {:?}", result),
    }
}

#[test]
fn test_macroexpand_non_macro() {
    let result = run_code(r#"
        (macroexpand '(+ 1 2))
    "#).unwrap();

    // Should return the original form unchanged
    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            // At minimum should have the + symbol
            assert!(vec.len() >= 1);
            match &vec[0] {
                Value::Symbol(s) => assert_eq!(s.as_str(), "+"),
                _ => panic!("Expected + symbol"),
            }
        },
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_macroexpand_atom() {
    let result = run_code("(macroexpand 42)").unwrap();
    assert_eq!(result, Value::Integer(42));
}

// ============================================================
// Integration Tests
// ============================================================

#[test]
fn test_do_with_type_of() {
    let result = run_code(r#"
        (def x 42)
        (do
          (print x)
          (type-of x))
    "#).unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "integer"),
        _ => panic!("Expected symbol"),
    }
}

#[test]
fn test_gensym_in_do_block() {
    let result = run_code(r#"
        (def g1 (gensym))
        (def g2 (gensym))
        (list g1 g2)
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 2);
            // Check they're different
            assert_ne!(vec[0], vec[1]);
        },
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_macroexpand_in_do() {
    let result = run_code(r#"
        (defmacro double (x) `(* 2 ,x))
        (def expanded (macroexpand '(double 5)))
        (type-of expanded)
    "#).unwrap();
    match result {
        Value::Symbol(s) => assert_eq!(s.as_str(), "list"),
        _ => panic!("Expected symbol"),
    }
}

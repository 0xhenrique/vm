use lisp_bytecode_vm::{Compiler, VM, parser::Parser, Value};

fn compile_and_run(source: &str) -> Result<String, String> {
    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).map_err(|e| format!("Compile error: {:?}", e))?;

    let mut vm = VM::new();
    // Merge user-defined functions with builtins (don't overwrite builtins!)
    for (name, bytecode) in functions {
        vm.functions.insert(name, bytecode);
    }
    vm.current_bytecode = main;
    vm.run().map_err(|e| format!("Runtime error: {:?}", e))?;

    // Get the top value from the stack and format it
    match vm.value_stack.last() {
        Some(value) => Ok(format_value(value)),
        None => Err("No value on stack".to_string()),
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 && !f.is_nan() && !f.is_infinite() {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        Value::Boolean(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::List(items) => {
            let formatted_items: Vec<String> = items.iter().map(format_value).collect();
            format!("({})", formatted_items.join(" "))
        }
        Value::String(s) => format!("\"{}\"", s),
        Value::Symbol(s) => s.clone(),
        Value::Function(name) => format!("<function:{}>", name),
        Value::Closure { params, rest_param, .. } => {
            if let Some(rest) = rest_param {
                format!("<closure:({:?} . {})>", params, rest)
            } else {
                format!("<closure:({:?})>", params)
            }
        }
        Value::HashMap(_) => "<hashmap>".to_string(),
        Value::Vector(_) => "<vector>".to_string(),
    }
}

// Basic eval tests

#[test]
fn test_eval_simple_expression() {
    let result = compile_and_run(r#"(eval "(+ 1 2)")"#);
    assert_eq!(result, Ok("3".to_string()));
}

#[test]
fn test_eval_arithmetic() {
    let result = compile_and_run(r#"(eval "(* 5 6)")"#);
    assert_eq!(result, Ok("30".to_string()));
}

#[test]
fn test_eval_nested_expression() {
    let result = compile_and_run(r#"(eval "(+ (* 2 3) (* 4 5))")"#);
    assert_eq!(result, Ok("26".to_string()));
}

#[test]
fn test_eval_string_manipulation() {
    // Test string operations within eval
    let result = compile_and_run(r#"(eval "(string-length (symbol->string (quote hello)))")"#);
    assert_eq!(result, Ok("5".to_string()));
}

#[test]
fn test_eval_list_operations() {
    let result = compile_and_run(r#"(eval "(cons 1 (cons 2 (list)))")"#);
    assert_eq!(result, Ok("(1 2)".to_string()));
}

// Eval with variables and definitions

#[test]
fn test_eval_with_defun() {
    let result = compile_and_run(r#"
        (eval "(defun double (x) (* x 2))")
        (double 21)
    "#);
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_eval_with_lambda() {
    let result = compile_and_run(r#"
        (eval "((lambda (x) (* x 3)) 7)")
    "#);
    assert_eq!(result, Ok("21".to_string()));
}

#[test]
fn test_eval_returns_value() {
    // Test that eval returns the last value
    let result = compile_and_run(r#"
        (eval "(+ 5 5)")
    "#);
    assert_eq!(result, Ok("10".to_string()));
}

// Eval with conditionals

#[test]
fn test_eval_if_expression() {
    let result = compile_and_run(r#"(eval "(if true 42 99)")"#);
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_eval_cond_expression() {
    let result = compile_and_run(r#"
        (eval "(cond ((< 5 3) 1) ((> 5 3) 2) (true 3))")
    "#);
    assert_eq!(result, Ok("2".to_string()));
}

// Eval with higher-order functions

#[test]
fn test_eval_with_lambdas() {
    // Use lambdas inside eval
    let result = compile_and_run(r#"
        (eval "((lambda (x) (* x x)) 5)")
    "#);
    assert_eq!(result, Ok("25".to_string()));
}

#[test]
fn test_eval_nested_lambdas() {
    // Nested lambda application
    let result = compile_and_run(r#"
        (eval "((lambda (x) ((lambda (y) (+ x y)) 10)) 5)")
    "#);
    assert_eq!(result, Ok("15".to_string()));
}

// Eval with dynamic code generation

#[test]
fn test_eval_dynamic_code() {
    let result = compile_and_run(r#"
        (eval "(+ 10 20)")
    "#);
    assert_eq!(result, Ok("30".to_string()));
}

#[test]
fn test_eval_computed_function_name() {
    let result = compile_and_run(r#"
        (eval "(defun add (x y) (+ x y))")
        (eval "(add 5 7)")
    "#);
    assert_eq!(result, Ok("12".to_string()));
}

// Eval with multiple expressions

#[test]
fn test_eval_multiple_expressions() {
    // Multiple independent expressions
    let result = compile_and_run(r#"
        (eval "(+ 10 20)")
        (eval "(* 3 4)")
    "#);
    assert_eq!(result, Ok("12".to_string()));
}

// Eval with recursion

#[test]
fn test_eval_recursive_function() {
    let result = compile_and_run(r#"
        (eval "(defun fact (n) (if (<= n 1) 1 (* n (fact (- n 1)))))")
        (fact 5)
    "#);
    assert_eq!(result, Ok("120".to_string()));
}

// Eval with closures

#[test]
fn test_eval_closure() {
    // Create and use a closure entirely within eval
    let result = compile_and_run(r#"
        (eval "((lambda (x) (lambda (y) (+ x y))) 5)")
    "#);
    // This returns a closure
    assert!(compile_and_run(r#"(eval "((lambda (x) (lambda (y) (+ x y))) 5)")"#).unwrap().contains("closure"));
}

// Eval error handling

#[test]
fn test_eval_parse_error() {
    let result = compile_and_run(r#"(eval "(+ 1 2")"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("parse"));
}

#[test]
fn test_eval_type_error() {
    let result = compile_and_run(r#"(eval 42)"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects a string"));
}

// Eval with floats

#[test]
fn test_eval_float_arithmetic() {
    let result = compile_and_run(r#"(eval "(+ 3.14 2.86)")"#);
    assert_eq!(result, Ok("6.0".to_string()));
}

#[test]
fn test_eval_math_functions() {
    let result = compile_and_run(r#"(eval "(sqrt 16.0)")"#);
    assert_eq!(result, Ok("4.0".to_string()));
}

// Multiple evals can build on each other through the VM's function table

#[test]
fn test_multiple_evals_with_functions() {
    // Each eval can define functions that subsequent code can use
    let result = compile_and_run(r#"
        (eval "(defun add (x y) (+ x y))")
        (eval "(defun mul (x y) (* x y))")
        (add (mul 2 3) 4)
    "#);
    assert_eq!(result, Ok("10".to_string()));
}

#[test]
fn test_eval_quote() {
    let result = compile_and_run(r#"(eval "(quote (1 2 3))")"#);
    assert_eq!(result, Ok("(1 2 3)".to_string()));
}

// Eval with data structures

#[test]
fn test_eval_vector() {
    let result = compile_and_run(r#"(eval "(vector 1 2 3)")"#);
    assert_eq!(result, Ok("<vector>".to_string()));
}

#[test]
fn test_eval_hashmap() {
    // Test hashmap creation
    let result = compile_and_run(r#"(eval "(hashmap? (hash-map))")"#);
    assert_eq!(result, Ok("true".to_string()));
}

// ========== Context-Aware Eval Tests ==========
// These tests verify that eval can access functions and globals from the parent context

#[test]
fn test_eval_can_call_parent_function() {
    // Eval can call a function defined in the main program
    let result = compile_and_run(r#"
        (defun square (x) (* x x))
        (eval "(square 7)")
    "#);
    assert_eq!(result, Ok("49".to_string()));
}

#[test]
fn test_eval_can_access_parent_global() {
    // Eval can access a global variable from the main program
    let result = compile_and_run(r#"
        (def myvar 42)
        (eval "myvar")
    "#);
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_eval_can_use_parent_global_in_computation() {
    // Eval can use a global variable from the main program in computations
    let result = compile_and_run(r#"
        (def counter 10)
        (def result (eval "(+ counter 5)"))
        result
    "#);
    assert_eq!(result, Ok("15".to_string()));
}

#[test]
fn test_eval_chain_with_functions() {
    // Multiple evals can build on each other's function definitions
    let result = compile_and_run(r#"
        (eval "(defun add (x y) (+ x y))")
        (eval "(defun mul (x y) (* x y))")
        (eval "(add (mul 3 4) 5)")
    "#);
    assert_eq!(result, Ok("17".to_string()));
}

#[test]
fn test_eval_chain_with_globals() {
    // Multiple evals can share global state
    let result = compile_and_run(r#"
        (eval "(def x 10)")
        (eval "(def y 20)")
        (eval "(+ x y)")
    "#);
    assert_eq!(result, Ok("30".to_string()));
}

#[test]
fn test_eval_uses_stdlib_function() {
    // Eval defined function can call a function from parent context
    let result = compile_and_run(r#"
        (defun double (x) (* x 2))
        (eval "(defun quadruple (x) (double (double x)))")
        (quadruple 5)
    "#);
    assert_eq!(result, Ok("20".to_string()));
}

#[test]
fn test_eval_closure_with_parent_function() {
    // Eval can create closures that use parent functions
    let result = compile_and_run(r#"
        (defun increment (x) (+ x 1))
        (eval "(defun make-add (x y) (+ (increment x) y))")
        (make-add 5 10)
    "#);
    assert_eq!(result, Ok("16".to_string()));
}

#[test]
fn test_parent_can_call_eval_defined_function() {
    // Parent context can call functions defined in eval
    let result = compile_and_run(r#"
        (eval "(defun triple (x) (* x 3))")
        (triple 7)
    "#);
    assert_eq!(result, Ok("21".to_string()));
}

#[test]
fn test_eval_complex_interaction() {
    // Complex interaction between parent and eval contexts
    let result = compile_and_run(r#"
        (def base 10)
        (defun add-base (x) (+ x base))
        (eval "(defun process (x) (add-base (* x 2)))")
        (def multiplier (eval "2"))
        (* (process 5) multiplier)
    "#);
    assert_eq!(result, Ok("40".to_string()));
}

#[test]
fn test_eval_with_higher_order_functions() {
    // Eval with higher-order functions from parent context
    let result = compile_and_run(r#"
        (defun apply-twice (f x) (f (f x)))
        (defun add-one (x) (+ x 1))
        (eval "(apply-twice add-one 10)")
    "#);
    assert_eq!(result, Ok("12".to_string()));
}

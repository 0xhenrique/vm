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

// ========== Function Arity Tests ==========

#[test]
fn test_function_arity_builtin_unary() {
    // Test arity of a builtin unary function
    let result = compile_and_run(r#"(function-arity car)"#);
    assert_eq!(result, Ok("1".to_string()));
}

#[test]
fn test_function_arity_builtin_binary() {
    // Test arity of a builtin binary function
    let result = compile_and_run(r#"(function-arity +)"#);
    assert_eq!(result, Ok("2".to_string()));
}

#[test]
fn test_function_arity_user_defined() {
    // Test arity of user-defined function
    let result = compile_and_run(r#"
        (defun triple (x y z) (+ x (+ y z)))
        (function-arity triple)
    "#);
    assert_eq!(result, Ok("3".to_string()));
}

#[test]
fn test_function_arity_closure() {
    // Test arity of a closure
    let result = compile_and_run(r#"
        (def my-closure (lambda (a b) (+ a b)))
        (function-arity my-closure)
    "#);
    assert_eq!(result, Ok("2".to_string()));
}

#[test]
fn test_function_arity_variadic_function() {
    // Test arity of variadic function - should return -1
    let result = compile_and_run(r#"
        (defun variadic (a . rest) (cons a rest))
        (function-arity variadic)
    "#);
    assert_eq!(result, Ok("-1".to_string()));
}

#[test]
fn test_function_arity_variadic_closure() {
    // Test arity of variadic closure - should return -1
    let result = compile_and_run(r#"
        (def my-var-closure (lambda (x . rest) (cons x rest)))
        (function-arity my-var-closure)
    "#);
    assert_eq!(result, Ok("-1".to_string()));
}

#[test]
fn test_function_arity_nullary() {
    // Test arity of function with no parameters
    let result = compile_and_run(r#"
        (defun get-constant () 42)
        (function-arity get-constant)
    "#);
    assert_eq!(result, Ok("0".to_string()));
}

#[test]
fn test_function_arity_type_error() {
    // Test error when calling on non-function
    let result = compile_and_run(r#"(function-arity 42)"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects a function or closure"));
}

// ========== Function Params Tests ==========

#[test]
fn test_function_params_simple_closure() {
    // Test getting parameters of a simple closure
    let result = compile_and_run(r#"
        (def my-closure (lambda (x y) (+ x y)))
        (function-params my-closure)
    "#);
    assert_eq!(result, Ok("(\"x\" \"y\")".to_string()));
}

#[test]
fn test_function_params_single_param() {
    // Test getting parameters of closure with single param
    let result = compile_and_run(r#"
        (def my-closure (lambda (x) (* x 2)))
        (function-params my-closure)
    "#);
    assert_eq!(result, Ok("(\"x\")".to_string()));
}

#[test]
fn test_function_params_no_params() {
    // Test getting parameters of closure with no params
    let result = compile_and_run(r#"
        (def my-closure (lambda () 42))
        (function-params my-closure)
    "#);
    assert_eq!(result, Ok("()".to_string()));
}

#[test]
fn test_function_params_variadic() {
    // Test getting parameters of variadic closure
    let result = compile_and_run(r#"
        (def my-closure (lambda (x . rest) (cons x rest)))
        (function-params my-closure)
    "#);
    assert_eq!(result, Ok("(\"x\" \". rest\")".to_string()));
}

#[test]
fn test_function_params_named_function_error() {
    // Test error when calling on named function (not closure)
    let result = compile_and_run(r#"
        (defun my-func (a b) (+ a b))
        (function-params my-func)
    "#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("only works with closures"));
}

#[test]
fn test_function_params_type_error() {
    // Test error when calling on non-closure
    let result = compile_and_run(r#"(function-params 42)"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects a closure"));
}

// ========== Closure Captured Tests ==========

#[test]
fn test_closure_captured_simple() {
    // Test getting captured variables from a closure
    // Closures capture variables from their enclosing function scope
    // Note: The compiler uses internal names like __captured_0 for captured variables
    let result = compile_and_run(r#"
        (defun make-adder (x)
            (lambda (y) (+ x y)))
        (def my-closure (make-adder 10))
        (closure-captured my-closure)
    "#);
    // Should return one captured variable with value 10
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("10"));
    assert!(output.starts_with("(("));
}

#[test]
fn test_closure_captured_multiple() {
    // Test getting multiple captured variables
    let result = compile_and_run(r#"
        (defun make-calculator (a b)
            (lambda (c) (+ a (+ b c))))
        (def my-closure (make-calculator 1 2))
        (closure-captured my-closure)
    "#);
    // Should return two captured variables with values 1 and 2
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    // Should be a list of two pairs
    let pair_count = output.matches("(").count() - 1; // -1 for outer list
    assert_eq!(pair_count, 2);
}

#[test]
fn test_closure_captured_none() {
    // Test closure with no captured variables
    let result = compile_and_run(r#"
        (def my-closure (lambda (x y) (+ x y)))
        (closure-captured my-closure)
    "#);
    assert_eq!(result, Ok("()".to_string()));
}

#[test]
fn test_closure_captured_nested() {
    // Test nested closure with captured variables
    let result = compile_and_run(r#"
        (defun make-inner (x y)
            (lambda (z) (+ x (+ y z))))
        (def inner (make-inner 10 5))
        (closure-captured inner)
    "#);
    // Should capture both x and y with values 10 and 5
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("10"));
    assert!(output.contains("5"));
}

#[test]
fn test_closure_captured_named_function() {
    // Named functions don't have captured variables
    let result = compile_and_run(r#"
        (defun my-func (x) (* x 2))
        (closure-captured my-func)
    "#);
    assert_eq!(result, Ok("()".to_string()));
}

#[test]
fn test_closure_captured_type_error() {
    // Test error when calling on non-function
    let result = compile_and_run(r#"(closure-captured 42)"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects a function or closure"));
}

// ========== Function Name Tests ==========

#[test]
fn test_function_name_builtin() {
    // Test getting name of builtin function
    let result = compile_and_run(r#"(function-name +)"#);
    assert_eq!(result, Ok("\"+\"".to_string()));
}

#[test]
fn test_function_name_user_defined() {
    // Test getting name of user-defined function
    let result = compile_and_run(r#"
        (defun my-function (x) (* x 2))
        (function-name my-function)
    "#);
    assert_eq!(result, Ok("\"my-function\"".to_string()));
}

#[test]
fn test_function_name_closure_error() {
    // Test error when calling on closure
    let result = compile_and_run(r#"
        (def my-closure (lambda (x) (* x 2)))
        (function-name my-closure)
    "#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects a named function, not a closure"));
}

#[test]
fn test_function_name_type_error() {
    // Test error when calling on non-function
    let result = compile_and_run(r#"(function-name 42)"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects a function"));
}

// ========== Integration Tests ==========

#[test]
fn test_reflection_with_multiple_functions() {
    // Test using reflection on multiple functions
    let result = compile_and_run(r#"
        (defun add (a b) (+ a b))
        (defun mul (a b c) (* a (* b c)))
        (list (function-arity add) (function-arity mul))
    "#);
    assert_eq!(result, Ok("(2 3)".to_string()));
}

#[test]
fn test_reflection_check_before_apply() {
    // Use reflection to check arity before calling
    let result = compile_and_run(r#"
        (defun safe-call (f x y)
            (if (== (function-arity f) 2)
                (f x y)
                0))
        (defun add (a b) (+ a b))
        (safe-call add 10 20)
    "#);
    assert_eq!(result, Ok("30".to_string()));
}

#[test]
fn test_reflection_inspect_closure_params_and_captured() {
    // Inspect both params and captured variables
    let result = compile_and_run(r#"
        (defun make-adder (base)
            (lambda (x) (+ base x)))
        (def adder (make-adder 100))
        (def params (function-params adder))
        (def captured (closure-captured adder))
        (list (car params) (car (cdr (car captured))))
    "#);
    // Should return the param name "x" and the captured value 100
    assert_eq!(result, Ok("(\"x\" 100)".to_string()));
}

#[test]
fn test_reflection_with_eval() {
    // Test reflection with eval'd code
    // Since eval'd functions can't be referenced as first-class values from outside eval,
    // we do the reflection inside the eval
    let result = compile_and_run(r#"
        (eval "(defun dynamic-func (a b c) (+ a (+ b c)))(function-arity dynamic-func)")
    "#);
    assert_eq!(result, Ok("3".to_string()));
}

#[test]
fn test_reflection_variadic_detection() {
    // Detect variadic functions by checking if arity is -1
    // Note: arity reflects the number of LoadArg instructions, not declared params
    let result = compile_and_run(r#"
        (defun var-func (a . rest) a)
        (defun normal-func (a b) (+ a b))
        (list (function-arity var-func) (function-arity normal-func))
    "#);
    assert_eq!(result, Ok("(-1 2)".to_string()));
}

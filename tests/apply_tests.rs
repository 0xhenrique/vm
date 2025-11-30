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
        Value::Symbol(s) => s.to_string(),
        Value::Function(name) => format!("<function:{}>", name),
        Value::Closure(closure_data) => {
            if let Some(rest) = &closure_data.rest_param {
                format!("<closure:({:?} . {})>", closure_data.params, rest)
            } else {
                format!("<closure:({:?})>", closure_data.params)
            }
        }
        Value::HashMap(_) => "<hashmap>".to_string(),
        Value::Vector(items) => {
            let formatted_items: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", formatted_items.join(" "))
        }
        Value::TcpListener(_) => "#<tcp-listener>".to_string(),
        Value::TcpStream(_) => "#<tcp-stream>".to_string(),
        Value::SharedTcpListener(_) => "#<shared-tcp-listener>".to_string(),
        Value::Pointer(p) => format!("#<pointer 0x{:x}>", p),
    }
}

// ==================== Basic Apply Tests ====================

#[test]
fn test_apply_with_builtin_addition() {
    let source = r#"
        (apply + (list 10 5))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "15");
}

#[test]
fn test_apply_with_builtin_multiplication() {
    let source = r#"
        (apply * (list 6 4))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "24");
}

#[test]
fn test_apply_with_builtin_max() {
    let source = r#"
        (defun max2 (a b) (if (> a b) a b))
        (apply max2 (list 5 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "5");
}

#[test]
fn test_apply_with_defun() {
    let source = r#"
        (defun add3 (a b c) (+ (+ a b) c))
        (apply add3 (list 10 20 30))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "60");
}

#[test]
fn test_apply_with_lambda() {
    let source = r#"
        (apply (lambda (x y) (* x y)) (list 7 8))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "56");
}

#[test]
fn test_apply_with_single_arg() {
    let source = r#"
        (defun square (x) (* x x))
        (apply square (list 9))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "81");
}

#[test]
fn test_apply_with_empty_list() {
    let source = r#"
        (defun zero-args () 42)
        (apply zero-args '())
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "42");
}

// ==================== Apply with Variadic Functions ====================

#[test]
fn test_apply_with_variadic_defun() {
    let source = r#"
        (defun make-list (a . rest)
            (cons a rest))
        (apply make-list (list 1 2 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(1 2 3)");
}

#[test]
fn test_apply_with_variadic_lambda() {
    let source = r#"
        (apply (lambda (. args) args) (list 1 2 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(1 2 3)");
}

#[test]
fn test_apply_with_mixed_variadic() {
    let source = r#"
        (defun first-and-rest (first . rest)
            (list first rest))
        (apply first-and-rest (list 1 2 3 4))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(1 (2 3 4))");
}

// ==================== Apply with Closures ====================

#[test]
fn test_apply_with_closure() {
    let source = r#"
        (defun make-adder (n)
            (lambda (x) (+ x n)))
        (let ((add5 (make-adder 5)))
            (apply add5 (list 10)))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "15");
}

#[test]
fn test_apply_with_closure_multiple_args() {
    let source = r#"
        (defun make-multiplier (factor)
            (lambda (a b) (* (* a b) factor)))
        (let ((times2 (make-multiplier 2)))
            (apply times2 (list 3 4)))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "24");
}

// ==================== Error Cases ====================

#[test]
fn test_apply_arity_mismatch() {
    let source = r#"
        (let ((add2 (lambda (a b) (+ a b))))
            (apply add2 (list 1 2 3)))
    "#;
    let result = compile_and_run(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("arity mismatch"));
}

#[test]
fn test_apply_with_non_list() {
    let source = r#"
        (apply + 42)
    "#;
    let result = compile_and_run(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("expected list"));
}

#[test]
fn test_apply_with_non_function() {
    let source = r#"
        (apply 42 (list 1 2 3))
    "#;
    let result = compile_and_run(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("expected function") || err.contains("expected function or closure"));
}

// ==================== Integration Tests ====================

#[test]
fn test_apply_in_higher_order_function() {
    let source = r#"
        (defun call-with-list (f lst)
            (apply f lst))
        (call-with-list + (list 7 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "10");
}

#[test]
fn test_apply_with_cons() {
    let source = r#"
        (apply cons (list 1 (list 2 3)))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(1 2 3)");
}

#[test]
fn test_apply_with_append() {
    let source = r#"
        (apply append (list (list 1 2) (list 3 4)))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(1 2 3 4)");
}

#[test]
fn test_nested_apply() {
    let source = r#"
        (defun add (a b) (+ a b))
        (defun make-pair (a b) (cons a (cons b '())))
        (apply add (apply make-pair (cons 5 (cons 10 '()))))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "15");
}

#[test]
fn test_apply_with_variadic_recursive_sum() {
    let source = r#"
        (defun sum-helper (nums)
            (if (null? nums)
                0
                (+ (car nums) (sum-helper (cdr nums)))))
        (defun sum (. nums)
            (sum-helper nums))
        (apply sum (list 10 20 30 40))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "100");
}

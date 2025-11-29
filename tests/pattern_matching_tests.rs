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
        Value::Boolean(b) => b.to_string(),
        Value::List(items) => {
            let formatted_items: Vec<String> = items.iter().map(|v| format_value(v)).collect();
            format!("({})", formatted_items.join(" "))
        }
        Value::Symbol(s) => s.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Function(name) => format!("<function {}>", name),
        Value::Closure(closure_data) => {
            let param_count = closure_data.params.len() + if closure_data.rest_param.is_some() { 1 } else { 0 };
            format!("<closure/{}>", param_count)
        }
        Value::HashMap(map) => {
            let mut items: Vec<String> = map.iter()
                .map(|(k, v)| format!("\"{}\" {}", k, format_value(v)))
                .collect();
            items.sort();
            format!("{{{}}}", items.join(" "))
        }
        Value::Vector(items) => {
            let formatted_items: Vec<String> = items.iter().map(|v| format_value(v)).collect();
            format!("[{}]", formatted_items.join(" "))
        }
        Value::TcpListener(_) => "#<tcp-listener>".to_string(),
        Value::TcpStream(_) => "#<tcp-stream>".to_string(),
        Value::SharedTcpListener(_) => "#<shared-tcp-listener>".to_string(),
    }
}

// ==================== Basic Pattern Matching Tests ====================

#[test]
fn test_pattern_literal_number() {
    let source = r#"
        (defun is-zero
          ((0) true)
          ((n) false))
        (is-zero 0)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_pattern_literal_number_fallthrough() {
    let source = r#"
        (defun is-zero
          ((0) true)
          ((n) false))
        (is-zero 5)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_pattern_variable_binding() {
    let source = r#"
        (defun double
          ((n) (* n 2)))
        (double 21)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "42");
}

#[test]
fn test_pattern_wildcard() {
    let source = r#"
        (defun always-42
          ((_) 42))
        (always-42 100)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "42");
}

#[test]
fn test_pattern_boolean_true() {
    let source = r#"
        (defun my-not
          ((true) false)
          ((false) true))
        (my-not true)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_pattern_boolean_false() {
    let source = r#"
        (defun my-not
          ((true) false)
          ((false) true))
        (my-not false)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

// ==================== Multiple Arguments Tests ====================

#[test]
fn test_pattern_multiple_args_first_zero() {
    let source = r#"
        (defun add-if-first-zero
          ((0 y) y)
          ((x y) (+ x y)))
        (add-if-first-zero 0 10)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "10");
}

#[test]
fn test_pattern_multiple_args_general() {
    let source = r#"
        (defun add-if-first-zero
          ((0 y) y)
          ((x y) (+ x y)))
        (add-if-first-zero 5 10)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "15");
}

#[test]
fn test_pattern_multiple_literals() {
    let source = r#"
        (defun check-pair
          ((1 1) 'both-one)
          ((1 y) 'first-one)
          ((x 1) 'second-one)
          ((x y) 'neither))
        (check-pair 1 1)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "both-one");
}

// ==================== Recursive Pattern Matching Tests ====================

#[test]
fn test_pattern_factorial() {
    let source = r#"
        (defun fact
          ((0) 1)
          ((n) (* n (fact (- n 1)))))
        (fact 5)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "120");
}

#[test]
fn test_pattern_fibonacci() {
    let source = r#"
        (defun fib
          ((0) 0)
          ((1) 1)
          ((n) (+ (fib (- n 1)) (fib (- n 2)))))
        (fib 10)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "55");
}

// ==================== Cons/List Pattern Tests ====================

#[test]
fn test_pattern_cons_first() {
    let source = r#"
        (defun first
          (((h . _)) h))
        (first '(1 2 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "1");
}

#[test]
fn test_pattern_cons_rest() {
    let source = r#"
        (defun rest
          (((_ . t)) t))
        (rest '(1 2 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(2 3)");
}

#[test]
fn test_pattern_empty_list() {
    let source = r#"
        (defun len
          (('()) 0)
          (((_ . t)) (+ 1 (len t))))
        (len '())
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "0");
}

#[test]
fn test_pattern_cons_length() {
    let source = r#"
        (defun len
          (('()) 0)
          (((_ . t)) (+ 1 (len t))))
        (len '(a b c d e))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "5");
}

#[test]
fn test_pattern_cons_sum() {
    let source = r#"
        (defun sum
          (('()) 0)
          (((h . t)) (+ h (sum t))))
        (sum '(1 2 3 4 5))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "15");
}

#[test]
fn test_pattern_cons_second_element() {
    let source = r#"
        (defun second
          (((_ x . _)) x))
        (second '(1 2 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "2");
}

// ==================== Multi-Argument with Cons Patterns ====================

#[test]
fn test_pattern_append() {
    let source = r#"
        (defun my-append
          (('() ys) ys)
          (((h . t) ys) (cons h (my-append t ys))))
        (my-append '(1 2) '(3 4))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(1 2 3 4)");
}

#[test]
fn test_pattern_reverse_helper() {
    let source = r#"
        (defun rev-helper
          (('() acc) acc)
          (((h . t) acc) (rev-helper t (cons h acc))))
        (rev-helper '(1 2 3) '())
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(3 2 1)");
}

// ==================== Edge Cases ====================

#[test]
fn test_pattern_single_clause() {
    // Single clause should still work
    let source = r#"
        (defun identity
          ((x) x))
        (identity 42)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "42");
}

#[test]
fn test_pattern_preserves_old_defun_syntax() {
    // Old syntax should still work
    let source = r#"
        (defun add (a b)
          (+ a b))
        (add 10 20)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "30");
}

#[test]
fn test_pattern_mixed_with_old_defun() {
    // Old and new syntax should coexist
    let source = r#"
        (defun old-style (x y) (+ x y))
        (defun new-style
          ((0 y) y)
          ((x y) (* x y)))
        (+ (old-style 1 2) (new-style 3 4))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "15"); // (1+2) + (3*4) = 3 + 12 = 15
}

#[test]
fn test_pattern_float_literal() {
    let source = r#"
        (defun check-pi
          ((3.14) true)
          ((x) false))
        (check-pi 3.14)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

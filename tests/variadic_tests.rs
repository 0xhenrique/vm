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
        Value::Symbol(s) => s.clone(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Function(name) => format!("<function {}>", name),
        Value::Closure { params, rest_param, .. } => {
            let param_count = params.len() + if rest_param.is_some() { 1 } else { 0 };
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
    }
}

// ==================== Variadic defun Tests ====================

#[test]
fn test_variadic_defun_basic() {
    let source = r#"
        (defun test (a b . rest)
            rest)
        (test 1 2 3 4 5)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(3 4 5)");
}

#[test]
fn test_variadic_defun_no_extra_args() {
    let source = r#"
        (defun test (a b . rest)
            rest)
        (test 10 20)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "()");
}

#[test]
fn test_variadic_defun_one_extra_arg() {
    let source = r#"
        (defun test (a . rest)
            rest)
        (test 1 2)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(2)");
}

#[test]
fn test_variadic_defun_use_required_and_rest() {
    let source = r#"
        (defun sum-with-base (base . nums)
            (+ base (if (null? nums) 0 (car nums))))
        (sum-with-base 100 42)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "142");
}

#[test]
fn test_variadic_defun_list_length_of_rest() {
    let source = r#"
        (defun count-rest (a . rest)
            (list-length rest))
        (count-rest 1 2 3 4 5 6 7)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "6");
}

#[test]
fn test_variadic_defun_insufficient_args() {
    let source = r#"
        (defun test (a b . rest)
            rest)
        (test 1)
    "#;
    let result = compile_and_run(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least 2"));
}

// ==================== Variadic lambda Tests ====================

#[test]
fn test_variadic_lambda_basic() {
    let source = r#"
        ((lambda (a b . rest) rest) 1 2 3 4 5)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(3 4 5)");
}

#[test]
fn test_variadic_lambda_no_extra_args() {
    let source = r#"
        ((lambda (a . rest) rest) 1)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "()");
}

#[test]
fn test_variadic_lambda_in_defun() {
    let source = r#"
        (defun make-adder (base)
            (lambda (. nums)
                (if (null? nums)
                    base
                    (+ base (car nums)))))
        (let ((add100 (make-adder 100)))
            (add100 42))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "142");
}

#[test]
fn test_variadic_lambda_captures_variable() {
    let source = r#"
        (let ((x 10))
            (let ((f (lambda (a . rest)
                        (+ x a (if (null? rest) 0 (car rest))))))
                (f 5 7)))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "22");
}

#[test]
fn test_variadic_closure_arity_check() {
    let source = r#"
        ((lambda (a b . rest) rest) 1)
    "#;
    let result = compile_and_run(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least 2"));
}

// ==================== Integration Tests ====================

#[test]
fn test_variadic_with_map() {
    let source = r#"
        (defun map (f lst)
            (if (null? lst)
                '()
                (cons (f (car lst))
                      (map f (cdr lst)))))
        (defun first-or-default (default . items)
            (if (null? items)
                default
                (car items)))
        (map (lambda (x) (first-or-default 0 x x)) (list 10 20 30))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(10 20 30)");
}

#[test]
fn test_variadic_recursive() {
    let source = r#"
        (defun sum-helper (nums)
            (if (null? nums)
                0
                (+ (car nums) (sum-helper (cdr nums)))))
        (defun sum-all (. nums)
            (sum-helper nums))
        (sum-all 1 2 3 4 5)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "15");
}

#[test]
fn test_variadic_with_filter() {
    let source = r#"
        (defun filter (pred lst)
            (if (null? lst)
                '()
                (if (pred (car lst))
                    (cons (car lst) (filter pred (cdr lst)))
                    (filter pred (cdr lst)))))
        (defun greater-than (threshold . nums)
            (filter (lambda (x) (> x threshold)) nums))
        (greater-than 5 1 3 5 7 9 11)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(7 9 11)");
}

#[test]
fn test_complex_variadic_scenario() {
    let source = r#"
        (defun make-list-processor (op)
            (lambda (initial . items)
                (if (null? items)
                    initial
                    (op initial (car items)))))
        (let ((adder (make-list-processor +))
              (multiplier (make-list-processor *)))
            (+ (adder 10 5) (multiplier 10 3)))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "45"); // (10 + 5) + (10 * 3) = 15 + 30 = 45
}

#[test]
fn test_variadic_zero_required_params() {
    let source = r#"
        (defun all-nums (. nums)
            nums)
        (all-nums 1 2 3)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "(1 2 3)");
}

#[test]
fn test_variadic_zero_required_no_args() {
    let source = r#"
        (defun all-nums (. nums)
            nums)
        (all-nums)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "()");
}

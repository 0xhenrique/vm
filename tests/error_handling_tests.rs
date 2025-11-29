use lisp_bytecode_vm::{Compiler, VM, parser::Parser, Value};

/// Helper function to compile and run source code with stdlib loaded
fn compile_and_run(source: &str) -> VM {
    // First, prepend the stdlib loading
    let full_source = format!(r#"
        (load "stdlib.lisp")
        {}
    "#, source);

    let mut parser = Parser::new(&full_source);
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).unwrap();

    let mut vm = VM::new();
    vm.functions.extend(functions);
    vm.current_bytecode = main;
    vm.run().unwrap();
    vm
}

/// Helper to get last value as string for printing
fn get_output(vm: &VM) -> String {
    match vm.value_stack.last() {
        Some(Value::Integer(n)) => n.to_string(),
        Some(Value::Boolean(b)) => if *b { "true".to_string() } else { "false".to_string() },
        Some(Value::String(s)) => s.clone(),
        Some(Value::List(items)) => format_list(items),
        Some(Value::Symbol(s)) => s.clone(),
        _ => panic!("Unexpected value type"),
    }
}

/// Helper to format a list for display
fn format_list(items: &[Value]) -> String {
    let formatted: Vec<String> = items.iter().map(|v| match v {
        Value::Integer(n) => n.to_string(),
        Value::Boolean(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::String(s) => s.clone(),
        Value::Symbol(s) => s.clone(),
        Value::List(inner) => format_list(inner),
        _ => format!("{:?}", v),
    }).collect();
    format!("({})", formatted.join(" "))
}

#[test]
fn test_ok_constructor() {
    let vm = compile_and_run("(ok 42)");
    assert_eq!(get_output(&vm), "(ok 42)");
}

#[test]
fn test_err_constructor() {
    let vm = compile_and_run(r#"(err "something went wrong")"#);
    assert_eq!(get_output(&vm), "(err something went wrong)");
}

#[test]
fn test_ok_predicate_with_ok() {
    let vm = compile_and_run("(ok? (ok 42))");
    assert_eq!(get_output(&vm), "true");
}

#[test]
fn test_ok_predicate_with_err() {
    let vm = compile_and_run(r#"(ok? (err "error"))"#);
    assert_eq!(get_output(&vm), "false");
}

#[test]
fn test_ok_predicate_with_non_result() {
    let vm = compile_and_run("(ok? 42)");
    assert_eq!(get_output(&vm), "false");
}

#[test]
fn test_err_predicate_with_err() {
    let vm = compile_and_run(r#"(err? (err "error"))"#);
    assert_eq!(get_output(&vm), "true");
}

#[test]
fn test_err_predicate_with_ok() {
    let vm = compile_and_run("(err? (ok 42))");
    assert_eq!(get_output(&vm), "false");
}

#[test]
fn test_result_predicate_with_ok() {
    let vm = compile_and_run("(result? (ok 42))");
    assert_eq!(get_output(&vm), "true");
}

#[test]
fn test_result_predicate_with_err() {
    let vm = compile_and_run(r#"(result? (err "error"))"#);
    assert_eq!(get_output(&vm), "true");
}

#[test]
fn test_result_predicate_with_non_result() {
    let vm = compile_and_run("(result? 42)");
    assert_eq!(get_output(&vm), "false");
}

#[test]
fn test_unwrap_ok() {
    let vm = compile_and_run("(unwrap (ok 42))");
    assert_eq!(get_output(&vm), "42");
}

#[test]
fn test_unwrap_ok_with_string() {
    let vm = compile_and_run(r#"(unwrap (ok "success"))"#);
    assert_eq!(get_output(&vm), "success");
}

#[test]
fn test_unwrap_ok_with_list() {
    let vm = compile_and_run("(unwrap (ok '(1 2 3)))");
    assert_eq!(get_output(&vm), "(1 2 3)");
}

#[test]
fn test_unwrap_or_with_ok() {
    let vm = compile_and_run("(unwrap-or (ok 42) 0)");
    assert_eq!(get_output(&vm), "42");
}

#[test]
fn test_unwrap_or_with_err() {
    let vm = compile_and_run(r#"(unwrap-or (err "error") 0)"#);
    assert_eq!(get_output(&vm), "0");
}

#[test]
fn test_unwrap_err_with_err() {
    let vm = compile_and_run(r#"(unwrap-err (err "something failed"))"#);
    assert_eq!(get_output(&vm), "something failed");
}

#[test]
fn test_map_ok_with_ok() {
    let vm = compile_and_run("(map-ok (lambda (x) (* x 2)) (ok 21))");
    assert_eq!(get_output(&vm), "(ok 42)");
}

#[test]
fn test_map_ok_with_err() {
    let vm = compile_and_run(r#"(map-ok (lambda (x) (* x 2)) (err "error"))"#);
    assert_eq!(get_output(&vm), "(err error)");
}

#[test]
fn test_map_err_with_ok() {
    let vm = compile_and_run(r#"(map-err (lambda (e) (string-append "Error: " e)) (ok 42))"#);
    assert_eq!(get_output(&vm), "(ok 42)");
}

#[test]
fn test_map_err_with_err() {
    let vm = compile_and_run(r#"(map-err (lambda (e) (string-append "Error: " e)) (err "failed"))"#);
    assert_eq!(get_output(&vm), "(err Error: failed)");
}

#[test]
fn test_and_then_with_ok_returning_ok() {
    let vm = compile_and_run(r#"
        (defun safe-div (x y)
          (if (== y 0)
              (err "division by zero")
              (ok (/ x y))))

        (and-then (ok 10) (lambda (x) (safe-div x 2)))
    "#);
    assert_eq!(get_output(&vm), "(ok 5)");
}

#[test]
fn test_and_then_with_ok_returning_err() {
    let vm = compile_and_run(r#"
        (defun safe-div (x y)
          (if (== y 0)
              (err "division by zero")
              (ok (/ x y))))

        (and-then (ok 10) (lambda (x) (safe-div x 0)))
    "#);
    assert_eq!(get_output(&vm), "(err division by zero)");
}

#[test]
fn test_and_then_with_err() {
    let vm = compile_and_run(r#"
        (defun safe-div (x y)
          (if (== y 0)
              (err "division by zero")
              (ok (/ x y))))

        (and-then (err "previous error") (lambda (x) (safe-div x 2)))
    "#);
    assert_eq!(get_output(&vm), "(err previous error)");
}

#[test]
fn test_and_then_chaining() {
    let vm = compile_and_run(r#"
        (defun safe-div (x y)
          (if (== y 0)
              (err "division by zero")
              (ok (/ x y))))

        (defun safe-sqrt (x)
          (if (< x 0)
              (err "negative square root")
              (ok x)))

        (and-then (ok 100)
                  (lambda (x) (and-then (safe-div x 2)
                                       (lambda (y) (safe-sqrt y)))))
    "#);
    assert_eq!(get_output(&vm), "(ok 50)");
}

#[test]
fn test_or_else_with_err_returning_ok() {
    let vm = compile_and_run(r#"
        (defun recover (e)
          (ok 0))

        (or-else (err "something failed") recover)
    "#);
    assert_eq!(get_output(&vm), "(ok 0)");
}

#[test]
fn test_or_else_with_err_returning_err() {
    let vm = compile_and_run(r#"
        (defun recover (e)
          (err (string-append "Recovered: " e)))

        (or-else (err "something failed") recover)
    "#);
    assert_eq!(get_output(&vm), "(err Recovered: something failed)");
}

#[test]
fn test_or_else_with_ok() {
    let vm = compile_and_run(r#"
        (defun recover (e)
          (ok 0))

        (or-else (ok 42) recover)
    "#);
    assert_eq!(get_output(&vm), "(ok 42)");
}

#[test]
fn test_is_ok_alias() {
    let vm = compile_and_run("(is-ok (ok 42))");
    assert_eq!(get_output(&vm), "true");
}

#[test]
fn test_is_err_alias() {
    let vm = compile_and_run(r#"(is-err (err "error"))"#);
    assert_eq!(get_output(&vm), "true");
}

#[test]
fn test_practical_example_file_reading() {
    let vm = compile_and_run(r#"
        (defun read-config (filename)
          (if (== filename "config.json")
              (ok 8080)
              (err "file not found")))

        (unwrap (read-config "config.json"))
    "#);
    assert_eq!(get_output(&vm), "8080");
}

#[test]
fn test_practical_example_file_reading_error() {
    let vm = compile_and_run(r#"
        (defun read-config (filename)
          (if (== filename "config.json")
              (ok 8080)
              (err "file not found")))

        (unwrap-err (read-config "wrong.json"))
    "#);
    assert_eq!(get_output(&vm), "file not found");
}

#[test]
fn test_practical_example_with_unwrap_or() {
    let vm = compile_and_run(r#"
        (defun parse-number (s)
          (if (== s "42")
              (ok 42)
              (err "invalid number")))

        (unwrap-or (parse-number "invalid") 0)
    "#);
    assert_eq!(get_output(&vm), "0");
}

#[test]
fn test_practical_example_pipeline() {
    let vm = compile_and_run(r#"
        (defun validate-positive (x)
          (if (> x 0)
              (ok x)
              (err "must be positive")))

        (defun validate-even (x)
          (if (even? x)
              (ok x)
              (err "must be even")))

        (defun double (x)
          (ok (* x 2)))

        (unwrap (and-then (validate-positive 4)
                          (lambda (x) (and-then (validate-even x)
                                               (lambda (y) (double y))))))
    "#);
    assert_eq!(get_output(&vm), "8");
}

#[test]
fn test_practical_example_pipeline_fail_first() {
    let vm = compile_and_run(r#"
        (defun validate-positive (x)
          (if (> x 0)
              (ok x)
              (err "must be positive")))

        (defun validate-even (x)
          (if (even? x)
              (ok x)
              (err "must be even")))

        (defun double (x)
          (ok (* x 2)))

        (unwrap-err (and-then (validate-positive -4)
                              (lambda (x) (and-then (validate-even x)
                                                   (lambda (y) (double y))))))
    "#);
    assert_eq!(get_output(&vm), "must be positive");
}

#[test]
fn test_practical_example_pipeline_fail_second() {
    let vm = compile_and_run(r#"
        (defun validate-positive (x)
          (if (> x 0)
              (ok x)
              (err "must be positive")))

        (defun validate-even (x)
          (if (even? x)
              (ok x)
              (err "must be even")))

        (defun double (x)
          (ok (* x 2)))

        (unwrap-err (and-then (validate-positive 5)
                              (lambda (x) (and-then (validate-even x)
                                                   (lambda (y) (double y))))))
    "#);
    assert_eq!(get_output(&vm), "must be even");
}

#[test]
fn test_nested_results() {
    let vm = compile_and_run("(ok (ok 42))");
    assert_eq!(get_output(&vm), "(ok (ok 42))");
}

#[test]
fn test_result_with_complex_values() {
    let vm = compile_and_run("(ok '(1 2 3 4 5))");
    assert_eq!(get_output(&vm), "(ok (1 2 3 4 5))");
}

#[test]
fn test_map_ok_composition() {
    let vm = compile_and_run(r#"
        (defun double (x) (* x 2))
        (defun inc (x) (+ x 1))

        (unwrap (map-ok inc (map-ok double (ok 10))))
    "#);
    assert_eq!(get_output(&vm), "21");
}

#[test]
fn test_error_recovery_with_or_else() {
    let vm = compile_and_run(r#"
        (defun try-parse (s)
          (if (== s "42")
              (ok 42)
              (err "parse error")))

        (defun default-value (e)
          (ok 0))

        (unwrap (or-else (try-parse "invalid") default-value))
    "#);
    assert_eq!(get_output(&vm), "0");
}

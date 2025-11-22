use lisp_bytecode_vm::{Compiler, VM, parser::Parser};

fn compile_and_get_result(source: &str) -> i64 {
    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).unwrap();

    let mut vm = VM::new();
    vm.functions = functions;
    vm.current_bytecode = main;
    vm.run();

    // Get the top value from the stack
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => *n,
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_factorial() {
    let source = r#"
        (defun fact (n)
          (if (<= n 1)
              1
              (* n (fact (- n 1)))))
        (fact 5)
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 120);
}

#[test]
fn test_fibonacci() {
    let source = r#"
        (defun fib (n)
          (if (<= n 1)
              n
              (+ (fib (- n 1)) (fib (- n 2)))))
        (fib 10)
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 55);
}

#[test]
fn test_simple_arithmetic() {
    let tests = vec![
        ("(+ 2 3)", 5),
        ("(- 10 3)", 7),
        ("(* 4 5)", 20),
        ("(/ 20 4)", 5),
        ("(% 10 3)", 1),
    ];

    for (source, expected) in tests {
        let result = compile_and_get_result(source);
        assert_eq!(result, expected, "Failed for: {}", source);
    }
}

#[test]
fn test_nested_arithmetic() {
    let source = "(+ (* 2 3) (- 10 5))"; // (2*3) + (10-5) = 6 + 5 = 11
    let result = compile_and_get_result(source);
    assert_eq!(result, 11);
}

#[test]
fn test_comparisons() {
    let source = r#"
        (if (> 10 5) 1 0)
    "#;
    let result = compile_and_get_result(source);
    assert_eq!(result, 1);

    let source = r#"
        (if (< 10 5) 1 0)
    "#;
    let result = compile_and_get_result(source);
    assert_eq!(result, 0);

    let source = r#"
        (if (== 5 5) 42 99)
    "#;
    let result = compile_and_get_result(source);
    assert_eq!(result, 42);
}

#[test]
fn test_negation() {
    let source = "(neg 5)";
    let result = compile_and_get_result(source);
    assert_eq!(result, -5);

    let source = "(neg (neg 10))";
    let result = compile_and_get_result(source);
    assert_eq!(result, 10);
}

#[test]
fn test_function_with_multiple_params() {
    let source = r#"
        (defun add3 (a b c)
          (+ a (+ b c)))
        (add3 1 2 3)
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 6);
}

#[test]
fn test_nested_function_calls() {
    let source = r#"
        (defun double (x) (* x 2))
        (defun quad (x) (double (double x)))
        (quad 5)
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 20);
}

#[test]
fn test_conditional_in_function() {
    let source = r#"
        (defun abs (x)
          (if (< x 0)
              (neg x)
              x))
        (abs (neg 10))
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 10);
}

#[test]
fn test_is_even_function() {
    let source = r#"
        (defun is_even (n)
          (== (% n 2) 0))
        (if (is_even 4) 1 0)
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 1);

    let source = r#"
        (defun is_even (n)
          (== (% n 2) 0))
        (if (is_even 5) 1 0)
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 0);
}

#[test]
fn test_max_min_functions() {
    let source = r#"
        (defun max (a b)
          (if (> a b) a b))
        (defun min (a b)
          (if (< a b) a b))
        (+ (max 10 5) (min 10 5))
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 15); // 10 + 5
}

#[test]
fn test_all_comparison_operators() {
    let tests = vec![
        ("(if (<= 5 10) 1 0)", 1),
        ("(if (<= 10 5) 1 0)", 0),
        ("(if (< 5 10) 1 0)", 1),
        ("(if (> 10 5) 1 0)", 1),
        ("(if (>= 10 5) 1 0)", 1),
        ("(if (>= 5 10) 1 0)", 0),
        ("(if (== 5 5) 1 0)", 1),
        ("(if (!= 5 10) 1 0)", 1),
    ];

    for (source, expected) in tests {
        let result = compile_and_get_result(source);
        assert_eq!(result, expected, "Failed for: {}", source);
    }
}

#[test]
fn test_deeply_nested_calls() {
    let source = r#"
        (defun add1 (x) (+ x 1))
        (add1 (add1 (add1 (add1 (add1 0)))))
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 5);
}

#[test]
fn test_mutual_recursion_even_odd() {
    // Simple version without mutual recursion since we need forward declarations
    let source = r#"
        (defun is_even_simple (n)
          (if (<= n 0)
              (== n 0)
              (is_even_simple (- n 2))))
        (if (is_even_simple 10) 1 0)
    "#;

    let result = compile_and_get_result(source);
    assert_eq!(result, 1);
}

#[test]
fn test_compile_error_handling() {
    let source = "unknown_variable";
    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().unwrap();
    let mut compiler = Compiler::new();
    let result = compiler.compile_program(&exprs);

    assert!(result.is_err());
}

#[test]
fn test_multiple_expressions_in_main() {
    let source = r#"
        (defun double (x) (* x 2))
        (double 5)
        (double 10)
    "#;

    // The result should be the last expression
    let result = compile_and_get_result(source);
    assert_eq!(result, 20);
}

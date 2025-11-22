use lisp_bytecode_vm::{repl::Repl, Value};

#[test]
fn test_is_complete_input_simple_expression() {
    let mut repl = Repl::new();
    repl.input_buffer = "(+ 1 2)".to_string();
    assert!(repl.is_complete_input());
}

#[test]
fn test_is_complete_input_incomplete() {
    let mut repl = Repl::new();
    repl.input_buffer = "(+ 1".to_string();
    assert!(!repl.is_complete_input());
}

#[test]
fn test_is_complete_input_nested() {
    let mut repl = Repl::new();
    repl.input_buffer = "(+ (* 2 3) (- 10 5))".to_string();
    assert!(repl.is_complete_input());
}

#[test]
fn test_is_complete_input_multiline_complete() {
    let mut repl = Repl::new();
    repl.input_buffer = "(defun square (x)\n  (* x x))".to_string();
    assert!(repl.is_complete_input());
}

#[test]
fn test_is_complete_input_multiline_incomplete() {
    let mut repl = Repl::new();
    repl.input_buffer = "(defun square (x)\n  (* x x)".to_string();
    assert!(!repl.is_complete_input());
}

#[test]
fn test_is_complete_input_empty() {
    let mut repl = Repl::new();
    repl.input_buffer = "".to_string();
    assert!(!repl.is_complete_input());
}

#[test]
fn test_is_complete_input_whitespace_only() {
    let mut repl = Repl::new();
    repl.input_buffer = "   \n  \t  ".to_string();
    assert!(!repl.is_complete_input());
}

#[test]
fn test_is_complete_input_deeply_nested() {
    let mut repl = Repl::new();
    repl.input_buffer = "(if (> x 0) (if (< x 10) (print x) (print 0)) (print -1))".to_string();
    assert!(repl.is_complete_input());
}

#[test]
fn test_is_complete_input_unbalanced_closing() {
    let mut repl = Repl::new();
    repl.input_buffer = "(+ 1 2))".to_string();
    // This has an extra closing paren, depth goes negative
    // The current implementation only checks depth == 0 at the end,
    // so depth -1 means this is not complete
    assert!(!repl.is_complete_input());
}

#[test]
fn test_format_value_integer() {
    let repl = Repl::new();
    let value = Value::Integer(42);
    assert_eq!(repl.format_value(&value), "42");
}

#[test]
fn test_format_value_boolean_true() {
    let repl = Repl::new();
    let value = Value::Boolean(true);
    assert_eq!(repl.format_value(&value), "true");
}

#[test]
fn test_format_value_boolean_false() {
    let repl = Repl::new();
    let value = Value::Boolean(false);
    assert_eq!(repl.format_value(&value), "false");
}

#[test]
fn test_format_value_negative_integer() {
    let repl = Repl::new();
    let value = Value::Integer(-123);
    assert_eq!(repl.format_value(&value), "-123");
}

#[test]
fn test_clear_state_clears_input_buffer() {
    let mut repl = Repl::new();
    repl.input_buffer = "some input".to_string();

    repl.clear_state();

    assert_eq!(repl.input_buffer.len(), 0);
}

use lisp_bytecode_vm::{Compiler, VM, parser::Parser, Value};
use std::sync::Arc;

/// Helper function to compile and run source code
fn compile_and_run(source: &str) -> VM {
    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).unwrap();

    let mut vm = VM::new();
    vm.functions.extend(functions);
    vm.current_bytecode = main;
    vm.run().unwrap();
    vm
}

/// Helper to get float result from VM
fn get_float_result(vm: &VM) -> f64 {
    match vm.value_stack.last() {
        Some(Value::Float(f)) => *f,
        Some(Value::Integer(n)) => *n as f64,
        _ => panic!("Expected float result, got {:?}", vm.value_stack.last()),
    }
}

/// Helper to get integer result from VM
fn get_int_result(vm: &VM) -> i64 {
    match vm.value_stack.last() {
        Some(Value::Integer(n)) => *n,
        _ => panic!("Expected integer result, got {:?}", vm.value_stack.last()),
    }
}

/// Helper to get string result from VM
fn get_string_result(vm: &VM) -> String {
    match vm.value_stack.last() {
        Some(Value::String(s)) => s.to_string(),
        _ => panic!("Expected string result, got {:?}", vm.value_stack.last()),
    }
}

/// Helper to get list result from VM
fn get_list_result(vm: &VM) -> Vec<Value> {
    match vm.value_stack.last() {
        Some(Value::List(lst)) => lst.to_vec(),
        _ => panic!("Expected list result, got {:?}", vm.value_stack.last()),
    }
}

// ============================================================================
// Math Functions Tests
// ============================================================================

#[test]
fn test_log() {
    let source = "(log 2.718281828459045)"; // e
    let vm = compile_and_run(source);
    let result = get_float_result(&vm);
    assert!((result - 1.0).abs() < 0.0001);
}

#[test]
fn test_exp() {
    let source = "(exp 1)"; // e^1
    let vm = compile_and_run(source);
    let result = get_float_result(&vm);
    assert!((result - 2.718281828459045).abs() < 0.0001);
}

#[test]
fn test_tan() {
    let source = "(tan 0)";
    let vm = compile_and_run(source);
    let result = get_float_result(&vm);
    assert!(result.abs() < 0.0001);
}

#[test]
fn test_atan() {
    let source = "(atan 1)";
    let vm = compile_and_run(source);
    let result = get_float_result(&vm);
    assert!((result - 0.7853981633974483).abs() < 0.0001); // pi/4
}

#[test]
fn test_atan2() {
    let source = "(atan2 1 1)";
    let vm = compile_and_run(source);
    let result = get_float_result(&vm);
    assert!((result - 0.7853981633974483).abs() < 0.0001); // pi/4
}

#[test]
fn test_random() {
    let source = "(random)";
    let vm = compile_and_run(source);
    let result = get_float_result(&vm);
    assert!(result >= 0.0 && result < 1.0);
}

#[test]
fn test_random_int() {
    let source = "(random-int 10)";
    let vm = compile_and_run(source);
    let result = get_int_result(&vm);
    assert!(result >= 0 && result < 10);
}

#[test]
fn test_seed_random() {
    let source = "(seed-random 42)";
    let vm = compile_and_run(source);
    let result = get_int_result(&vm);
    assert_eq!(result, 42);
}

// ============================================================================
// String Functions Tests
// ============================================================================

#[test]
fn test_string_split() {
    let source = r#"(string-split "hello,world,test" ",")"#;
    let vm = compile_and_run(source);
    let result = get_list_result(&vm);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Value::String(Arc::new("hello".to_string())));
    assert_eq!(result[1], Value::String(Arc::new("world".to_string())));
    assert_eq!(result[2], Value::String(Arc::new("test".to_string())));
}

#[test]
fn test_string_split_empty_delimiter() {
    let source = r#"(string-split "abc" "")"#;
    let vm = compile_and_run(source);
    let result = get_list_result(&vm);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Value::String(Arc::new("a".to_string())));
    assert_eq!(result[1], Value::String(Arc::new("b".to_string())));
    assert_eq!(result[2], Value::String(Arc::new("c".to_string())));
}

#[test]
fn test_string_join() {
    let source = r#"(string-join (list "hello" "world" "test") ",")"#;
    let vm = compile_and_run(source);
    let result = get_string_result(&vm);
    assert_eq!(result, "hello,world,test");
}

#[test]
fn test_string_trim() {
    let source = r#"(string-trim "  hello world  ")"#;
    let vm = compile_and_run(source);
    let result = get_string_result(&vm);
    assert_eq!(result, "hello world");
}

#[test]
fn test_string_replace() {
    let source = r#"(string-replace "hello world" "world" "rust")"#;
    let vm = compile_and_run(source);
    let result = get_string_result(&vm);
    assert_eq!(result, "hello rust");
}

// ============================================================================
// Date/Time Functions Tests
// ============================================================================

#[test]
fn test_current_timestamp() {
    let source = "(current-timestamp)";
    let vm = compile_and_run(source);
    let result = get_int_result(&vm);
    // Check that timestamp is reasonable (after Jan 1, 2020 and before Jan 1, 2030)
    assert!(result > 1577836800); // Jan 1, 2020
    assert!(result < 1893456000); // Jan 1, 2030
}

#[test]
fn test_format_timestamp() {
    let source = r#"(format-timestamp 1609459200 "%Y-%m-%d")"#; // 2021-01-01 00:00:00 UTC
    let vm = compile_and_run(source);
    let result = get_string_result(&vm);
    assert_eq!(result, "2021-01-01");
}

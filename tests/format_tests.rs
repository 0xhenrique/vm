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
        Some(Value::String(s)) => Ok(s.to_string()),
        Some(value) => Ok(format!("{:?}", value)),
        None => Err("No value on stack".to_string()),
    }
}

#[test]
fn test_format_simple_string() {
    let source = r#"
        (format "Hello, {}!" (list "world"))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "Hello, world!");
}

#[test]
fn test_format_multiple_placeholders() {
    let source = r#"
        (format "x={}, y={}" (list 10 20))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "x=10, y=20");
}

#[test]
fn test_format_no_placeholders() {
    let source = r#"
        (format "Just a string" (list))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "Just a string");
}

#[test]
fn test_format_single_placeholder() {
    let source = r#"
        (format "The answer is {}" (list 42))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "The answer is 42");
}

#[test]
fn test_format_with_boolean() {
    let source = r#"
        (format "Success: {}" (list true))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "Success: true");
}

#[test]
fn test_format_with_float() {
    let source = r#"
        (format "Pi is approximately {}" (list 3.14))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "Pi is approximately 3.14");
}

#[test]
fn test_format_mixed_types() {
    let source = r#"
        (format "String: {}, Number: {}, Bool: {}" (list "test" 123 false))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "String: test, Number: 123, Bool: false");
}

#[test]
fn test_format_with_list() {
    let source = r#"
        (format "List: {}" (list (list 1 2 3)))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "List: (1 2 3)");
}

#[test]
fn test_format_escape_braces() {
    let source = r#"
        (format "Not a placeholder: { }" (list))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "Not a placeholder: { }");
}

#[test]
fn test_format_with_computation() {
    let source = r#"
        (format "Sum: {}" (list (+ 10 20)))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "Sum: 30");
}

#[test]
fn test_format_error_not_enough_args() {
    let source = r#"
        (format "x={}, y={}" (list 10))
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not enough arguments"));
}

#[test]
fn test_format_error_wrong_type() {
    let source = r#"
        (format "Hello" 42)
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects a list"));
}

#[test]
fn test_format_in_function() {
    let source = r#"
        (defun greet (name age)
            (format "Hello, {}! You are {} years old." (list name age)))

        (greet "Alice" 30)
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "Hello, Alice! You are 30 years old.");
}

#[test]
fn test_format_nested() {
    let source = r#"
        (format "Outer: {}" (list (format "Inner: {}" (list 42))))
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result, "Outer: Inner: 42");
}

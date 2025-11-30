use lisp_bytecode_vm::{Compiler, VM, parser::Parser};

/// Helper function to compile Lisp code and capture compile errors
fn compile_lisp(source: &str) -> Result<(), String> {
    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().map_err(|e| format!("{:?}", e))?;
    let mut compiler = Compiler::new();
    let (_functions, _main) = compiler.compile_program(&exprs).map_err(|e| e.format(Some(source)))?;
    Ok(())
}

/// Helper function to compile and run Lisp code, capturing runtime errors
fn compile_and_run(source: &str) -> Result<String, String> {
    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().map_err(|e| format!("{:?}", e))?;

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).map_err(|e| e.format(Some(source)))?;

    let mut vm = VM::new();
    for (name, bytecode) in functions {
        vm.functions.insert(name, bytecode);
    }
    vm.current_bytecode = main;
    vm.run().map_err(|e| e.format())?;

    Ok(String::new())
}

#[test]
fn test_undefined_variable_suggestion() {
    let source = r#"
        (defun test (correct-name)
            (print corect-name))
    "#;

    let result = compile_lisp(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    // Should suggest the correct name
    assert!(error.contains("Undefined variable"));
    assert!(error.contains("Suggestion"));
    assert!(error.contains("correct-name"));
}

#[test]
fn test_undefined_variable_no_match_suggestion() {
    let source = r#"
        (defun test ()
            (print totally-undefined-variable-xyz))
    "#;

    let result = compile_lisp(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    // Should provide generic suggestion when no close match
    assert!(error.contains("Undefined variable"));
    assert!(error.contains("Suggestion"));
    assert!(error.contains("Make sure the variable is defined"));
}

#[test]
fn test_empty_list_suggestion() {
    let source = r#"
        (defun test () ())
    "#;

    let result = compile_lisp(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert!(error.contains("Empty list cannot be compiled"));
    assert!(error.contains("Suggestion"));
    assert!(error.contains("quote"));
}

#[test]
fn test_addition_arity_suggestion() {
    let source = r#"
        (+ 5)
    "#;

    let result = compile_lisp(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert!(error.contains("+ expects at least 2 arguments"));
    assert!(error.contains("Suggestion"));
    assert!(error.contains("(+ 1 2)"));
}

#[test]
fn test_subtraction_arity_suggestion() {
    let source = r#"
        (- 5)
    "#;

    let result = compile_lisp(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert!(error.contains("- expects at least 2 arguments"));
    assert!(error.contains("Suggestion"));
    assert!(error.contains("(- 10 3)"));
}

#[test]
fn test_dotted_list_suggestion() {
    let source = r#"
        (def x (1 . 2))
    "#;

    let result = compile_lisp(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert!(error.contains("Dotted lists can only be used in patterns"));
    assert!(error.contains("Suggestion"));
    assert!(error.contains("cons"));
}

#[test]
fn test_division_by_zero_suggestion() {
    let source = r#"
        (/ 10 0)
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert!(error.contains("Division by zero"));
    assert!(error.contains("Suggestion"));
    assert!(error.contains("Check your divisor"));
    assert!(error.contains("if-expression"));
}

#[test]
fn test_modulo_by_zero_suggestion() {
    let source = r#"
        (% 10 0)
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert!(error.contains("Modulo by zero"));
    assert!(error.contains("Suggestion"));
    assert!(error.contains("Check your divisor"));
}

#[test]
fn test_suggestion_formatting() {
    // Test that suggestions are properly word-wrapped and formatted
    let source = r#"
        (print undefined-var)
    "#;

    let result = compile_lisp(source);
    assert!(result.is_err());
    let error = result.unwrap_err();

    // Should contain the formatted suggestion section
    assert!(error.contains("├─ Suggestion"));
    assert!(error.contains("│"));
    assert!(error.contains("╰─"));
}

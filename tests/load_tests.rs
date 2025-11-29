use lisp_bytecode_vm::{Compiler, VM, parser::Parser, Value};
use std::fs;

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

/// Helper to get integer result from VM
fn get_int_result(vm: &VM) -> i64 {
    match vm.value_stack.last() {
        Some(Value::Integer(n)) => *n,
        _ => panic!("Expected integer result"),
    }
}

/// Helper to get boolean result from VM
fn get_bool_result(vm: &VM) -> bool {
    match vm.value_stack.last() {
        Some(Value::Boolean(b)) => *b,
        _ => panic!("Expected boolean result"),
    }
}

#[test]
fn test_load_basic() {
    // Create a temporary file to load
    let lib_content = r#"
        (defun add-ten (x)
          (+ x 10))
    "#;
    fs::write("/tmp/test-lib-basic.lisp", lib_content).unwrap();

    let source = r#"
        (load "/tmp/test-lib-basic.lisp")
        (add-ten 5)
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 15);
}

#[test]
fn test_load_multiple_functions() {
    let lib_content = r#"
        (defun square (x)
          (* x x))

        (defun cube (x)
          (* x (* x x)))
    "#;
    fs::write("/tmp/test-lib-multi.lisp", lib_content).unwrap();

    let source = r#"
        (load "/tmp/test-lib-multi.lisp")
        (+ (square 5) (cube 2))
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 33); // 25 + 8 = 33
}

#[test]
fn test_load_with_closures() {
    let lib_content = r#"
        (defun make-adder (n)
          (lambda (x) (+ x n)))
    "#;
    fs::write("/tmp/test-lib-closure.lisp", lib_content).unwrap();

    let source = r#"
        (load "/tmp/test-lib-closure.lisp")
        ((make-adder 100) 42)
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 142);
}

#[test]
fn test_load_returns_true() {
    let lib_content = r#"
        (defun dummy () 42)
    "#;
    fs::write("/tmp/test-lib-return.lisp", lib_content).unwrap();

    let source = r#"
        (load "/tmp/test-lib-return.lisp")
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), true);
}

#[test]
fn test_load_file_not_found() {
    let source = r#"
        (load "/tmp/nonexistent-file-12345.lisp")
    "#;

    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).unwrap();

    let mut vm = VM::new();
    vm.functions.extend(functions);
    vm.current_bytecode = main;

    let result = vm.run();
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("failed to read"));
}

#[test]
fn test_load_with_global_vars() {
    let lib_content = r#"
        (def *greeting* "Hello")

        (defun greet (name)
          (string-append *greeting* (string-append " " name)))
    "#;
    fs::write("/tmp/test-lib-globals.lisp", lib_content).unwrap();

    let source = r#"
        (load "/tmp/test-lib-globals.lisp")
        (greet "World")
    "#;

    let vm = compile_and_run(source);
    match vm.value_stack.last() {
        Some(Value::String(s)) => assert_eq!(s.as_str(), "Hello World"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_load_with_lists() {
    let lib_content = r#"
        (defun list-sum (lst)
          (if (null? lst)
              0
              (+ (car lst) (list-sum (cdr lst)))))
    "#;
    fs::write("/tmp/test-lib-lists.lisp", lib_content).unwrap();

    let source = r#"
        (load "/tmp/test-lib-lists.lisp")
        (list-sum '(1 2 3 4 5))
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 15);
}

// ============================================================================
// REQUIRE TESTS (with caching)
// ============================================================================

#[test]
fn test_require_basic() {
    // Create a temporary file to require
    let lib_content = r#"
        (defun multiply-by-five (x)
          (* x 5))
    "#;
    fs::write("/tmp/test-require-basic.lisp", lib_content).unwrap();

    let source = r#"
        (require "/tmp/test-require-basic.lisp")
        (multiply-by-five 7)
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 35);
}

#[test]
fn test_require_returns_true() {
    let lib_content = r#"
        (defun helper () 42)
    "#;
    fs::write("/tmp/test-require-return.lisp", lib_content).unwrap();

    let source = r#"
        (require "/tmp/test-require-return.lisp")
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), true);
}

#[test]
fn test_require_caching_no_redefine() {
    // This test verifies that require caches and doesn't reload
    // With immutable bindings, if require loads the file multiple times,
    // it will error trying to redefine the constant.
    // If it caches properly, no error should occur.

    let lib_content = r#"
        (def *lib-loaded* true)
        (defun lib-function () 42)
    "#;
    fs::write("/tmp/test-require-cache.lisp", lib_content).unwrap();

    let source = r#"
        ; First require - loads the file
        (require "/tmp/test-require-cache.lisp")

        ; Second require - should be cached, won't try to redefine *lib-loaded*
        (require "/tmp/test-require-cache.lisp")

        ; Third require - cached again
        (require "/tmp/test-require-cache.lisp")

        ; Return result - if we got here without error, caching works!
        (lib-function)
    "#;

    let vm = compile_and_run(source);
    match vm.value_stack.last() {
        Some(Value::Integer(n)) => assert_eq!(*n, 42), // Success - no redefinition error!
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_require_vs_load() {
    // This test demonstrates the difference between require and load
    // by directly checking the VM's loaded_modules set

    let lib_content = r#"
        (defun test-function () 42)
    "#;
    fs::write("/tmp/test-caching-behavior.lisp", lib_content).unwrap();

    // Test 1: Multiple load calls - loaded_modules should be empty (load doesn't cache)
    let source_load = r#"
        (load "/tmp/test-caching-behavior.lisp")
        (load "/tmp/test-caching-behavior.lisp")
        (load "/tmp/test-caching-behavior.lisp")
        (test-function)
    "#;

    let vm_load = compile_and_run(source_load);
    // load doesn't use the loaded_modules cache
    assert_eq!(vm_load.loaded_modules.len(), 0);
    assert_eq!(get_int_result(&vm_load), 42);

    // Test 2: Multiple require calls - loaded_modules should contain 1 entry
    let source_require = r#"
        (require "/tmp/test-caching-behavior.lisp")
        (require "/tmp/test-caching-behavior.lisp")
        (require "/tmp/test-caching-behavior.lisp")
        (test-function)
    "#;

    let vm_require = compile_and_run(source_require);
    // require caches the module
    assert_eq!(vm_require.loaded_modules.len(), 1);
    assert!(vm_require.loaded_modules.iter().any(|p| p.contains("test-caching-behavior.lisp")));
    assert_eq!(get_int_result(&vm_require), 42);
}

#[test]
fn test_require_multiple_files() {
    let lib1_content = r#"
        (defun add-100 (x)
          (+ x 100))
    "#;
    fs::write("/tmp/test-require-lib1.lisp", lib1_content).unwrap();

    let lib2_content = r#"
        (defun multiply-by-2 (x)
          (* x 2))
    "#;
    fs::write("/tmp/test-require-lib2.lisp", lib2_content).unwrap();

    let source = r#"
        (require "/tmp/test-require-lib1.lisp")
        (require "/tmp/test-require-lib2.lisp")
        (multiply-by-2 (add-100 5))
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 210); // (5 + 100) * 2 = 210
}

#[test]
fn test_require_file_not_found() {
    let source = r#"
        (require "/tmp/nonexistent-require-file-99999.lisp")
    "#;

    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).unwrap();

    let mut vm = VM::new();
    vm.functions.extend(functions);
    vm.current_bytecode = main;

    let result = vm.run();
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("'require' failed to read"));
}

#[test]
fn test_require_idempotent() {
    // Verify that requiring the same file many times is safe and efficient
    let lib_content = r#"
        (defun test-func () 42)
    "#;
    fs::write("/tmp/test-require-idem.lisp", lib_content).unwrap();

    let source = r#"
        (require "/tmp/test-require-idem.lisp")
        (require "/tmp/test-require-idem.lisp")
        (require "/tmp/test-require-idem.lisp")
        (require "/tmp/test-require-idem.lisp")
        (require "/tmp/test-require-idem.lisp")
        (test-func)
    "#;

    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 42);
    // Also verify that the module was only loaded once
    assert_eq!(vm.loaded_modules.len(), 1);
}

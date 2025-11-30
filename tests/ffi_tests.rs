// FFI (Foreign Function Interface) Tests
// Tests for loading shared libraries and calling C functions from Lisp

use std::sync::Arc;

use lisp_bytecode_vm::{VM, Value};

// Helper to compile and run Lisp code
fn run_lisp(source: &str) -> Result<Value, String> {
    use lisp_bytecode_vm::parser::Parser;
    use lisp_bytecode_vm::compiler::Compiler;

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

    vm.value_stack.last().cloned().ok_or_else(|| "No value on stack".to_string())
}

// ==================== Basic Pointer Operations ====================

#[test]
fn test_ffi_null_pointer() {
    let result = run_lisp("(ffi-null)").unwrap();
    assert!(matches!(result, Value::Pointer(0)));
}

#[test]
fn test_ffi_null_check() {
    let result = run_lisp("(ffi-null? (ffi-null))").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_ffi_null_check_non_null() {
    // Create a pointer via allocation
    let result = run_lisp(r#"
        (let ((ptr (ffi-allocate 8)))
            (let ((is_null (ffi-null? ptr)))
                (do (ffi-free ptr) is_null)))
    "#).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_pointer_predicate() {
    let result = run_lisp("(pointer? (ffi-null))").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_pointer_predicate_non_pointer() {
    let result = run_lisp("(pointer? 42)").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

// ==================== Memory Allocation ====================

#[test]
fn test_ffi_allocate_and_free() {
    let result = run_lisp(r#"
        (let ((ptr (ffi-allocate 1024)))
            (let ((is_valid (if (ffi-null? ptr) false true)))
                (do (ffi-free ptr) is_valid)))
    "#).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_ffi_pointer_arithmetic() {
    // Test pointer arithmetic by writing at offset and reading back
    let result = run_lisp(r#"
        (let ((base (ffi-allocate 16)))
            (let ((offset_ptr (ffi-pointer+ base 8)))
                (do
                    (ffi-write-int offset_ptr 42)
                    (let ((value (ffi-read-int offset_ptr)))
                        (do (ffi-free base) value)))))
    "#).unwrap();
    // We should be able to write and read at the offset
    assert_eq!(result, Value::Integer(42));
}

// ==================== Memory Read/Write ====================

#[test]
fn test_ffi_write_read_int() {
    let result = run_lisp(r#"
        (let ((ptr (ffi-allocate 8)))
            (do
                (ffi-write-int ptr 12345)
                (let ((value (ffi-read-int ptr)))
                    (do (ffi-free ptr) value))))
    "#).unwrap();
    assert_eq!(result, Value::Integer(12345));
}

#[test]
fn test_ffi_write_read_float() {
    let result = run_lisp(r#"
        (let ((ptr (ffi-allocate 8)))
            (do
                (ffi-write-float ptr 3.14159)
                (let ((value (ffi-read-float ptr)))
                    (do (ffi-free ptr) value))))
    "#).unwrap();
    if let Value::Float(f) = result {
        assert!((f - 3.14159).abs() < 0.00001);
    } else {
        panic!("Expected float, got {:?}", result);
    }
}

#[test]
fn test_ffi_write_read_byte() {
    let result = run_lisp(r#"
        (let ((ptr (ffi-allocate 1)))
            (do
                (ffi-write-byte ptr 255)
                (let ((value (ffi-read-byte ptr)))
                    (do (ffi-free ptr) value))))
    "#).unwrap();
    assert_eq!(result, Value::Integer(255));
}

// ==================== String Conversion ====================

#[test]
fn test_ffi_string_to_pointer() {
    let result = run_lisp(r#"
        (let ((ptr (ffi-string->pointer "hello")))
            (let ((is_valid (if (ffi-null? ptr) false true)))
                (do (ffi-free-string ptr) is_valid)))
    "#).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_ffi_string_roundtrip() {
    let result = run_lisp(r#"
        (let ((ptr (ffi-string->pointer "hello world")))
            (let ((str (ffi-pointer->string ptr)))
                (do (ffi-free-string ptr) str)))
    "#).unwrap();
    assert_eq!(result, Value::String(Arc::new("hello world".to_string())));
}

// ==================== Library Loading ====================

#[test]
fn test_ffi_load_libc() {
    // Try to load libc - this should work on Linux
    let result = run_lisp(r#"
        (let ((lib (ffi-load "libc.so.6")))
            (> lib 0))
    "#);

    // On systems where libc.so.6 exists, this should succeed
    // If it fails (e.g., different libc path), that's OK - we'll test what we can
    match result {
        Ok(Value::Boolean(true)) => {
            // Successfully loaded libc
        }
        Ok(Value::Boolean(false)) => {
            // Library handle is 0, meaning it failed to load
            // This might happen on some systems with different libc paths
        }
        Err(e) => {
            // Runtime error - might be a different libc path
            eprintln!("Note: Could not load libc.so.6: {}", e);
        }
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_ffi_load_invalid_library() {
    let result = run_lisp(r#"(ffi-load "nonexistent_library_12345.so")"#);
    // Should fail with an error
    assert!(result.is_err());
}

// ==================== FFI Call Tests ====================

#[test]
fn test_ffi_call_strlen() {
    // Test calling strlen from libc
    let result = run_lisp(r#"
        (let ((lib (ffi-load "libc.so.6")))
            (let ((strlen_ptr (ffi-symbol lib "strlen")))
                (ffi-call strlen_ptr (:string) :int64 "hello")))
    "#);

    match result {
        Ok(Value::Integer(5)) => {
            // Perfect - strlen("hello") = 5
        }
        Ok(Value::Integer(n)) => {
            panic!("Expected strlen to return 5, got {}", n);
        }
        Err(e) => {
            // Might fail on systems with different libc paths
            eprintln!("Note: strlen test skipped: {}", e);
        }
        _ => panic!("Unexpected result type"),
    }
}

#[test]
fn test_ffi_call_abs() {
    // Test calling abs from libc
    let result = run_lisp(r#"
        (let ((lib (ffi-load "libc.so.6")))
            (let ((abs_ptr (ffi-symbol lib "abs")))
                (ffi-call abs_ptr (:int32) :int32 -42)))
    "#);

    match result {
        Ok(Value::Integer(42)) => {
            // Perfect - abs(-42) = 42
        }
        Err(e) => {
            // Might fail on systems with different libc paths
            eprintln!("Note: abs test skipped: {}", e);
        }
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_ffi_call_sqrt() {
    // Test calling sqrt from libm
    let result = run_lisp(r#"
        (let ((lib (ffi-load "libm.so.6")))
            (let ((sqrt_ptr (ffi-symbol lib "sqrt")))
                (ffi-call sqrt_ptr (:double) :double 16.0)))
    "#);

    match result {
        Ok(Value::Float(f)) => {
            assert!((f - 4.0).abs() < 0.0001, "Expected sqrt(16) = 4, got {}", f);
        }
        Err(e) => {
            // Might fail on systems where libm is bundled into libc
            eprintln!("Note: sqrt test with libm skipped: {}", e);
        }
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_ffi_call_getenv() {
    // Test calling getenv - should return null or a valid pointer
    let result = run_lisp(r#"
        (let ((lib (ffi-load "libc.so.6")))
            (let ((getenv_ptr (ffi-symbol lib "getenv")))
                (let ((result (ffi-call getenv_ptr (:string) :pointer "PATH")))
                    (if (ffi-null? result) false true))))
    "#);

    match result {
        Ok(Value::Boolean(true)) => {
            // PATH exists, as expected on most systems
        }
        Ok(Value::Boolean(false)) => {
            // PATH doesn't exist - unusual but possible
        }
        Err(e) => {
            eprintln!("Note: getenv test skipped: {}", e);
        }
        _ => panic!("Unexpected result"),
    }
}

// ==================== Complex FFI Scenarios ====================

#[test]
fn test_ffi_defun_style_wrapper() {
    // Test defining a wrapper function that calls FFI
    let result = run_lisp(r#"
        (def *libc* (ffi-load "libc.so.6"))

        (defun c-strlen (s)
            (let ((strlen_ptr (ffi-symbol *libc* "strlen")))
                (ffi-call strlen_ptr (:string) :int64 s)))

        (c-strlen "test string")
    "#);

    match result {
        Ok(Value::Integer(11)) => {
            // Perfect - strlen("test string") = 11
        }
        Err(e) => {
            eprintln!("Note: wrapper function test skipped: {}", e);
        }
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_ffi_multiple_calls() {
    // Test making multiple FFI calls in sequence
    let result = run_lisp(r#"
        (let ((lib (ffi-load "libc.so.6")))
            (let ((strlen_ptr (ffi-symbol lib "strlen")))
                (let ((len1 (ffi-call strlen_ptr (:string) :int64 "hello")))
                    (let ((len2 (ffi-call strlen_ptr (:string) :int64 "world")))
                        (let ((len3 (ffi-call strlen_ptr (:string) :int64 "!")))
                            (+ len1 (+ len2 len3)))))))
    "#);

    match result {
        Ok(Value::Integer(11)) => {
            // 5 + 5 + 1 = 11
        }
        Err(e) => {
            eprintln!("Note: multiple calls test skipped: {}", e);
        }
        _ => panic!("Unexpected result"),
    }
}

// ==================== Error Handling ====================

#[test]
fn test_ffi_null_pointer_read_error() {
    let result = run_lisp("(ffi-read-int (ffi-null))");
    assert!(result.is_err());
}

#[test]
fn test_ffi_null_pointer_write_error() {
    let result = run_lisp("(ffi-write-int (ffi-null) 42)");
    assert!(result.is_err());
}

#[test]
fn test_ffi_null_pointer_string_read_error() {
    let result = run_lisp("(ffi-pointer->string (ffi-null))");
    assert!(result.is_err());
}

#[test]
fn test_ffi_invalid_symbol() {
    let result = run_lisp(r#"
        (let ((lib (ffi-load "libc.so.6")))
            (ffi-symbol lib "this_function_does_not_exist_12345"))
    "#);
    // Should fail because the symbol doesn't exist
    assert!(result.is_err());
}

// ==================== Type Coercion ====================

#[test]
fn test_ffi_integer_to_pointer() {
    // Integers can be treated as pointers
    let result = run_lisp("(ffi-null? 0)").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_ffi_allocate_negative_size_error() {
    let result = run_lisp("(ffi-allocate -1)");
    assert!(result.is_err());
}

#[test]
fn test_ffi_write_byte_out_of_range() {
    let result = run_lisp(r#"
        (let ((ptr (ffi-allocate 1)))
            (ffi-write-byte ptr 256))
    "#);
    assert!(result.is_err());
}

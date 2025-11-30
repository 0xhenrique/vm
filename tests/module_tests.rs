use lisp_bytecode_vm::{Compiler, VM, parser::Parser, Value};
use std::sync::Arc;

fn compile_and_run(source: &str) -> Result<VM, String> {
    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).map_err(|e| format!("Compile error: {:?}", e))?;

    let mut vm = VM::new();
    // Merge user-defined functions with builtins
    for (name, bytecode) in functions {
        vm.functions.insert(name, bytecode);
    }
    // Merge module exports
    for (module, exports) in compiler.module_exports {
        vm.module_exports.insert(module, exports);
    }
    vm.current_bytecode = main;
    vm.run().map_err(|e| format!("Runtime error: {:?}", e))?;

    Ok(vm)
}

fn get_stack_top(vm: &VM) -> Option<Value> {
    vm.value_stack.last().cloned()
}

// ==================== BASIC MODULE TESTS ====================

#[test]
fn test_module_basic_definition() {
    let source = r#"
        (module math
            (export add)
            (defun add (x y) (+ x y)))

        (math/add 1 2)
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(3)));
}

#[test]
fn test_module_multiple_exports() {
    let source = r#"
        (module math
            (export add subtract multiply)
            (defun add (x y) (+ x y))
            (defun subtract (x y) (- x y))
            (defun multiply (x y) (* x y)))

        (let ((a (math/add 10 5))
              (b (math/subtract 10 5))
              (c (math/multiply 10 5)))
          (+ a (+ b c)))
    "#;

    let vm = compile_and_run(source).unwrap();
    // a=15, b=5, c=50, result = 15 + 5 + 50 = 70
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(70)));
}

#[test]
fn test_module_private_functions() {
    // Private function should be accessible within the module
    let source = r#"
        (module math
            (export double)
            (defun helper (x) (+ x x))
            (defun double (x) (helper x)))

        (math/double 21)
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(42)));
}

#[test]
fn test_module_private_function_qualified_access() {
    // Note: Private functions ARE accessible via qualified names (like Python's underscore convention)
    // This is by design - the module system uses qualified names for all functions.
    // Privacy is by convention (only exported functions are "officially public")
    let source = r#"
        (module math
            (export double)
            (defun helper (x) (+ x x))
            (defun double (x) (helper x)))

        (math/helper 21)
    "#;

    // Private functions can be accessed via qualified names
    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(42)));
}

// ==================== IMPORT TESTS ====================

#[test]
fn test_import_module() {
    let source = r#"
        (module math
            (export add subtract)
            (defun add (x y) (+ x y))
            (defun subtract (x y) (- x y)))

        (import math)
        (math/add 10 5)
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(15)));
}

#[test]
fn test_import_selective() {
    let source = r#"
        (module math
            (export add subtract)
            (defun add (x y) (+ x y))
            (defun subtract (x y) (- x y)))

        (import math add)
        (add 10 5)
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(15)));
}

#[test]
fn test_import_multiple_symbols() {
    let source = r#"
        (module math
            (export add subtract multiply)
            (defun add (x y) (+ x y))
            (defun subtract (x y) (- x y))
            (defun multiply (x y) (* x y)))

        (import math add multiply)
        (+ (add 2 3) (multiply 4 5))
    "#;

    let vm = compile_and_run(source).unwrap();
    // (add 2 3) = 5, (multiply 4 5) = 20, result = 25
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(25)));
}

#[test]
fn test_import_non_exported_fails() {
    let source = r#"
        (module math
            (export add)
            (defun add (x y) (+ x y))
            (defun private-fn (x) x))

        (import math private-fn)
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err(), "Importing non-exported symbol should fail");
}

// ==================== MODULE GLOBALS (def) TESTS ====================

#[test]
fn test_module_with_def() {
    let source = r#"
        (module constants
            (export pi)
            (def pi 314))

        constants/pi
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(314)));
}

#[test]
fn test_module_def_used_by_function() {
    let source = r#"
        (module circle
            (export area)
            (def pi 314)
            (defun area (r) (* pi (* r r))))

        (circle/area 10)
    "#;

    let vm = compile_and_run(source).unwrap();
    // pi=314, r=10, area = 314 * 100 = 31400
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(31400)));
}

// ==================== MULTIPLE MODULES TESTS ====================

#[test]
fn test_multiple_modules() {
    let source = r#"
        (module math
            (export add)
            (defun add (x y) (+ x y)))

        (module strings
            (export greet)
            (defun greet (name) (string-append "Hello, " name)))

        (let ((sum (math/add 1 2))
              (msg (strings/greet "World")))
          sum)
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(3)));
}

#[test]
fn test_module_using_another_module() {
    let source = r#"
        (module helpers
            (export double)
            (defun double (x) (+ x x)))

        (module math
            (export quadruple)
            (import helpers double)
            (defun quadruple (x) (double (double x))))

        (math/quadruple 5)
    "#;

    let vm = compile_and_run(source).unwrap();
    // double(5) = 10, double(10) = 20
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(20)));
}

// ==================== PATTERN MATCHING IN MODULES ====================

#[test]
fn test_module_with_pattern_matching() {
    // Test simple pattern matching (without variadic patterns, which have limitations)
    let source = r#"
        (module utils
            (export classify)
            (defun classify
                ((0) "zero")
                ((1) "one")
                ((_) "other")))

        (list (utils/classify 0) (utils/classify 1) (utils/classify 42))
    "#;

    let vm = compile_and_run(source).unwrap();
    match get_stack_top(&vm) {
        Some(Value::List(lst)) => {
            let values: Vec<Value> = lst.to_vec();
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], Value::String(std::sync::Arc::new("zero".to_string())));
            assert_eq!(values[1], Value::String(std::sync::Arc::new("one".to_string())));
            assert_eq!(values[2], Value::String(std::sync::Arc::new("other".to_string())));
        }
        _ => panic!("Expected a list"),
    }
}

// ==================== EXPORT SYNTAX VARIATIONS ====================

#[test]
fn test_export_list_syntax() {
    // Test (export (sym1 sym2)) syntax
    let source = r#"
        (module math
            (export (add subtract))
            (defun add (x y) (+ x y))
            (defun subtract (x y) (- x y)))

        (+ (math/add 5 3) (math/subtract 5 3))
    "#;

    let vm = compile_and_run(source).unwrap();
    // add(5,3)=8, subtract(5,3)=2, result=10
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(10)));
}

#[test]
fn test_export_single_symbol() {
    let source = r#"
        (module math
            (export add)
            (defun add (x y) (+ x y)))

        (math/add 7 8)
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(15)));
}

// ==================== ERROR HANDLING TESTS ====================

#[test]
fn test_nested_modules_error() {
    let source = r#"
        (module outer
            (export foo)
            (module inner
                (export bar)
                (defun bar (x) x))
            (defun foo (x) x))
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err(), "Nested modules should not be allowed");
}

#[test]
fn test_import_unknown_module_error() {
    let source = r#"
        (import nonexistent)
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err(), "Importing unknown module should fail");
}

// ==================== CLOSURES IN MODULES ====================

#[test]
fn test_module_returns_closure() {
    let source = r#"
        (module math
            (export make-adder)
            (defun make-adder (n)
                (lambda (x) (+ x n))))

        (let ((add5 (math/make-adder 5)))
            (add5 10))
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(15)));
}

// ==================== MODULE EXPORTS TRACKING ====================

#[test]
fn test_module_exports_tracked() {
    let source = r#"
        (module math
            (export add subtract multiply)
            (defun add (x y) (+ x y))
            (defun subtract (x y) (- x y))
            (defun multiply (x y) (* x y)))

        (math/add 1 1)
    "#;

    let vm = compile_and_run(source).unwrap();

    // Check that module exports are tracked
    assert!(vm.module_exports.contains_key("math"));
    let math_exports = vm.module_exports.get("math").unwrap();
    assert!(math_exports.contains("add"));
    assert!(math_exports.contains("subtract"));
    assert!(math_exports.contains("multiply"));
    assert!(!math_exports.contains("private"));
}

// ==================== FUNCTIONS STORED WITH QUALIFIED NAMES ====================

#[test]
fn test_functions_stored_qualified() {
    let source = r#"
        (module math
            (export add)
            (defun add (x y) (+ x y)))

        (math/add 1 1)
    "#;

    let vm = compile_and_run(source).unwrap();

    // Check that the function is stored with qualified name
    assert!(vm.functions.contains_key("math/add"));
}

// ==================== BUILTINS ACCESSIBLE IN MODULES ====================

#[test]
fn test_builtins_in_module() {
    let source = r#"
        (module utils
            (export double triple)
            (defun double (x) (* x 2))
            (defun triple (x) (* x 3)))

        (+ (utils/double 5) (utils/triple 5))
    "#;

    let vm = compile_and_run(source).unwrap();
    // double(5)=10, triple(5)=15, result=25
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(25)));
}

// ==================== STDLIB FUNCTIONS IN MODULES ====================

#[test]
fn test_stdlib_functions_in_module() {
    let source = r#"
        (module list-ops
            (export len)
            (defun len (lst)
                (list-length lst)))

        (list-ops/len '(1 2 3 4 5))
    "#;

    let vm = compile_and_run(source).unwrap();
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(5)));
}

// ==================== RECURSIVE FUNCTIONS IN MODULES ====================

#[test]
fn test_recursive_function_in_module() {
    let source = r#"
        (module math
            (export factorial)
            (defun factorial (n)
                (if (<= n 1)
                    1
                    (* n (factorial (- n 1))))))

        (math/factorial 5)
    "#;

    let vm = compile_and_run(source).unwrap();
    // 5! = 120
    assert_eq!(get_stack_top(&vm), Some(Value::Integer(120)));
}

// ==================== MUTUAL RECURSION IN MODULES ====================

#[test]
fn test_mutual_recursion_in_module() {
    let source = r#"
        (module parity
            (export even odd)
            (defun even (n)
                (if (== n 0)
                    true
                    (odd (- n 1))))
            (defun odd (n)
                (if (== n 0)
                    false
                    (even (- n 1)))))

        (list (parity/even 4) (parity/odd 4) (parity/even 3) (parity/odd 3))
    "#;

    let vm = compile_and_run(source).unwrap();
    // even(4)=true, odd(4)=false, even(3)=false, odd(3)=true
    match get_stack_top(&vm) {
        Some(Value::List(lst)) => {
            let values: Vec<Value> = lst.to_vec();
            assert_eq!(values.len(), 4);
            assert_eq!(values[0], Value::Boolean(true));
            assert_eq!(values[1], Value::Boolean(false));
            assert_eq!(values[2], Value::Boolean(false));
            assert_eq!(values[3], Value::Boolean(true));
        }
        _ => panic!("Expected a list"),
    }
}

// ==================== IMPORTED SYMBOLS SHADOW NOTHING ====================

#[test]
fn test_import_does_not_shadow_builtins() {
    // This should work - imported add doesn't shadow + builtin
    let source = r#"
        (module math
            (export add)
            (defun add (x y) (+ x y 100)))

        (import math add)
        (list (add 1 2) (+ 1 2))
    "#;

    let vm = compile_and_run(source).unwrap();
    match get_stack_top(&vm) {
        Some(Value::List(lst)) => {
            let values: Vec<Value> = lst.to_vec();
            assert_eq!(values.len(), 2);
            // add(1,2) = 1 + 2 + 100 = 103
            assert_eq!(values[0], Value::Integer(103));
            // (+ 1 2) = 3
            assert_eq!(values[1], Value::Integer(3));
        }
        _ => panic!("Expected a list"),
    }
}

// ==================== COMPLEX MODULE INTERACTION ====================

#[test]
fn test_complex_module_interaction() {
    let source = r#"
        (module validators
            (export is-positive is-even)
            (defun is-positive (n) (> n 0))
            (defun is-even (n) (== (% n 2) 0)))

        (module filters
            (export filter-positives)
            (import validators is-positive)

            (defun filter-helper (lst acc)
                (if (null? lst)
                    acc
                    (if (is-positive (car lst))
                        (filter-helper (cdr lst) (append acc (list (car lst))))
                        (filter-helper (cdr lst) acc))))

            (defun filter-positives (lst)
                (filter-helper lst '())))

        (filters/filter-positives '(-1 2 -3 4 -5 6))
    "#;

    let vm = compile_and_run(source).unwrap();
    match get_stack_top(&vm) {
        Some(Value::List(lst)) => {
            let values: Vec<Value> = lst.to_vec();
            assert_eq!(values, vec![
                Value::Integer(2),
                Value::Integer(4),
                Value::Integer(6)
            ]);
        }
        _ => panic!("Expected a list"),
    }
}

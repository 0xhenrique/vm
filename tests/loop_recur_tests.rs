use lisp_bytecode_vm::{Compiler, VM, parser::Parser, Value};

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
        other => panic!("Expected integer result, got {:?}", other),
    }
}

/// Helper to get list result from VM
fn get_list_result(vm: &VM) -> Vec<Value> {
    match vm.value_stack.last() {
        Some(Value::List(lst)) => lst.to_vec(),
        other => panic!("Expected list result, got {:?}", other),
    }
}

// ============================================================================
// Basic loop/recur Tests
// ============================================================================

#[test]
fn test_loop_simple_countdown() {
    let source = r#"
        (loop ((n 5))
          (if (<= n 0)
              n
              (recur (- n 1))))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 0);
}

#[test]
fn test_loop_factorial() {
    let source = r#"
        (loop ((n 5) (acc 1))
          (if (<= n 1)
              acc
              (recur (- n 1) (* acc n))))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 120); // 5! = 120
}

#[test]
fn test_loop_sum_range() {
    let source = r#"
        (loop ((i 0) (sum 0))
          (if (> i 10)
              sum
              (recur (+ i 1) (+ sum i))))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 55); // sum of 0..10 = 55
}

#[test]
fn test_loop_fibonacci() {
    let source = r#"
        (loop ((n 10) (a 0) (b 1))
          (if (<= n 0)
              a
              (recur (- n 1) b (+ a b))))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 55); // 10th Fibonacci number
}

#[test]
fn test_loop_build_list() {
    let source = r#"
        (loop ((n 5) (lst '()))
          (if (<= n 0)
              lst
              (recur (- n 1) (cons n lst))))
    "#;
    let vm = compile_and_run(source);
    let result = get_list_result(&vm);
    assert_eq!(result.len(), 5);
    assert_eq!(result[0], Value::Integer(1));
    assert_eq!(result[4], Value::Integer(5));
}

#[test]
fn test_loop_multiple_bindings() {
    let source = r#"
        (loop ((a 1) (b 2) (c 3))
          (if (> a 5)
              (+ a (+ b c))
              (recur (+ a 1) (+ b 1) (+ c 1))))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 21); // 6 + 7 + 8 = 21
}

// ============================================================================
// Loop in Function Tests
// ============================================================================

#[test]
fn test_loop_in_function() {
    let source = r#"
        (defun factorial (n)
          (loop ((i n) (acc 1))
            (if (<= i 1)
                acc
                (recur (- i 1) (* acc i)))))
        (factorial 6)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 720); // 6! = 720
}

#[test]
fn test_loop_nested_in_function() {
    let source = r#"
        (defun sum-n (n)
          (loop ((i 0) (sum 0))
            (if (> i n)
                sum
                (recur (+ i 1) (+ sum i)))))
        (sum-n 100)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 5050); // sum of 0..100 = 5050
}

// ============================================================================
// Large Iteration Tests (No Stack Overflow)
// ============================================================================

#[test]
fn test_loop_large_iteration() {
    // This would cause stack overflow with normal recursion
    let source = r#"
        (loop ((n 100000) (acc 0))
          (if (<= n 0)
              acc
              (recur (- n 1) (+ acc 1))))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 100000);
}

#[test]
fn test_loop_even_larger_iteration() {
    // Even larger iteration to prove no stack growth
    let source = r#"
        (loop ((n 1000000))
          (if (<= n 0)
              42
              (recur (- n 1))))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 42);
}

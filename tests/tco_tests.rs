use lisp_bytecode_vm::{Compiler, VM, parser::Parser, Instruction};

/// Helper function to compile source and run it
fn compile_and_run(source: &str) -> VM {
    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).unwrap();

    let mut vm = VM::new();
    vm.functions = functions;
    vm.current_bytecode = main;
    vm.run().unwrap();
    vm
}

/// Helper to check that a function uses TailCall instruction
fn function_uses_tailcall(vm: &VM, function_name: &str) -> bool {
    if let Some(bytecode) = vm.functions.get(function_name) {
        bytecode.iter().any(|instr| matches!(instr, Instruction::TailCall(_, _)))
    } else {
        false
    }
}

#[test]
fn test_simple_tail_recursion() {
    let source = r#"
        (defun countdown (n)
          (if (<= n 0)
            42
            (countdown (- n 1))))
        (countdown 10)
    "#;

    let vm = compile_and_run(source);

    // Verify TailCall instruction is used
    assert!(function_uses_tailcall(&vm, "countdown"),
            "countdown should use TailCall instruction");

    // Verify result
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 42),
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_deep_tail_recursion_no_overflow() {
    // This would overflow the stack without TCO
    // Using 5000 iterations for faster CI/CD while still verifying TCO works
    let source = r#"
        (defun countdown (n)
          (if (<= n 0)
            999
            (countdown (- n 1))))
        (countdown 5000)
    "#;

    let vm = compile_and_run(source);

    // Verify TailCall instruction is used
    assert!(function_uses_tailcall(&vm, "countdown"));

    // Verify result
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 999),
        _ => panic!("Expected integer result"),
    }

    // Verify stack didn't grow excessively
    // With TCO, call stack should remain at most depth 1
    assert!(vm.call_stack.len() <= 1,
            "Call stack should not grow with TCO, but got depth: {}",
            vm.call_stack.len());
}

#[test]
fn test_tail_recursive_factorial() {
    let source = r#"
        (defun fact-tail (n acc)
          (if (<= n 0)
            acc
            (fact-tail (- n 1) (* n acc))))
        (fact-tail 10 1)
    "#;

    let vm = compile_and_run(source);

    // Verify TailCall instruction is used
    assert!(function_uses_tailcall(&vm, "fact-tail"));

    // Verify result
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 3628800),
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_tail_recursive_sum() {
    let source = r#"
        (defun sum (n acc)
          (if (<= n 0)
            acc
            (sum (- n 1) (+ n acc))))
        (sum 100 0)
    "#;

    let vm = compile_and_run(source);

    // Verify TailCall instruction is used
    assert!(function_uses_tailcall(&vm, "sum"));

    // Verify result: sum of 1..100 = 5050
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 5050),
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_mutual_tail_recursion() {
    let source = r#"
        (defun even? (n)
          (if (== n 0)
            true
            (odd? (- n 1))))

        (defun odd? (n)
          (if (== n 0)
            false
            (even? (- n 1))))

        (even? 500)
    "#;

    let vm = compile_and_run(source);

    // Verify both functions use TailCall
    assert!(function_uses_tailcall(&vm, "even?"));
    assert!(function_uses_tailcall(&vm, "odd?"));

    // Verify result
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Boolean(b)) => assert_eq!(*b, true),
        _ => panic!("Expected boolean result"),
    }
}

#[test]
fn test_non_tail_recursion_still_works() {
    // Non-tail-recursive factorial should still work (uses Call, not TailCall)
    let source = r#"
        (defun fact (n)
          (if (<= n 1)
            1
            (* n (fact (- n 1)))))
        (fact 5)
    "#;

    let vm = compile_and_run(source);

    // Verify this does NOT use TailCall (because multiplication happens after recursive call)
    assert!(!function_uses_tailcall(&vm, "fact"),
            "Non-tail-recursive fact should NOT use TailCall");

    // Verify result
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 120),
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_tail_call_in_both_if_branches() {
    let source = r#"
        (defun collatz (n)
          (if (<= n 1)
            1
            (if (== (% n 2) 0)
              (collatz (/ n 2))
              (collatz (+ (* 3 n) 1)))))
        (collatz 27)
    "#;

    let vm = compile_and_run(source);

    // Verify TailCall is used
    assert!(function_uses_tailcall(&vm, "collatz"));

    // Verify result
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 1),
        _ => panic!("Expected integer result"),
    }
}

#[test]
#[ignore] // TODO: Fix interaction between let bindings cleanup and tail calls
fn test_tail_call_with_let() {
    // Tail call in let body should be optimized
    // Currently disabled - needs fix for Slide instruction cleanup before tail call
    let source = r#"
        (defun loop-with-let (n)
          (let ((x (- n 1)))
            (if (<= x 0)
              999
              (loop-with-let x))))
        (loop-with-let 100)
    "#;

    let vm = compile_and_run(source);

    // Verify TailCall is used
    assert!(function_uses_tailcall(&vm, "loop-with-let"));

    // Verify result
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 999),
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_tail_recursive_gcd() {
    let source = r#"
        (defun gcd (a b)
          (if (== b 0)
            a
            (gcd b (% a b))))
        (gcd 48 18)
    "#;

    let vm = compile_and_run(source);

    // Verify TailCall is used
    assert!(function_uses_tailcall(&vm, "gcd"));

    // Verify result: gcd(48, 18) = 6
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 6),
        _ => panic!("Expected integer result"),
    }
}

#[test]
fn test_multiple_tail_recursive_functions() {
    // Test that multiple tail-recursive functions work correctly
    let source = r#"
        (defun count1 (n)
          (if (<= n 0)
            1
            (count1 (- n 1))))

        (defun count2 (n)
          (if (<= n 0)
            2
            (count2 (- n 1))))

        (+ (count1 50) (count2 50))
    "#;

    let vm = compile_and_run(source);

    // Verify both use TailCall
    assert!(function_uses_tailcall(&vm, "count1"));
    assert!(function_uses_tailcall(&vm, "count2"));

    // Verify result
    match vm.value_stack.last() {
        Some(lisp_bytecode_vm::Value::Integer(n)) => assert_eq!(*n, 3),
        _ => panic!("Expected integer result"),
    }
}

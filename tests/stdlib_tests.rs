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

/// Helper to get list result from VM
fn get_list_result(vm: &VM) -> Vec<Value> {
    match vm.value_stack.last() {
        Some(Value::List(lst)) => lst.clone(),
        _ => panic!("Expected list result"),
    }
}

// ============================================================================
// List Utilities Tests
// ============================================================================

#[test]
fn test_length() {
    let source = r#"
        (defun length (lst)
          (if (null? lst)
              0
              (+ 1 (length (cdr lst)))))
        (length '(1 2 3 4 5))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 5);

    let source = r#"
        (defun length (lst)
          (if (null? lst)
              0
              (+ 1 (length (cdr lst)))))
        (length '())
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 0);
}

#[test]
fn test_nth() {
    let source = r#"
        (defun nth (n lst)
          (if (== n 0)
              (car lst)
              (nth (- n 1) (cdr lst))))
        (nth 0 '(10 20 30))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 10);

    let source = r#"
        (defun nth (n lst)
          (if (== n 0)
              (car lst)
              (nth (- n 1) (cdr lst))))
        (nth 2 '(10 20 30))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 30);
}

#[test]
fn test_last() {
    let source = r#"
        (defun last (lst)
          (if (null? lst)
              '()
              (if (null? (cdr lst))
                  (car lst)
                  (last (cdr lst)))))
        (last '(1 2 3 4 5))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 5);
}

#[test]
fn test_reverse() {
    let source = r#"
        (defun reverse-helper (acc lst)
          (if (null? lst)
              acc
              (reverse-helper (cons (car lst) acc) (cdr lst))))
        (defun reverse (lst)
          (reverse-helper '() lst))
        (reverse '(1 2 3))
    "#;
    let vm = compile_and_run(source);
    let result = get_list_result(&vm);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Value::Integer(3));
    assert_eq!(result[1], Value::Integer(2));
    assert_eq!(result[2], Value::Integer(1));
}

#[test]
fn test_append() {
    let source = r#"
        (defun append (xs ys)
          (if (null? xs)
              ys
              (cons (car xs) (append (cdr xs) ys))))
        (append '(1 2) '(3 4))
    "#;
    let vm = compile_and_run(source);
    let result = get_list_result(&vm);
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], Value::Integer(1));
    assert_eq!(result[1], Value::Integer(2));
    assert_eq!(result[2], Value::Integer(3));
    assert_eq!(result[3], Value::Integer(4));
}

// ============================================================================
// Numeric Utilities Tests
// ============================================================================

#[test]
fn test_abs() {
    let source = r#"
        (defun abs (n)
          (if (< n 0)
            (- 0 n)
            n))
        (abs -5)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 5);

    let source = r#"
        (defun abs (n)
          (if (< n 0)
            (- 0 n)
            n))
        (abs 5)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 5);
}

#[test]
fn test_min() {
    let source = r#"
        (defun min (a b) (if (< a b) a b))
        (min 3 5)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 3);

    let source = r#"
        (defun min (a b) (if (< a b) a b))
        (min 5 3)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 3);
}

#[test]
fn test_max() {
    let source = r#"
        (defun max (a b) (if (> a b) a b))
        (max 3 5)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 5);

    let source = r#"
        (defun max (a b) (if (> a b) a b))
        (max 5 3)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 5);
}

#[test]
fn test_even_odd() {
    let source = r#"
        (defun even? (n)
          (if (== n 0)
              true
              (if (== n 1)
                  false
                  (if (< n 0)
                      (even? (- 0 n))
                      (even? (- n 2))))))
        (even? 4)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), true);

    let source = r#"
        (defun even? (n)
          (if (== n 0)
              true
              (if (== n 1)
                  false
                  (if (< n 0)
                      (even? (- 0 n))
                      (even? (- n 2))))))
        (even? 3)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), false);

    let source = r#"
        (defun even? (n)
          (if (== n 0)
              true
              (if (== n 1)
                  false
                  (if (< n 0)
                      (even? (- 0 n))
                      (even? (- n 2))))))
        (defun odd? (n)
          (if (even? n) false true))
        (odd? 3)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), true);
}

// ============================================================================
// Functional Utilities Tests
// ============================================================================

#[test]
fn test_identity() {
    let source = r#"
        (defun identity (x) x)
        (identity 42)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 42);
}

#[test]
fn test_compose() {
    let source = r#"
        (defun add1 (x) (+ x 1))
        (defun double (x) (* x 2))
        (defun compose (f g) (lambda (x) (f (g x))))
        ((compose (lambda (x) (+ x 1)) (lambda (x) (* x 2))) 5)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 11); // double(5) = 10, add1(10) = 11
}

#[test]
fn test_partial() {
    let source = r#"
        (defun add (a b) (+ a b))
        (defun partial (f arg) (lambda (x) (f arg x)))
        ((partial (lambda (a b) (+ a b)) 10) 5)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 15);
}

// ============================================================================
// Higher-Order Functions Tests
// ============================================================================

#[test]
fn test_map() {
    let source = r#"
        (defun map (f lst)
          (if (null? lst)
              '()
              (cons (f (car lst))
                    (map f (cdr lst)))))
        (map (lambda (x) (* x 2)) '(1 2 3))
    "#;
    let vm = compile_and_run(source);
    let result = get_list_result(&vm);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Value::Integer(2));
    assert_eq!(result[1], Value::Integer(4));
    assert_eq!(result[2], Value::Integer(6));
}

#[test]
fn test_filter() {
    let source = r#"
        (defun filter (pred lst)
          (if (null? lst)
              '()
              (if (pred (car lst))
                  (cons (car lst) (filter pred (cdr lst)))
                  (filter pred (cdr lst)))))
        (filter (lambda (x) (> x 2)) '(1 2 3 4 5))
    "#;
    let vm = compile_and_run(source);
    let result = get_list_result(&vm);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Value::Integer(3));
    assert_eq!(result[1], Value::Integer(4));
    assert_eq!(result[2], Value::Integer(5));
}

#[test]
fn test_reduce() {
    let source = r#"
        (defun reduce (f init lst)
          (if (null? lst)
              init
              (reduce f (f init (car lst)) (cdr lst))))
        (reduce (lambda (acc x) (+ acc x)) 0 '(1 2 3 4 5))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 15);
}

// ============================================================================
// Combined Tests
// ============================================================================

#[test]
fn test_sum_of_squares() {
    let source = r#"
        (defun map (f lst)
          (if (null? lst)
              '()
              (cons (f (car lst))
                    (map f (cdr lst)))))
        (defun reduce (f init lst)
          (if (null? lst)
              init
              (reduce f (f init (car lst)) (cdr lst))))
        (reduce (lambda (acc x) (+ acc x))
                0
                (map (lambda (x) (* x x)) '(1 2 3 4)))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 30); // 1 + 4 + 9 + 16 = 30
}

#[test]
fn test_filter_and_sum() {
    let source = r#"
        (defun even? (n)
          (if (== n 0)
              true
              (if (== n 1)
                  false
                  (if (< n 0)
                      (even? (- 0 n))
                      (even? (- n 2))))))
        (defun filter (pred lst)
          (if (null? lst)
              '()
              (if (pred (car lst))
                  (cons (car lst) (filter pred (cdr lst)))
                  (filter pred (cdr lst)))))
        (defun reduce (f init lst)
          (if (null? lst)
              init
              (reduce f (f init (car lst)) (cdr lst))))
        (reduce (lambda (acc x) (+ acc x))
                0
                (filter (lambda (x) (even? x)) '(1 2 3 4 5 6 7 8)))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 20); // 2 + 4 + 6 + 8 = 20
}

#[test]
fn test_not_helper() {
    let source = r#"
        (defun not (x)
          (if x false true))
        (not true)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), false);

    let source = r#"
        (defun not (x)
          (if x false true))
        (not false)
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), true);
}

#[test]
fn test_null_predicate() {
    let source = r#"
        ; null? is already a builtin, but we can test the behavior
        (null? '())
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), true);

    let source = r#"
        ; null? is already a builtin, but we can test the behavior
        (null? '(1 2 3))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_bool_result(&vm), false);
}

#[test]
fn test_list_length_with_filter() {
    let source = r#"
        (defun even? (n)
          (if (== n 0)
              true
              (if (== n 1)
                  false
                  (if (< n 0)
                      (even? (- 0 n))
                      (even? (- n 2))))))
        (defun length (lst)
          (if (null? lst)
              0
              (+ 1 (length (cdr lst)))))
        (defun filter (pred lst)
          (if (null? lst)
              '()
              (if (pred (car lst))
                  (cons (car lst) (filter pred (cdr lst)))
                  (filter pred (cdr lst)))))
        (length (filter (lambda (x) (even? x)) '(1 2 3 4 5 6 7 8 9 10)))
    "#;
    let vm = compile_and_run(source);
    assert_eq!(get_int_result(&vm), 5); // 5 even numbers in range 1-10
}

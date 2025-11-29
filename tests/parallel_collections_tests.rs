// Tests for Phase 12a: Parallel Collections
// pmap, pfilter, preduce

use lisp_bytecode_vm::*;

fn run_code(source: &str) -> Result<Value, String> {
    let mut parser = parser::Parser::new(source);
    let exprs = parser.parse_all().map_err(|e| e.to_string())?;

    let mut compiler = Compiler::new();
    let (functions, main_bytecode) = compiler.compile_program(&exprs)
        .map_err(|e| e.message)?;

    let mut vm = VM::new();
    vm.functions.extend(functions);
    vm.current_bytecode = main_bytecode;

    vm.run().map_err(|e| e.message.clone())?;

    Ok(vm.value_stack.last().cloned().unwrap_or(Value::Boolean(false)))
}

// ============================================================
// pmap Tests
// ============================================================

#[test]
fn test_pmap_simple() {
    let result = run_code(r#"
        (defun square (x) (* x x))
        (pmap square '(1 2 3 4 5))
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 5);
            assert_eq!(vec[0], &Value::Integer(1));
            assert_eq!(vec[1], &Value::Integer(4));
            assert_eq!(vec[2], &Value::Integer(9));
            assert_eq!(vec[3], &Value::Integer(16));
            assert_eq!(vec[4], &Value::Integer(25));
        }
        _ => panic!("Expected list, got {:?}", result),
    }
}

#[test]
fn test_pmap_with_lambda() {
    let result = run_code(r#"
        (pmap (lambda (x) (+ x 10)) '(1 2 3))
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 3);
            assert_eq!(vec[0], &Value::Integer(11));
            assert_eq!(vec[1], &Value::Integer(12));
            assert_eq!(vec[2], &Value::Integer(13));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_pmap_empty_list() {
    let result = run_code(r#"
        (defun square (x) (* x x))
        (pmap square '())
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 0);
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_pmap_single_element() {
    let result = run_code(r#"
        (pmap (lambda (x) (* x 2)) '(42))
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 1);
            assert_eq!(vec[0], &Value::Integer(84));
        }
        _ => panic!("Expected list"),
    }
}

// TODO: Enable this test once range function is implemented
// #[test]
// fn test_pmap_large_list() {
//     let result = run_code(r#"
//         (defun inc (x) (+ x 1))
//         (pmap inc (range 1 1001))
//     "#).unwrap();
//
//     match result {
//         Value::List(items) => {
//             let vec: Vec<_> = items.iter().collect();
//             assert_eq!(vec.len(), 1000);
//             assert_eq!(vec[0], &Value::Integer(2));
//             assert_eq!(vec[999], &Value::Integer(1001));
//         }
//         _ => panic!("Expected list"),
//     }
// }

// ============================================================
// pfilter Tests
// ============================================================

#[test]
fn test_pfilter_simple() {
    let result = run_code(r#"
        (defun is_even (x) (== (% x 2) 0))
        (pfilter is_even '(1 2 3 4 5 6))
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 3);
            assert_eq!(vec[0], &Value::Integer(2));
            assert_eq!(vec[1], &Value::Integer(4));
            assert_eq!(vec[2], &Value::Integer(6));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_pfilter_with_lambda() {
    let result = run_code(r#"
        (pfilter (lambda (x) (> x 5)) '(1 3 5 7 9 11))
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 3);
            assert_eq!(vec[0], &Value::Integer(7));
            assert_eq!(vec[1], &Value::Integer(9));
            assert_eq!(vec[2], &Value::Integer(11));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_pfilter_empty_list() {
    let result = run_code(r#"
        (pfilter (lambda (x) (> x 0)) '())
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 0);
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_pfilter_none_match() {
    let result = run_code(r#"
        (pfilter (lambda (x) (> x 100)) '(1 2 3 4 5))
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 0);
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_pfilter_all_match() {
    let result = run_code(r#"
        (pfilter (lambda (x) (> x 0)) '(1 2 3 4 5))
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 5);
        }
        _ => panic!("Expected list"),
    }
}

// ============================================================
// preduce Tests
// ============================================================

#[test]
fn test_preduce_sum() {
    let result = run_code(r#"
        (defun add (a b) (+ a b))
        (preduce '(1 2 3 4 5) 0 add)
    "#).unwrap();

    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_preduce_product() {
    let result = run_code(r#"
        (defun mul (a b) (* a b))
        (preduce '(1 2 3 4 5) 1 mul)
    "#).unwrap();

    assert_eq!(result, Value::Integer(120));
}

#[test]
fn test_preduce_with_lambda() {
    let result = run_code(r#"
        (preduce '(1 2 3 4) 0 (lambda (acc x) (+ acc (* x x))))
    "#).unwrap();

    // 0 + 1^2 + 2^2 + 3^2 + 4^2 = 0 + 1 + 4 + 9 + 16 = 30
    assert_eq!(result, Value::Integer(30));
}

#[test]
fn test_preduce_empty_list() {
    let result = run_code(r#"
        (preduce '() 42 (lambda (a b) (+ a b)))
    "#).unwrap();

    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_preduce_single_element() {
    let result = run_code(r#"
        (preduce '(5) 10 (lambda (a b) (+ a b)))
    "#).unwrap();

    assert_eq!(result, Value::Integer(15));
}

// ============================================================
// Combined/Integration Tests
// ============================================================

#[test]
fn test_pmap_then_pfilter() {
    let result = run_code(r#"
        (def squared (pmap (lambda (x) (* x x)) '(1 2 3 4 5)))
        (pfilter (lambda (x) (> x 10)) squared)
    "#).unwrap();

    match result {
        Value::List(items) => {
            let vec: Vec<_> = items.iter().collect();
            assert_eq!(vec.len(), 2);
            assert_eq!(vec[0], &Value::Integer(16));
            assert_eq!(vec[1], &Value::Integer(25));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_pmap_pfilter_preduce_chain() {
    let result = run_code(r#"
        (def doubled (pmap (lambda (x) (* x 2)) '(1 2 3 4 5)))
        (def evens (pfilter (lambda (x) (== (% x 4) 0)) doubled))
        (preduce evens 0 (lambda (a b) (+ a b)))
    "#).unwrap();

    // doubled: (2 4 6 8 10)
    // evens (divisible by 4): (4 8)
    // sum: 12
    assert_eq!(result, Value::Integer(12));
}

// TODO: Enable this test once map and reduce functions are implemented
// #[test]
// fn test_parallel_vs_sequential_same_result() {
//     let par_result = run_code(r#"
//         (def par (pmap (lambda (x) (* x x)) '(1 2 3 4 5)))
//         (preduce par 0 (lambda (a b) (+ a b)))
//     "#).unwrap();
//
//     let seq_result = run_code(r#"
//         (def seq (map (lambda (x) (* x x)) '(1 2 3 4 5)))
//         (reduce (lambda (a b) (+ a b)) 0 seq)
//     "#).unwrap();
//
//     assert_eq!(par_result, seq_result);
//     assert_eq!(par_result, Value::Integer(55)); // 1+4+9+16+25
// }

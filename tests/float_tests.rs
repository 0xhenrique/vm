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

/// Helper to get last value as f64 (handles both Integer and Float)
fn get_float(vm: &VM) -> f64 {
    match vm.value_stack.last() {
        Some(Value::Float(f)) => *f,
        Some(Value::Integer(n)) => *n as f64,
        _ => panic!("Expected number value, got {:?}", vm.value_stack.last()),
    }
}

/// Helper to get last value as i64
fn get_int(vm: &VM) -> i64 {
    match vm.value_stack.last() {
        Some(Value::Integer(n)) => *n,
        _ => panic!("Expected integer value, got {:?}", vm.value_stack.last()),
    }
}

/// Helper to get last value as boolean
fn get_bool(vm: &VM) -> bool {
    match vm.value_stack.last() {
        Some(Value::Boolean(b)) => *b,
        _ => panic!("Expected boolean value, got {:?}", vm.value_stack.last()),
    }
}

// ============================================================
// Float Literals
// ============================================================

#[test]
fn test_float_literal() {
    let vm = compile_and_run("3.14");
    assert_eq!(get_float(&vm), 3.14);
}

#[test]
fn test_float_literal_with_zero_decimal() {
    let vm = compile_and_run("42.0");
    assert_eq!(get_float(&vm), 42.0);
}

#[test]
fn test_float_literal_scientific_notation() {
    let vm = compile_and_run("1.5e2");
    assert_eq!(get_float(&vm), 150.0);
}

#[test]
fn test_float_literal_negative_exponent() {
    let vm = compile_and_run("2.5e-1");
    assert_eq!(get_float(&vm), 0.25);
}

// ============================================================
// Type Predicates
// ============================================================

#[test]
fn test_float_predicate_with_float() {
    let vm = compile_and_run("(float? 3.14)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_float_predicate_with_integer() {
    let vm = compile_and_run("(float? 42)");
    assert_eq!(get_bool(&vm), false);
}

#[test]
fn test_integer_predicate_with_integer() {
    let vm = compile_and_run("(integer? 42)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_integer_predicate_with_float() {
    let vm = compile_and_run("(integer? 3.14)");
    assert_eq!(get_bool(&vm), false);
}

#[test]
fn test_number_predicate_with_integer() {
    let vm = compile_and_run("(number? 42)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_number_predicate_with_float() {
    let vm = compile_and_run("(number? 3.14)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_number_predicate_with_string() {
    let vm = compile_and_run(r#"(number? "hello")"#);
    assert_eq!(get_bool(&vm), false);
}

// ============================================================
// Type Conversions
// ============================================================

#[test]
fn test_int_to_float() {
    let vm = compile_and_run("(int->float 42)");
    assert_eq!(get_float(&vm), 42.0);
}

#[test]
fn test_float_to_int_truncates() {
    let vm = compile_and_run("(float->int 3.14)");
    assert_eq!(get_int(&vm), 3);
}

#[test]
fn test_float_to_int_negative() {
    let vm = compile_and_run("(float->int -3.9)");
    assert_eq!(get_int(&vm), -3);
}

// ============================================================
// Arithmetic with Type Coercion
// ============================================================

#[test]
fn test_add_int_int() {
    let vm = compile_and_run("(+ 10 20)");
    assert_eq!(get_int(&vm), 30);
}

#[test]
fn test_add_float_float() {
    let vm = compile_and_run("(+ 1.5 2.5)");
    assert_eq!(get_float(&vm), 4.0);
}

#[test]
fn test_add_int_float_coerces_to_float() {
    let vm = compile_and_run("(+ 10 2.5)");
    assert_eq!(get_float(&vm), 12.5);
}

#[test]
fn test_add_float_int_coerces_to_float() {
    let vm = compile_and_run("(+ 1.5 10)");
    assert_eq!(get_float(&vm), 11.5);
}

#[test]
fn test_sub_int_int() {
    let vm = compile_and_run("(- 30 10)");
    assert_eq!(get_int(&vm), 20);
}

#[test]
fn test_sub_float_float() {
    let vm = compile_and_run("(- 5.5 2.5)");
    assert_eq!(get_float(&vm), 3.0);
}

#[test]
fn test_sub_int_float_coerces_to_float() {
    let vm = compile_and_run("(- 10 2.5)");
    assert_eq!(get_float(&vm), 7.5);
}

#[test]
fn test_mul_int_int() {
    let vm = compile_and_run("(* 6 7)");
    assert_eq!(get_int(&vm), 42);
}

#[test]
fn test_mul_float_float() {
    let vm = compile_and_run("(* 2.5 4.0)");
    assert_eq!(get_float(&vm), 10.0);
}

#[test]
fn test_mul_int_float_coerces_to_float() {
    let vm = compile_and_run("(* 5 2.5)");
    assert_eq!(get_float(&vm), 12.5);
}

#[test]
fn test_div_int_int_stays_integer() {
    let vm = compile_and_run("(/ 10 2)");
    assert_eq!(get_int(&vm), 5);
}

#[test]
fn test_div_int_int_truncates() {
    let vm = compile_and_run("(/ 7 2)");
    assert_eq!(get_int(&vm), 3);
}

#[test]
fn test_div_float_float() {
    let vm = compile_and_run("(/ 7.0 2.0)");
    assert_eq!(get_float(&vm), 3.5);
}

#[test]
fn test_div_int_float_coerces_to_float() {
    let vm = compile_and_run("(/ 7 2.0)");
    assert_eq!(get_float(&vm), 3.5);
}

#[test]
fn test_div_float_int_coerces_to_float() {
    let vm = compile_and_run("(/ 7.0 2)");
    assert_eq!(get_float(&vm), 3.5);
}

#[test]
fn test_mod_int_int() {
    let vm = compile_and_run("(% 17 5)");
    assert_eq!(get_int(&vm), 2);
}

#[test]
fn test_mod_float_float() {
    let vm = compile_and_run("(% 5.5 2.0)");
    assert_eq!(get_float(&vm), 1.5);
}

#[test]
fn test_neg_integer() {
    let vm = compile_and_run("(neg 42)");
    assert_eq!(get_int(&vm), -42);
}

#[test]
fn test_neg_float() {
    let vm = compile_and_run("(neg 3.14)");
    assert_eq!(get_float(&vm), -3.14);
}

// ============================================================
// Comparisons with Type Coercion
// ============================================================

#[test]
fn test_leq_int_int() {
    let vm = compile_and_run("(<= 5 10)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_leq_float_float() {
    let vm = compile_and_run("(<= 3.14 3.14)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_leq_int_float() {
    let vm = compile_and_run("(<= 3 3.5)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_lt_int_float() {
    let vm = compile_and_run("(< 3 3.5)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_gt_float_int() {
    let vm = compile_and_run("(> 3.5 3)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_gte_float_int() {
    let vm = compile_and_run("(>= 3.5 3)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_eq_int_int() {
    let vm = compile_and_run("(== 42 42)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_eq_float_float() {
    let vm = compile_and_run("(== 3.14 3.14)");
    assert_eq!(get_bool(&vm), true);
}

#[test]
fn test_eq_int_float_different() {
    let vm = compile_and_run("(== 3 3.5)");
    assert_eq!(get_bool(&vm), false);
}

#[test]
fn test_eq_int_float_same_value() {
    let vm = compile_and_run("(== 3 3.0)");
    assert_eq!(get_bool(&vm), true);
}

// ============================================================
// Math Functions
// ============================================================

#[test]
fn test_sqrt_of_integer() {
    let vm = compile_and_run("(sqrt 16)");
    assert_eq!(get_float(&vm), 4.0);
}

#[test]
fn test_sqrt_of_float() {
    let vm = compile_and_run("(sqrt 2.25)");
    assert_eq!(get_float(&vm), 1.5);
}

#[test]
fn test_abs_positive_integer() {
    let vm = compile_and_run("(abs 42)");
    assert_eq!(get_int(&vm), 42);
}

#[test]
fn test_abs_negative_integer() {
    let vm = compile_and_run("(abs -42)");
    assert_eq!(get_int(&vm), 42);
}

#[test]
fn test_abs_positive_float() {
    let vm = compile_and_run("(abs 3.14)");
    assert_eq!(get_float(&vm), 3.14);
}

#[test]
fn test_abs_negative_float() {
    let vm = compile_and_run("(abs -3.14)");
    assert_eq!(get_float(&vm), 3.14);
}

#[test]
fn test_floor_positive() {
    let vm = compile_and_run("(floor 3.7)");
    assert_eq!(get_int(&vm), 3);
}

#[test]
fn test_floor_negative() {
    let vm = compile_and_run("(floor -3.7)");
    assert_eq!(get_int(&vm), -4);
}

#[test]
fn test_floor_integer() {
    let vm = compile_and_run("(floor 5)");
    assert_eq!(get_int(&vm), 5);
}

#[test]
fn test_ceil_positive() {
    let vm = compile_and_run("(ceil 3.2)");
    assert_eq!(get_int(&vm), 4);
}

#[test]
fn test_ceil_negative() {
    let vm = compile_and_run("(ceil -3.2)");
    assert_eq!(get_int(&vm), -3);
}

#[test]
fn test_ceil_integer() {
    let vm = compile_and_run("(ceil 5)");
    assert_eq!(get_int(&vm), 5);
}

#[test]
fn test_pow_integer_base() {
    let vm = compile_and_run("(pow 2 3)");
    assert_eq!(get_float(&vm), 8.0);
}

#[test]
fn test_pow_float_base() {
    let vm = compile_and_run("(pow 2.0 3.0)");
    assert_eq!(get_float(&vm), 8.0);
}

#[test]
fn test_pow_fractional_exponent() {
    let vm = compile_and_run("(pow 4.0 0.5)");
    assert_eq!(get_float(&vm), 2.0);
}

#[test]
fn test_sin_zero() {
    let vm = compile_and_run("(sin 0.0)");
    assert_eq!(get_float(&vm), 0.0);
}

#[test]
fn test_sin_integer() {
    let vm = compile_and_run("(sin 0)");
    assert_eq!(get_float(&vm), 0.0);
}

#[test]
fn test_cos_zero() {
    let vm = compile_and_run("(cos 0.0)");
    assert_eq!(get_float(&vm), 1.0);
}

#[test]
fn test_cos_integer() {
    let vm = compile_and_run("(cos 0)");
    assert_eq!(get_float(&vm), 1.0);
}

// ============================================================
// Complex Expressions
// ============================================================

#[test]
fn test_complex_arithmetic() {
    let vm = compile_and_run("(+ (* 2.5 4.0) (/ 10.0 2.0))");
    assert_eq!(get_float(&vm), 15.0);
}

#[test]
fn test_mixed_type_expression() {
    let vm = compile_and_run("(+ (* 3 2.5) 10)");
    assert_eq!(get_float(&vm), 17.5);
}

#[test]
fn test_pythagorean_theorem() {
    // sqrt(3^2 + 4^2) = 5
    let vm = compile_and_run("(sqrt (+ (* 3 3) (* 4 4)))");
    assert_eq!(get_float(&vm), 5.0);
}

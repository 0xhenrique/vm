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
        Some(value) => Ok(format_value(value)),
        None => Err("No value on stack".to_string()),
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 && !f.is_nan() && !f.is_infinite() {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        Value::Boolean(b) => b.to_string(),
        Value::List(items) => {
            let formatted_items: Vec<String> = items.iter().map(|v| format_value(v)).collect();
            format!("({})", formatted_items.join(" "))
        }
        Value::Symbol(s) => s.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Function(name) => format!("<function {}>", name),
        Value::Closure(closure_data) => format!("<closure/{}>", closure_data.params.len()),
        Value::HashMap(map) => {
            let mut items: Vec<String> = map.iter()
                .map(|(k, v)| format!("\"{}\" {}", k, format_value(v)))
                .collect();
            items.sort(); // Sort for consistent output
            format!("{{{}}}", items.join(" "))
        }
        Value::Vector(items) => {
            let formatted_items: Vec<String> = items.iter().map(|v| format_value(v)).collect();
            format!("[{}]", formatted_items.join(" "))
        }
    }
}

// ==================== HashMap Tests ====================

#[test]
fn test_hash_map_creation() {
    let source = r#"
        (hash-map "name" "Alice" "age" 30 "city" "NYC")
    "#;
    let result = compile_and_run(source).unwrap();
    // Should create a hashmap (exact output format tested below)
    assert!(result.contains("\"name\" \"Alice\""));
    assert!(result.contains("\"age\" 30"));
    assert!(result.contains("\"city\" \"NYC\""));
}

#[test]
fn test_hash_map_empty() {
    let source = r#"
        (hash-map)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "{}");
}

#[test]
fn test_hash_map_get() {
    let source = r#"
        (defun test-get ()
            (let ((m (hash-map "key1" 100 "key2" 200)))
                (hashmap-get m "key1")))
        (test-get)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "100");
}

#[test]
fn test_hash_map_get_nested() {
    let source = r#"
        (defun test-nested ()
            (let ((m (hash-map "a" 1 "b" 2 "c" 3)))
                (+ (hashmap-get m "a") (hashmap-get m "b"))))
        (test-nested)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "3");
}

#[test]
fn test_hash_map_set() {
    let source = r#"
        (defun test-set ()
            (let ((m (hash-map "x" 10)))
                (let ((m2 (hashmap-set m "y" 20)))
                    (hashmap-get m2 "y"))))
        (test-set)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "20");
}

#[test]
fn test_hash_map_set_overwrite() {
    let source = r#"
        (defun test-overwrite ()
            (let ((m (hash-map "key" 100)))
                (let ((m2 (hashmap-set m "key" 999)))
                    (hashmap-get m2 "key"))))
        (test-overwrite)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "999");
}

#[test]
fn test_hash_map_keys() {
    let source = r#"
        (defun test-keys ()
            (let ((m (hash-map "a" 1 "b" 2)))
                (list-length (hashmap-keys m))))
        (test-keys)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_hash_map_values() {
    let source = r#"
        (defun test-values ()
            (let ((m (hash-map "x" 10 "y" 20)))
                (list-length (hashmap-values m))))
        (test-values)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_hash_map_contains_key_true() {
    let source = r#"
        (defun test-contains ()
            (let ((m (hash-map "found" 42)))
                (hashmap-contains-key? m "found")))
        (test-contains)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_hash_map_contains_key_false() {
    let source = r#"
        (defun test-not-contains ()
            (let ((m (hash-map "a" 1)))
                (hashmap-contains-key? m "b")))
        (test-not-contains)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_hash_map_predicate_true() {
    let source = r#"
        (hashmap? (hash-map "x" 1))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_hash_map_predicate_false() {
    let source = r#"
        (hashmap? (list 1 2 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_hash_map_nested_values() {
    let source = r#"
        (defun test-nested-maps ()
            (let ((inner (hash-map "nested" 100)))
                (let ((outer (hash-map "inner" 1 "data" 2)))
                    (hashmap-get outer "data"))))
        (test-nested-maps)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "2");
}

// ==================== Vector Tests ====================

#[test]
fn test_vector_creation() {
    let source = r#"
        (vector 1 2 3 4 5)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "[1 2 3 4 5]");
}

#[test]
fn test_vector_empty() {
    let source = r#"
        (vector)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "[]");
}

#[test]
fn test_vector_ref() {
    let source = r#"
        (defun test-ref ()
            (let ((v (vector 10 20 30)))
                (vector-ref v 1)))
        (test-ref)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "20");
}

#[test]
fn test_vector_ref_first() {
    let source = r#"
        (vector-ref (vector 100 200 300) 0)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "100");
}

#[test]
fn test_vector_ref_last() {
    let source = r#"
        (vector-ref (vector 1 2 3 4 5) 4)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "5");
}

#[test]
fn test_vector_set() {
    let source = r#"
        (defun test-set ()
            (let ((v (vector 1 2 3)))
                (let ((v2 (vector-set v 1 999)))
                    (vector-ref v2 1))))
        (test-set)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "999");
}

#[test]
fn test_vector_set_immutable() {
    let source = r#"
        (defun test-immutable ()
            (let ((v (vector 1 2 3)))
                (let ((v2 (vector-set v 0 100)))
                    (vector-ref v 0))))
        (test-immutable)
    "#;
    let result = compile_and_run(source).unwrap();
    // Original vector should be unchanged
    assert_eq!(result.trim(), "1");
}

#[test]
fn test_vector_push() {
    let source = r#"
        (defun test-push ()
            (let ((v (vector 1 2)))
                (let ((v2 (vector-push v 3)))
                    (vector-length v2))))
        (test-push)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "3");
}

#[test]
fn test_vector_push_value() {
    let source = r#"
        (defun test-push-val ()
            (let ((v (vector 10)))
                (let ((v2 (vector-push v 20)))
                    (vector-ref v2 1))))
        (test-push-val)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "20");
}

#[test]
fn test_vector_pop() {
    let source = r#"
        (defun test-pop ()
            (let ((v (vector 1 2 3)))
                (vector-pop v)))
        (test-pop)
    "#;
    let result = compile_and_run(source).unwrap();
    // vector-pop returns two values: the popped element is on top of stack
    assert_eq!(result.trim(), "3");
}

#[test]
fn test_vector_length() {
    let source = r#"
        (vector-length (vector 1 2 3 4))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "4");
}

#[test]
fn test_vector_length_empty() {
    let source = r#"
        (vector-length (vector))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "0");
}

#[test]
fn test_vector_predicate_true() {
    let source = r#"
        (vector? (vector 1 2 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_vector_predicate_false() {
    let source = r#"
        (vector? (list 1 2 3))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_vector_mixed_types() {
    let source = r#"
        (defun test-mixed ()
            (let ((v (vector 42 "hello" true)))
                (vector-length v)))
        (test-mixed)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "3");
}

#[test]
fn test_vector_nested() {
    let source = r#"
        (defun test-nested ()
            (let ((inner (vector 1 2)))
                (let ((outer (vector inner 3 4)))
                    (vector-length outer))))
        (test-nested)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "3");
}

// ==================== Integration Tests ====================

#[test]
fn test_vector_and_hashmap_together() {
    let source = r#"
        (defun test-both ()
            (let ((v (vector 1 2 3)))
                (let ((m (hash-map "vec" 100)))
                    (+ (vector-ref v 0) (hashmap-get m "vec")))))
        (test-both)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "101");
}

#[test]
fn test_vector_of_hashmaps() {
    let source = r#"
        (defun test-vec-of-maps ()
            (let ((m1 (hash-map "id" 1))
                  (m2 (hash-map "id" 2)))
                (let ((v (vector m1 m2)))
                    (vector-length v))))
        (test-vec-of-maps)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_complex_data_structure_operations() {
    let source = r#"
        (defun test-complex ()
            (let ((v (vector 10 20 30)))
                (let ((v2 (vector-set v 1 999)))
                    (let ((m (hash-map "result" (vector-ref v2 1))))
                        (hashmap-get m "result")))))
        (test-complex)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "999");
}

// ==================== Type Predicate Tests (Phase 2) ====================

#[test]
fn test_integer_predicate_true() {
    let source = r#"
        (integer? 42)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_integer_predicate_false() {
    let source = r#"
        (integer? "not a number")
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_boolean_predicate_true() {
    let source = r#"
        (boolean? true)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_boolean_predicate_false() {
    let source = r#"
        (boolean? 42)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_function_predicate_true() {
    let source = r#"
        (function? +)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_function_predicate_false() {
    let source = r#"
        (function? 42)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_closure_predicate_true() {
    let source = r#"
        (defun make-closure ()
            (let ((x 10))
                (lambda (y) (+ x y))))
        (closure? (make-closure))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_closure_predicate_false() {
    let source = r#"
        (closure? +)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_procedure_predicate_on_function() {
    let source = r#"
        (procedure? +)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_procedure_predicate_on_closure() {
    let source = r#"
        (defun make-closure ()
            (let ((x 10))
                (lambda (y) (+ x y))))
        (procedure? (make-closure))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_procedure_predicate_false() {
    let source = r#"
        (procedure? 42)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_number_predicate_true() {
    let source = r#"
        (number? 123)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_number_predicate_false() {
    let source = r#"
        (number? "123")
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_multiple_type_predicates() {
    let source = r#"
        (defun test-types ()
            (let ((num 42)
                  (str "hello")
                  (lst (list 1 2 3))
                  (vec (vector 1 2 3))
                  (map (hash-map "a" 1)))
                (+ (if (integer? num) 1 0)
                   (if (string? str) 1 0)
                   (if (list? lst) 1 0)
                   (if (vector? vec) 1 0)
                   (if (hashmap? map) 1 0))))
        (test-types)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "5");
}

// ==================== Type Conversion Tests (Phase 2) ====================

#[test]
fn test_string_to_number_valid() {
    let source = r#"
        (string->number "42")
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "42");
}

#[test]
fn test_string_to_number_negative() {
    let source = r#"
        (string->number "-123")
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "-123");
}

#[test]
fn test_string_to_number_with_whitespace() {
    let source = r#"
        (string->number "  100  ")
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "100");
}

#[test]
fn test_string_to_number_invalid() {
    let source = r#"
        (string->number "not-a-number")
    "#;
    let result = compile_and_run(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot parse"));
}

#[test]
fn test_number_to_string_and_back() {
    let source = r#"
        (defun test-roundtrip ()
            (let ((original 999))
                (let ((as-string (number->string original)))
                    (string->number as-string))))
        (test-roundtrip)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "999");
}

#[test]
fn test_list_to_vector() {
    let source = r#"
        (defun test-conversion ()
            (let ((lst (list 1 2 3 4 5)))
                (let ((vec (list->vector lst)))
                    (vector? vec))))
        (test-conversion)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_list_to_vector_preserves_values() {
    let source = r#"
        (defun test-values ()
            (let ((lst (list 10 20 30)))
                (let ((vec (list->vector lst)))
                    (vector-ref vec 1))))
        (test-values)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "20");
}

#[test]
fn test_list_to_vector_empty() {
    let source = r#"
        (list->vector (list))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "[]");
}

#[test]
fn test_vector_to_list() {
    let source = r#"
        (defun test-conversion ()
            (let ((vec (vector 1 2 3 4 5)))
                (let ((lst (vector->list vec)))
                    (list? lst))))
        (test-conversion)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_vector_to_list_preserves_values() {
    let source = r#"
        (defun test-values ()
            (let ((vec (vector 100 200 300)))
                (let ((lst (vector->list vec)))
                    (car (cdr lst)))))
        (test-values)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "200");
}

#[test]
fn test_vector_to_list_empty() {
    let source = r#"
        (vector->list (vector))
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "()");
}

#[test]
fn test_list_vector_roundtrip() {
    let source = r#"
        (defun test-roundtrip ()
            (let ((original (list 1 2 3 4 5)))
                (let ((as-vec (list->vector original)))
                    (let ((back-to-list (vector->list as-vec)))
                        (list-length back-to-list)))))
        (test-roundtrip)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "5");
}

#[test]
fn test_conversion_with_mixed_types() {
    let source = r#"
        (defun test-mixed ()
            (let ((lst (list 42 "hello" true)))
                (let ((vec (list->vector lst)))
                    (let ((back (vector->list vec)))
                        (list-length back)))))
        (test-mixed)
    "#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(result.trim(), "3");
}

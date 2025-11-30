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
            format!("#({})", formatted_items.join(" "))
        }
        Value::TcpListener(_) => "#<tcp-listener>".to_string(),
        Value::TcpStream(_) => "#<tcp-stream>".to_string(),
        Value::SharedTcpListener(_) => "#<shared-tcp-listener>".to_string(),
        Value::Pointer(p) => format!("#<pointer 0x{:x}>", p),
    }
}

// Built-in reader macro tests: #()

#[test]
fn test_vector_literal_basic() {
    let result = compile_and_run("#(1 2 3)");
    assert_eq!(result, Ok("#(1 2 3)".to_string()));
}

#[test]
fn test_vector_literal_empty() {
    let result = compile_and_run("#()");
    assert_eq!(result, Ok("#()".to_string()));
}

#[test]
fn test_vector_literal_with_expressions() {
    let result = compile_and_run("#((+ 1 2) (* 3 4) (- 10 5))");
    assert_eq!(result, Ok("#(3 12 5)".to_string()));
}

#[test]
fn test_vector_literal_nested() {
    let result = compile_and_run("#(1 #(2 3) 4)");
    assert_eq!(result, Ok("#(1 #(2 3) 4)".to_string()));
}

#[test]
fn test_vector_literal_with_strings() {
    let result = compile_and_run(r#"#("hello" "world")"#);
    assert_eq!(result, Ok(r#"#("hello" "world")"#.to_string()));
}

#[test]
fn test_vector_literal_mixed_types() {
    let result = compile_and_run(r#"#(1 "hello" true 3.14)"#);
    assert_eq!(result, Ok(r#"#(1 "hello" true 3.14)"#.to_string()));
}

#[test]
fn test_vector_literal_in_expression() {
    let result = compile_and_run("(vector-length #(1 2 3 4 5))");
    assert_eq!(result, Ok("5".to_string()));
}

#[test]
fn test_vector_literal_with_vector_ref() {
    let result = compile_and_run("(vector-ref #(10 20 30) 1)");
    assert_eq!(result, Ok("20".to_string()));
}

#[test]
fn test_vector_literal_stored_in_def() {
    let result = compile_and_run(r#"
        (def my-vec #(1 2 3))
        (vector-ref my-vec 2)
    "#);
    assert_eq!(result, Ok("3".to_string()));
}

#[test]
fn test_vector_literal_passed_to_function() {
    let result = compile_and_run(r#"
        (defun sum-vector (v)
            (+ (vector-ref v 0)
               (vector-ref v 1)
               (vector-ref v 2)))
        (sum-vector #(10 20 30))
    "#);
    assert_eq!(result, Ok("60".to_string()));
}

#[test]
fn test_vector_literal_with_lambda() {
    let result = compile_and_run(r#"
        (def funcs #((lambda (x) (* x 2))
                     (lambda (x) (* x 3))
                     (lambda (x) (* x 4))))
        ((vector-ref funcs 1) 5)
    "#);
    assert_eq!(result, Ok("15".to_string()));
}

#[test]
fn test_vector_literal_deeply_nested() {
    let result = compile_and_run("#(#(#(1 2) #(3 4)) #(#(5 6) #(7 8)))");
    assert_eq!(result, Ok("#(#(#(1 2) #(3 4)) #(#(5 6) #(7 8)))".to_string()));
}

#[test]
fn test_vector_literal_with_quoted_expressions() {
    let result = compile_and_run("#('a 'b 'c)");
    assert_eq!(result, Ok("#(a b c)".to_string()));
}

#[test]
fn test_vector_literal_comparison_with_vector() {
    let result = compile_and_run(r#"
        (def v1 #(1 2 3))
        (def v2 (vector 1 2 3))
        (== (vector-ref v1 0) (vector-ref v2 0))
    "#);
    assert_eq!(result, Ok("true".to_string()));
}

#[test]
fn test_multiple_vector_literals_in_expression() {
    let result = compile_and_run(r#"
        (+ (vector-ref #(10 20 30) 0)
           (vector-ref #(5 15 25) 1))
    "#);
    assert_eq!(result, Ok("25".to_string()));
}

// Built-in reader macro tests: #t and #f

#[test]
fn test_boolean_true_reader_macro() {
    let result = compile_and_run("#t");
    assert_eq!(result, Ok("true".to_string()));
}

#[test]
fn test_boolean_false_reader_macro() {
    let result = compile_and_run("#f");
    assert_eq!(result, Ok("false".to_string()));
}

#[test]
fn test_boolean_reader_macros_in_conditionals() {
    let result = compile_and_run(r#"
        (if #t 42 99)
    "#);
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_boolean_reader_macros_mixed() {
    let result = compile_and_run(r#"
        (if #f
            99
            (if #t 42 0))
    "#);
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_boolean_reader_macros_in_vector() {
    let result = compile_and_run(r#"
        #(#t #f true false)
    "#);
    assert_eq!(result, Ok("#(true false true false)".to_string()));
}

// Built-in reader macro tests: #;

#[test]
fn test_expression_comment_simple() {
    let result = compile_and_run(r#"
        (+ 1 #;2 3)
    "#);
    assert_eq!(result, Ok("4".to_string()));
}

#[test]
fn test_expression_comment_complex() {
    let result = compile_and_run(r#"
        (+ 1 #;(* 10 20) 3 #;(- 100 50) 5)
    "#);
    assert_eq!(result, Ok("9".to_string()));
}

#[test]
fn test_expression_comment_nested() {
    let result = compile_and_run(r#"
        (+ 1 #;#;2 3 4)
    "#);
    assert_eq!(result, Ok("5".to_string())); // 2 and 3 are both commented
}

#[test]
fn test_expression_comment_at_top_level() {
    let result = compile_and_run(r#"
        #;(def x 10)
        42
    "#);
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_expression_comment_with_defun() {
    let result = compile_and_run(r#"
        #;(defun bad-function (x) (/ x 0))
        (defun good-function (x) (* x 2))
        (good-function 21)
    "#);
    assert_eq!(result, Ok("42".to_string()));
}

// Built-in reader macro tests: #'
// Note: #'func is syntactic sugar for func
// It provides visual clarity that you're referring to a function

#[test]
fn test_function_quote_basic() {
    let result = compile_and_run(r#"
        (defun double (x) (* x 2))
        (#'double 21)
    "#);
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_function_quote_equivalent_to_bare_name() {
    // #'func and func are equivalent
    let result1 = compile_and_run(r#"
        (defun add-one (x) (+ x 1))
        (#'add-one 41)
    "#);
    let result2 = compile_and_run(r#"
        (defun add-one (x) (+ x 1))
        (add-one 41)
    "#);
    assert_eq!(result1, result2);
    assert_eq!(result1, Ok("42".to_string()));
}

#[test]
fn test_function_quote_builtin() {
    // Works with builtin functions too
    let result = compile_and_run(r#"
        (#'+ 20 22)
    "#);
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_function_quote_in_list() {
    // Can use #' in any context where you'd use a function name
    let result = compile_and_run(r#"
        (defun triple (x) (* x 3))
        '(#'triple)
    "#);
    assert_eq!(result, Ok("(triple)".to_string()));
}

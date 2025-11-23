use lisp_bytecode_vm::{Compiler, parser::Parser, Instruction, Value};

#[test]
fn test_compile_number() {
    let mut parser = Parser::new("42");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).unwrap();

    assert_eq!(functions.len(), 0);
    assert_eq!(main.len(), 2); // Push + Halt
    assert!(matches!(main[0], Instruction::Push(Value::Integer(42))));
    assert!(matches!(main[1], Instruction::Halt));
}

#[test]
fn test_compile_arithmetic() {
    let mut parser = Parser::new("(+ 5 3)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    // Should have: Push(5), Push(3), Add, Halt
    assert_eq!(main.len(), 4);
    assert!(matches!(main[0], Instruction::Push(Value::Integer(5))));
    assert!(matches!(main[1], Instruction::Push(Value::Integer(3))));
    assert!(matches!(main[2], Instruction::Add));
    assert!(matches!(main[3], Instruction::Halt));
}

#[test]
fn test_compile_all_arithmetic_operators() {
    let operators = vec![
        ("+", Instruction::Add),
        ("-", Instruction::Sub),
        ("*", Instruction::Mul),
        ("/", Instruction::Div),
        ("%", Instruction::Mod),
    ];

    for (op, expected_instr) in operators {
        let source = format!("({} 10 2)", op);
        let mut parser = Parser::new(&source);
        let exprs = parser.parse_all().unwrap();

        let mut compiler = Compiler::new();
        let (_, main) = compiler.compile_program(&exprs).unwrap();

        assert_eq!(main.len(), 4);
        assert!(matches!(main[2], ref instr if std::mem::discriminant(instr) == std::mem::discriminant(&expected_instr)));
    }
}

#[test]
fn test_compile_comparison_operators() {
    let operators = vec![
        ("<=", Instruction::Leq),
        ("<", Instruction::Lt),
        (">", Instruction::Gt),
        (">=", Instruction::Gte),
        ("==", Instruction::Eq),
        ("!=", Instruction::Neq),
    ];

    for (op, expected_instr) in operators {
        let source = format!("({} 5 3)", op);
        let mut parser = Parser::new(&source);
        let exprs = parser.parse_all().unwrap();

        let mut compiler = Compiler::new();
        let (_, main) = compiler.compile_program(&exprs).unwrap();

        assert_eq!(main.len(), 4);
        assert!(matches!(main[2], ref instr if std::mem::discriminant(instr) == std::mem::discriminant(&expected_instr)));
    }
}

#[test]
fn test_compile_negation() {
    let mut parser = Parser::new("(neg 5)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    assert_eq!(main.len(), 3);
    assert!(matches!(main[0], Instruction::Push(Value::Integer(5))));
    assert!(matches!(main[1], Instruction::Neg));
}

#[test]
fn test_compile_if_expression() {
    let mut parser = Parser::new("(if (> 5 3) 10 20)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    // Should have comparison, conditional jump, branches, and halt
    assert!(main.len() > 5);

    // Check for JmpIfFalse and Jmp instructions
    let has_jmp_if_false = main.iter().any(|i| matches!(i, Instruction::JmpIfFalse(_)));
    let has_jmp = main.iter().any(|i| matches!(i, Instruction::Jmp(_)));

    assert!(has_jmp_if_false);
    assert!(has_jmp);
}

#[test]
fn test_compile_function_definition() {
    let mut parser = Parser::new("(defun double (x) (* x 2))");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, _main) = compiler.compile_program(&exprs).unwrap();

    assert_eq!(functions.len(), 1);
    assert!(functions.contains_key("double"));

    let double_bytecode = &functions["double"];
    assert!(double_bytecode.len() > 0);

    // Should have LoadArg, Push(2), Mul, Ret
    assert!(matches!(double_bytecode[0], Instruction::LoadArg(0)));
    assert!(matches!(double_bytecode.last(), Some(Instruction::Ret)));
}

#[test]
fn test_compile_function_call() {
    let mut parser = Parser::new("(defun add1 (x) (+ x 1)) (add1 5)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, main) = compiler.compile_program(&exprs).unwrap();

    assert_eq!(functions.len(), 1);

    // Main should call the function
    let has_call = main.iter().any(|i| matches!(i, Instruction::Call(name, 1) if name == "add1"));
    assert!(has_call);
}

#[test]
fn test_compile_nested_expressions() {
    let mut parser = Parser::new("(+ (* 2 3) (- 10 5))");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    // Should compile nested multiplications and subtractions before the addition
    assert!(main.len() > 5);

    let mul_pos = main.iter().position(|i| matches!(i, Instruction::Mul));
    let sub_pos = main.iter().position(|i| matches!(i, Instruction::Sub));
    let add_pos = main.iter().position(|i| matches!(i, Instruction::Add));

    assert!(mul_pos.is_some());
    assert!(sub_pos.is_some());
    assert!(add_pos.is_some());

    // Both mul and sub should come before add
    assert!(mul_pos.unwrap() < add_pos.unwrap());
    assert!(sub_pos.unwrap() < add_pos.unwrap());
}

#[test]
fn test_compile_error_undefined_variable() {
    let mut parser = Parser::new("unknown_var");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let result = compiler.compile_program(&exprs);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("not found"));
}

#[test]
fn test_compile_error_wrong_arg_count() {
    let mut parser = Parser::new("(+ 1)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let result = compiler.compile_program(&exprs);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("at least 2 arguments"));
}

#[test]
fn test_compile_multiple_functions() {
    let source = r#"
        (defun add (a b) (+ a b))
        (defun mul (a b) (* a b))
        (defun square (x) (mul x x))
    "#;

    let mut parser = Parser::new(source);
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (functions, _) = compiler.compile_program(&exprs).unwrap();

    assert_eq!(functions.len(), 3);
    assert!(functions.contains_key("add"));
    assert!(functions.contains_key("mul"));
    assert!(functions.contains_key("square"));
}

#[test]
fn test_compile_variadic_addition() {
    let mut parser = Parser::new("(+ 1 2 3 4)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    // Should have: Push(1), Push(2), Add, Push(3), Add, Push(4), Add, Halt
    assert_eq!(main.len(), 8);
    assert!(matches!(main[0], Instruction::Push(Value::Integer(1))));
    assert!(matches!(main[1], Instruction::Push(Value::Integer(2))));
    assert!(matches!(main[2], Instruction::Add));
    assert!(matches!(main[3], Instruction::Push(Value::Integer(3))));
    assert!(matches!(main[4], Instruction::Add));
    assert!(matches!(main[5], Instruction::Push(Value::Integer(4))));
    assert!(matches!(main[6], Instruction::Add));
    assert!(matches!(main[7], Instruction::Halt));
}

#[test]
fn test_compile_variadic_multiplication() {
    let mut parser = Parser::new("(* 2 3 4)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    // Should have: Push(2), Push(3), Mul, Push(4), Mul, Halt
    assert_eq!(main.len(), 6);
    assert!(matches!(main[0], Instruction::Push(Value::Integer(2))));
    assert!(matches!(main[1], Instruction::Push(Value::Integer(3))));
    assert!(matches!(main[2], Instruction::Mul));
    assert!(matches!(main[3], Instruction::Push(Value::Integer(4))));
    assert!(matches!(main[4], Instruction::Mul));
    assert!(matches!(main[5], Instruction::Halt));
}

#[test]
fn test_compile_variadic_subtraction() {
    let mut parser = Parser::new("(- 10 2 3)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    // Should have: Push(10), Push(2), Sub, Push(3), Sub, Halt
    assert_eq!(main.len(), 6);
    assert!(matches!(main[0], Instruction::Push(Value::Integer(10))));
    assert!(matches!(main[1], Instruction::Push(Value::Integer(2))));
    assert!(matches!(main[2], Instruction::Sub));
    assert!(matches!(main[3], Instruction::Push(Value::Integer(3))));
    assert!(matches!(main[4], Instruction::Sub));
    assert!(matches!(main[5], Instruction::Halt));
}

#[test]
fn test_compile_variadic_division() {
    let mut parser = Parser::new("(/ 20 2 5)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    // Should have: Push(20), Push(2), Div, Push(5), Div, Halt
    assert_eq!(main.len(), 6);
    assert!(matches!(main[0], Instruction::Push(Value::Integer(20))));
    assert!(matches!(main[1], Instruction::Push(Value::Integer(2))));
    assert!(matches!(main[2], Instruction::Div));
    assert!(matches!(main[3], Instruction::Push(Value::Integer(5))));
    assert!(matches!(main[4], Instruction::Div));
    assert!(matches!(main[5], Instruction::Halt));
}

#[test]
fn test_compile_variadic_modulo() {
    let mut parser = Parser::new("(% 10 4 2)");
    let exprs = parser.parse_all().unwrap();

    let mut compiler = Compiler::new();
    let (_, main) = compiler.compile_program(&exprs).unwrap();

    // Should have: Push(10), Push(4), Mod, Push(2), Mod, Halt
    assert_eq!(main.len(), 6);
    assert!(matches!(main[0], Instruction::Push(Value::Integer(10))));
    assert!(matches!(main[1], Instruction::Push(Value::Integer(4))));
    assert!(matches!(main[2], Instruction::Mod));
    assert!(matches!(main[3], Instruction::Push(Value::Integer(2))));
    assert!(matches!(main[4], Instruction::Mod));
    assert!(matches!(main[5], Instruction::Halt));
}

#[test]
fn test_compile_error_single_arg_arithmetic() {
    // Test that operators still require at least 2 arguments
    let operators = vec!["+", "-", "*", "/", "%"];

    for op in operators {
        let source = format!("({} 1)", op);
        let mut parser = Parser::new(&source);
        let exprs = parser.parse_all().unwrap();

        let mut compiler = Compiler::new();
        let result = compiler.compile_program(&exprs);

        assert!(result.is_err(), "Expected error for {} with 1 argument", op);
        let err = result.unwrap_err();
        assert!(err.message.contains("at least 2 arguments"),
                "Error message for {} should mention 'at least 2 arguments'", op);
    }
}

use lisp_bytecode_vm::{VM, Instruction, Value};

#[test]
fn test_vm_push_and_halt() {
    let mut vm = VM::new();
    vm.current_bytecode = vec![
        Instruction::Push(Value::Integer(42)),
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(42));
    assert!(vm.halted);
}

#[test]
fn test_vm_arithmetic_add() {
    let mut vm = VM::new();
    vm.current_bytecode = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Add,
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(8));
}

#[test]
fn test_vm_arithmetic_operations() {
    let tests = vec![
        (Instruction::Add, 10, 3, 13),
        (Instruction::Sub, 10, 3, 7),
        (Instruction::Mul, 10, 3, 30),
        (Instruction::Div, 10, 2, 5),
        (Instruction::Mod, 10, 3, 1),
    ];

    for (op, a, b, expected) in tests {
        let mut vm = VM::new();
        vm.current_bytecode = vec![
            Instruction::Push(Value::Integer(a)),
            Instruction::Push(Value::Integer(b)),
            op,
            Instruction::Halt,
        ];

        vm.run();

        assert_eq!(vm.value_stack.len(), 1);
        assert_eq!(vm.value_stack[0], Value::Integer(expected));
    }
}

#[test]
fn test_vm_negation() {
    let mut vm = VM::new();
    vm.current_bytecode = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Neg,
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(-5));
}

#[test]
fn test_vm_comparison_operations() {
    let tests = vec![
        (Instruction::Lt, 5, 10, true),
        (Instruction::Lt, 10, 5, false),
        (Instruction::Gt, 10, 5, true),
        (Instruction::Gt, 5, 10, false),
        (Instruction::Leq, 5, 5, true),
        (Instruction::Leq, 5, 10, true),
        (Instruction::Leq, 10, 5, false),
        (Instruction::Gte, 5, 5, true),
        (Instruction::Gte, 10, 5, true),
        (Instruction::Gte, 5, 10, false),
        (Instruction::Eq, 5, 5, true),
        (Instruction::Eq, 5, 10, false),
        (Instruction::Neq, 5, 10, true),
        (Instruction::Neq, 5, 5, false),
    ];

    for (op, a, b, expected) in tests {
        let mut vm = VM::new();
        vm.current_bytecode = vec![
            Instruction::Push(Value::Integer(a)),
            Instruction::Push(Value::Integer(b)),
            op,
            Instruction::Halt,
        ];

        vm.run();

        assert_eq!(vm.value_stack.len(), 1);
        assert_eq!(vm.value_stack[0], Value::Boolean(expected));
    }
}

#[test]
fn test_vm_conditional_jump_true() {
    let mut vm = VM::new();
    vm.current_bytecode = vec![
        Instruction::Push(Value::Boolean(true)),
        Instruction::JmpIfFalse(4),
        Instruction::Push(Value::Integer(10)),
        Instruction::Jmp(5),
        Instruction::Push(Value::Integer(20)),
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(10));
}

#[test]
fn test_vm_conditional_jump_false() {
    let mut vm = VM::new();
    vm.current_bytecode = vec![
        Instruction::Push(Value::Boolean(false)),
        Instruction::JmpIfFalse(4),
        Instruction::Push(Value::Integer(10)),
        Instruction::Jmp(5),
        Instruction::Push(Value::Integer(20)),
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(20));
}

#[test]
fn test_vm_function_call() {
    let mut vm = VM::new();

    // Define a simple function: double(x) = x * 2
    let double_fn = vec![
        Instruction::LoadArg(0),
        Instruction::Push(Value::Integer(2)),
        Instruction::Mul,
        Instruction::Ret,
    ];

    vm.functions.insert("double".to_string(), double_fn);

    // Main: call double(5)
    vm.current_bytecode = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Call("double".to_string(), 1),
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(10));
}

#[test]
fn test_vm_recursive_function() {
    let mut vm = VM::new();

    // Define factorial function: fact(n) = if n <= 1 then 1 else n * fact(n-1)
    let fact_fn = vec![
        Instruction::LoadArg(0),
        Instruction::Push(Value::Integer(1)),
        Instruction::Leq,
        Instruction::JmpIfFalse(6),
        Instruction::Push(Value::Integer(1)),
        Instruction::Jmp(12),
        Instruction::LoadArg(0),
        Instruction::LoadArg(0),
        Instruction::Push(Value::Integer(1)),
        Instruction::Sub,
        Instruction::Call("fact".to_string(), 1),
        Instruction::Mul,
        Instruction::Ret,
    ];

    vm.functions.insert("fact".to_string(), fact_fn);

    // Main: call fact(5) = 120
    vm.current_bytecode = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Call("fact".to_string(), 1),
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(120));
}

#[test]
fn test_vm_multiple_function_calls() {
    let mut vm = VM::new();

    // add(a, b) = a + b
    let add_fn = vec![
        Instruction::LoadArg(0),
        Instruction::LoadArg(1),
        Instruction::Add,
        Instruction::Ret,
    ];

    vm.functions.insert("add".to_string(), add_fn);

    // Main: add(3, 4) + add(5, 6)
    vm.current_bytecode = vec![
        Instruction::Push(Value::Integer(3)),
        Instruction::Push(Value::Integer(4)),
        Instruction::Call("add".to_string(), 2),
        Instruction::Push(Value::Integer(5)),
        Instruction::Push(Value::Integer(6)),
        Instruction::Call("add".to_string(), 2),
        Instruction::Add,
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(18)); // 7 + 11
}

#[test]
fn test_vm_load_arg() {
    let mut vm = VM::new();

    // Function with 3 parameters
    let test_fn = vec![
        Instruction::LoadArg(0),
        Instruction::LoadArg(1),
        Instruction::Add,
        Instruction::LoadArg(2),
        Instruction::Mul,
        Instruction::Ret,
    ];

    vm.functions.insert("test".to_string(), test_fn);

    // Main: test(2, 3, 4) = (2 + 3) * 4 = 20
    vm.current_bytecode = vec![
        Instruction::Push(Value::Integer(2)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Push(Value::Integer(4)),
        Instruction::Call("test".to_string(), 3),
        Instruction::Halt,
    ];

    vm.run();

    assert_eq!(vm.value_stack.len(), 1);
    assert_eq!(vm.value_stack[0], Value::Integer(20));
}

#[test]
fn test_vm_stack_trace() {
    let mut vm = VM::new();

    // Inner function
    let inner_fn = vec![
        Instruction::Push(Value::Integer(42)),
        Instruction::Ret,
    ];

    // Outer function that calls inner
    let outer_fn = vec![
        Instruction::Call("inner".to_string(), 0),
        Instruction::Ret,
    ];

    vm.functions.insert("inner".to_string(), inner_fn);
    vm.functions.insert("outer".to_string(), outer_fn);

    vm.current_bytecode = vec![
        Instruction::Call("outer".to_string(), 0),
        Instruction::Halt,
    ];

    // Manually step through to check stack trace mid-execution
    vm.execute_one_instruction(); // Call outer
    assert_eq!(vm.get_stack_trace(), vec!["outer"]);

    vm.execute_one_instruction(); // Call inner
    assert_eq!(vm.get_stack_trace(), vec!["outer", "inner"]);
}

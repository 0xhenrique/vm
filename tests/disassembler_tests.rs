use lisp_bytecode_vm::{disassembler, Instruction, Value};
use std::collections::HashMap;

#[test]
fn test_disassemble_simple_program() {
    let functions = HashMap::new();
    let main = vec![
        Instruction::Push(Value::Integer(42)),
        Instruction::Halt,
    ];

    let output = disassembler::disassemble_bytecode(&functions, &main);

    assert!(output.contains("=== Main ==="));
    assert!(output.contains("2 instruction(s)"));
    assert!(output.contains("0: Push(Integer(42))"));
    assert!(output.contains("1: Halt"));
}

#[test]
fn test_disassemble_with_functions() {
    let mut functions = HashMap::new();

    functions.insert(
        "double".to_string(),
        vec![
            Instruction::LoadArg(0),
            Instruction::Push(Value::Integer(2)),
            Instruction::Mul,
            Instruction::Ret,
        ],
    );

    let main = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Call("double".to_string(), 1),
        Instruction::Halt,
    ];

    let output = disassembler::disassemble_bytecode(&functions, &main);

    assert!(output.contains("=== Functions ==="));
    assert!(output.contains("Function: double"));
    assert!(output.contains("4 instruction(s)"));
    assert!(output.contains("LoadArg(0)"));
    assert!(output.contains("Mul"));
    assert!(output.contains("Ret"));
    assert!(output.contains("=== Main ==="));
    assert!(output.contains("Call(\"double\", 1)"));
}

#[test]
fn test_disassemble_all_instructions() {
    let functions = HashMap::new();
    let main = vec![
        Instruction::Push(Value::Integer(10)),
        Instruction::Push(Value::Boolean(true)),
        Instruction::Add,
        Instruction::Sub,
        Instruction::Mul,
        Instruction::Div,
        Instruction::Mod,
        Instruction::Neg,
        Instruction::Leq,
        Instruction::Lt,
        Instruction::Gt,
        Instruction::Gte,
        Instruction::Eq,
        Instruction::Neq,
        Instruction::JmpIfFalse(10),
        Instruction::Jmp(5),
        Instruction::Call("test".to_string(), 2),
        Instruction::Ret,
        Instruction::LoadArg(3),
        Instruction::Print,
        Instruction::Halt,
    ];

    let output = disassembler::disassemble_bytecode(&functions, &main);

    // Check that all instruction types appear
    assert!(output.contains("Push(Integer(10))"));
    assert!(output.contains("Push(Boolean(true))"));
    assert!(output.contains("Add"));
    assert!(output.contains("Sub"));
    assert!(output.contains("Mul"));
    assert!(output.contains("Div"));
    assert!(output.contains("Mod"));
    assert!(output.contains("Neg"));
    assert!(output.contains("Leq"));
    assert!(output.contains("Lt"));
    assert!(output.contains("Gt"));
    assert!(output.contains("Gte"));
    assert!(output.contains("Eq"));
    assert!(output.contains("Neq"));
    assert!(output.contains("JmpIfFalse"));
    assert!(output.contains("Jmp"));
    assert!(output.contains("Call"));
    assert!(output.contains("Ret"));
    assert!(output.contains("LoadArg"));
    assert!(output.contains("Print"));
    assert!(output.contains("Halt"));
}

#[test]
fn test_disassemble_statistics() {
    let mut functions = HashMap::new();

    functions.insert(
        "func1".to_string(),
        vec![Instruction::Push(Value::Integer(1)), Instruction::Ret],
    );

    functions.insert(
        "func2".to_string(),
        vec![
            Instruction::Push(Value::Integer(2)),
            Instruction::Push(Value::Integer(3)),
            Instruction::Add,
            Instruction::Ret,
        ],
    );

    let main = vec![Instruction::Halt];

    let output = disassembler::disassemble_bytecode(&functions, &main);

    assert!(output.contains("=== Statistics ==="));
    assert!(output.contains("Total instructions: 7")); // 2 + 4 + 1
    assert!(output.contains("Functions: 2"));
    assert!(output.contains("Main instructions: 1"));
    assert!(output.contains("func1: 2"));
    assert!(output.contains("func2: 4"));
}

#[test]
fn test_get_statistics() {
    let mut functions = HashMap::new();

    functions.insert(
        "test".to_string(),
        vec![Instruction::Push(Value::Integer(1)), Instruction::Ret],
    );

    let main = vec![Instruction::Halt, Instruction::Halt];

    let stats = disassembler::get_statistics(&functions, &main);

    assert_eq!(stats.total_instructions, 4); // 2 + 2
    assert_eq!(stats.function_count, 1);
    assert_eq!(stats.main_instruction_count, 2);
    assert_eq!(stats.per_function_counts.get("test"), Some(&2));
}

#[test]
fn test_disassemble_empty_program() {
    let functions = HashMap::new();
    let main = vec![Instruction::Halt];

    let output = disassembler::disassemble_bytecode(&functions, &main);

    assert!(output.contains("=== Main ==="));
    assert!(output.contains("1 instruction(s)"));
    assert!(output.contains("=== Statistics ==="));
    assert!(output.contains("Functions: 0"));
}

#[test]
fn test_disassemble_address_format() {
    let functions = HashMap::new();
    let main = vec![
        Instruction::Push(Value::Integer(1)),
        Instruction::Push(Value::Integer(2)),
        Instruction::Add,
        Instruction::Halt,
    ];

    let output = disassembler::disassemble_bytecode(&functions, &main);

    // Check that addresses are properly formatted
    assert!(output.contains("0: Push"));
    assert!(output.contains("1: Push"));
    assert!(output.contains("2: Add"));
    assert!(output.contains("3: Halt"));
}

#[test]
fn test_disassemble_multiple_functions_sorted() {
    let mut functions = HashMap::new();

    functions.insert("zebra".to_string(), vec![Instruction::Ret]);
    functions.insert("apple".to_string(), vec![Instruction::Ret]);
    functions.insert("monkey".to_string(), vec![Instruction::Ret]);

    let main = vec![Instruction::Halt];

    let output = disassembler::disassemble_bytecode(&functions, &main);

    // Check that functions appear in sorted order
    let apple_pos = output.find("Function: apple").unwrap();
    let monkey_pos = output.find("Function: monkey").unwrap();
    let zebra_pos = output.find("Function: zebra").unwrap();

    assert!(apple_pos < monkey_pos);
    assert!(monkey_pos < zebra_pos);
}

#[test]
fn test_disassemble_conditional_jumps() {
    let functions = HashMap::new();
    let main = vec![
        Instruction::Push(Value::Boolean(true)),
        Instruction::JmpIfFalse(4),
        Instruction::Push(Value::Integer(10)),
        Instruction::Jmp(5),
        Instruction::Push(Value::Integer(20)),
        Instruction::Halt,
    ];

    let output = disassembler::disassemble_bytecode(&functions, &main);

    assert!(output.contains("JmpIfFalse(4)"));
    assert!(output.contains("Jmp(5)"));
}

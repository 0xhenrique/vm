use lisp_bytecode_vm::{bytecode, Instruction, Value};
use std::collections::HashMap;

#[test]
fn test_serialize_deserialize_simple() {
    let functions = HashMap::new();
    let main = vec![
        Instruction::Push(Value::Integer(42)),
        Instruction::Halt,
    ];

    let bytes = bytecode::serialize_bytecode(&functions, &main);
    let (loaded_functions, loaded_main) = bytecode::deserialize_bytecode(&bytes).unwrap();

    assert_eq!(loaded_functions.len(), 0);
    assert_eq!(loaded_main.len(), 2);
    assert!(matches!(loaded_main[0], Instruction::Push(Value::Integer(42))));
    assert!(matches!(loaded_main[1], Instruction::Halt));
}

#[test]
fn test_serialize_all_instructions() {
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

    let bytes = bytecode::serialize_bytecode(&functions, &main);
    let (_, loaded_main) = bytecode::deserialize_bytecode(&bytes).unwrap();

    assert_eq!(loaded_main.len(), main.len());

    // Verify a few key instructions
    assert!(matches!(loaded_main[0], Instruction::Push(Value::Integer(10))));
    assert!(matches!(loaded_main[1], Instruction::Push(Value::Boolean(true))));
    assert!(matches!(loaded_main[2], Instruction::Add));
    assert!(matches!(loaded_main[7], Instruction::Neg));
    assert!(matches!(loaded_main[14], Instruction::JmpIfFalse(10)));
    assert!(matches!(loaded_main[16], Instruction::Call(ref name, 2) if name == "test"));
}

#[test]
fn test_serialize_with_functions() {
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

    functions.insert(
        "add".to_string(),
        vec![
            Instruction::LoadArg(0),
            Instruction::LoadArg(1),
            Instruction::Add,
            Instruction::Ret,
        ],
    );

    let main = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Call("double".to_string(), 1),
        Instruction::Halt,
    ];

    let bytes = bytecode::serialize_bytecode(&functions, &main);
    let (loaded_functions, loaded_main) = bytecode::deserialize_bytecode(&bytes).unwrap();

    assert_eq!(loaded_functions.len(), 2);
    assert!(loaded_functions.contains_key("double"));
    assert!(loaded_functions.contains_key("add"));

    let double_fn = &loaded_functions["double"];
    assert_eq!(double_fn.len(), 4);
    assert!(matches!(double_fn[0], Instruction::LoadArg(0)));
    assert!(matches!(double_fn[3], Instruction::Ret));

    assert_eq!(loaded_main.len(), 3);
    assert!(matches!(loaded_main[1], Instruction::Call(ref name, 1) if name == "double"));
}

#[test]
fn test_round_trip_complex_program() {
    let mut functions = HashMap::new();

    // Factorial function
    functions.insert(
        "fact".to_string(),
        vec![
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
        ],
    );

    let main = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Call("fact".to_string(), 1),
        Instruction::Print,
        Instruction::Halt,
    ];

    let bytes = bytecode::serialize_bytecode(&functions, &main);
    let (loaded_functions, loaded_main) = bytecode::deserialize_bytecode(&bytes).unwrap();

    assert_eq!(loaded_functions.len(), 1);
    assert_eq!(loaded_functions["fact"].len(), 13);
    assert_eq!(loaded_main.len(), 4);
}

#[test]
fn test_file_save_load() {
    let mut functions = HashMap::new();
    functions.insert(
        "test".to_string(),
        vec![Instruction::Push(Value::Integer(123)), Instruction::Ret],
    );

    let main = vec![Instruction::Halt];

    let temp_file = "/tmp/test_bytecode.bc";

    // Save
    bytecode::save_bytecode_file(temp_file, &functions, &main).unwrap();

    // Load
    let (loaded_functions, loaded_main) = bytecode::load_bytecode_file(temp_file).unwrap();

    assert_eq!(loaded_functions.len(), 1);
    assert!(loaded_functions.contains_key("test"));
    assert_eq!(loaded_main.len(), 1);

    // Clean up
    std::fs::remove_file(temp_file).ok();
}

#[test]
fn test_deserialize_invalid_magic() {
    let bad_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Wrong magic number
    let result = bytecode::deserialize_bytecode(&bad_bytes);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("magic number"));
}

#[test]
fn test_serialize_negative_numbers() {
    let functions = HashMap::new();
    let main = vec![
        Instruction::Push(Value::Integer(-42)),
        Instruction::Push(Value::Integer(-1)),
        Instruction::Halt,
    ];

    let bytes = bytecode::serialize_bytecode(&functions, &main);
    let (_, loaded_main) = bytecode::deserialize_bytecode(&bytes).unwrap();

    assert!(matches!(loaded_main[0], Instruction::Push(Value::Integer(-42))));
    assert!(matches!(loaded_main[1], Instruction::Push(Value::Integer(-1))));
}

#[test]
fn test_serialize_boolean_values() {
    let functions = HashMap::new();
    let main = vec![
        Instruction::Push(Value::Boolean(true)),
        Instruction::Push(Value::Boolean(false)),
        Instruction::Halt,
    ];

    let bytes = bytecode::serialize_bytecode(&functions, &main);
    let (_, loaded_main) = bytecode::deserialize_bytecode(&bytes).unwrap();

    assert!(matches!(loaded_main[0], Instruction::Push(Value::Boolean(true))));
    assert!(matches!(loaded_main[1], Instruction::Push(Value::Boolean(false))));
}

#[test]
fn test_serialize_large_numbers() {
    let functions = HashMap::new();
    let main = vec![
        Instruction::Push(Value::Integer(i64::MAX)),
        Instruction::Push(Value::Integer(i64::MIN)),
        Instruction::Halt,
    ];

    let bytes = bytecode::serialize_bytecode(&functions, &main);
    let (_, loaded_main) = bytecode::deserialize_bytecode(&bytes).unwrap();

    assert!(matches!(loaded_main[0], Instruction::Push(Value::Integer(i64::MAX))));
    assert!(matches!(loaded_main[1], Instruction::Push(Value::Integer(i64::MIN))));
}

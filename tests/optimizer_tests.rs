use lisp_bytecode_vm::{Instruction, Value, optimizer::Optimizer};
use std::collections::HashMap;

#[test]
fn test_constant_folding_add() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Add,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Integer(8))));
    assert!(matches!(optimized[1], Instruction::Halt));
    assert_eq!(optimizer.get_stats().constant_folds, 1);
}

#[test]
fn test_constant_folding_sub() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(10)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Sub,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Integer(7))));
}

#[test]
fn test_constant_folding_mul() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(4)),
        Instruction::Push(Value::Integer(5)),
        Instruction::Mul,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Integer(20))));
}

#[test]
fn test_constant_folding_div() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(20)),
        Instruction::Push(Value::Integer(4)),
        Instruction::Div,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Integer(5))));
}

#[test]
fn test_constant_folding_mod() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(10)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Mod,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Integer(1))));
}

#[test]
fn test_constant_folding_neg() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(42)),
        Instruction::Neg,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Integer(-42))));
}

#[test]
fn test_constant_folding_comparisons() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Gt,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Boolean(true))));
}

#[test]
fn test_constant_folding_leq() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(3)),
        Instruction::Push(Value::Integer(5)),
        Instruction::Leq,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Boolean(true))));
}

#[test]
fn test_constant_folding_eq() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Push(Value::Integer(5)),
        Instruction::Eq,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert!(matches!(optimized[0], Instruction::Push(Value::Boolean(true))));
}

#[test]
fn test_constant_folding_multiple() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(2)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Add,
        Instruction::Push(Value::Integer(4)),
        Instruction::Push(Value::Integer(5)),
        Instruction::Mul,
        Instruction::Add,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 4);
    assert!(matches!(optimized[0], Instruction::Push(Value::Integer(5))));
    assert!(matches!(optimized[1], Instruction::Push(Value::Integer(20))));
    assert!(matches!(optimized[2], Instruction::Add));
    assert_eq!(optimizer.get_stats().constant_folds, 2);
}

#[test]
fn test_constant_folding_no_division_by_zero() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(10)),
        Instruction::Push(Value::Integer(0)),
        Instruction::Div,
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 4);
    assert_eq!(optimizer.get_stats().constant_folds, 0);
}

#[test]
fn test_dead_code_after_halt() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(42)),
        Instruction::Halt,
        Instruction::Push(Value::Integer(99)),
        Instruction::Print,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert_eq!(optimizer.get_stats().dead_code_removed, 2);
}

#[test]
fn test_dead_code_after_ret() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(42)),
        Instruction::Ret,
        Instruction::Push(Value::Integer(99)),
    ];

    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 2);
    assert_eq!(optimizer.get_stats().dead_code_removed, 1);
}

#[test]
fn test_dead_code_unreachable_branch() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(1)),
        Instruction::Jmp(3),
        Instruction::Push(Value::Integer(99)),
        Instruction::Halt,
    ];

    let original_len = bytecode.len();
    let optimized = optimizer.optimize(bytecode);

    assert!(optimized.len() < original_len);
    assert_eq!(optimizer.get_stats().dead_code_removed, 1);
}

#[test]
fn test_jump_to_jump_elimination() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Jmp(1),
        Instruction::Jmp(2),
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert!(matches!(optimized[0], Instruction::Jmp(2)));
    assert_eq!(optimizer.get_stats().jump_chains_simplified, 1);
}

#[test]
fn test_jump_to_jump_chain() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Jmp(1),
        Instruction::Jmp(2),
        Instruction::Jmp(3),
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert!(matches!(optimized[0], Instruction::Jmp(_)));
    assert!(optimizer.get_stats().jump_chains_simplified >= 1);
    assert!(optimizer.get_stats().dead_code_removed >= 2);
}

#[test]
fn test_conditional_jump_elimination() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Boolean(false)),
        Instruction::JmpIfFalse(3),
        Instruction::Print,
        Instruction::Jmp(4),
        Instruction::Halt,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert!(matches!(optimized[1], Instruction::JmpIfFalse(4)));
}

#[test]
fn test_optimize_empty_bytecode() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![];
    let optimized = optimizer.optimize(bytecode);

    assert_eq!(optimized.len(), 0);
}

#[test]
fn test_optimize_functions() {
    let mut optimizer = Optimizer::new();

    let mut functions = HashMap::new();
    functions.insert(
        "test".to_string(),
        vec![
            Instruction::Push(Value::Integer(2)),
            Instruction::Push(Value::Integer(3)),
            Instruction::Add,
            Instruction::Ret,
        ],
    );

    let optimized_functions = optimizer.optimize_functions(functions);

    assert_eq!(optimized_functions.len(), 1);
    let test_fn = &optimized_functions["test"];
    assert_eq!(test_fn.len(), 2);
    assert!(matches!(test_fn[0], Instruction::Push(Value::Integer(5))));
    assert!(matches!(test_fn[1], Instruction::Ret));
}

#[test]
fn test_optimization_stats_reduction_percentage() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(2)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Add,
        Instruction::Halt,
        Instruction::Print,
    ];

    optimizer.optimize(bytecode);

    let stats = optimizer.get_stats();
    assert_eq!(stats.original_instruction_count, 5);
    assert_eq!(stats.optimized_instruction_count, 2);
    assert_eq!(stats.reduction_percentage(), 60.0);
}

#[test]
fn test_no_optimization_needed() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::LoadArg(0),
        Instruction::Push(Value::Integer(1)),
        Instruction::Add,
        Instruction::Ret,
    ];

    let optimized = optimizer.optimize(bytecode.clone());

    assert_eq!(optimized.len(), bytecode.len());
    assert_eq!(optimizer.get_stats().constant_folds, 0);
    assert_eq!(optimizer.get_stats().dead_code_removed, 0);
}

#[test]
fn test_complex_optimization() {
    let mut optimizer = Optimizer::new();

    let bytecode = vec![
        Instruction::Push(Value::Integer(10)),
        Instruction::Push(Value::Integer(5)),
        Instruction::Sub,
        Instruction::Push(Value::Integer(2)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Mul,
        Instruction::Add,
        Instruction::Halt,
        Instruction::Print,
        Instruction::Print,
    ];

    let optimized = optimizer.optimize(bytecode);

    assert!(optimized.len() < 10);
    assert!(optimizer.get_stats().constant_folds > 0);
    assert!(optimizer.get_stats().dead_code_removed > 0);
}

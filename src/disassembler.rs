use crate::Instruction;
use std::collections::HashMap;

pub struct DisassemblerStats {
    pub total_instructions: usize,
    pub function_count: usize,
    pub per_function_counts: HashMap<String, usize>,
    pub main_instruction_count: usize,
}

pub fn disassemble_bytecode(
    functions: &HashMap<String, Vec<Instruction>>,
    main: &[Instruction],
) -> String {
    let mut output = String::new();

    // Disassemble functions
    if !functions.is_empty() {
        output.push_str("=== Functions ===\n\n");

        let mut sorted_functions: Vec<_> = functions.iter().collect();
        sorted_functions.sort_by_key(|(name, _)| *name);

        for (name, bytecode) in sorted_functions {
            output.push_str(&format!("Function: {}\n", name));
            output.push_str(&format!("  {} instruction(s)\n", bytecode.len()));
            output.push_str(&disassemble_instructions(bytecode));
            output.push_str("\n");
        }
    }

    // Disassemble main bytecode
    output.push_str("=== Main ===\n");
    output.push_str(&format!("  {} instruction(s)\n", main.len()));
    output.push_str(&disassemble_instructions(main));

    // Add statistics
    output.push_str("\n");
    output.push_str(&format_statistics(functions, main));

    output
}

fn disassemble_instructions(bytecode: &[Instruction]) -> String {
    let mut output = String::new();

    for (addr, instr) in bytecode.iter().enumerate() {
        output.push_str(&format!("  {:4}: {}\n", addr, format_instruction(instr)));
    }

    output
}

fn format_instruction(instr: &Instruction) -> String {
    match instr {
        Instruction::Push(val) => format!("Push({:?})", val),
        Instruction::Add => "Add".to_string(),
        Instruction::Sub => "Sub".to_string(),
        Instruction::Mul => "Mul".to_string(),
        Instruction::Div => "Div".to_string(),
        Instruction::Mod => "Mod".to_string(),
        Instruction::Neg => "Neg".to_string(),
        Instruction::Leq => "Leq".to_string(),
        Instruction::Lt => "Lt".to_string(),
        Instruction::Gt => "Gt".to_string(),
        Instruction::Gte => "Gte".to_string(),
        Instruction::Eq => "Eq".to_string(),
        Instruction::Neq => "Neq".to_string(),
        Instruction::JmpIfFalse(addr) => format!("JmpIfFalse({})", addr),
        Instruction::Jmp(addr) => format!("Jmp({})", addr),
        Instruction::Call(name, argc) => format!("Call(\"{}\", {})", name, argc),
        Instruction::TailCall(name, argc) => format!("TailCall(\"{}\", {})", name, argc),
        Instruction::Ret => "Ret".to_string(),
        Instruction::LoadArg(idx) => format!("LoadArg({})", idx),
        Instruction::Print => "Print".to_string(),
        Instruction::Halt => "Halt".to_string(),
        Instruction::Cons => "Cons".to_string(),
        Instruction::Car => "Car".to_string(),
        Instruction::Cdr => "Cdr".to_string(),
        Instruction::IsList => "IsList".to_string(),
        Instruction::IsString => "IsString".to_string(),
        Instruction::IsSymbol => "IsSymbol".to_string(),
        Instruction::SymbolToString => "SymbolToString".to_string(),
        Instruction::StringToSymbol => "StringToSymbol".to_string(),
        Instruction::GetLocal(pos) => format!("GetLocal({})", pos),
        Instruction::PopN(n) => format!("PopN({})", n),
        Instruction::Slide(n) => format!("Slide({})", n),
        Instruction::CheckArity(arity, addr) => format!("CheckArity({}, {})", arity, addr),
        Instruction::MakeClosure(params, body, num_captured) => {
            format!("MakeClosure({:?}, {} instructions, {} captured)", params, body.len(), num_captured)
        }
        Instruction::CallClosure(argc) => format!("CallClosure({})", argc),
        Instruction::LoadCaptured(idx) => format!("LoadCaptured({})", idx),
        Instruction::Append => "Append".to_string(),
        Instruction::MakeList(n) => format!("MakeList({})", n),
    }
}

fn format_statistics(
    functions: &HashMap<String, Vec<Instruction>>,
    main: &[Instruction],
) -> String {
    let mut output = String::new();

    output.push_str("=== Statistics ===\n");

    let total_function_instructions: usize = functions.values().map(|v| v.len()).sum();
    let total_instructions = total_function_instructions + main.len();

    output.push_str(&format!("Total instructions: {}\n", total_instructions));
    output.push_str(&format!("Functions: {}\n", functions.len()));
    output.push_str(&format!("Main instructions: {}\n", main.len()));

    if !functions.is_empty() {
        output.push_str("\nPer-function instruction counts:\n");
        let mut sorted_functions: Vec<_> = functions.iter().collect();
        sorted_functions.sort_by_key(|(name, _)| *name);

        for (name, bytecode) in sorted_functions {
            output.push_str(&format!("  {}: {}\n", name, bytecode.len()));
        }
    }

    output
}

pub fn get_statistics(
    functions: &HashMap<String, Vec<Instruction>>,
    main: &[Instruction],
) -> DisassemblerStats {
    let mut per_function_counts = HashMap::new();
    for (name, bytecode) in functions {
        per_function_counts.insert(name.clone(), bytecode.len());
    }

    let total_function_instructions: usize = functions.values().map(|v| v.len()).sum();
    let total_instructions = total_function_instructions + main.len();

    DisassemblerStats {
        total_instructions,
        function_count: functions.len(),
        per_function_counts,
        main_instruction_count: main.len(),
    }
}

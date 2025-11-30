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
        Instruction::SetLocal(pos) => format!("SetLocal({})", pos),
        Instruction::BeginLoop(count) => format!("BeginLoop({})", count),
        Instruction::Recur(count) => format!("Recur({})", count),
        Instruction::PopN(n) => format!("PopN({})", n),
        Instruction::Slide(n) => format!("Slide({})", n),
        Instruction::CheckArity(arity, addr) => format!("CheckArity({}, {})", arity, addr),
        Instruction::MakeClosure(params, body, num_captured) => {
            format!("MakeClosure({:?}, {} instructions, {} captured)", params, body.len(), num_captured)
        }
        Instruction::CallClosure(argc) => format!("CallClosure({})", argc),
        Instruction::Apply => "Apply".to_string(),
        Instruction::LoadCaptured(idx) => format!("LoadCaptured({})", idx),
        Instruction::Append => "Append".to_string(),
        Instruction::MakeList(n) => format!("MakeList({})", n),
        Instruction::LoadGlobal(name) => format!("LoadGlobal(\"{}\")", name),
        Instruction::StoreGlobal(name) => format!("StoreGlobal(\"{}\")", name),
        Instruction::StringLength => "StringLength".to_string(),
        Instruction::Substring => "Substring".to_string(),
        Instruction::StringAppend => "StringAppend".to_string(),
        Instruction::StringToList => "StringToList".to_string(),
        Instruction::ListToString => "ListToString".to_string(),
        Instruction::CharCode => "CharCode".to_string(),
        Instruction::ReadFile => "ReadFile".to_string(),
        Instruction::WriteFile => "WriteFile".to_string(),
        Instruction::FileExists => "FileExists".to_string(),
        Instruction::GetArgs => "GetArgs".to_string(),
        Instruction::WriteBinaryFile => "WriteBinaryFile".to_string(),
        Instruction::LoadFile => "LoadFile".to_string(),
        Instruction::RequireFile => "RequireFile".to_string(),
        Instruction::ListRef => "ListRef".to_string(),
        Instruction::ListLength => "ListLength".to_string(),
        Instruction::NumberToString => "NumberToString".to_string(),
        // HashMap operations
        Instruction::MakeHashMap(n) => format!("MakeHashMap({})", n),
        Instruction::HashMapGet => "HashMapGet".to_string(),
        Instruction::HashMapSet => "HashMapSet".to_string(),
        Instruction::HashMapKeys => "HashMapKeys".to_string(),
        Instruction::HashMapValues => "HashMapValues".to_string(),
        Instruction::HashMapContainsKey => "HashMapContainsKey".to_string(),
        Instruction::IsHashMap => "IsHashMap".to_string(),
        // Vector operations
        Instruction::MakeVector(n) => format!("MakeVector({})", n),
        Instruction::VectorGet => "VectorGet".to_string(),
        Instruction::VectorSet => "VectorSet".to_string(),
        Instruction::VectorPush => "VectorPush".to_string(),
        Instruction::VectorPop => "VectorPop".to_string(),
        Instruction::VectorLength => "VectorLength".to_string(),
        Instruction::IsVector => "IsVector".to_string(),
        // Type predicates
        Instruction::IsInteger => "IsInteger".to_string(),
        Instruction::IsBoolean => "IsBoolean".to_string(),
        Instruction::IsFunction => "IsFunction".to_string(),
        Instruction::IsClosure => "IsClosure".to_string(),
        Instruction::IsProcedure => "IsProcedure".to_string(),
        // Type conversions
        Instruction::StringToNumber => "StringToNumber".to_string(),
        Instruction::ListToVector => "ListToVector".to_string(),
        Instruction::VectorToList => "VectorToList".to_string(),
        // Variadic function support
        Instruction::PackRestArgs(n) => format!("PackRestArgs({})", n),
        Instruction::MakeVariadicClosure(params, rest_param, body, num_captured) => {
            format!("MakeVariadicClosure({:?} . {}, {} instrs, {} captured)",
                    params, rest_param, body.len(), num_captured)
        }
        // Float type predicates and conversions
        Instruction::IsFloat => "IsFloat".to_string(),
        Instruction::IsNumber => "IsNumber".to_string(),
        Instruction::IntToFloat => "IntToFloat".to_string(),
        Instruction::FloatToInt => "FloatToInt".to_string(),
        // Math functions
        Instruction::Sqrt => "Sqrt".to_string(),
        Instruction::Sin => "Sin".to_string(),
        Instruction::Cos => "Cos".to_string(),
        Instruction::Tan => "Tan".to_string(),
        Instruction::Atan => "Atan".to_string(),
        Instruction::Atan2 => "Atan2".to_string(),
        Instruction::Log => "Log".to_string(),
        Instruction::Exp => "Exp".to_string(),
        Instruction::Floor => "Floor".to_string(),
        Instruction::Ceil => "Ceil".to_string(),
        Instruction::Abs => "Abs".to_string(),
        Instruction::Pow => "Pow".to_string(),
        Instruction::Random => "Random".to_string(),
        Instruction::RandomInt => "RandomInt".to_string(),
        Instruction::SeedRandom => "SeedRandom".to_string(),
        // String operations
        Instruction::StringSplit => "StringSplit".to_string(),
        Instruction::StringJoin => "StringJoin".to_string(),
        Instruction::StringTrim => "StringTrim".to_string(),
        Instruction::StringReplace => "StringReplace".to_string(),
        // Date/Time operations
        Instruction::CurrentTimestamp => "CurrentTimestamp".to_string(),
        Instruction::FormatTimestamp => "FormatTimestamp".to_string(),
        // Metaprogramming
        Instruction::Eval => "Eval".to_string(),
        // Reflection
        Instruction::FunctionArity => "FunctionArity".to_string(),
        Instruction::FunctionParams => "FunctionParams".to_string(),
        Instruction::ClosureCaptured => "ClosureCaptured".to_string(),
        Instruction::FunctionName => "FunctionName".to_string(),
        // Type inspection and symbol generation
        Instruction::TypeOf => "TypeOf".to_string(),
        Instruction::GenSym => "GenSym".to_string(),
        // Parallel Collections
        Instruction::PMap => "PMap".to_string(),
        Instruction::PFilter => "PFilter".to_string(),
        Instruction::PReduce => "PReduce".to_string(),
        // HTTP/Networking
        Instruction::HttpListen => "HttpListen".to_string(),
        Instruction::HttpAccept => "HttpAccept".to_string(),
        Instruction::HttpReadRequest => "HttpReadRequest".to_string(),
        Instruction::HttpSendResponse => "HttpSendResponse".to_string(),
        Instruction::HttpClose => "HttpClose".to_string(),
        // Multi-threaded HTTP
        Instruction::HttpListenShared => "HttpListenShared".to_string(),
        Instruction::HttpServeParallel => "HttpServeParallel".to_string(),
        // String predicates and utilities
        Instruction::StringStartsWith => "StringStartsWith".to_string(),
        Instruction::StringEndsWith => "StringEndsWith".to_string(),
        Instruction::StringContains => "StringContains".to_string(),
        Instruction::StringUpcase => "StringUpcase".to_string(),
        Instruction::StringDowncase => "StringDowncase".to_string(),
        Instruction::Format => "Format".to_string(),
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

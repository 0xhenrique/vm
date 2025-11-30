use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;

use super::instructions::Instruction;
use super::value::{Value, List, ClosureData};

// Bytecode file format serialization

pub fn serialize_bytecode(
    functions: &HashMap<String, Vec<Instruction>>,
    main_bytecode: &[Instruction],
) -> Vec<u8> {
    let mut bytes = Vec::new();

    // Magic number: "LISP" in ASCII
    bytes.extend_from_slice(b"LISP");

    // Version: 7 (added HashMap and Vector support)
    bytes.push(7);

    // Serialize functions
    write_u32(&mut bytes, functions.len() as u32);
    for (name, bytecode) in functions {
        write_string(&mut bytes, name);
        write_bytecode(&mut bytes, bytecode);
    }

    // Serialize main bytecode
    write_bytecode(&mut bytes, main_bytecode);

    bytes
}

pub fn deserialize_bytecode(bytes: &[u8]) -> Result<(HashMap<String, Vec<Instruction>>, Vec<Instruction>), String> {
    let mut pos = 0;

    // Check magic number
    if bytes.len() < 5 || &bytes[0..4] != b"LISP" {
        return Err("Invalid bytecode file: bad magic number".to_string());
    }
    pos += 4;

    // Check version
    let version = bytes[pos];
    if version != 7 {
        return Err(format!("Unsupported bytecode version: {} (expected 7)", version));
    }
    pos += 1;

    // Deserialize functions
    let func_count = read_u32(bytes, &mut pos)?;
    let mut functions = HashMap::new();

    for _ in 0..func_count {
        let name = read_string(bytes, &mut pos)?;
        let bytecode = read_bytecode(bytes, &mut pos)?;
        functions.insert(name, bytecode);
    }

    // Deserialize main bytecode
    let main_bytecode = read_bytecode(bytes, &mut pos)?;

    Ok((functions, main_bytecode))
}

pub fn save_bytecode_file(
    path: &str,
    functions: &HashMap<String, Vec<Instruction>>,
    main_bytecode: &[Instruction],
) -> Result<(), String> {
    let bytes = serialize_bytecode(functions, main_bytecode);
    let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

pub fn load_bytecode_file(path: &str) -> Result<(HashMap<String, Vec<Instruction>>, Vec<Instruction>), String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| format!("Failed to read file: {}", e))?;
    deserialize_bytecode(&bytes)
}

// Helper functions for serialization

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn read_u32(bytes: &[u8], pos: &mut usize) -> Result<u32, String> {
    if *pos + 4 > bytes.len() {
        return Err("Unexpected end of bytecode".to_string());
    }
    let value = u32::from_le_bytes([bytes[*pos], bytes[*pos + 1], bytes[*pos + 2], bytes[*pos + 3]]);
    *pos += 4;
    Ok(value)
}

fn write_string(bytes: &mut Vec<u8>, s: &str) {
    write_u32(bytes, s.len() as u32);
    bytes.extend_from_slice(s.as_bytes());
}

fn read_string(bytes: &[u8], pos: &mut usize) -> Result<String, String> {
    let len = read_u32(bytes, pos)? as usize;
    if *pos + len > bytes.len() {
        return Err("Unexpected end of bytecode".to_string());
    }
    let s = String::from_utf8(bytes[*pos..*pos + len].to_vec())
        .map_err(|_| "Invalid UTF-8 in bytecode".to_string())?;
    *pos += len;
    Ok(s)
}

fn write_bytecode(bytes: &mut Vec<u8>, bytecode: &[Instruction]) {
    write_u32(bytes, bytecode.len() as u32);
    for instr in bytecode {
        write_instruction(bytes, instr);
    }
}

fn read_bytecode(bytes: &[u8], pos: &mut usize) -> Result<Vec<Instruction>, String> {
    let len = read_u32(bytes, pos)? as usize;
    let mut bytecode = Vec::new();
    for _ in 0..len {
        bytecode.push(read_instruction(bytes, pos)?);
    }
    Ok(bytecode)
}

fn write_instruction(bytes: &mut Vec<u8>, instr: &Instruction) {
    match instr {
        Instruction::Push(value) => {
            bytes.push(0);
            write_value(bytes, value);
        }
        Instruction::Add => bytes.push(1),
        Instruction::Sub => bytes.push(2),
        Instruction::Mul => bytes.push(3),
        Instruction::Div => bytes.push(4),
        Instruction::Leq => bytes.push(5),
        Instruction::JmpIfFalse(addr) => {
            bytes.push(6);
            write_u32(bytes, *addr as u32);
        }
        Instruction::Jmp(addr) => {
            bytes.push(7);
            write_u32(bytes, *addr as u32);
        }
        Instruction::Call(name, argc) => {
            bytes.push(8);
            write_string(bytes, name);
            write_u32(bytes, *argc as u32);
        }
        Instruction::Ret => bytes.push(9),
        Instruction::LoadArg(idx) => {
            bytes.push(10);
            write_u32(bytes, *idx as u32);
        }
        Instruction::Print => bytes.push(11),
        Instruction::Halt => bytes.push(12),
        Instruction::Mod => bytes.push(13),
        Instruction::Neg => bytes.push(14),
        Instruction::Lt => bytes.push(15),
        Instruction::Gt => bytes.push(16),
        Instruction::Gte => bytes.push(17),
        Instruction::Eq => bytes.push(18),
        Instruction::Neq => bytes.push(19),
        Instruction::Cons => bytes.push(20),
        Instruction::Car => bytes.push(21),
        Instruction::Cdr => bytes.push(22),
        Instruction::IsList => bytes.push(23),
        Instruction::IsString => bytes.push(24),
        Instruction::IsSymbol => bytes.push(25),
        Instruction::SymbolToString => bytes.push(26),
        Instruction::StringToSymbol => bytes.push(27),
        Instruction::GetLocal(pos) => {
            bytes.push(28);
            write_u32(bytes, *pos as u32);
        }
        Instruction::PopN(n) => {
            bytes.push(29);
            write_u32(bytes, *n as u32);
        }
        Instruction::Slide(n) => {
            bytes.push(30);
            write_u32(bytes, *n as u32);
        }
        Instruction::CheckArity(arity, addr) => {
            bytes.push(31);
            write_u32(bytes, *arity as u32);
            write_u32(bytes, *addr as u32);
        }
        Instruction::MakeClosure(params, body, num_captured) => {
            bytes.push(32);
            // Write params
            write_u32(bytes, params.len() as u32);
            for param in params {
                write_string(bytes, param);
            }
            // Write body
            write_u32(bytes, body.len() as u32);
            for instr in body {
                write_instruction(bytes, instr);
            }
            // Write num_captured
            write_u32(bytes, *num_captured as u32);
        }
        Instruction::CallClosure(argc) => {
            bytes.push(33);
            write_u32(bytes, *argc as u32);
        }
        Instruction::Apply => {
            bytes.push(78);
        }
        Instruction::LoadFile => {
            bytes.push(79);
        }
        Instruction::RequireFile => {
            bytes.push(80);
        }
        Instruction::LoadCaptured(idx) => {
            bytes.push(34);
            write_u32(bytes, *idx as u32);
        }
        Instruction::Append => bytes.push(35),
        Instruction::MakeList(n) => {
            bytes.push(36);
            write_u32(bytes, *n as u32);
        }
        Instruction::TailCall(name, argc) => {
            bytes.push(37);
            write_string(bytes, name);
            write_u32(bytes, *argc as u32);
        }
        Instruction::LoadGlobal(name) => {
            bytes.push(38);
            write_string(bytes, name);
        }
        Instruction::StoreGlobal(name) => {
            bytes.push(39);
            write_string(bytes, name);
        }
        Instruction::StringLength => bytes.push(40),
        Instruction::Substring => bytes.push(41),
        Instruction::StringAppend => bytes.push(42),
        Instruction::StringToList => bytes.push(43),
        Instruction::ListToString => bytes.push(44),
        Instruction::CharCode => bytes.push(50),
        Instruction::ReadFile => bytes.push(45),
        Instruction::WriteFile => bytes.push(46),
        Instruction::FileExists => bytes.push(47),
        Instruction::GetArgs => bytes.push(48),
        Instruction::WriteBinaryFile => bytes.push(49),
        Instruction::ListRef => bytes.push(51),
        Instruction::ListLength => bytes.push(52),
        Instruction::NumberToString => bytes.push(53),
        // HashMap operations (54-60)
        Instruction::MakeHashMap(n) => {
            bytes.push(54);
            write_u32(bytes, *n as u32);
        }
        Instruction::HashMapGet => bytes.push(55),
        Instruction::HashMapSet => bytes.push(56),
        Instruction::HashMapKeys => bytes.push(57),
        Instruction::HashMapValues => bytes.push(58),
        Instruction::HashMapContainsKey => bytes.push(59),
        Instruction::IsHashMap => bytes.push(60),
        // Vector operations (61-67)
        Instruction::MakeVector(n) => {
            bytes.push(61);
            write_u32(bytes, *n as u32);
        }
        Instruction::VectorGet => bytes.push(62),
        Instruction::VectorSet => bytes.push(63),
        Instruction::VectorPush => bytes.push(64),
        Instruction::VectorPop => bytes.push(65),
        Instruction::VectorLength => bytes.push(66),
        Instruction::IsVector => bytes.push(67),
        // Type predicates (68-72)
        Instruction::IsInteger => bytes.push(68),
        Instruction::IsBoolean => bytes.push(69),
        Instruction::IsFunction => bytes.push(70),
        Instruction::IsClosure => bytes.push(71),
        Instruction::IsProcedure => bytes.push(72),
        // Type conversions (73-75)
        Instruction::StringToNumber => bytes.push(73),
        Instruction::ListToVector => bytes.push(74),
        Instruction::VectorToList => bytes.push(75),
        // Variadic function support (76-77)
        Instruction::PackRestArgs(required_count) => {
            bytes.push(76);
            write_u32(bytes, *required_count as u32);
        }
        Instruction::MakeVariadicClosure(params, rest_param, body, num_captured) => {
            bytes.push(77);
            // Write required params
            write_u32(bytes, params.len() as u32);
            for param in params {
                write_string(bytes, param);
            }
            // Write rest param name
            write_string(bytes, rest_param);
            // Write body
            write_u32(bytes, body.len() as u32);
            for instr in body {
                write_instruction(bytes, instr);
            }
            // Write num_captured
            write_u32(bytes, *num_captured as u32);
        }
        // Float type predicates and conversions (81-84)
        Instruction::IsFloat => bytes.push(81),
        Instruction::IsNumber => bytes.push(82),
        Instruction::IntToFloat => bytes.push(83),
        Instruction::FloatToInt => bytes.push(84),
        // Math functions (85-103)
        Instruction::Sqrt => bytes.push(85),
        Instruction::Sin => bytes.push(86),
        Instruction::Cos => bytes.push(87),
        Instruction::Floor => bytes.push(88),
        Instruction::Ceil => bytes.push(89),
        Instruction::Abs => bytes.push(90),
        Instruction::Pow => bytes.push(91),
        Instruction::Tan => bytes.push(97),
        Instruction::Atan => bytes.push(98),
        Instruction::Atan2 => bytes.push(99),
        Instruction::Log => bytes.push(100),
        Instruction::Exp => bytes.push(101),
        Instruction::Random => bytes.push(102),
        Instruction::RandomInt => bytes.push(103),
        Instruction::SeedRandom => bytes.push(104),
        // String operations (105-108)
        Instruction::StringSplit => bytes.push(105),
        Instruction::StringJoin => bytes.push(106),
        Instruction::StringTrim => bytes.push(107),
        Instruction::StringReplace => bytes.push(108),
        // String predicates and utilities (126-130)
        Instruction::StringStartsWith => bytes.push(126),
        Instruction::StringEndsWith => bytes.push(127),
        Instruction::StringContains => bytes.push(128),
        Instruction::StringUpcase => bytes.push(129),
        Instruction::StringDowncase => bytes.push(130),
        Instruction::Format => bytes.push(131),
        // Date/Time operations (109-110)
        Instruction::CurrentTimestamp => bytes.push(109),
        Instruction::FormatTimestamp => bytes.push(110),
        // Metaprogramming (92+)
        Instruction::Eval => bytes.push(92),
        // Reflection (93+)
        Instruction::FunctionArity => bytes.push(93),
        Instruction::FunctionParams => bytes.push(94),
        Instruction::ClosureCaptured => bytes.push(95),
        Instruction::FunctionName => bytes.push(96),
        // Type inspection and symbol generation (114-115)
        Instruction::TypeOf => bytes.push(114),
        Instruction::GenSym => bytes.push(115),
        // Parallel Collections (116-118)
        Instruction::PMap => bytes.push(116),
        Instruction::PFilter => bytes.push(117),
        Instruction::PReduce => bytes.push(118),
        // HTTP/Networking (119-123)
        Instruction::HttpListen => bytes.push(119),
        Instruction::HttpAccept => bytes.push(120),
        Instruction::HttpReadRequest => bytes.push(121),
        Instruction::HttpSendResponse => bytes.push(122),
        Instruction::HttpClose => bytes.push(123),
        // Loop/recur instructions (111-113)
        Instruction::SetLocal(pos) => {
            bytes.push(111);
            write_u32(bytes, *pos as u32);
        }
        Instruction::BeginLoop(count) => {
            bytes.push(112);
            write_u32(bytes, *count as u32);
        }
        Instruction::Recur(count) => {
            bytes.push(113);
            write_u32(bytes, *count as u32);
        }
        // Multi-threaded HTTP (124-125)
        Instruction::HttpListenShared => bytes.push(124),
        Instruction::HttpServeParallel => bytes.push(125),
    }
}

fn read_instruction(bytes: &[u8], pos: &mut usize) -> Result<Instruction, String> {
    if *pos >= bytes.len() {
        return Err("Unexpected end of bytecode".to_string());
    }
    let opcode = bytes[*pos];
    *pos += 1;

    match opcode {
        0 => Ok(Instruction::Push(read_value(bytes, pos)?)),
        1 => Ok(Instruction::Add),
        2 => Ok(Instruction::Sub),
        3 => Ok(Instruction::Mul),
        4 => Ok(Instruction::Div),
        5 => Ok(Instruction::Leq),
        6 => Ok(Instruction::JmpIfFalse(read_u32(bytes, pos)? as usize)),
        7 => Ok(Instruction::Jmp(read_u32(bytes, pos)? as usize)),
        8 => {
            let name = read_string(bytes, pos)?;
            let argc = read_u32(bytes, pos)? as usize;
            Ok(Instruction::Call(name, argc))
        }
        9 => Ok(Instruction::Ret),
        10 => Ok(Instruction::LoadArg(read_u32(bytes, pos)? as usize)),
        11 => Ok(Instruction::Print),
        12 => Ok(Instruction::Halt),
        13 => Ok(Instruction::Mod),
        14 => Ok(Instruction::Neg),
        15 => Ok(Instruction::Lt),
        16 => Ok(Instruction::Gt),
        17 => Ok(Instruction::Gte),
        18 => Ok(Instruction::Eq),
        19 => Ok(Instruction::Neq),
        20 => Ok(Instruction::Cons),
        21 => Ok(Instruction::Car),
        22 => Ok(Instruction::Cdr),
        23 => Ok(Instruction::IsList),
        24 => Ok(Instruction::IsString),
        25 => Ok(Instruction::IsSymbol),
        26 => Ok(Instruction::SymbolToString),
        27 => Ok(Instruction::StringToSymbol),
        28 => Ok(Instruction::GetLocal(read_u32(bytes, pos)? as usize)),
        29 => Ok(Instruction::PopN(read_u32(bytes, pos)? as usize)),
        30 => Ok(Instruction::Slide(read_u32(bytes, pos)? as usize)),
        31 => {
            let arity = read_u32(bytes, pos)? as usize;
            let addr = read_u32(bytes, pos)? as usize;
            Ok(Instruction::CheckArity(arity, addr))
        }
        32 => {
            // Read params
            let params_len = read_u32(bytes, pos)? as usize;
            let mut params = Vec::new();
            for _ in 0..params_len {
                params.push(read_string(bytes, pos)?);
            }
            // Read body
            let body_len = read_u32(bytes, pos)? as usize;
            let mut body = Vec::new();
            for _ in 0..body_len {
                body.push(read_instruction(bytes, pos)?);
            }
            // Read num_captured
            let num_captured = read_u32(bytes, pos)? as usize;
            Ok(Instruction::MakeClosure(params, body, num_captured))
        }
        33 => Ok(Instruction::CallClosure(read_u32(bytes, pos)? as usize)),
        34 => Ok(Instruction::LoadCaptured(read_u32(bytes, pos)? as usize)),
        35 => Ok(Instruction::Append),
        36 => Ok(Instruction::MakeList(read_u32(bytes, pos)? as usize)),
        37 => {
            let name = read_string(bytes, pos)?;
            let argc = read_u32(bytes, pos)? as usize;
            Ok(Instruction::TailCall(name, argc))
        }
        38 => Ok(Instruction::LoadGlobal(read_string(bytes, pos)?)),
        39 => Ok(Instruction::StoreGlobal(read_string(bytes, pos)?)),
        40 => Ok(Instruction::StringLength),
        41 => Ok(Instruction::Substring),
        42 => Ok(Instruction::StringAppend),
        43 => Ok(Instruction::StringToList),
        44 => Ok(Instruction::ListToString),
        50 => Ok(Instruction::CharCode),
        45 => Ok(Instruction::ReadFile),
        46 => Ok(Instruction::WriteFile),
        47 => Ok(Instruction::FileExists),
        48 => Ok(Instruction::GetArgs),
        49 => Ok(Instruction::WriteBinaryFile),
        51 => Ok(Instruction::ListRef),
        52 => Ok(Instruction::ListLength),
        53 => Ok(Instruction::NumberToString),
        // HashMap operations (54-60)
        54 => Ok(Instruction::MakeHashMap(read_u32(bytes, pos)? as usize)),
        55 => Ok(Instruction::HashMapGet),
        56 => Ok(Instruction::HashMapSet),
        57 => Ok(Instruction::HashMapKeys),
        58 => Ok(Instruction::HashMapValues),
        59 => Ok(Instruction::HashMapContainsKey),
        60 => Ok(Instruction::IsHashMap),
        // Vector operations (61-67)
        61 => Ok(Instruction::MakeVector(read_u32(bytes, pos)? as usize)),
        62 => Ok(Instruction::VectorGet),
        63 => Ok(Instruction::VectorSet),
        64 => Ok(Instruction::VectorPush),
        65 => Ok(Instruction::VectorPop),
        66 => Ok(Instruction::VectorLength),
        67 => Ok(Instruction::IsVector),
        // Type predicates (68-72)
        68 => Ok(Instruction::IsInteger),
        69 => Ok(Instruction::IsBoolean),
        70 => Ok(Instruction::IsFunction),
        71 => Ok(Instruction::IsClosure),
        72 => Ok(Instruction::IsProcedure),
        // Type conversions (73-75)
        73 => Ok(Instruction::StringToNumber),
        74 => Ok(Instruction::ListToVector),
        75 => Ok(Instruction::VectorToList),
        // Variadic function support (76-77)
        76 => Ok(Instruction::PackRestArgs(read_u32(bytes, pos)? as usize)),
        77 => {
            // Read MakeVariadicClosure
            let params_len = read_u32(bytes, pos)? as usize;
            let mut params = Vec::new();
            for _ in 0..params_len {
                params.push(read_string(bytes, pos)?);
            }
            let rest_param = read_string(bytes, pos)?;
            let body_len = read_u32(bytes, pos)? as usize;
            let mut body = Vec::new();
            for _ in 0..body_len {
                body.push(read_instruction(bytes, pos)?);
            }
            let num_captured = read_u32(bytes, pos)? as usize;
            Ok(Instruction::MakeVariadicClosure(params, rest_param, body, num_captured))
        }
        78 => Ok(Instruction::Apply),
        79 => Ok(Instruction::LoadFile),
        80 => Ok(Instruction::RequireFile),
        // Float type predicates and conversions (81-84)
        81 => Ok(Instruction::IsFloat),
        82 => Ok(Instruction::IsNumber),
        83 => Ok(Instruction::IntToFloat),
        84 => Ok(Instruction::FloatToInt),
        // Math functions (85-91, 97-104)
        85 => Ok(Instruction::Sqrt),
        86 => Ok(Instruction::Sin),
        87 => Ok(Instruction::Cos),
        88 => Ok(Instruction::Floor),
        89 => Ok(Instruction::Ceil),
        90 => Ok(Instruction::Abs),
        91 => Ok(Instruction::Pow),
        97 => Ok(Instruction::Tan),
        98 => Ok(Instruction::Atan),
        99 => Ok(Instruction::Atan2),
        100 => Ok(Instruction::Log),
        101 => Ok(Instruction::Exp),
        102 => Ok(Instruction::Random),
        103 => Ok(Instruction::RandomInt),
        104 => Ok(Instruction::SeedRandom),
        // String operations (105-108)
        105 => Ok(Instruction::StringSplit),
        106 => Ok(Instruction::StringJoin),
        107 => Ok(Instruction::StringTrim),
        108 => Ok(Instruction::StringReplace),
        // Date/Time operations (109-110)
        109 => Ok(Instruction::CurrentTimestamp),
        110 => Ok(Instruction::FormatTimestamp),
        // Metaprogramming (92+)
        92 => Ok(Instruction::Eval),
        // Reflection (93+)
        93 => Ok(Instruction::FunctionArity),
        94 => Ok(Instruction::FunctionParams),
        95 => Ok(Instruction::ClosureCaptured),
        96 => Ok(Instruction::FunctionName),
        // Type inspection and symbol generation (114-115)
        114 => Ok(Instruction::TypeOf),
        115 => Ok(Instruction::GenSym),
        // Parallel Collections (116-118)
        116 => Ok(Instruction::PMap),
        117 => Ok(Instruction::PFilter),
        118 => Ok(Instruction::PReduce),
        // HTTP/Networking (119-123)
        119 => Ok(Instruction::HttpListen),
        120 => Ok(Instruction::HttpAccept),
        121 => Ok(Instruction::HttpReadRequest),
        122 => Ok(Instruction::HttpSendResponse),
        123 => Ok(Instruction::HttpClose),
        // Loop/recur instructions (111-113)
        111 => Ok(Instruction::SetLocal(read_u32(bytes, pos)? as usize)),
        112 => Ok(Instruction::BeginLoop(read_u32(bytes, pos)? as usize)),
        113 => Ok(Instruction::Recur(read_u32(bytes, pos)? as usize)),
        // Multi-threaded HTTP (124-125)
        124 => Ok(Instruction::HttpListenShared),
        125 => Ok(Instruction::HttpServeParallel),
        // String predicates and utilities (126-130)
        126 => Ok(Instruction::StringStartsWith),
        127 => Ok(Instruction::StringEndsWith),
        128 => Ok(Instruction::StringContains),
        129 => Ok(Instruction::StringUpcase),
        130 => Ok(Instruction::StringDowncase),
        131 => Ok(Instruction::Format),
        _ => Err(format!("Unknown opcode: {}", opcode)),
    }
}

fn write_value(bytes: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Integer(n) => {
            bytes.push(0);
            bytes.extend_from_slice(&n.to_le_bytes());
        }
        Value::Boolean(b) => {
            bytes.push(1);
            bytes.push(if *b { 1 } else { 0 });
        }
        Value::Float(f) => {
            bytes.push(9);
            bytes.extend_from_slice(&f.to_le_bytes());
        }
        Value::List(list) => {
            bytes.push(2);
            write_u32(bytes, list.len() as u32);
            for item in list.iter() {
                write_value(bytes, item);
            }
        }
        Value::Symbol(s) => {
            bytes.push(3);
            write_string(bytes, s);
        }
        Value::String(s) => {
            bytes.push(4);
            write_string(bytes, s);
        }
        Value::Function(name) => {
            bytes.push(5);
            write_string(bytes, name);
        }
        Value::Closure(closure_data) => {
            bytes.push(6);
            // Write params
            write_u32(bytes, closure_data.params.len() as u32);
            for param in &closure_data.params {
                write_string(bytes, param);
            }
            // Write rest_param (Option<String>)
            match &closure_data.rest_param {
                None => bytes.push(0),
                Some(rest_name) => {
                    bytes.push(1);
                    write_string(bytes, rest_name);
                }
            }
            // Write body
            write_u32(bytes, closure_data.body.len() as u32);
            for instr in &closure_data.body {
                write_instruction(bytes, instr);
            }
            // Write captured environment
            write_u32(bytes, closure_data.captured.len() as u32);
            for (name, value) in &closure_data.captured {
                write_string(bytes, name);
                write_value(bytes, value);
            }
        }
        Value::HashMap(map) => {
            bytes.push(7);
            // Write number of entries
            write_u32(bytes, map.len() as u32);
            // Write key-value pairs
            for (key, value) in map.iter() {
                write_string(bytes, key);
                write_value(bytes, value);
            }
        }
        Value::Vector(vec) => {
            bytes.push(8);
            // Write number of elements
            write_u32(bytes, vec.len() as u32);
            // Write elements
            for value in vec.iter() {
                write_value(bytes, value);
            }
        }
        Value::TcpListener(_) => {
            panic!("Cannot serialize TcpListener to bytecode - runtime value only");
        }
        Value::TcpStream(_) => {
            panic!("Cannot serialize TcpStream to bytecode - runtime value only");
        }
        Value::SharedTcpListener(_) => {
            panic!("Cannot serialize SharedTcpListener to bytecode - runtime value only");
        }
    }
}

fn read_value(bytes: &[u8], pos: &mut usize) -> Result<Value, String> {
    if *pos >= bytes.len() {
        return Err("Unexpected end of bytecode".to_string());
    }
    let tag = bytes[*pos];
    *pos += 1;

    match tag {
        0 => {
            if *pos + 8 > bytes.len() {
                return Err("Unexpected end of bytecode".to_string());
            }
            let n = i64::from_le_bytes([
                bytes[*pos],
                bytes[*pos + 1],
                bytes[*pos + 2],
                bytes[*pos + 3],
                bytes[*pos + 4],
                bytes[*pos + 5],
                bytes[*pos + 6],
                bytes[*pos + 7],
            ]);
            *pos += 8;
            Ok(Value::Integer(n))
        }
        1 => {
            if *pos >= bytes.len() {
                return Err("Unexpected end of bytecode".to_string());
            }
            let b = bytes[*pos] != 0;
            *pos += 1;
            Ok(Value::Boolean(b))
        }
        2 => {
            let len = read_u32(bytes, pos)? as usize;
            let mut items = Vec::new();
            for _ in 0..len {
                items.push(read_value(bytes, pos)?);
            }
            Ok(Value::List(List::from_vec(items)))
        }
        3 => Ok(Value::Symbol(Arc::new(read_string(bytes, pos)?))),
        4 => Ok(Value::String(Arc::new(read_string(bytes, pos)?))),
        5 => Ok(Value::Function(Arc::new(read_string(bytes, pos)?))),
        6 => {
            // Read params
            let params_len = read_u32(bytes, pos)? as usize;
            let mut params = Vec::new();
            for _ in 0..params_len {
                params.push(read_string(bytes, pos)?);
            }
            // Read rest_param (Option<String>)
            let rest_param = if bytes[*pos] == 0 {
                *pos += 1;
                None
            } else {
                *pos += 1;
                Some(read_string(bytes, pos)?)
            };
            // Read body
            let body_len = read_u32(bytes, pos)? as usize;
            let mut body = Vec::new();
            for _ in 0..body_len {
                body.push(read_instruction(bytes, pos)?);
            }
            // Read captured environment
            let captured_len = read_u32(bytes, pos)? as usize;
            let mut captured = Vec::new();
            for _ in 0..captured_len {
                let name = read_string(bytes, pos)?;
                let value = read_value(bytes, pos)?;
                captured.push((name, value));
            }
            Ok(Value::Closure(Arc::new(ClosureData { params, rest_param, body, captured })))
        }
        7 => {
            // Read HashMap
            let len = read_u32(bytes, pos)? as usize;
            let mut map = HashMap::new();
            for _ in 0..len {
                let key = read_string(bytes, pos)?;
                let value = read_value(bytes, pos)?;
                map.insert(key, value);
            }
            Ok(Value::HashMap(Arc::new(map)))
        }
        8 => {
            // Read Vector
            let len = read_u32(bytes, pos)? as usize;
            let mut vec = Vec::new();
            for _ in 0..len {
                vec.push(read_value(bytes, pos)?);
            }
            Ok(Value::Vector(Arc::new(vec)))
        }
        9 => {
            // Read Float
            if *pos + 8 > bytes.len() {
                return Err("Unexpected end of bytecode".to_string());
            }
            let f = f64::from_le_bytes([
                bytes[*pos],
                bytes[*pos + 1],
                bytes[*pos + 2],
                bytes[*pos + 3],
                bytes[*pos + 4],
                bytes[*pos + 5],
                bytes[*pos + 6],
                bytes[*pos + 7],
            ]);
            *pos += 8;
            Ok(Value::Float(f))
        }
        _ => Err(format!("Unknown value tag: {}", tag)),
    }
}

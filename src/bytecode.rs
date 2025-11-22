use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use crate::{Instruction, Value};

// Bytecode file format serialization

pub fn serialize_bytecode(
    functions: &HashMap<String, Vec<Instruction>>,
    main_bytecode: &[Instruction],
) -> Vec<u8> {
    let mut bytes = Vec::new();

    // Magic number: "LISP" in ASCII
    bytes.extend_from_slice(b"LISP");

    // Version: 1
    bytes.push(1);

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
    if version != 1 {
        return Err(format!("Unsupported bytecode version: {}", version));
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
        _ => Err(format!("Unknown value tag: {}", tag)),
    }
}

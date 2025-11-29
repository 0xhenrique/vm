use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use super::value::{Value, List, ClosureData};
use super::instructions::Instruction;
use super::stack::Frame;
use super::errors::RuntimeError;
use crate::parser::Parser;
use crate::compiler::Compiler;

pub struct VM {
    pub instruction_pointer: usize,
    pub value_stack: Vec<Value>,
    pub call_stack: Vec<Frame>,
    pub functions: HashMap<String, Vec<Instruction>>,
    pub current_bytecode: Vec<Instruction>,
    pub halted: bool,
    pub global_vars: HashMap<String, Value>, // Global variables
    pub args: Vec<String>, // Command-line arguments
    pub loaded_modules: HashSet<String>,     // Track loaded modules for require
}

impl VM {

    pub fn new() -> Self {
        let mut vm = VM {
            instruction_pointer: 0,
            value_stack: Vec::new(),
            call_stack: Vec::new(),
            functions: HashMap::new(),
            current_bytecode: Vec::new(),
            halted: false,
            global_vars: HashMap::new(),
            args: Vec::new(),
            loaded_modules: HashSet::new(),
        };
        vm.register_builtins();
        vm
    }

    fn register_builtins(&mut self) {
        use Instruction::*;

        // Arithmetic operations (binary)
        self.functions.insert("+".to_string(), vec![LoadArg(0), LoadArg(1), Add, Ret]);
        self.functions.insert("-".to_string(), vec![LoadArg(0), LoadArg(1), Sub, Ret]);
        self.functions.insert("*".to_string(), vec![LoadArg(0), LoadArg(1), Mul, Ret]);
        self.functions.insert("/".to_string(), vec![LoadArg(0), LoadArg(1), Div, Ret]);
        self.functions.insert("%".to_string(), vec![LoadArg(0), LoadArg(1), Mod, Ret]);
        // Arithmetic operations (unary)
        self.functions.insert("neg".to_string(), vec![LoadArg(0), Neg, Ret]);

        // Comparison operations
        self.functions.insert("<=".to_string(), vec![LoadArg(0), LoadArg(1), Leq, Ret]);
        self.functions.insert("<".to_string(), vec![LoadArg(0), LoadArg(1), Lt, Ret]);
        self.functions.insert(">".to_string(), vec![LoadArg(0), LoadArg(1), Gt, Ret]);
        self.functions.insert(">=".to_string(), vec![LoadArg(0), LoadArg(1), Gte, Ret]);
        self.functions.insert("==".to_string(), vec![LoadArg(0), LoadArg(1), Eq, Ret]);
        self.functions.insert("!=".to_string(), vec![LoadArg(0), LoadArg(1), Neq, Ret]);

        // List operations
        self.functions.insert("cons".to_string(), vec![LoadArg(0), LoadArg(1), Cons, Ret]);
        self.functions.insert("car".to_string(), vec![LoadArg(0), Car, Ret]);
        self.functions.insert("cdr".to_string(), vec![LoadArg(0), Cdr, Ret]);
        self.functions.insert("list?".to_string(), vec![LoadArg(0), IsList, Ret]);
        self.functions.insert("append".to_string(), vec![LoadArg(0), LoadArg(1), Append, Ret]);
        self.functions.insert("list-ref".to_string(), vec![LoadArg(0), LoadArg(1), ListRef, Ret]);
        self.functions.insert("list-length".to_string(), vec![LoadArg(0), ListLength, Ret]);
        self.functions.insert("null?".to_string(), vec![LoadArg(0), ListLength, Push(Value::Integer(0)), Eq, Ret]);

        // Type predicates
        self.functions.insert("integer?".to_string(), vec![LoadArg(0), IsInteger, Ret]);
        self.functions.insert("float?".to_string(), vec![LoadArg(0), IsFloat, Ret]);
        self.functions.insert("number?".to_string(), vec![LoadArg(0), IsNumber, Ret]); // int or float
        self.functions.insert("boolean?".to_string(), vec![LoadArg(0), IsBoolean, Ret]);
        self.functions.insert("function?".to_string(), vec![LoadArg(0), IsFunction, Ret]);
        self.functions.insert("closure?".to_string(), vec![LoadArg(0), IsClosure, Ret]);
        self.functions.insert("procedure?".to_string(), vec![LoadArg(0), IsProcedure, Ret]);

        // String operations
        self.functions.insert("string?".to_string(), vec![LoadArg(0), IsString, Ret]);
        self.functions.insert("symbol?".to_string(), vec![LoadArg(0), IsSymbol, Ret]);
        self.functions.insert("symbol->string".to_string(), vec![LoadArg(0), SymbolToString, Ret]);
        self.functions.insert("string->symbol".to_string(), vec![LoadArg(0), StringToSymbol, Ret]);
        self.functions.insert("string-length".to_string(), vec![LoadArg(0), StringLength, Ret]);
        self.functions.insert("substring".to_string(), vec![LoadArg(0), LoadArg(1), LoadArg(2), Substring, Ret]);
        self.functions.insert("string-append".to_string(), vec![LoadArg(0), LoadArg(1), StringAppend, Ret]);
        self.functions.insert("string->list".to_string(), vec![LoadArg(0), StringToList, Ret]);
        self.functions.insert("list->string".to_string(), vec![LoadArg(0), ListToString, Ret]);
        self.functions.insert("char-code".to_string(), vec![LoadArg(0), CharCode, Ret]);
        self.functions.insert("number->string".to_string(), vec![LoadArg(0), NumberToString, Ret]);
        self.functions.insert("string->number".to_string(), vec![LoadArg(0), StringToNumber, Ret]);
        self.functions.insert("string-split".to_string(), vec![LoadArg(0), LoadArg(1), StringSplit, Ret]);
        self.functions.insert("string-join".to_string(), vec![LoadArg(0), LoadArg(1), StringJoin, Ret]);
        self.functions.insert("string-trim".to_string(), vec![LoadArg(0), StringTrim, Ret]);
        self.functions.insert("string-replace".to_string(), vec![LoadArg(0), LoadArg(1), LoadArg(2), StringReplace, Ret]);

        // File I/O operations
        self.functions.insert("read-file".to_string(), vec![LoadArg(0), ReadFile, Ret]);
        self.functions.insert("write-file".to_string(), vec![LoadArg(0), LoadArg(1), WriteFile, Ret]);
        self.functions.insert("file-exists?".to_string(), vec![LoadArg(0), FileExists, Ret]);
        self.functions.insert("write-binary-file".to_string(), vec![LoadArg(0), LoadArg(1), WriteBinaryFile, Ret]);
        self.functions.insert("load".to_string(), vec![LoadArg(0), LoadFile, Ret]);
        self.functions.insert("require".to_string(), vec![LoadArg(0), RequireFile, Ret]);

        // Date/Time operations
        self.functions.insert("current-timestamp".to_string(), vec![CurrentTimestamp, Ret]);
        self.functions.insert("format-timestamp".to_string(), vec![LoadArg(0), LoadArg(1), FormatTimestamp, Ret]);

        // Other operations
        self.functions.insert("get-args".to_string(), vec![GetArgs, Ret]);
        self.functions.insert("print".to_string(), vec![LoadArg(0), Print, Ret]);
        self.functions.insert("apply".to_string(), vec![LoadArg(0), LoadArg(1), Apply, Ret]);

        // HashMap operations
        self.functions.insert("hashmap?".to_string(), vec![LoadArg(0), IsHashMap, Ret]);
        self.functions.insert("hashmap-get".to_string(), vec![LoadArg(0), LoadArg(1), HashMapGet, Ret]);
        self.functions.insert("hashmap-set".to_string(), vec![LoadArg(0), LoadArg(1), LoadArg(2), HashMapSet, Ret]);
        self.functions.insert("hashmap-keys".to_string(), vec![LoadArg(0), HashMapKeys, Ret]);
        self.functions.insert("hashmap-values".to_string(), vec![LoadArg(0), HashMapValues, Ret]);
        self.functions.insert("hashmap-contains-key?".to_string(), vec![LoadArg(0), LoadArg(1), HashMapContainsKey, Ret]);

        // Vector operations
        self.functions.insert("vector?".to_string(), vec![LoadArg(0), IsVector, Ret]);
        self.functions.insert("vector-ref".to_string(), vec![LoadArg(0), LoadArg(1), VectorGet, Ret]);
        self.functions.insert("vector-set".to_string(), vec![LoadArg(0), LoadArg(1), LoadArg(2), VectorSet, Ret]);
        self.functions.insert("vector-push".to_string(), vec![LoadArg(0), LoadArg(1), VectorPush, Ret]);
        self.functions.insert("vector-pop".to_string(), vec![LoadArg(0), VectorPop, Ret]);
        self.functions.insert("vector-length".to_string(), vec![LoadArg(0), VectorLength, Ret]);

        // Type conversions
        self.functions.insert("list->vector".to_string(), vec![LoadArg(0), ListToVector, Ret]);
        self.functions.insert("vector->list".to_string(), vec![LoadArg(0), VectorToList, Ret]);
        self.functions.insert("int->float".to_string(), vec![LoadArg(0), IntToFloat, Ret]);
        self.functions.insert("float->int".to_string(), vec![LoadArg(0), FloatToInt, Ret]);

        // Math functions
        self.functions.insert("sqrt".to_string(), vec![LoadArg(0), Sqrt, Ret]);
        self.functions.insert("sin".to_string(), vec![LoadArg(0), Sin, Ret]);
        self.functions.insert("cos".to_string(), vec![LoadArg(0), Cos, Ret]);
        self.functions.insert("tan".to_string(), vec![LoadArg(0), Tan, Ret]);
        self.functions.insert("atan".to_string(), vec![LoadArg(0), Atan, Ret]);
        self.functions.insert("atan2".to_string(), vec![LoadArg(0), LoadArg(1), Atan2, Ret]);
        self.functions.insert("log".to_string(), vec![LoadArg(0), Log, Ret]);
        self.functions.insert("exp".to_string(), vec![LoadArg(0), Exp, Ret]);
        self.functions.insert("floor".to_string(), vec![LoadArg(0), Floor, Ret]);
        self.functions.insert("ceil".to_string(), vec![LoadArg(0), Ceil, Ret]);
        self.functions.insert("abs".to_string(), vec![LoadArg(0), Abs, Ret]);
        self.functions.insert("pow".to_string(), vec![LoadArg(0), LoadArg(1), Pow, Ret]);
        self.functions.insert("random".to_string(), vec![Random, Ret]);
        self.functions.insert("random-int".to_string(), vec![LoadArg(0), RandomInt, Ret]);
        self.functions.insert("seed-random".to_string(), vec![LoadArg(0), SeedRandom, Ret]);

        // Metaprogramming
        self.functions.insert("eval".to_string(), vec![LoadArg(0), Eval, Ret]);

        // Reflection - Function Introspection
        self.functions.insert("function-arity".to_string(), vec![LoadArg(0), FunctionArity, Ret]);
        self.functions.insert("function-params".to_string(), vec![LoadArg(0), FunctionParams, Ret]);
        self.functions.insert("closure-captured".to_string(), vec![LoadArg(0), ClosureCaptured, Ret]);
        self.functions.insert("function-name".to_string(), vec![LoadArg(0), FunctionName, Ret]);

        // Type inspection
        self.functions.insert("type-of".to_string(), vec![LoadArg(0), TypeOf, Ret]);

        // Symbol generation
        self.functions.insert("gensym".to_string(), vec![GenSym, Ret]);

        // Parallel Collections (Phase 12a)
        self.functions.insert("pmap".to_string(), vec![LoadArg(0), LoadArg(1), PMap, Ret]);
        self.functions.insert("pfilter".to_string(), vec![LoadArg(0), LoadArg(1), PFilter, Ret]);
        self.functions.insert("preduce".to_string(), vec![LoadArg(0), LoadArg(1), LoadArg(2), PReduce, Ret]);
    }

    pub fn execute_one_instruction(&mut self) -> Result<(), RuntimeError> {
        if self.instruction_pointer >= self.current_bytecode.len() {
            self.halted = true;
            return Ok(());
        }

        let instruction = self.current_bytecode[self.instruction_pointer].clone();

        match instruction {
            Instruction::Push(value) => {
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::Add => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Add operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Add operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Integer(x + y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Float(x + y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Float(*x as f64 + y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Float(x + *y as f64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '+' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Sub => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Sub operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Sub operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Integer(x - y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Float(x - y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Float(*x as f64 - y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Float(x - *y as f64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '-' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Mul => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Mul operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Mul operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Integer(x * y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Float(x * y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Float(*x as f64 * y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Float(x * *y as f64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '*' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Div => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Div operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Div operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if *y == 0 {
                            return Err(RuntimeError::new("Division by zero".to_string()));
                        }
                        self.value_stack.push(Value::Integer(x / y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if *y == 0.0 {
                            return Err(RuntimeError::new("Division by zero".to_string()));
                        }
                        self.value_stack.push(Value::Float(x / y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        if *y == 0.0 {
                            return Err(RuntimeError::new("Division by zero".to_string()));
                        }
                        self.value_stack.push(Value::Float(*x as f64 / y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        if *y == 0 {
                            return Err(RuntimeError::new("Division by zero".to_string()));
                        }
                        self.value_stack.push(Value::Float(x / *y as f64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '/' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Mod => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Mod operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Mod operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if *y == 0 {
                            return Err(RuntimeError::new("Modulo by zero".to_string()));
                        }
                        self.value_stack.push(Value::Integer(x % y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if *y == 0.0 {
                            return Err(RuntimeError::new("Modulo by zero".to_string()));
                        }
                        self.value_stack.push(Value::Float(x % y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        if *y == 0.0 {
                            return Err(RuntimeError::new("Modulo by zero".to_string()));
                        }
                        self.value_stack.push(Value::Float((*x as f64) % y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        if *y == 0 {
                            return Err(RuntimeError::new("Modulo by zero".to_string()));
                        }
                        self.value_stack.push(Value::Float(x % (*y as f64)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '%' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Neg => {
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Neg operation".to_string()))?;
                match &a {
                    Value::Integer(x) => {
                        self.value_stack.push(Value::Integer(-x));
                    }
                    Value::Float(x) => {
                        self.value_stack.push(Value::Float(-x));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'neg' expects a number, got {}",
                            Self::type_name(&a)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Leq => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Leq operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Leq operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x <= y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Boolean(x <= y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Boolean((*x as f64) <= *y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(*x <= (*y as f64)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '<=' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Lt => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Lt operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Lt operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x < y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Boolean(x < y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Boolean((*x as f64) < *y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(*x < (*y as f64)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '<' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Gt => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Gt operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Gt operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x > y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Boolean(x > y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Boolean((*x as f64) > *y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(*x > (*y as f64)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '>' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Gte => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Gte operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Gte operation".to_string()))?;
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x >= y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Boolean(x >= y));
                    }
                    (Value::Integer(x), Value::Float(y)) => {
                        self.value_stack.push(Value::Boolean((*x as f64) >= *y));
                    }
                    (Value::Float(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(*x >= (*y as f64)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '>=' expects two numbers, got {} and {}",
                            Self::type_name(&a),
                            Self::type_name(&b)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Eq => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Eq operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Eq operation".to_string()))?;
                // Handle numeric type coercion for equality
                let result = match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => x == y,
                    (Value::Float(x), Value::Float(y)) => x == y,
                    (Value::Integer(x), Value::Float(y)) => *x as f64 == *y,
                    (Value::Float(x), Value::Integer(y)) => *x == *y as f64,
                    _ => a == b, // For non-numeric types, use standard PartialEq
                };
                self.value_stack.push(Value::Boolean(result));
                self.instruction_pointer += 1;
            }
            Instruction::Neq => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Neq operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Neq operation".to_string()))?;
                // Handle numeric type coercion for inequality
                let result = match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => x != y,
                    (Value::Float(x), Value::Float(y)) => x != y,
                    (Value::Integer(x), Value::Float(y)) => *x as f64 != *y,
                    (Value::Float(x), Value::Integer(y)) => *x != *y as f64,
                    _ => a != b, // For non-numeric types, use standard PartialEq
                };
                self.value_stack.push(Value::Boolean(result));
                self.instruction_pointer += 1;
            }
            Instruction::Jmp(addr) => {
                self.instruction_pointer = addr;
            }
            Instruction::JmpIfFalse(addr) => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in JmpIfFalse operation".to_string()))?;
                match value {
                    Value::Boolean(false) => {
                        self.instruction_pointer = addr;
                    }
                    Value::Boolean(true) => {
                        self.instruction_pointer += 1;
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: conditional expects boolean, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
            }
            Instruction::LoadArg(idx) => {
                let frame = self.call_stack.last().ok_or_else(|| RuntimeError::new("No frame to load arg from".to_string()))?;
                let value = frame.locals.get(idx).ok_or_else(|| RuntimeError::new(format!("Arg index {} out of bounds", idx)))?.clone();
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::GetLocal(pos) => {
                // Load from value stack at position relative to current frame's stack base
                let stack_base = if let Some(frame) = self.call_stack.last() {
                    frame.stack_base
                } else {
                    0  // Main execution has stack_base 0
                };
                let absolute_pos = stack_base + pos;
                let value = self.value_stack.get(absolute_pos)
                    .ok_or_else(|| RuntimeError::new(format!(
                        "Stack position {} (base {} + offset {}) out of bounds",
                        absolute_pos, stack_base, pos
                    )))?
                    .clone();
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::SetLocal(pos) => {
                // Set local variable at position on value stack
                let stack_base = if let Some(frame) = self.call_stack.last() {
                    frame.stack_base
                } else {
                    0  // Main execution has stack_base 0
                };
                let absolute_pos = stack_base + pos;
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in SetLocal".to_string()))?;

                if absolute_pos >= self.value_stack.len() {
                    return Err(RuntimeError::new(format!(
                        "Stack position {} (base {} + offset {}) out of bounds for SetLocal",
                        absolute_pos, stack_base, pos
                    )));
                }

                self.value_stack[absolute_pos] = value;
                self.instruction_pointer += 1;
            }
            Instruction::BeginLoop(bindings_count) => {
                // Mark the current position as a loop start
                // The loop bindings are already on the value stack (pushed by the compiler)
                // We just need to record where they are and where to jump back to
                let loop_start = self.instruction_pointer + 1; // Next instruction after BeginLoop
                let stack_base = if let Some(frame) = self.call_stack.last() {
                    frame.stack_base
                } else {
                    0
                };
                let bindings_start = self.value_stack.len() - bindings_count;

                // Update the current frame or create loop context on main
                if let Some(frame) = self.call_stack.last_mut() {
                    frame.loop_start = Some(loop_start);
                    frame.loop_bindings_start = Some(bindings_start);
                    frame.loop_bindings_count = Some(bindings_count);
                } else {
                    // We're in main execution - we need to track loop info differently
                    // For now, we'll store it in a way that recur can access
                    // Actually, let's create a dummy frame for main execution's loop
                    // Or better, let's ensure main always has a frame
                    // For simplicity, let's require that loops are only in functions
                    // Actually, let's support loops in main too by creating a frame
                    let frame = Frame::new(0, Vec::new(), "<main>".to_string(), stack_base);
                    self.call_stack.push(frame);
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.loop_start = Some(loop_start);
                        frame.loop_bindings_start = Some(bindings_start);
                        frame.loop_bindings_count = Some(bindings_count);
                    }
                }

                self.instruction_pointer += 1;
            }
            Instruction::Recur(arg_count) => {
                // Get loop information from current frame
                let (loop_start, bindings_start, bindings_count) = if let Some(frame) = self.call_stack.last() {
                    (
                        frame.loop_start.ok_or_else(|| RuntimeError::new("recur used outside of loop".to_string()))?,
                        frame.loop_bindings_start.ok_or_else(|| RuntimeError::new("recur used outside of loop".to_string()))?,
                        frame.loop_bindings_count.ok_or_else(|| RuntimeError::new("recur used outside of loop".to_string()))?,
                    )
                } else {
                    return Err(RuntimeError::new("recur used outside of loop".to_string()));
                };

                // Verify arg_count matches bindings_count
                if arg_count != bindings_count {
                    return Err(RuntimeError::new(format!(
                        "recur expects {} arguments but got {}",
                        bindings_count, arg_count
                    )));
                }

                // Pop the new values from the stack (in reverse order)
                let mut new_values = Vec::new();
                for _ in 0..arg_count {
                    new_values.push(self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Recur".to_string()))?);
                }
                new_values.reverse();

                // Update the loop bindings with new values
                for (i, value) in new_values.into_iter().enumerate() {
                    self.value_stack[bindings_start + i] = value;
                }

                // Jump back to loop start
                self.instruction_pointer = loop_start;
            }
            Instruction::PopN(n) => {
                // Pop N values from the stack
                for _ in 0..n {
                    self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow during PopN".to_string()))?;
                }
                self.instruction_pointer += 1;
            }
            Instruction::Slide(n) => {
                // Pop the top value (result), pop N values (bindings), push result back
                let result = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow during Slide".to_string()))?;
                for _ in 0..n {
                    self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow during Slide".to_string()))?;
                }
                self.value_stack.push(result);
                self.instruction_pointer += 1;
            }
            Instruction::CheckArity(expected_arity, jump_addr) => {
                // Check if current frame has the expected number of arguments
                let frame = self.call_stack.last().ok_or_else(|| RuntimeError::new("No frame for arity check".to_string()))?;
                if frame.locals.len() != expected_arity {
                    // Arity doesn't match, jump to next clause
                    self.instruction_pointer = jump_addr;
                } else {
                    // Arity matches, continue
                    self.instruction_pointer += 1;
                }
            }
            Instruction::PackRestArgs(required_count) => {
                // Collect args from required_count onwards into a list
                // Replace them with a single list in frame.locals
                let frame = self.call_stack.last_mut().ok_or_else(|| RuntimeError::new("No frame for PackRestArgs".to_string()))?;

                if frame.locals.len() < required_count {
                    return Err(RuntimeError::new(format!(
                        "Not enough arguments: expected at least {}, got {}",
                        required_count,
                        frame.locals.len()
                    )));
                }

                // Collect rest args into a list
                let rest_args: Vec<Value> = frame.locals.drain(required_count..).collect();
                // Push the rest list as the next parameter
                frame.locals.push(Value::List(List::from_vec(rest_args)));

                self.instruction_pointer += 1;
            }
            Instruction::MakeClosure(params, body, num_captured) => {
                // Pop captured values from stack (compiler pushed them in order)
                let mut captured_values = Vec::new();
                for _ in 0..num_captured {
                    captured_values.push(self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow during MakeClosure".to_string()))?);
                }
                captured_values.reverse(); // They were pushed in order, so reverse after popping

                // Create closure with captured values
                // We store as (name, value) pairs, but for now we don't have names at runtime
                // So we'll just use indices and the compiler will emit LoadCaptured(idx)
                let captured: Vec<(String, Value)> = captured_values
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (format!("__captured_{}", i), v))
                    .collect();

                let closure = Value::Closure(Arc::new(ClosureData {
                    params: params.clone(),
                    rest_param: None, // Regular closure, no variadic support
                    body: body.clone(),
                    captured,
                }));

                self.value_stack.push(closure);
                self.instruction_pointer += 1;
            }
            Instruction::MakeVariadicClosure(required_params, rest_param, body, num_captured) => {
                // Pop captured values from stack (compiler pushed them in order)
                let mut captured_values = Vec::new();
                for _ in 0..num_captured {
                    captured_values.push(self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow during MakeVariadicClosure".to_string()))?);
                }
                captured_values.reverse(); // They were pushed in order, so reverse after popping

                // Create closure with captured values
                let captured: Vec<(String, Value)> = captured_values
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (format!("__captured_{}", i), v))
                    .collect();

                let closure = Value::Closure(Arc::new(ClosureData {
                    params: required_params.clone(),
                    rest_param: Some(rest_param.clone()), // Variadic closure
                    body: body.clone(),
                    captured,
                }));

                self.value_stack.push(closure);
                self.instruction_pointer += 1;
            }
            Instruction::CallClosure(arg_count) => {
                // Pop arguments from stack (in reverse order)
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in CallClosure".to_string()))?);
                }
                args.reverse();

                // Pop the function/closure
                let callable = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in CallClosure".to_string()))?;

                match callable {
                    Value::Function(ref fn_name) => {
                        // Call a named function (same as Call instruction)
                        let fn_bytecode = self.functions.get(fn_name.as_str())
                            .ok_or_else(|| RuntimeError::new(format!("Undefined function '{}'", fn_name)))?
                            .clone();

                        let frame = Frame {
                            return_address: self.instruction_pointer + 1,
                            locals: args,
                            return_bytecode: self.current_bytecode.clone(),
                            function_name: fn_name.to_string(),
                            captured: Vec::new(),
                            stack_base: self.value_stack.len(),
                            loop_start: None,
                            loop_bindings_start: None,
                            loop_bindings_count: None,
                        };
                        self.call_stack.push(frame);

                        self.current_bytecode = fn_bytecode;
                        self.instruction_pointer = 0;
                    }
                    Value::Closure(ref closure_data) => {
                        // Verify arity
                        match &closure_data.rest_param {
                            None => {
                                // Regular closure - exact arity match required
                                if closure_data.params.len() != args.len() {
                                    return Err(RuntimeError::new(format!(
                                        "Closure arity mismatch: expected {} argument(s), got {}",
                                        closure_data.params.len(),
                                        args.len()
                                    )));
                                }
                            }
                            Some(_rest_name) => {
                                // Variadic closure - need at least the required params
                                if args.len() < closure_data.params.len() {
                                    return Err(RuntimeError::new(format!(
                                        "Variadic closure arity mismatch: expected at least {} argument(s), got {}",
                                        closure_data.params.len(),
                                        args.len()
                                    )));
                                }
                                // Pack extra args into a list and append to args
                                let rest_args: Vec<Value> = args.drain(closure_data.params.len()..).collect();
                                args.push(Value::List(List::from_vec(rest_args)));
                            }
                        }

                        // Create frame with arguments and captured environment
                        let frame = Frame {
                            return_address: self.instruction_pointer + 1,
                            locals: args,
                            return_bytecode: self.current_bytecode.to_vec(),
                            function_name: "<closure>".to_string(),
                            captured: closure_data.captured.iter().map(|(_, v)| v.clone()).collect(),
                            stack_base: self.value_stack.len(), // Current stack top is base for this function
                            loop_start: None,
                            loop_bindings_start: None,
                            loop_bindings_count: None,
                        };

                        self.call_stack.push(frame);

                        // Switch to closure body bytecode
                        self.current_bytecode = closure_data.body.clone();
                        self.instruction_pointer = 0;
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: expected function or closure, got {}",
                            Self::type_name(&callable)
                        )));
                    }
                }
            }
            Instruction::Apply => {
                // Apply function to a list of arguments
                // Stack: ... <function/closure> <list> (top)

                // Pop the argument list
                let arg_list = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Apply".to_string()))?;

                // Extract arguments from list
                let args = match arg_list {
                    Value::List(list) => list.to_vec(),
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error in apply: expected list of arguments, got {}",
                            Self::type_name(&arg_list)
                        )));
                    }
                };

                // Pop the function/closure
                let callable = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Apply".to_string()))?;

                match callable {
                    Value::Function(ref fn_name) => {
                        // Call a named function
                        let fn_bytecode = self.functions.get(fn_name.as_str())
                            .ok_or_else(|| RuntimeError::new(format!("Undefined function '{}'", fn_name)))?
                            .clone();

                        let frame = Frame {
                            return_address: self.instruction_pointer + 1,
                            locals: args,
                            return_bytecode: self.current_bytecode.clone(),
                            function_name: fn_name.to_string(),
                            captured: Vec::new(),
                            stack_base: self.value_stack.len(),
                            loop_start: None,
                            loop_bindings_start: None,
                            loop_bindings_count: None,
                        };
                        self.call_stack.push(frame);

                        self.current_bytecode = fn_bytecode;
                        self.instruction_pointer = 0;
                    }
                    Value::Closure(ref closure_data) => {
                        // Call a closure with variadic support
                        let mut args = args;

                        // Verify arity and handle variadic parameters
                        match &closure_data.rest_param {
                            None => {
                                // Regular closure - exact arity match required
                                if closure_data.params.len() != args.len() {
                                    return Err(RuntimeError::new(format!(
                                        "Closure arity mismatch in apply: expected {} argument(s), got {}",
                                        closure_data.params.len(),
                                        args.len()
                                    )));
                                }
                            }
                            Some(_rest_name) => {
                                // Variadic closure - need at least the required params
                                if args.len() < closure_data.params.len() {
                                    return Err(RuntimeError::new(format!(
                                        "Variadic closure arity mismatch in apply: expected at least {} argument(s), got {}",
                                        closure_data.params.len(),
                                        args.len()
                                    )));
                                }
                                // Pack extra arguments into a list for the rest parameter
                                let rest_args: Vec<Value> = args.drain(closure_data.params.len()..).collect();
                                args.push(Value::List(List::from_vec(rest_args)));
                            }
                        }

                        let frame = Frame {
                            return_address: self.instruction_pointer + 1,
                            locals: args,
                            return_bytecode: self.current_bytecode.clone(),
                            function_name: "<closure>".to_string(),
                            captured: closure_data.captured.iter().map(|(_, v)| v.clone()).collect(),
                            stack_base: self.value_stack.len(),
                            loop_start: None,
                            loop_bindings_start: None,
                            loop_bindings_count: None,
                        };
                        self.call_stack.push(frame);

                        // Switch to closure body bytecode
                        self.current_bytecode = closure_data.body.clone();
                        self.instruction_pointer = 0;
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error in apply: expected function or closure, got {}",
                            Self::type_name(&callable)
                        )));
                    }
                }
            }
            Instruction::LoadCaptured(idx) => {
                // Load a captured variable from the current closure's environment
                let frame = self.call_stack.last().ok_or_else(|| RuntimeError::new("No frame for LoadCaptured".to_string()))?;
                let value = frame.captured.get(idx)
                    .ok_or_else(|| RuntimeError::new(format!("Captured variable index {} out of bounds", idx)))?
                    .clone();
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::Print => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Print".to_string()))?;
                println!("{}", Self::format_value(&value));
                // Push the value back so print can be used in expressions
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::Ret => {
                let frame = self.call_stack.pop().ok_or_else(|| RuntimeError::new("No frame to return from".to_string()))?;
                self.current_bytecode = frame.return_bytecode;
                self.instruction_pointer = frame.return_address;
            }
            Instruction::Call(fn_name, arg_count) => {
                let fn_bytecode = self.functions.get(&fn_name)
                    .ok_or_else(|| RuntimeError::new(format!("Undefined function '{}'", fn_name)))?
                    .clone();

                // Pop arguments from value stack in reverse order
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Call".to_string()))?);
                }
                args.reverse();

                // Create new frame with return bytecode and function name for stack traces
                let frame = Frame {
                    return_address: self.instruction_pointer + 1,
                    locals: args,
                    return_bytecode: self.current_bytecode.clone(),
                    function_name: fn_name.clone(),
                    captured: Vec::new(), // Regular functions don't have captured variables
                    stack_base: self.value_stack.len(), // Current stack top is base for this function
                    loop_start: None,
                    loop_bindings_start: None,
                    loop_bindings_count: None,
                };
                self.call_stack.push(frame);

                // Switch to function bytecode
                self.current_bytecode = fn_bytecode;
                self.instruction_pointer = 0;
            }
            Instruction::TailCall(fn_name, arg_count) => {
                let fn_bytecode = self.functions.get(&fn_name)
                    .ok_or_else(|| RuntimeError::new(format!("Undefined function '{}'", fn_name)))?
                    .clone();

                // Pop arguments from value stack in reverse order
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in TailCall".to_string()))?);
                }
                args.reverse();

                // Reuse current frame instead of pushing a new one
                // This is the key to tail call optimization!
                if let Some(frame) = self.call_stack.last_mut() {
                    // Clear the value_stack back to this frame's base
                    // This is crucial - any let bindings or temporary values should be removed
                    self.value_stack.truncate(frame.stack_base);

                    // Replace the locals (arguments) in the current frame
                    frame.locals = args;
                    // Update function name for stack traces
                    frame.function_name = fn_name.clone();
                    // Keep the same return address, return bytecode, and stack_base
                } else {
                    // No frame exists (top-level call), treat as regular call
                    let frame = Frame {
                        return_address: self.instruction_pointer + 1,
                        locals: args,
                        return_bytecode: self.current_bytecode.clone(),
                        function_name: fn_name.clone(),
                        captured: Vec::new(),
                        stack_base: self.value_stack.len(), // Current stack top is base for this function
                        loop_start: None,
                        loop_bindings_start: None,
                        loop_bindings_count: None,
                    };
                    self.call_stack.push(frame);
                }

                // Switch to function bytecode
                self.current_bytecode = fn_bytecode;
                self.instruction_pointer = 0;
            }
            Instruction::Halt => {
                self.halted = true;
            }
            Instruction::Cons => {
                let second = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Cons".to_string()))?;
                let first = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Cons".to_string()))?;

                // cons creates a list by prepending first to second
                // (cons 1 '(2 3)) -> '(1 2 3)
                // (cons 1 2) -> '(1 2) [improper list, to be represented as proper list]
                let new_list = match second {
                    Value::List(tail) => List::cons(first, tail),
                    other => List::cons(first, List::cons(other, List::Nil)),
                };
                self.value_stack.push(Value::List(new_list));
                self.instruction_pointer += 1;
            }
            Instruction::Car => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Car".to_string()))?;
                match value {
                    Value::List(list) => {
                        match list.car() {
                            Some(head) => self.value_stack.push(head.clone()),
                            None => return Err(RuntimeError::new("'car' cannot take the first element of an empty list".to_string())),
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'car' expects a list, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Cdr => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Cdr".to_string()))?;
                match value {
                    Value::List(list) => {
                        match list.cdr() {
                            Some(tail) => self.value_stack.push(Value::List(tail)),
                            None => return Err(RuntimeError::new("'cdr' cannot take the rest of an empty list".to_string())),
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'cdr' expects a list, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::IsList => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsList".to_string()))?;
                let is_list = matches!(value, Value::List(_));
                self.value_stack.push(Value::Boolean(is_list));
                self.instruction_pointer += 1;
            }
            Instruction::IsInteger => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsInteger".to_string()))?;
                let is_integer = matches!(value, Value::Integer(_));
                self.value_stack.push(Value::Boolean(is_integer));
                self.instruction_pointer += 1;
            }
            Instruction::IsFloat => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsFloat".to_string()))?;
                let is_float = matches!(value, Value::Float(_));
                self.value_stack.push(Value::Boolean(is_float));
                self.instruction_pointer += 1;
            }
            Instruction::IsNumber => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsNumber".to_string()))?;
                let is_number = matches!(value, Value::Integer(_) | Value::Float(_));
                self.value_stack.push(Value::Boolean(is_number));
                self.instruction_pointer += 1;
            }
            Instruction::IsBoolean => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsBoolean".to_string()))?;
                let is_boolean = matches!(value, Value::Boolean(_));
                self.value_stack.push(Value::Boolean(is_boolean));
                self.instruction_pointer += 1;
            }
            Instruction::IsFunction => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsFunction".to_string()))?;
                let is_function = matches!(value, Value::Function(_));
                self.value_stack.push(Value::Boolean(is_function));
                self.instruction_pointer += 1;
            }
            Instruction::IsClosure => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsClosure".to_string()))?;
                let is_closure = matches!(value, Value::Closure { .. });
                self.value_stack.push(Value::Boolean(is_closure));
                self.instruction_pointer += 1;
            }
            Instruction::IsProcedure => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsProcedure".to_string()))?;
                let is_procedure = matches!(value, Value::Function(_) | Value::Closure { .. });
                self.value_stack.push(Value::Boolean(is_procedure));
                self.instruction_pointer += 1;
            }
            Instruction::IsString => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsString".to_string()))?;
                let is_string = matches!(value, Value::String(_));
                self.value_stack.push(Value::Boolean(is_string));
                self.instruction_pointer += 1;
            }
            Instruction::IsSymbol => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsSymbol".to_string()))?;
                let is_symbol = matches!(value, Value::Symbol(_));
                self.value_stack.push(Value::Boolean(is_symbol));
                self.instruction_pointer += 1;
            }
            Instruction::SymbolToString => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in SymbolToString".to_string()))?;
                match value {
                    Value::Symbol(s) => {
                        self.value_stack.push(Value::String(s));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'symbol->string' expects a symbol, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringToSymbol => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringToSymbol".to_string()))?;
                match value {
                    Value::String(s) => {
                        self.value_stack.push(Value::Symbol(s));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string->symbol' expects a string, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Append => {
                // Pop two lists and append them
                let second = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Append".to_string()))?;
                let first = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Append".to_string()))?;

                match (&first, &second) {
                    (Value::List(first_list), Value::List(second_list)) => {
                        // For append, we need to copy the first list and attach second to the end
                        // This is O(n) in the first list length - append is inherently expensive
                        let first_vec = first_list.to_vec();
                        let second_vec = second_list.to_vec();
                        let mut result = first_vec;
                        result.extend(second_vec);
                        self.value_stack.push(Value::List(List::from_vec(result)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'append' expects two lists, got {} and {}",
                            Self::type_name(&first),
                            Self::type_name(&second)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::MakeList(n) => {
                // Pop n values and create a list
                let mut items = Vec::new();
                for _ in 0..n {
                    items.push(self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in MakeList".to_string()))?);
                }
                items.reverse(); // Reverse because we popped in reverse order
                self.value_stack.push(Value::List(List::from_vec(items)));
                self.instruction_pointer += 1;
            }
            Instruction::ListRef => {
                // Pop index and list, push element at that index
                let index = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ListRef".to_string()))?;
                let list_val = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ListRef".to_string()))?;

                match (&list_val, &index) {
                    (Value::List(list), Value::Integer(idx)) => {
                        if *idx < 0 {
                            return Err(RuntimeError::new(format!("'list-ref' index cannot be negative: {}", idx)));
                        }
                        let idx_usize = *idx as usize;
                        // Walk the list to the index position
                        let mut current = list.clone();
                        for _ in 0..idx_usize {
                            match current.cdr() {
                                Some(tail) => current = tail,
                                None => return Err(RuntimeError::new(format!(
                                    "'list-ref' index {} out of bounds for list of length {}",
                                    idx, list.len()
                                ))),
                            }
                        }
                        match current.car() {
                            Some(val) => self.value_stack.push(val.clone()),
                            None => return Err(RuntimeError::new(format!(
                                "'list-ref' index {} out of bounds for list of length {}",
                                idx, list.len()
                            ))),
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'list-ref' expects a list and an integer, got {} and {}",
                            Self::type_name(&list_val),
                            Self::type_name(&index)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::ListLength => {
                // Pop list and push its length
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ListLength".to_string()))?;
                match value {
                    Value::List(items) => {
                        self.value_stack.push(Value::Integer(items.len() as i64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'list-length' expects a list, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::NumberToString => {
                // Pop integer and push string representation
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in NumberToString".to_string()))?;
                match value {
                    Value::Integer(n) => {
                        self.value_stack.push(Value::String(Arc::new(n.to_string())));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'number->string' expects an integer, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringToNumber => {
                // Pop string and push integer (or error if not a valid number)
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringToNumber".to_string()))?;
                match value {
                    Value::String(s) => {
                        match s.trim().parse::<i64>() {
                            Ok(n) => {
                                self.value_stack.push(Value::Integer(n));
                            }
                            Err(_) => {
                                return Err(RuntimeError::new(format!(
                                    "Type error: 'string->number' cannot parse '{}' as a number",
                                    s
                                )));
                            }
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string->number' expects a string, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::LoadGlobal(name) => {
                let value = self.global_vars.get(&name)
                    .ok_or_else(|| RuntimeError::new(format!("Undefined global variable '{}'", name)))?
                    .clone();
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::StoreGlobal(name) => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StoreGlobal".to_string()))?;
                self.global_vars.insert(name, value);
                self.instruction_pointer += 1;
            }
            Instruction::StringLength => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringLength".to_string()))?;
                match value {
                    Value::String(s) => {
                        self.value_stack.push(Value::Integer(s.len() as i64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string-length' expects a string, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Substring => {
                let end = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Substring".to_string()))?;
                let start = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Substring".to_string()))?;
                let string = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Substring".to_string()))?;

                match (&string, &start, &end) {
                    (Value::String(s), Value::Integer(start_idx), Value::Integer(end_idx)) => {
                        let start = (*start_idx).max(0) as usize;
                        let end = (*end_idx).min(s.len() as i64) as usize;
                        if start <= end && end <= s.len() {
                            let result = s.chars().skip(start).take(end - start).collect::<String>();
                            self.value_stack.push(Value::String(Arc::new(result)));
                        } else {
                            return Err(RuntimeError::new(format!(
                                "'substring' invalid indices: start={}, end={}, string length={}",
                                start_idx, end_idx, s.len()
                            )));
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'substring' expects a string and two integers, got {}, {}, and {}",
                            Self::type_name(&string),
                            Self::type_name(&start),
                            Self::type_name(&end)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringAppend => {
                let second = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringAppend".to_string()))?;
                let first = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringAppend".to_string()))?;

                match (&first, &second) {
                    (Value::String(s1), Value::String(s2)) => {
                        self.value_stack.push(Value::String(Arc::new(format!("{}{}", s1, s2))));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string-append' expects two strings, got {} and {}",
                            Self::type_name(&first),
                            Self::type_name(&second)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringToList => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringToList".to_string()))?;
                match value {
                    Value::String(s) => {
                        let char_list: Vec<Value> = s.chars()
                            .map(|c| Value::String(Arc::new(c.to_string())))
                            .collect();
                        self.value_stack.push(Value::List(List::from_vec(char_list)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string->list' expects a string, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::ListToString => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ListToString".to_string()))?;
                match value {
                    Value::List(list) => {
                        let mut result = String::new();
                        for item in list.iter() {
                            match item {
                                Value::String(s) => result.push_str(&s),
                                _ => {
                                    return Err(RuntimeError::new(format!(
                                        "Type error: 'list->string' expects a list of strings, but found {}",
                                        Self::type_name(item)
                                    )));
                                }
                            }
                        }
                        self.value_stack.push(Value::String(Arc::new(result)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'list->string' expects a list, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::CharCode => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in CharCode".to_string()))?;
                match &value {
                    Value::String(s) => {
                        if s.len() != 1 {
                            return Err(RuntimeError::new(format!(
                                "'char-code' expects a single-character string, got {} characters",
                                s.len()
                            )));
                        }
                        let code = s.chars().next().unwrap() as i64;
                        self.value_stack.push(Value::Integer(code));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'char-code' expects a string, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringSplit => {
                let delimiter = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringSplit".to_string()))?;
                let string = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringSplit".to_string()))?;
                match (&string, &delimiter) {
                    (Value::String(s), Value::String(delim)) => {
                        let parts: Vec<Value> = if delim.is_empty() {
                            // Split into characters
                            s.chars().map(|c| Value::String(Arc::new(c.to_string()))).collect()
                        } else {
                            s.split(delim.as_str()).map(|part| Value::String(Arc::new(part.to_string()))).collect()
                        };
                        self.value_stack.push(Value::List(List::from_vec(parts)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string-split' expects two strings, got {} and {}",
                            Self::type_name(&string),
                            Self::type_name(&delimiter)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringJoin => {
                let delimiter = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringJoin".to_string()))?;
                let list_val = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringJoin".to_string()))?;
                match (&list_val, &delimiter) {
                    (Value::List(list), Value::String(delim)) => {
                        let mut parts = Vec::new();
                        for item in list.iter() {
                            match item {
                                Value::String(s) => parts.push(s.to_string()),
                                _ => {
                                    return Err(RuntimeError::new(format!(
                                        "Type error: 'string-join' expects a list of strings, but found {}",
                                        Self::type_name(item)
                                    )));
                                }
                            }
                        }
                        let result = parts.join(delim.as_str());
                        self.value_stack.push(Value::String(Arc::new(result)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string-join' expects a list and a string, got {} and {}",
                            Self::type_name(&list_val),
                            Self::type_name(&delimiter)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringTrim => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringTrim".to_string()))?;
                match value {
                    Value::String(s) => {
                        self.value_stack.push(Value::String(Arc::new(s.trim().to_string())));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string-trim' expects a string, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringReplace => {
                let new_str = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringReplace".to_string()))?;
                let old_str = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringReplace".to_string()))?;
                let string = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in StringReplace".to_string()))?;
                match (&string, &old_str, &new_str) {
                    (Value::String(s), Value::String(old), Value::String(new)) => {
                        let result = s.replace(old.as_str(), new.as_str());
                        self.value_stack.push(Value::String(Arc::new(result)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'string-replace' expects three strings, got {}, {}, and {}",
                            Self::type_name(&string),
                            Self::type_name(&old_str),
                            Self::type_name(&new_str)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::ReadFile => {
                let path = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ReadFile".to_string()))?;
                match path {
                    Value::String(path_str) => {
                        match std::fs::read_to_string(path_str.as_str()) {
                            Ok(contents) => {
                                self.value_stack.push(Value::String(Arc::new(contents)));
                            }
                            Err(e) => {
                                return Err(RuntimeError::new(format!(
                                    "'read-file' failed to read '{}': {}",
                                    path_str, e
                                )));
                            }
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'read-file' expects a string path, got {}",
                            Self::type_name(&path)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::WriteFile => {
                let content = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in WriteFile".to_string()))?;
                let path = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in WriteFile".to_string()))?;

                match (&path, &content) {
                    (Value::String(path_str), Value::String(content_str)) => {
                        match std::fs::write(path_str.as_str(), content_str.as_str()) {
                            Ok(_) => {
                                self.value_stack.push(Value::Boolean(true));
                            }
                            Err(e) => {
                                eprintln!("write-file: failed to write '{}': {}", path_str, e);
                                self.value_stack.push(Value::Boolean(false));
                            }
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'write-file' expects a string path and string content, got {} and {}",
                            Self::type_name(&path),
                            Self::type_name(&content)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::FileExists => {
                let path = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in FileExists".to_string()))?;
                match path {
                    Value::String(path_str) => {
                        let exists = std::path::Path::new(path_str.as_str()).exists();
                        self.value_stack.push(Value::Boolean(exists));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'file-exists?' expects a string path, got {}",
                            Self::type_name(&path)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::WriteBinaryFile => {
                let bytes_list = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in WriteBinaryFile".to_string()))?;
                let path = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in WriteBinaryFile".to_string()))?;

                match (&path, &bytes_list) {
                    (Value::String(path_str), Value::List(bytes)) => {
                        // Convert list of integers to Vec<u8>
                        let mut byte_vec = Vec::new();
                        for byte_val in bytes.iter() {
                            match byte_val {
                                Value::Integer(n) if n >= &0 && n <= &255 => {
                                    byte_vec.push(*n as u8);
                                }
                                _ => {
                                    return Err(RuntimeError::new(format!(
                                        "'write-binary-file' expects a list of integers 0-255, but found {}",
                                        Self::type_name(byte_val)
                                    )));
                                }
                            }
                        }

                        // Write bytes to file
                        match std::fs::write(path_str.as_str(), &byte_vec) {
                            Ok(_) => self.value_stack.push(Value::Boolean(true)),
                            Err(e) => {
                                eprintln!("write-binary-file: failed to write '{}': {}", path_str, e);
                                self.value_stack.push(Value::Boolean(false));
                            }
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'write-binary-file' expects a string path and a list of bytes, got {} and {}",
                            Self::type_name(&path),
                            Self::type_name(&bytes_list)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::LoadFile => {
                let path = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in LoadFile".to_string()))?;
                match path {
                    Value::String(path_str) => {
                        // Read the file
                        let source = std::fs::read_to_string(path_str.as_str()).map_err(|e| {
                            RuntimeError::new(format!("'load' failed to read '{}': {}", path_str, e))
                        })?;

                        // Parse the file
                        let mut parser = Parser::new(&source);
                        let exprs = parser.parse_all().map_err(|e| {
                            RuntimeError::new(format!("'load' failed to parse '{}': {}", path_str, e))
                        })?;

                        // Compile the file
                        let mut compiler = Compiler::new();
                        let (functions, main) = compiler.compile_program(&exprs).map_err(|e| {
                            RuntimeError::new(format!("'load' failed to compile '{}': {}", path_str, e.message))
                        })?;

                        // Merge compiled functions into VM's function table
                        self.functions.extend(functions);

                        // Execute the main bytecode from the loaded file
                        // Save current state
                        let saved_bytecode = std::mem::replace(&mut self.current_bytecode, main);
                        let saved_ip = self.instruction_pointer;

                        // Execute the loaded file's main code
                        self.instruction_pointer = 0;
                        while !self.halted && self.instruction_pointer < self.current_bytecode.len() {
                            self.execute_one_instruction()?;
                        }

                        // Restore previous state
                        self.current_bytecode = saved_bytecode;
                        self.instruction_pointer = saved_ip;
                        self.halted = false;

                        // Push a success value (true) onto the stack
                        self.value_stack.push(Value::Boolean(true));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'load' expects a string path, got {}",
                            Self::type_name(&path)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::RequireFile => {
                let path = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in RequireFile".to_string()))?;
                match path {
                    Value::String(path_str) => {
                        // Normalize the path to canonical form for consistent tracking
                        let canonical_path = std::fs::canonicalize(path_str.as_str())
                            .unwrap_or_else(|_| std::path::PathBuf::from(path_str.as_str()))
                            .to_string_lossy()
                            .to_string();

                        // Check if already loaded
                        if self.loaded_modules.contains(&canonical_path) {
                            // Already loaded, just return true
                            self.value_stack.push(Value::Boolean(true));
                        } else {
                            // Not loaded yet, load it
                            // Read the file
                            let source = std::fs::read_to_string(path_str.as_str()).map_err(|e| {
                                RuntimeError::new(format!("'require' failed to read '{}': {}", path_str, e))
                            })?;

                            // Parse the file
                            let mut parser = Parser::new(&source);
                            let exprs = parser.parse_all().map_err(|e| {
                                RuntimeError::new(format!("'require' failed to parse '{}': {}", path_str, e))
                            })?;

                            // Compile the file
                            let mut compiler = Compiler::new();
                            let (functions, main) = compiler.compile_program(&exprs).map_err(|e| {
                                RuntimeError::new(format!("'require' failed to compile '{}': {}", path_str, e.message))
                            })?;

                            // Merge compiled functions into VM's function table
                            self.functions.extend(functions);

                            // Execute the main bytecode from the loaded file
                            // Save current state
                            let saved_bytecode = std::mem::replace(&mut self.current_bytecode, main);
                            let saved_ip = self.instruction_pointer;

                            // Execute the loaded file's main code
                            self.instruction_pointer = 0;
                            while !self.halted && self.instruction_pointer < self.current_bytecode.len() {
                                self.execute_one_instruction()?;
                            }

                            // Restore previous state
                            self.current_bytecode = saved_bytecode;
                            self.instruction_pointer = saved_ip;
                            self.halted = false;

                            // Mark as loaded
                            self.loaded_modules.insert(canonical_path);

                            // Push a success value (true) onto the stack
                            self.value_stack.push(Value::Boolean(true));
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'require' expects a string path, got {}",
                            Self::type_name(&path)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::GetArgs => {
                // Convert Vec<String> to Value::List of Value::String
                let args_list: Vec<Value> = self.args.iter()
                    .map(|s| Value::String(Arc::new(s.clone())))
                    .collect();
                self.value_stack.push(Value::List(List::from_vec(args_list)));
                self.instruction_pointer += 1;
            }
            // HashMap operations
            Instruction::MakeHashMap(n) => {
                // Pop n key-value pairs and create a hashmap
                let mut pairs = Vec::new();
                for _ in 0..n {
                    let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in MakeHashMap".to_string()))?;
                    let key = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in MakeHashMap".to_string()))?;
                    pairs.push((key, value));
                }
                pairs.reverse(); // Reverse because we popped in reverse order

                let mut map = std::collections::HashMap::new();
                for (key, value) in pairs {
                    match key {
                        Value::String(s) => {
                            map.insert(s.to_string(), value);
                        }
                        _ => {
                            return Err(RuntimeError::new(format!(
                                "Type error: hashmap keys must be strings, got {}",
                                Self::type_name(&key)
                            )));
                        }
                    }
                }
                self.value_stack.push(Value::HashMap(Arc::new(map)));
                self.instruction_pointer += 1;
            }
            Instruction::HashMapGet => {
                // Pop key and hashmap, push value
                let key = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapGet".to_string()))?;
                let map = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapGet".to_string()))?;

                match (&map, &key) {
                    (Value::HashMap(m), Value::String(k)) => {
                        match m.get(k.as_str()) {
                            Some(v) => self.value_stack.push(v.clone()),
                            None => {
                                return Err(RuntimeError::new(format!(
                                    "Key '{}' not found in hashmap",
                                    k
                                )));
                            }
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'get' expects a hashmap and a string key, got {} and {}",
                            Self::type_name(&map),
                            Self::type_name(&key)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::HashMapSet => {
                // Pop value, key, and hashmap, push new hashmap with key-value set
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapSet".to_string()))?;
                let key = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapSet".to_string()))?;
                let map = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapSet".to_string()))?;

                match (&map, &key) {
                    (Value::HashMap(m), Value::String(k)) => {
                        let mut new_map = (**m).clone();
                        new_map.insert(k.to_string(), value);
                        self.value_stack.push(Value::HashMap(Arc::new(new_map)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'set' expects a hashmap and a string key, got {} and {}",
                            Self::type_name(&map),
                            Self::type_name(&key)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::HashMapKeys => {
                // Pop hashmap and push list of keys
                let map = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapKeys".to_string()))?;

                match map {
                    Value::HashMap(m) => {
                        let keys: Vec<Value> = m.keys().map(|k| Value::String(Arc::new(k.clone()))).collect();
                        self.value_stack.push(Value::List(List::from_vec(keys)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'keys' expects a hashmap, got {}",
                            Self::type_name(&map)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::HashMapValues => {
                // Pop hashmap and push list of values
                let map = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapValues".to_string()))?;

                match map {
                    Value::HashMap(m) => {
                        let values: Vec<Value> = m.values().cloned().collect();
                        self.value_stack.push(Value::List(List::from_vec(values)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'values' expects a hashmap, got {}",
                            Self::type_name(&map)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::HashMapContainsKey => {
                // Pop key and hashmap, push boolean
                let key = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapContainsKey".to_string()))?;
                let map = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in HashMapContainsKey".to_string()))?;

                match (&map, &key) {
                    (Value::HashMap(m), Value::String(k)) => {
                        self.value_stack.push(Value::Boolean(m.contains_key(k.as_str())));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'contains-key?' expects a hashmap and a string key, got {} and {}",
                            Self::type_name(&map),
                            Self::type_name(&key)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::IsHashMap => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsHashMap".to_string()))?;
                let is_hashmap = matches!(value, Value::HashMap(_));
                self.value_stack.push(Value::Boolean(is_hashmap));
                self.instruction_pointer += 1;
            }
            // Vector operations
            Instruction::MakeVector(n) => {
                // Pop n values and create a vector
                let mut items = Vec::new();
                for _ in 0..n {
                    items.push(self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in MakeVector".to_string()))?);
                }
                items.reverse(); // Reverse because we popped in reverse order
                self.value_stack.push(Value::Vector(Arc::new(items)));
                self.instruction_pointer += 1;
            }
            Instruction::VectorGet => {
                // Pop index and vector, push element at that index
                let index = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorGet".to_string()))?;
                let vec = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorGet".to_string()))?;

                match (&vec, &index) {
                    (Value::Vector(items), Value::Integer(idx)) => {
                        if *idx < 0 {
                            return Err(RuntimeError::new(format!("'vector-ref' index cannot be negative: {}", idx)));
                        }
                        let idx_usize = *idx as usize;
                        if idx_usize >= items.len() {
                            return Err(RuntimeError::new(format!(
                                "'vector-ref' index {} out of bounds for vector of length {}",
                                idx, items.len()
                            )));
                        }
                        self.value_stack.push(items[idx_usize].clone());
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'vector-ref' expects a vector and an integer, got {} and {}",
                            Self::type_name(&vec),
                            Self::type_name(&index)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::VectorSet => {
                // Pop value, index, and vector, push new vector with element at index set
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorSet".to_string()))?;
                let index = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorSet".to_string()))?;
                let vec = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorSet".to_string()))?;

                match (&vec, &index) {
                    (Value::Vector(items), Value::Integer(idx)) => {
                        if *idx < 0 {
                            return Err(RuntimeError::new(format!("'vector-set!' index cannot be negative: {}", idx)));
                        }
                        let idx_usize = *idx as usize;
                        if idx_usize >= items.len() {
                            return Err(RuntimeError::new(format!(
                                "'vector-set!' index {} out of bounds for vector of length {}",
                                idx, items.len()
                            )));
                        }
                        let mut new_vec = (**items).clone();
                        new_vec[idx_usize] = value;
                        self.value_stack.push(Value::Vector(Arc::new(new_vec)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'vector-set!' expects a vector, an integer, and a value, got {} and {}",
                            Self::type_name(&vec),
                            Self::type_name(&index)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::VectorPush => {
                // Pop value and vector, push new vector with value appended
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorPush".to_string()))?;
                let vec = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorPush".to_string()))?;

                match vec {
                    Value::Vector(items) => {
                        let mut new_items = (*items).clone();
                        new_items.push(value);
                        self.value_stack.push(Value::Vector(Arc::new(new_items)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'vector-push!' expects a vector, got {}",
                            Self::type_name(&vec)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::VectorPop => {
                // Pop vector, push vector without last element and the last element (two values on stack)
                let vec = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorPop".to_string()))?;

                match vec {
                    Value::Vector(items) => {
                        if items.is_empty() {
                            return Err(RuntimeError::new("'vector-pop!' cannot pop from empty vector".to_string()));
                        }
                        let mut new_vec = (*items).clone();
                        let last = new_vec.pop().unwrap();
                        self.value_stack.push(Value::Vector(Arc::new(new_vec)));
                        self.value_stack.push(last);
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'vector-pop!' expects a vector, got {}",
                            Self::type_name(&vec)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::VectorLength => {
                // Pop vector and push its length
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorLength".to_string()))?;
                match value {
                    Value::Vector(items) => {
                        self.value_stack.push(Value::Integer(items.len() as i64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'vector-length' expects a vector, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::IsVector => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IsVector".to_string()))?;
                let is_vector = matches!(value, Value::Vector(_));
                self.value_stack.push(Value::Boolean(is_vector));
                self.instruction_pointer += 1;
            }
            Instruction::ListToVector => {
                // Pop list and push vector with same elements
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ListToVector".to_string()))?;
                match value {
                    Value::List(list) => {
                        self.value_stack.push(Value::Vector(Arc::new(list.to_vec())));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'list->vector' expects a list, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::VectorToList => {
                // Pop vector and push list with same elements
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in VectorToList".to_string()))?;
                match value {
                    Value::Vector(vec) => {
                        self.value_stack.push(Value::List(List::from_vec((*vec).clone())));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'vector->list' expects a vector, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::IntToFloat => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in IntToFloat".to_string()))?;
                match value {
                    Value::Integer(n) => {
                        self.value_stack.push(Value::Float(n as f64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'int->float' expects an integer, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::FloatToInt => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in FloatToInt".to_string()))?;
                match value {
                    Value::Float(f) => {
                        self.value_stack.push(Value::Integer(f as i64));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'float->int' expects a float, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Sqrt => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Sqrt".to_string()))?;
                let f = match value {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'sqrt' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Float(f.sqrt()));
                self.instruction_pointer += 1;
            }
            Instruction::Sin => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Sin".to_string()))?;
                let f = match value {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'sin' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Float(f.sin()));
                self.instruction_pointer += 1;
            }
            Instruction::Cos => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Cos".to_string()))?;
                let f = match value {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'cos' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Float(f.cos()));
                self.instruction_pointer += 1;
            }
            Instruction::Floor => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Floor".to_string()))?;
                let result = match value {
                    Value::Float(f) => f.floor() as i64,
                    Value::Integer(n) => n,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'floor' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Integer(result));
                self.instruction_pointer += 1;
            }
            Instruction::Ceil => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Ceil".to_string()))?;
                let result = match value {
                    Value::Float(f) => f.ceil() as i64,
                    Value::Integer(n) => n,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'ceil' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Integer(result));
                self.instruction_pointer += 1;
            }
            Instruction::Abs => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Abs".to_string()))?;
                match value {
                    Value::Integer(n) => {
                        self.value_stack.push(Value::Integer(n.abs()));
                    }
                    Value::Float(f) => {
                        self.value_stack.push(Value::Float(f.abs()));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'abs' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Pow => {
                let exponent = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Pow".to_string()))?;
                let base = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Pow".to_string()))?;
                let base_f = match base {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'pow' expects numbers, got {} and {}",
                            Self::type_name(&base),
                            Self::type_name(&exponent)
                        )));
                    }
                };
                let exp_f = match exponent {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'pow' expects numbers, got {} and {}",
                            Self::type_name(&base),
                            Self::type_name(&exponent)
                        )));
                    }
                };
                self.value_stack.push(Value::Float(base_f.powf(exp_f)));
                self.instruction_pointer += 1;
            }
            Instruction::Log => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Log".to_string()))?;
                let f = match value {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'log' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                if f <= 0.0 {
                    return Err(RuntimeError::new(format!(
                        "Math error: 'log' expects positive number, got {}",
                        f
                    )));
                }
                self.value_stack.push(Value::Float(f.ln()));
                self.instruction_pointer += 1;
            }
            Instruction::Exp => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Exp".to_string()))?;
                let f = match value {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'exp' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Float(f.exp()));
                self.instruction_pointer += 1;
            }
            Instruction::Tan => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Tan".to_string()))?;
                let f = match value {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'tan' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Float(f.tan()));
                self.instruction_pointer += 1;
            }
            Instruction::Atan => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Atan".to_string()))?;
                let f = match value {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'atan' expects a number, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Float(f.atan()));
                self.instruction_pointer += 1;
            }
            Instruction::Atan2 => {
                let x = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Atan2".to_string()))?;
                let y = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Atan2".to_string()))?;
                let y_f = match y {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'atan2' expects numbers, got {} and {}",
                            Self::type_name(&y),
                            Self::type_name(&x)
                        )));
                    }
                };
                let x_f = match x {
                    Value::Float(f) => f,
                    Value::Integer(n) => n as f64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'atan2' expects numbers, got {} and {}",
                            Self::type_name(&y),
                            Self::type_name(&x)
                        )));
                    }
                };
                self.value_stack.push(Value::Float(y_f.atan2(x_f)));
                self.instruction_pointer += 1;
            }
            Instruction::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                self.value_stack.push(Value::Float(rng.gen::<f64>()));
                self.instruction_pointer += 1;
            }
            Instruction::RandomInt => {
                use rand::Rng;
                let max = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in RandomInt".to_string()))?;
                let max_i = match max {
                    Value::Integer(n) => {
                        if n <= 0 {
                            return Err(RuntimeError::new(format!(
                                "Argument error: 'random-int' expects positive integer, got {}",
                                n
                            )));
                        }
                        n
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'random-int' expects integer, got {}",
                            Self::type_name(&max)
                        )));
                    }
                };
                let mut rng = rand::thread_rng();
                self.value_stack.push(Value::Integer(rng.gen_range(0..max_i)));
                self.instruction_pointer += 1;
            }
            Instruction::SeedRandom => {
                let seed_val = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in SeedRandom".to_string()))?;
                let seed = match seed_val {
                    Value::Integer(n) => n as u64,
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'seed-random' expects integer, got {}",
                            Self::type_name(&seed_val)
                        )));
                    }
                };
                // Note: This sets the seed for the thread_rng, but Rust's thread_rng doesn't support seeding.
                // We'll just return the seed value as confirmation. For true seeded randomness,
                // users would need to use a dedicated RNG stored in VM state.
                // For now, this is a placeholder implementation.
                self.value_stack.push(Value::Integer(seed as i64));
                self.instruction_pointer += 1;
            }
            Instruction::CurrentTimestamp => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| RuntimeError::new(format!("System time error: {}", e)))?;
                self.value_stack.push(Value::Integer(now.as_secs() as i64));
                self.instruction_pointer += 1;
            }
            Instruction::FormatTimestamp => {
                let format = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in FormatTimestamp".to_string()))?;
                let timestamp = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in FormatTimestamp".to_string()))?;
                match (&timestamp, &format) {
                    (Value::Integer(ts), Value::String(fmt)) => {
                        use chrono::DateTime;
                        let datetime = DateTime::from_timestamp(*ts, 0)
                            .ok_or_else(|| RuntimeError::new(format!("Invalid timestamp: {}", ts)))?;
                        let formatted = datetime.format(fmt).to_string();
                        self.value_stack.push(Value::String(Arc::new(formatted)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'format-timestamp' expects integer and string, got {} and {}",
                            Self::type_name(&timestamp),
                            Self::type_name(&format)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
            Instruction::Eval => {
                let code = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Eval".to_string()))?;
                match code {
                    Value::String(source) => {
                        // Parse the code
                        let mut parser = Parser::new(&source);
                        let exprs = parser.parse_all().map_err(|e| {
                            RuntimeError::new(format!("'eval' failed to parse code: {}", e))
                        })?;

                        // Compile the code with runtime context
                        // This allows eval'd code to reference functions and globals from parent context
                        let mut compiler = Compiler::new();
                        compiler.with_known_functions(self.functions.keys());
                        compiler.with_known_globals(self.global_vars.keys());
                        let (functions, main) = compiler.compile_program(&exprs).map_err(|e| {
                            RuntimeError::new(format!("'eval' failed to compile code: {}", e.message))
                        })?;

                        // Merge compiled functions into VM's function table
                        self.functions.extend(functions);

                        // Execute the compiled code
                        // Save current state
                        let saved_bytecode = std::mem::replace(&mut self.current_bytecode, main);
                        let saved_ip = self.instruction_pointer;

                        // Execute the eval'd code
                        self.instruction_pointer = 0;
                        while !self.halted && self.instruction_pointer < self.current_bytecode.len() {
                            self.execute_one_instruction()?;
                        }

                        // Restore previous state
                        self.current_bytecode = saved_bytecode;
                        self.instruction_pointer = saved_ip;
                        self.halted = false;

                        // The result is already on the stack from the eval'd code
                        // If nothing was pushed, push nil (empty list)
                        if self.value_stack.is_empty() {
                            self.value_stack.push(Value::List(List::Nil));
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'eval' expects a string, got {}",
                            Self::type_name(&code)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }

            // Reflection - Function Introspection
            Instruction::FunctionArity => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in FunctionArity".to_string()))?;
                let arity = match &value {
                    Value::Function(name) => {
                        // For built-in or user-defined functions, we need to inspect the bytecode
                        if let Some(bytecode) = self.functions.get(name.as_str()) {
                            // Look for PackRestArgs (variadic), CheckArity (explicit), or count LoadArg
                            let mut max_arg_index: Option<usize> = None;
                            let mut is_variadic = false;
                            let mut has_check_arity = false;

                            for instr in bytecode {
                                match instr {
                                    Instruction::CheckArity(n, _) => {
                                        // Explicit arity check - this is the arity
                                        max_arg_index = Some(*n);
                                        has_check_arity = true;
                                        break;
                                    }
                                    Instruction::PackRestArgs(n) => {
                                        // Variadic function - n is the number of required params
                                        is_variadic = true;
                                        max_arg_index = Some(*n);
                                        break;
                                    }
                                    Instruction::LoadArg(idx) => {
                                        // Track the highest argument index
                                        max_arg_index = Some(max_arg_index.map_or(*idx, |m| m.max(*idx)));
                                    }
                                    _ => {}
                                }
                            }

                            if is_variadic {
                                -1 // -1 indicates variadic
                            } else if let Some(idx) = max_arg_index {
                                if has_check_arity {
                                    idx as i64 // CheckArity already has the arity count
                                } else {
                                    (idx + 1) as i64 // LoadArg index -> arity is index + 1
                                }
                            } else {
                                0 // No arguments
                            }
                        } else {
                            return Err(RuntimeError::new(format!("Unknown function: {}", name)));
                        }
                    }
                    Value::Closure(closure_data) => {
                        if closure_data.rest_param.is_some() {
                            -1 // Variadic closure
                        } else {
                            closure_data.params.len() as i64
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'function-arity' expects a function or closure, got {}",
                            Self::type_name(&value)
                        )));
                    }
                };
                self.value_stack.push(Value::Integer(arity));
                self.instruction_pointer += 1;
            }

            Instruction::FunctionParams => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in FunctionParams".to_string()))?;
                match value {
                    Value::Closure(closure_data) => {
                        let mut param_values: Vec<Value> = closure_data.params.iter()
                            .map(|p| Value::String(Arc::new(p.clone())))
                            .collect();

                        // If there's a rest parameter, add it with a dotted notation indicator
                        if let Some(ref rest) = closure_data.rest_param {
                            param_values.push(Value::String(Arc::new(format!(". {}", rest))));
                        }

                        self.value_stack.push(Value::List(List::from_vec(param_values)));
                    }
                    Value::Function(name) => {
                        // For named functions, we can't easily extract parameter names from bytecode
                        // Return empty list or error
                        return Err(RuntimeError::new(format!(
                            "Type error: 'function-params' only works with closures, not named functions. Got function '{}'",
                            name
                        )));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'function-params' expects a closure, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }

            Instruction::ClosureCaptured => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ClosureCaptured".to_string()))?;
                match value {
                    Value::Closure(closure_data) => {
                        // Return list of (name, value) pairs
                        let captured_pairs: Vec<Value> = closure_data.captured.iter()
                            .map(|(name, val)| {
                                Value::List(List::from_vec(vec![
                                    Value::String(Arc::new(name.clone())),
                                    val.clone()
                                ]))
                            })
                            .collect();
                        self.value_stack.push(Value::List(List::from_vec(captured_pairs)));
                    }
                    Value::Function(_) => {
                        // Named functions don't have captured variables
                        self.value_stack.push(Value::List(List::Nil));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'closure-captured' expects a function or closure, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }

            Instruction::FunctionName => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in FunctionName".to_string()))?;
                match value {
                    Value::Function(name) => {
                        self.value_stack.push(Value::String(name));
                    }
                    Value::Closure { .. } => {
                        return Err(RuntimeError::new(
                            "Type error: 'function-name' expects a named function, not a closure".to_string()
                        ));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'function-name' expects a function, got {}",
                            Self::type_name(&value)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }

            Instruction::TypeOf => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in TypeOf".to_string()))?;
                let type_symbol = match value {
                    Value::Integer(_) => "integer",
                    Value::Float(_) => "float",
                    Value::Boolean(_) => "boolean",
                    Value::List(_) => "list",
                    Value::Symbol(_) => "symbol",
                    Value::String(_) => "string",
                    Value::Function(_) => "function",
                    Value::Closure(_) => "closure",
                    Value::HashMap(_) => "hashmap",
                    Value::Vector(_) => "vector",
                };
                self.value_stack.push(Value::Symbol(Arc::new(type_symbol.to_string())));
                self.instruction_pointer += 1;
            }

            Instruction::GenSym => {
                use std::sync::atomic::{AtomicUsize, Ordering};
                static GENSYM_COUNTER: AtomicUsize = AtomicUsize::new(0);
                let counter = GENSYM_COUNTER.fetch_add(1, Ordering::SeqCst);
                let sym = format!("G__{}", counter);
                self.value_stack.push(Value::Symbol(Arc::new(sym)));
                self.instruction_pointer += 1;
            }

            // ============================================================
            // Parallel Collections (Phase 12a)
            // ============================================================

            Instruction::PMap => {
                use rayon::prelude::*;

                let list = self.value_stack.pop()
                    .ok_or_else(|| RuntimeError::new("Stack underflow in PMap".to_string()))?;
                let function = self.value_stack.pop()
                    .ok_or_else(|| RuntimeError::new("Stack underflow in PMap".to_string()))?;

                match list {
                    Value::List(items) => {
                        // Collect items into a vector for parallel processing
                        let vec: Vec<Value> = items.iter().cloned().collect();

                        // Prepare function data for parallel execution
                        let (func_bytecode, func_params, func_rest, func_captured) = match &function {
                            Value::Closure(closure_data) => {
                                (closure_data.body.clone(),
                                 closure_data.params.clone(),
                                 closure_data.rest_param.clone(),
                                 closure_data.captured.clone())
                            }
                            Value::Function(name) => {
                                let bytecode = self.functions.get(name.as_str())
                                    .ok_or_else(|| RuntimeError::new(format!("Undefined function: {}", name)))?
                                    .clone();
                                (bytecode, vec!["x".to_string()], None, vec![])
                            }
                            _ => {
                                return Err(RuntimeError::new(format!(
                                    "Type error: pmap expects function or closure, got {}",
                                    Self::type_name(&function)
                                )));
                            }
                        };

                        // Clone the full function table for parallel execution
                        let functions = self.functions.clone();

                        // Parallel map operation
                        let results: Result<Vec<Value>, RuntimeError> = vec.par_iter()
                            .map(|item| {
                                // Create a mini-VM for this thread
                                let mut thread_vm = VM::new();
                                thread_vm.functions = functions.clone();

                                // Execute the function with this item
                                thread_vm.execute_closure_call(
                                    &func_bytecode,
                                    &func_params,
                                    &func_rest,
                                    &func_captured,
                                    &[item.clone()]
                                )
                            })
                            .collect();

                        let result_vec = results?;
                        self.value_stack.push(Value::List(List::from_vec(result_vec)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: pmap expects list, got {}",
                            Self::type_name(&list)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }

            Instruction::PFilter => {
                use rayon::prelude::*;

                let list = self.value_stack.pop()
                    .ok_or_else(|| RuntimeError::new("Stack underflow in PFilter".to_string()))?;
                let predicate = self.value_stack.pop()
                    .ok_or_else(|| RuntimeError::new("Stack underflow in PFilter".to_string()))?;

                match list {
                    Value::List(items) => {
                        let vec: Vec<Value> = items.iter().cloned().collect();

                        let (func_bytecode, func_params, func_rest, func_captured) = match &predicate {
                            Value::Closure(closure_data) => {
                                (closure_data.body.clone(),
                                 closure_data.params.clone(),
                                 closure_data.rest_param.clone(),
                                 closure_data.captured.clone())
                            }
                            Value::Function(name) => {
                                let bytecode = self.functions.get(name.as_str())
                                    .ok_or_else(|| RuntimeError::new(format!("Undefined function: {}", name)))?
                                    .clone();
                                (bytecode, vec!["x".to_string()], None, vec![])
                            }
                            _ => {
                                return Err(RuntimeError::new(format!(
                                    "Type error: pfilter expects function or closure, got {}",
                                    Self::type_name(&predicate)
                                )));
                            }
                        };

                        let functions = self.functions.clone();

                        // Parallel filter operation
                        let results: Result<Vec<(Value, bool)>, RuntimeError> = vec.par_iter()
                            .map(|item| {
                                let mut thread_vm = VM::new();
                                thread_vm.functions = functions.clone();

                                let result = thread_vm.execute_closure_call(
                                    &func_bytecode,
                                    &func_params,
                                    &func_rest,
                                    &func_captured,
                                    &[item.clone()]
                                )?;

                                Ok((item.clone(), matches!(result, Value::Boolean(true))))
                            })
                            .collect();

                        let filtered: Vec<Value> = results?
                            .into_iter()
                            .filter_map(|(item, keep)| if keep { Some(item) } else { None })
                            .collect();

                        self.value_stack.push(Value::List(List::from_vec(filtered)));
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: pfilter expects list, got {}",
                            Self::type_name(&list)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }

            Instruction::PReduce => {
                let function = self.value_stack.pop()
                    .ok_or_else(|| RuntimeError::new("Stack underflow in PReduce".to_string()))?;
                let initial = self.value_stack.pop()
                    .ok_or_else(|| RuntimeError::new("Stack underflow in PReduce".to_string()))?;
                let list = self.value_stack.pop()
                    .ok_or_else(|| RuntimeError::new("Stack underflow in PReduce".to_string()))?;

                match list {
                    Value::List(items) => {
                        let vec: Vec<Value> = items.iter().cloned().collect();

                        // Handle empty list: just return the initial value
                        if vec.is_empty() {
                            self.value_stack.push(initial);
                        } else {
                            let (func_bytecode, func_params, func_rest, func_captured) = match &function {
                                Value::Closure(closure_data) => {
                                    (closure_data.body.clone(),
                                     closure_data.params.clone(),
                                     closure_data.rest_param.clone(),
                                     closure_data.captured.clone())
                                }
                                Value::Function(name) => {
                                    let bytecode = self.functions.get(name.as_str())
                                        .ok_or_else(|| RuntimeError::new(format!("Undefined function: {}", name)))?
                                        .clone();
                                    (bytecode, vec!["acc".to_string(), "x".to_string()], None, vec![])
                                }
                                _ => {
                                    return Err(RuntimeError::new(format!(
                                        "Type error: preduce expects function or closure, got {}",
                                        Self::type_name(&function)
                                    )));
                                }
                            };

                            let functions = self.functions.clone();

                            // Simple sequential reduce after collecting items (can be optimized later)
                            let mut accumulator = initial.clone();
                            for item in vec.iter() {
                                let mut thread_vm = VM::new();
                                thread_vm.functions = functions.clone();

                                accumulator = thread_vm.execute_closure_call(
                                    &func_bytecode,
                                    &func_params,
                                    &func_rest,
                                    &func_captured,
                                    &[accumulator, item.clone()]
                                )?;
                            }

                            self.value_stack.push(accumulator);
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: preduce expects list, got {}",
                            Self::type_name(&list)
                        )));
                    }
                }
                self.instruction_pointer += 1;
            }
        }

        Ok(())
    }

    fn type_name(value: &Value) -> &str {
        match value {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Boolean(_) => "boolean",
            Value::List(_) => "list",
            Value::Symbol(_) => "symbol",
            Value::String(_) => "string",
            Value::Function(_) => "function",
            Value::Closure { .. } => "closure",
            Value::HashMap(_) => "hashmap",
            Value::Vector(_) => "vector",
        }
    }

    fn format_value(value: &Value) -> String {
        match value {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => {
                // Format float nicely - show decimal point even for whole numbers
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{}.0", f)
                } else {
                    f.to_string()
                }
            }
            Value::Boolean(b) => b.to_string(),
            Value::List(list) => {
                let formatted_items: Vec<String> = list
                    .iter()
                    .map(|v| Self::format_value(v))
                    .collect();
                format!("({})", formatted_items.join(" "))
            }
            Value::Symbol(s) => s.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Function(name) => format!("<function {}>", name),
            Value::Closure(closure_data) => {
                format!("<closure/{}>", closure_data.params.len())
            }
            Value::HashMap(map) => {
                let mut items: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{} {}", Self::format_value(&Value::String(Arc::new(k.clone()))), Self::format_value(v)))
                    .collect();
                items.sort(); // Sort for consistent output
                format!("{{{}}}", items.join(" "))
            }
            Value::Vector(items) => {
                let formatted_items: Vec<String> = items
                    .iter()
                    .map(|v| Self::format_value(v))
                    .collect();
                format!("[{}]", formatted_items.join(" "))
            }
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        while !self.halted {
            self.execute_one_instruction()?;
        }
        Ok(())
    }

    /// Execute a closure call in isolation (used for parallel operations)
    /// Returns the result value
    fn execute_closure_call(
        &mut self,
        bytecode: &[Instruction],
        params: &[String],
        rest_param: &Option<String>,
        _captured: &[(String, Value)],  // TODO: Handle captured variables properly
        args: &[Value]
    ) -> Result<Value, RuntimeError> {
        // Validate argument count
        let required_args = params.len();
        let has_rest = rest_param.is_some();

        if has_rest {
            if args.len() < required_args {
                return Err(RuntimeError::new(format!(
                    "Wrong number of arguments: expected at least {}, got {}",
                    required_args, args.len()
                )));
            }
        } else if args.len() != required_args {
            return Err(RuntimeError::new(format!(
                "Wrong number of arguments: expected {}, got {}",
                required_args, args.len()
            )));
        }

        // Set up execution environment
        self.current_bytecode = bytecode.to_vec();
        self.instruction_pointer = 0;
        self.halted = false;

        // Create a call frame with the arguments as locals
        let frame = Frame {
            return_address: 0,  // Not used for parallel calls
            locals: args.to_vec(),
            return_bytecode: Vec::new(),  // Not used for parallel calls
            function_name: "<parallel>".to_string(),
            captured: Vec::new(),  // TODO: Pass captured variables
            stack_base: self.value_stack.len(),
            loop_start: None,
            loop_bindings_start: None,
            loop_bindings_count: None,
        };
        self.call_stack.push(frame);

        // Execute the bytecode
        while !self.halted && self.instruction_pointer < self.current_bytecode.len() {
            self.execute_one_instruction()?;
        }

        // Pop the call frame
        self.call_stack.pop();

        // Return the result (should be on top of stack)
        self.value_stack.pop()
            .ok_or_else(|| RuntimeError::new("No return value from closure call".to_string()))
    }

    pub fn get_stack_trace(&self) -> Vec<String> {
        self.call_stack
            .iter()
            .map(|frame| frame.function_name.clone())
            .collect()
    }
}

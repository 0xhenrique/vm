use std::collections::HashMap;

use super::value::Value;
use super::instructions::Instruction;
use super::stack::Frame;
use super::errors::RuntimeError;

pub struct VM {
    pub instruction_pointer: usize,
    pub value_stack: Vec<Value>,
    pub call_stack: Vec<Frame>,
    pub functions: HashMap<String, Vec<Instruction>>,
    pub current_bytecode: Vec<Instruction>,
    pub halted: bool,
    pub global_vars: HashMap<String, Value>, // Global variables
    pub args: Vec<String>, // Command-line arguments
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
        };
        vm.register_builtins();
        vm
    }

    fn register_builtins(&mut self) {
        use Instruction::*;

        // Arithmetic operations
        self.functions.insert("+".to_string(), vec![Add, Ret]);
        self.functions.insert("-".to_string(), vec![Sub, Ret]);
        self.functions.insert("*".to_string(), vec![Mul, Ret]);
        self.functions.insert("/".to_string(), vec![Div, Ret]);
        self.functions.insert("%".to_string(), vec![Mod, Ret]);
        self.functions.insert("neg".to_string(), vec![Neg, Ret]);

        // Comparison operations
        self.functions.insert("<=".to_string(), vec![Leq, Ret]);
        self.functions.insert("<".to_string(), vec![Lt, Ret]);
        self.functions.insert(">".to_string(), vec![Gt, Ret]);
        self.functions.insert(">=".to_string(), vec![Gte, Ret]);
        self.functions.insert("==".to_string(), vec![Eq, Ret]);
        self.functions.insert("!=".to_string(), vec![Neq, Ret]);

        // List operations
        self.functions.insert("cons".to_string(), vec![Cons, Ret]);
        self.functions.insert("car".to_string(), vec![Car, Ret]);
        self.functions.insert("cdr".to_string(), vec![Cdr, Ret]);
        self.functions.insert("list?".to_string(), vec![IsList, Ret]);
        self.functions.insert("append".to_string(), vec![Append, Ret]);
        self.functions.insert("list-ref".to_string(), vec![ListRef, Ret]);
        self.functions.insert("list-length".to_string(), vec![ListLength, Ret]);

        // String operations
        self.functions.insert("string?".to_string(), vec![IsString, Ret]);
        self.functions.insert("symbol?".to_string(), vec![IsSymbol, Ret]);
        self.functions.insert("symbol->string".to_string(), vec![SymbolToString, Ret]);
        self.functions.insert("string->symbol".to_string(), vec![StringToSymbol, Ret]);
        self.functions.insert("string-length".to_string(), vec![StringLength, Ret]);
        self.functions.insert("substring".to_string(), vec![Substring, Ret]);
        self.functions.insert("string-append".to_string(), vec![StringAppend, Ret]);
        self.functions.insert("string->list".to_string(), vec![StringToList, Ret]);
        self.functions.insert("list->string".to_string(), vec![ListToString, Ret]);
        self.functions.insert("char-code".to_string(), vec![CharCode, Ret]);
        self.functions.insert("number->string".to_string(), vec![NumberToString, Ret]);

        // File I/O operations
        self.functions.insert("read-file".to_string(), vec![ReadFile, Ret]);
        self.functions.insert("write-file".to_string(), vec![WriteFile, Ret]);
        self.functions.insert("file-exists?".to_string(), vec![FileExists, Ret]);
        self.functions.insert("write-binary-file".to_string(), vec![WriteBinaryFile, Ret]);

        // Other operations
        self.functions.insert("get-args".to_string(), vec![GetArgs, Ret]);
        self.functions.insert("print".to_string(), vec![Print, Ret]);
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '+' expects two integers, got {} and {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '-' expects two integers, got {} and {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '*' expects two integers, got {} and {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '/' expects two integers, got {} and {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '%' expects two integers, got {} and {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'neg' expects an integer, got {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '<=' expects two integers, got {} and {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '<' expects two integers, got {} and {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '>' expects two integers, got {} and {}",
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
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: '>=' expects two integers, got {} and {}",
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
                // Use PartialEq to compare all value types
                self.value_stack.push(Value::Boolean(a == b));
                self.instruction_pointer += 1;
            }
            Instruction::Neq => {
                let b = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Neq operation".to_string()))?;
                let a = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Neq operation".to_string()))?;
                // Use PartialEq to compare all value types
                self.value_stack.push(Value::Boolean(a != b));
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

                let closure = Value::Closure {
                    params: params.clone(),
                    body: body.clone(),
                    captured,
                };

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

                // Pop the closure
                let closure = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in CallClosure".to_string()))?;

                match closure {
                    Value::Closure { params, body, captured } => {
                        // Verify arity
                        if params.len() != args.len() {
                            return Err(RuntimeError::new(format!(
                                "Closure arity mismatch: expected {} argument(s), got {}",
                                params.len(),
                                args.len()
                            )));
                        }

                        // Create frame with arguments and captured environment
                        let frame = Frame {
                            return_address: self.instruction_pointer + 1,
                            locals: args,
                            return_bytecode: self.current_bytecode.to_vec(),
                            function_name: "<closure>".to_string(),
                            captured: captured.iter().map(|(_, v)| v.clone()).collect(),
                            stack_base: self.value_stack.len(), // Current stack top is base for this function
                        };

                        self.call_stack.push(frame);

                        // Switch to closure body bytecode
                        self.current_bytecode = body;
                        self.instruction_pointer = 0;
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: expected closure, got {}",
                            Self::type_name(&closure)
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
                let mut new_list = vec![first];
                match second {
                    Value::List(mut items) => {
                        new_list.append(&mut items);
                    }
                    other => {
                        new_list.push(other);
                    }
                }
                self.value_stack.push(Value::List(new_list));
                self.instruction_pointer += 1;
            }
            Instruction::Car => {
                let value = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in Car".to_string()))?;
                match value {
                    Value::List(items) => {
                        if items.is_empty() {
                            return Err(RuntimeError::new("'car' cannot take the first element of an empty list".to_string()));
                        }
                        self.value_stack.push(items[0].clone());
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
                    Value::List(items) => {
                        if items.is_empty() {
                            return Err(RuntimeError::new("'cdr' cannot take the rest of an empty list".to_string()));
                        }
                        let rest = items[1..].to_vec();
                        self.value_stack.push(Value::List(rest));
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
                    (Value::List(first_items), Value::List(second_items)) => {
                        let mut result = first_items.clone();
                        result.extend(second_items.clone());
                        self.value_stack.push(Value::List(result));
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
                self.value_stack.push(Value::List(items));
                self.instruction_pointer += 1;
            }
            Instruction::ListRef => {
                // Pop index and list, push element at that index
                let index = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ListRef".to_string()))?;
                let list = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ListRef".to_string()))?;

                match (&list, &index) {
                    (Value::List(items), Value::Integer(idx)) => {
                        if *idx < 0 {
                            return Err(RuntimeError::new(format!("'list-ref' index cannot be negative: {}", idx)));
                        }
                        let idx_usize = *idx as usize;
                        if idx_usize >= items.len() {
                            return Err(RuntimeError::new(format!(
                                "'list-ref' index {} out of bounds for list of length {}",
                                idx, items.len()
                            )));
                        }
                        self.value_stack.push(items[idx_usize].clone());
                    }
                    _ => {
                        return Err(RuntimeError::new(format!(
                            "Type error: 'list-ref' expects a list and an integer, got {} and {}",
                            Self::type_name(&list),
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
                        self.value_stack.push(Value::String(n.to_string()));
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
                            self.value_stack.push(Value::String(result));
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
                        self.value_stack.push(Value::String(format!("{}{}", s1, s2)));
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
                            .map(|c| Value::String(c.to_string()))
                            .collect();
                        self.value_stack.push(Value::List(char_list));
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
                    Value::List(items) => {
                        let mut result = String::new();
                        for item in &items {
                            match item {
                                Value::String(s) => result.push_str(s),
                                _ => {
                                    return Err(RuntimeError::new(format!(
                                        "Type error: 'list->string' expects a list of strings, but found {}",
                                        Self::type_name(item)
                                    )));
                                }
                            }
                        }
                        self.value_stack.push(Value::String(result));
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
            Instruction::ReadFile => {
                let path = self.value_stack.pop().ok_or_else(|| RuntimeError::new("Stack underflow in ReadFile".to_string()))?;
                match path {
                    Value::String(path_str) => {
                        match std::fs::read_to_string(&path_str) {
                            Ok(contents) => {
                                self.value_stack.push(Value::String(contents));
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
                        match std::fs::write(path_str, content_str) {
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
                        let exists = std::path::Path::new(&path_str).exists();
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
                        for byte_val in bytes {
                            match byte_val {
                                Value::Integer(n) if *n >= 0 && *n <= 255 => {
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
                        match std::fs::write(path_str, &byte_vec) {
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
            Instruction::GetArgs => {
                // Convert Vec<String> to Value::List of Value::String
                let args_list = self.args.iter()
                    .map(|s| Value::String(s.clone()))
                    .collect();
                self.value_stack.push(Value::List(args_list));
                self.instruction_pointer += 1;
            }
        }

        Ok(())
    }

    fn type_name(value: &Value) -> &str {
        match value {
            Value::Integer(_) => "integer",
            Value::Boolean(_) => "boolean",
            Value::List(_) => "list",
            Value::Symbol(_) => "symbol",
            Value::String(_) => "string",
            Value::Closure { .. } => "closure",
        }
    }

    fn format_value(value: &Value) -> String {
        match value {
            Value::Integer(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::List(items) => {
                let formatted_items: Vec<String> = items
                    .iter()
                    .map(|v| Self::format_value(v))
                    .collect();
                format!("({})", formatted_items.join(" "))
            }
            Value::Symbol(s) => s.clone(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Closure { params, .. } => {
                format!("<closure/{}>", params.len())
            }
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        while !self.halted {
            self.execute_one_instruction()?;
        }
        Ok(())
    }

    pub fn get_stack_trace(&self) -> Vec<String> {
        self.call_stack
            .iter()
            .map(|frame| frame.function_name.clone())
            .collect()
    }
}

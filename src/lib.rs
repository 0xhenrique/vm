use std::collections::HashMap;

pub mod parser;
pub mod bytecode;
pub mod disassembler;
pub mod repl;
pub mod optimizer;


#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    List(Vec<Value>),
    Symbol(String),
    String(String),
    Closure {
        params: Vec<String>,
        body: Vec<Instruction>,
        captured: Vec<(String, Value)>, // Captured environment as ordered pairs
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Push(Value),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Leq,
    Lt,
    Gt,
    Gte,
    Eq,
    Neq,
    JmpIfFalse(usize),
    Jmp(usize),
    Call(String, usize),
    TailCall(String, usize), // Tail call: reuse current frame instead of pushing new one
    Ret,
    LoadArg(usize),
    GetLocal(usize), // Load from value stack at position (from bottom)
    PopN(usize),     // Pop N values from the stack
    Slide(usize),    // Pop top value, pop N values, push top value back (cleanup let bindings)
    CheckArity(usize, usize), // Check if frame.locals.len() == expected_arity, jump to addr if not
    MakeClosure(Vec<String>, Vec<Instruction>, usize), // Create closure: (params, body, num_captured_vars)
    CallClosure(usize), // Call closure with N arguments (pops closure + args from stack)
    LoadCaptured(usize), // Load captured variable at index from current closure's environment
    Print,
    Halt,
    // List operations
    Cons,    // Pop two values, push cons cell (list)
    Car,     // Pop list, push first element
    Cdr,     // Pop list, push rest of list
    IsList,  // Pop value, push boolean indicating if it's a list
    // String/Symbol operations
    IsString,       // Pop value, push boolean indicating if it's a string
    IsSymbol,       // Pop value, push boolean indicating if it's a symbol
    SymbolToString, // Pop symbol, push string
    StringToSymbol, // Pop string, push symbol
    // List manipulation
    Append,         // Pop two lists, push their concatenation (second appended to first)
    MakeList(usize), // Pop N values from stack and create a list from them (in order)
    // Global variables
    LoadGlobal(String),  // Push value of global variable onto stack
    StoreGlobal(String), // Pop value from stack and store in global variable
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub file: String,
}

impl Location {
    pub fn new(line: usize, column: usize, file: String) -> Self {
        Location { line, column, file }
    }

    pub fn unknown() -> Self {
        Location {
            line: 0,
            column: 0,
            file: "<unknown>".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LispExpr {
    Number(i64),
    Boolean(bool),
    Symbol(String),
    List(Vec<SourceExpr>),
    DottedList(Vec<SourceExpr>, Box<SourceExpr>), // (a b . rest) - for cons patterns
}

// Wrapper that includes source location
#[derive(Debug, Clone, PartialEq)]
pub struct SourceExpr {
    pub expr: LispExpr,
    pub location: Location,
}

impl SourceExpr {
    pub fn new(expr: LispExpr, location: Location) -> Self {
        SourceExpr { expr, location }
    }

    pub fn unknown(expr: LispExpr) -> Self {
        SourceExpr {
            expr,
            location: Location::unknown(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub location: Location,
}

impl CompileError {
    pub fn new(message: String, location: Location) -> Self {
        CompileError { message, location }
    }

    pub fn format(&self, source_line: Option<&str>) -> String {
        let mut output = format!(
            "Compile error at {}:{}:{}\n  {}\n",
            self.location.file, self.location.line, self.location.column, self.message
        );

        if let Some(line) = source_line {
            output.push_str(&format!("\n  | {}\n", line));
            output.push_str(&format!("  | {}^\n", " ".repeat(self.location.column.saturating_sub(1))));
        }

        output
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub call_stack: Vec<String>,
}

impl RuntimeError {
    pub fn new(message: String) -> Self {
        RuntimeError {
            message,
            call_stack: Vec::new(),
        }
    }

    pub fn with_stack(message: String, call_stack: Vec<String>) -> Self {
        RuntimeError {
            message,
            call_stack,
        }
    }

    pub fn format(&self) -> String {
        let mut output = format!("Runtime error: {}\n", self.message);

        if !self.call_stack.is_empty() {
            output.push_str("\nCall stack:\n");
            for (i, frame) in self.call_stack.iter().rev().enumerate() {
                output.push_str(&format!("  #{}: {}\n", i, frame));
            }
        }

        output
    }
}

#[allow(dead_code)]
fn number(n: i64) -> SourceExpr {
    SourceExpr::unknown(LispExpr::Number(n))
}

#[allow(dead_code)]
fn boolean(b: bool) -> SourceExpr {
    SourceExpr::unknown(LispExpr::Boolean(b))
}

#[allow(dead_code)]
fn symbol(s: &str) -> SourceExpr {
    SourceExpr::unknown(LispExpr::Symbol(s.to_string()))
}

#[allow(dead_code)]
fn list(items: Vec<SourceExpr>) -> SourceExpr {
    SourceExpr::unknown(LispExpr::List(items))
}

#[derive(Debug)]
pub struct Frame {
    pub return_address: usize,
    pub locals: Vec<Value>,
    pub return_bytecode: Vec<Instruction>, // Bytecode to return to after function call
    pub function_name: String, // For stack traces
    pub captured: Vec<Value>, // Captured variables for closures
}

pub struct VM {
    pub instruction_pointer: usize,
    pub value_stack: Vec<Value>,
    pub call_stack: Vec<Frame>,
    pub functions: HashMap<String, Vec<Instruction>>,
    pub current_bytecode: Vec<Instruction>,
    pub halted: bool,
    pub global_vars: HashMap<String, Value>, // Global variables
}

impl VM {

    pub fn new() -> Self {
        VM {
            instruction_pointer: 0,
            value_stack: Vec::new(),
            call_stack: Vec::new(),
            functions: HashMap::new(),
            current_bytecode: Vec::new(),
            halted: false,
            global_vars: HashMap::new(),
        }
    }

    pub fn execute_one_instruction(&mut self) {
        if self.instruction_pointer >= self.current_bytecode.len() {
            self.halted = true;
            return;
        }

        let instruction = self.current_bytecode[self.instruction_pointer].clone();

        match instruction {
            Instruction::Push(value) => {
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::Add => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Integer(x + y));
                    }
                    _ => panic!("Type error: Add expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Sub => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Integer(x - y));
                    }
                    _ => panic!("Type error: Sub expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Mul => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Integer(x * y));
                    }
                    _ => panic!("Type error: Mul expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Div => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if y == 0 {
                            panic!("Division by zero");
                        }
                        self.value_stack.push(Value::Integer(x / y));
                    }
                    _ => panic!("Type error: Div expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Mod => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if y == 0 {
                            panic!("Modulo by zero");
                        }
                        self.value_stack.push(Value::Integer(x % y));
                    }
                    _ => panic!("Type error: Mod expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Neg => {
                let a = self.value_stack.pop().expect("Stack underflow");
                match a {
                    Value::Integer(x) => {
                        self.value_stack.push(Value::Integer(-x));
                    }
                    _ => panic!("Type error: Neg expects integer"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Leq => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x <= y));
                    }
                    _ => panic!("Type error: Leq expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Lt => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x < y));
                    }
                    _ => panic!("Type error: Lt expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Gt => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x > y));
                    }
                    _ => panic!("Type error: Gt expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Gte => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x >= y));
                    }
                    _ => panic!("Type error: Gte expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Eq => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                // Use PartialEq to compare all value types
                self.value_stack.push(Value::Boolean(a == b));
                self.instruction_pointer += 1;
            }
            Instruction::Neq => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                // Use PartialEq to compare all value types
                self.value_stack.push(Value::Boolean(a != b));
                self.instruction_pointer += 1;
            }
            Instruction::Jmp(addr) => {
                self.instruction_pointer = addr;
            }
            Instruction::JmpIfFalse(addr) => {
                let value = self.value_stack.pop().expect("Stack underflow");
                match value {
                    Value::Boolean(false) => {
                        self.instruction_pointer = addr;
                    }
                    Value::Boolean(true) => {
                        self.instruction_pointer += 1;
                    }
                    _ => panic!("Type error: JmpIfFalse expects boolean"),
                }
            }
            Instruction::LoadArg(idx) => {
                let frame = self.call_stack.last().expect("No frame to load arg from");
                let value = frame.locals.get(idx).expect("Arg index out of bounds").clone();
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::GetLocal(pos) => {
                // Load from value stack at position (from bottom)
                let value = self.value_stack.get(pos)
                    .expect(&format!("Stack position {} out of bounds", pos))
                    .clone();
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::PopN(n) => {
                // Pop N values from the stack
                for _ in 0..n {
                    self.value_stack.pop().expect("Stack underflow during PopN");
                }
                self.instruction_pointer += 1;
            }
            Instruction::Slide(n) => {
                // Pop the top value (result), pop N values (bindings), push result back
                let result = self.value_stack.pop().expect("Stack underflow during Slide");
                for _ in 0..n {
                    self.value_stack.pop().expect("Stack underflow during Slide");
                }
                self.value_stack.push(result);
                self.instruction_pointer += 1;
            }
            Instruction::CheckArity(expected_arity, jump_addr) => {
                // Check if current frame has the expected number of arguments
                let frame = self.call_stack.last().expect("No frame for arity check");
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
                    captured_values.push(self.value_stack.pop().expect("Stack underflow during MakeClosure"));
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
                    args.push(self.value_stack.pop().expect("Stack underflow"));
                }
                args.reverse();

                // Pop the closure
                let closure = self.value_stack.pop().expect("Stack underflow");

                match closure {
                    Value::Closure { params, body, captured } => {
                        // Verify arity
                        if params.len() != args.len() {
                            panic!("Closure arity mismatch: expected {}, got {}", params.len(), args.len());
                        }

                        // Create frame with arguments and captured environment
                        let frame = Frame {
                            return_address: self.instruction_pointer + 1,
                            locals: args,
                            return_bytecode: self.current_bytecode.to_vec(),
                            function_name: "<closure>".to_string(),
                            captured: captured.iter().map(|(_, v)| v.clone()).collect(),
                        };

                        self.call_stack.push(frame);

                        // Switch to closure body bytecode
                        self.current_bytecode = body;
                        self.instruction_pointer = 0;
                    }
                    _ => panic!("CallClosure: expected closure, got {:?}", closure),
                }
            }
            Instruction::LoadCaptured(idx) => {
                // Load a captured variable from the current closure's environment
                let frame = self.call_stack.last().expect("No frame for LoadCaptured");
                let value = frame.captured.get(idx)
                    .expect(&format!("Captured variable index {} out of bounds", idx))
                    .clone();
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::Print => {
                let value = self.value_stack.pop().expect("Stack underflow");
                println!("{}", Self::format_value(&value));
                self.instruction_pointer += 1;
            }
            Instruction::Ret => {
                let frame = self.call_stack.pop().expect("No frame to return from");
                self.current_bytecode = frame.return_bytecode;
                self.instruction_pointer = frame.return_address;
            }
            Instruction::Call(fn_name, arg_count) => {
                let fn_bytecode = self.functions.get(&fn_name)
                    .expect(&format!("Function '{}' not found", fn_name))
                    .clone();

                // Pop arguments from value stack in reverse order
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.value_stack.pop().expect("Stack underflow"));
                }
                args.reverse();

                // Create new frame with return bytecode and function name for stack traces
                let frame = Frame {
                    return_address: self.instruction_pointer + 1,
                    locals: args,
                    return_bytecode: self.current_bytecode.clone(),
                    function_name: fn_name.clone(),
                    captured: Vec::new(), // Regular functions don't have captured variables
                };
                self.call_stack.push(frame);

                // Switch to function bytecode
                self.current_bytecode = fn_bytecode;
                self.instruction_pointer = 0;
            }
            Instruction::TailCall(fn_name, arg_count) => {
                let fn_bytecode = self.functions.get(&fn_name)
                    .expect(&format!("Function '{}' not found", fn_name))
                    .clone();

                // Pop arguments from value stack in reverse order
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.value_stack.pop().expect("Stack underflow"));
                }
                args.reverse();

                // Reuse current frame instead of pushing a new one
                // This is the key to tail call optimization!
                if let Some(frame) = self.call_stack.last_mut() {
                    // Replace the locals (arguments) in the current frame
                    frame.locals = args;
                    // Update function name for stack traces
                    frame.function_name = fn_name.clone();
                    // Keep the same return address and return bytecode
                } else {
                    // No frame exists (top-level call), treat as regular call
                    let frame = Frame {
                        return_address: self.instruction_pointer + 1,
                        locals: args,
                        return_bytecode: self.current_bytecode.clone(),
                        function_name: fn_name.clone(),
                        captured: Vec::new(),
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
                let second = self.value_stack.pop().expect("Stack underflow");
                let first = self.value_stack.pop().expect("Stack underflow");

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
                let value = self.value_stack.pop().expect("Stack underflow");
                match value {
                    Value::List(items) => {
                        if items.is_empty() {
                            panic!("car: cannot take car of empty list");
                        }
                        self.value_stack.push(items[0].clone());
                    }
                    _ => panic!("car: expected list"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Cdr => {
                let value = self.value_stack.pop().expect("Stack underflow");
                match value {
                    Value::List(items) => {
                        if items.is_empty() {
                            panic!("cdr: cannot take cdr of empty list");
                        }
                        let rest = items[1..].to_vec();
                        self.value_stack.push(Value::List(rest));
                    }
                    _ => panic!("cdr: expected list"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::IsList => {
                let value = self.value_stack.pop().expect("Stack underflow");
                let is_list = matches!(value, Value::List(_));
                self.value_stack.push(Value::Boolean(is_list));
                self.instruction_pointer += 1;
            }
            Instruction::IsString => {
                let value = self.value_stack.pop().expect("Stack underflow");
                let is_string = matches!(value, Value::String(_));
                self.value_stack.push(Value::Boolean(is_string));
                self.instruction_pointer += 1;
            }
            Instruction::IsSymbol => {
                let value = self.value_stack.pop().expect("Stack underflow");
                let is_symbol = matches!(value, Value::Symbol(_));
                self.value_stack.push(Value::Boolean(is_symbol));
                self.instruction_pointer += 1;
            }
            Instruction::SymbolToString => {
                let value = self.value_stack.pop().expect("Stack underflow");
                match value {
                    Value::Symbol(s) => {
                        self.value_stack.push(Value::String(s));
                    }
                    _ => panic!("symbol->string: expected symbol"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::StringToSymbol => {
                let value = self.value_stack.pop().expect("Stack underflow");
                match value {
                    Value::String(s) => {
                        self.value_stack.push(Value::Symbol(s));
                    }
                    _ => panic!("string->symbol: expected string"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Append => {
                // Pop two lists and append them
                let second = self.value_stack.pop().expect("Stack underflow");
                let first = self.value_stack.pop().expect("Stack underflow");

                match (first, second) {
                    (Value::List(mut first_items), Value::List(second_items)) => {
                        // Append second to first
                        first_items.extend(second_items);
                        self.value_stack.push(Value::List(first_items));
                    }
                    _ => panic!("append: expected two lists"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::MakeList(n) => {
                // Pop n values and create a list
                let mut items = Vec::new();
                for _ in 0..n {
                    items.push(self.value_stack.pop().expect("Stack underflow"));
                }
                items.reverse(); // Reverse because we popped in reverse order
                self.value_stack.push(Value::List(items));
                self.instruction_pointer += 1;
            }
            Instruction::LoadGlobal(name) => {
                let value = self.global_vars.get(&name)
                    .expect(&format!("Undefined global variable: {}", name))
                    .clone();
                self.value_stack.push(value);
                self.instruction_pointer += 1;
            }
            Instruction::StoreGlobal(name) => {
                let value = self.value_stack.pop().expect("Stack underflow");
                self.global_vars.insert(name, value);
                self.instruction_pointer += 1;
            }
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

    pub fn run(&mut self) {
        while !self.halted {
            self.execute_one_instruction();
        }
    }

    pub fn get_stack_trace(&self) -> Vec<String> {
        self.call_stack
            .iter()
            .map(|frame| frame.function_name.clone())
            .collect()
    }
}

// Represents where a value is stored for pattern matching
#[derive(Debug, Clone)]
enum ValueLocation {
    Arg(usize),                                    // Direct argument
    Local(usize),                                  // Local variable on value stack
    Captured(usize),                               // Captured variable in closure
    ListElement(Box<ValueLocation>, usize),        // i-th element of a list
    ListRest(Box<ValueLocation>, usize),           // Rest after skipping n elements
}

impl ValueLocation {
    // Emit instructions to load the value at this location onto the stack
    fn emit_load(&self, compiler: &mut Compiler) {
        match self {
            ValueLocation::Arg(idx) => {
                compiler.emit(Instruction::LoadArg(*idx));
            }
            ValueLocation::Local(pos) => {
                compiler.emit(Instruction::GetLocal(*pos));
            }
            ValueLocation::Captured(idx) => {
                compiler.emit(Instruction::LoadCaptured(*idx));
            }
            ValueLocation::ListElement(list_loc, idx) => {
                // Load the list
                list_loc.emit_load(compiler);
                // Extract the i-th element using car/cdr
                for _ in 0..*idx {
                    compiler.emit(Instruction::Cdr);
                }
                compiler.emit(Instruction::Car);
            }
            ValueLocation::ListRest(list_loc, skip_count) => {
                // Load the list
                list_loc.emit_load(compiler);
                // Skip elements using cdr
                for _ in 0..*skip_count {
                    compiler.emit(Instruction::Cdr);
                }
            }
        }
    }
}

// Macro definition
#[derive(Debug, Clone)]
struct MacroDef {
    params: Vec<String>,
    body: SourceExpr,
}

pub struct Compiler {
    bytecode: Vec<Instruction>,
    functions: HashMap<String, Vec<Instruction>>,
    macros: HashMap<String, MacroDef>, // Macro definitions
    global_vars: HashMap<String, bool>, // Track global variables (value is mutable flag)
    instruction_address: usize,
    param_names: Vec<String>, // Track parameter names for LoadArg
    pattern_bindings: HashMap<String, ValueLocation>, // Track pattern match bindings
    local_bindings: HashMap<String, ValueLocation>, // Track let-bound variables
    stack_depth: usize, // Track current stack depth for let bindings
    in_tail_position: bool, // Track if current expression is in tail position (for TCO)
}

impl Compiler {

    pub fn new() -> Self {
        Compiler {
            bytecode: Vec::new(),
            functions: HashMap::new(),
            macros: HashMap::new(),
            global_vars: HashMap::new(),
            instruction_address: 0,
            param_names: Vec::new(),
            pattern_bindings: HashMap::new(),
            local_bindings: HashMap::new(),
            stack_depth: 0,
            in_tail_position: false,
        }
    }


    fn emit(&mut self, instruction: Instruction) {
        self.bytecode.push(instruction);
        self.instruction_address += 1;
    }


    // Returns the starting address of compiled bytecode
    fn compile_expr(&mut self, expr: &SourceExpr) -> Result<usize, CompileError> {
        let start_address = self.instruction_address;

        match &expr.expr {
            // Case: Number or Boolean - emit Push instruction
            LispExpr::Number(n) => {
                self.emit(Instruction::Push(Value::Integer(*n)));
            }
            LispExpr::Boolean(b) => {
                self.emit(Instruction::Push(Value::Boolean(*b)));
            }

            // Case: DottedList - only valid in patterns
            LispExpr::DottedList(_, _) => {
                return Err(CompileError::new(
                    "Dotted lists can only be used in patterns, not in expressions".to_string(),
                    expr.location.clone(),
                ));
            }

            // Case: Symbol - check if it's a parameter or string literal
            LispExpr::Symbol(s) => {
                // Check if it's a string literal (hack from parser)
                if s.starts_with("__STRING__") {
                    let string_content = s["__STRING__".len()..].to_string();
                    self.emit(Instruction::Push(Value::String(string_content)));
                } else {
                    // Check local bindings first (let bindings)
                    if let Some(location) = self.local_bindings.get(s) {
                        location.clone().emit_load(self);
                    } else if let Some(location) = self.pattern_bindings.get(s) {
                        // Check pattern bindings (for nested pattern matches)
                        location.clone().emit_load(self);
                    } else if let Some(idx) = self.param_names.iter().position(|p| p == s) {
                        // Check if this symbol is a parameter
                        self.emit(Instruction::LoadArg(idx));
                    } else if self.global_vars.contains_key(s) {
                        // Check if this is a global variable
                        self.emit(Instruction::LoadGlobal(s.clone()));
                    } else {
                        return Err(CompileError::new(
                            format!("Undefined variable '{}'", s),
                            expr.location.clone(),
                        ));
                    }
                }
            }

            // Case: List (function call or special form)
            LispExpr::List(items) => {
                if items.is_empty() {
                    return Err(CompileError::new(
                        "Empty list cannot be compiled".to_string(),
                        expr.location.clone(),
                    ));
                }

                // Check if operator is a symbol
                if let LispExpr::Symbol(operator) = &items[0].expr {
                    // Operator is a symbol - might be special form, built-in, or function call
                    match operator.as_str() {
                    // Arithmetic operators: +, -, *, /
                    "+" => {
                        if items.len() < 3 {
                            return Err(CompileError::new(
                                "+ expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Arguments to + are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Add
                        // This transforms (+ 1 2 3 4) into (+ 1 (+ 2 (+ 3 4)))
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Add);
                        }

                        // Restore tail position
                        self.in_tail_position = saved_tail;
                    }
                    "-" => {
                        if items.len() < 3 {
                            return Err(CompileError::new(
                                "- expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Arguments are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Sub
                        // This does left-associative subtraction: (- 10 2 3) = (- (- 10 2) 3) = 5
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Sub);
                        }

                        self.in_tail_position = saved_tail;
                    }
                    "*" => {
                        if items.len() < 3 {
                            return Err(CompileError::new(
                                "* expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Arguments are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Mul
                        // This transforms (* 2 3 4) into (* 2 (* 3 4)) = (* 2 12) = 24
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Mul);
                        }

                        self.in_tail_position = saved_tail;
                    }
                    "/" => {
                        if items.len() < 3 {
                            return Err(CompileError::new(
                                "/ expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Arguments are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Div
                        // This transforms (/ 20 2 2) into (/ (/ 20 2) 2) = (/ 10 2) = 5
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Div);
                        }

                        self.in_tail_position = saved_tail;
                    }
                    "%" => {
                        if items.len() < 3 {
                            return Err(CompileError::new(
                                "% expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Arguments are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Mod
                        // This transforms (% 10 3 2) into (% (% 10 3) 2) = (% 1 2) = 1
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Mod);
                        }

                        self.in_tail_position = saved_tail;
                    }
                    "neg" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "neg expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Neg);
                        self.in_tail_position = saved_tail;
                    }

                    // Comparison operators
                    "<=" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "<= expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Leq);
                        self.in_tail_position = saved_tail;
                    }
                    "<" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "< expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Lt);
                    }
                    ">" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "> expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Gt);
                    }
                    ">=" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                ">= expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Gte);
                    }
                    "==" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "== expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Eq);
                    }
                    "!=" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "!= expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Neq);
                    }

                    // Conditional: (if condition then-branch else-branch)
                    "if" => {
                        if items.len() != 4 {
                            return Err(CompileError::new(
                                "if expects exactly 3 arguments (condition, then, else)".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        // Save tail position for branches (they inherit from if)
                        let saved_tail = self.in_tail_position;

                        // Compile condition (not in tail position)
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;

                        // Emit JmpIfFalse with placeholder address
                        let jmp_if_false_index = self.bytecode.len();
                        self.emit(Instruction::JmpIfFalse(0)); // placeholder

                        // Compile then-branch (inherits tail position from if)
                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[2])?;

                        // Emit Jmp to skip else-branch, with placeholder address
                        let jmp_to_end_index = self.bytecode.len();
                        self.emit(Instruction::Jmp(0)); // placeholder

                        // Record else-branch start address
                        let else_addr = self.instruction_address;

                        // Patch the JmpIfFalse to jump here
                        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(else_addr);

                        // Compile else-branch (inherits tail position from if)
                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[3])?;

                        // Record end address
                        let end_addr = self.instruction_address;

                        // Patch the Jmp to jump to end
                        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);

                        // Restore tail position
                        self.in_tail_position = saved_tail;
                    }

                    // Print: (print expr)
                    "print" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "print expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Print);
                    }

                    // Quote: (quote expr) - return expr unevaluated as a list
                    "quote" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "quote expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Convert the quoted expression to a runtime Value
                        let value = self.expr_to_value(&items[1])?;
                        self.emit(Instruction::Push(value));
                    }

                    // Quasiquote: (quasiquote expr) - like quote but allows unquote and unquote-splicing
                    "quasiquote" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "quasiquote expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_quasiquote(&items[1])?;
                    }

                    // Let: (let ((var val) ...) body)
                    "let" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "let expects exactly 2 arguments: bindings and body".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        self.compile_let(&items[1], &items[2])?;
                    }

                    // Lambda: (lambda (params) body)
                    "lambda" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "lambda expects exactly 2 arguments: parameters and body".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_lambda(&items[1], &items[2])?;
                    }

                    // List operations
                    "cons" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "cons expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Cons);
                        self.in_tail_position = saved_tail;
                    }
                    "car" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "car expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Car);
                        self.in_tail_position = saved_tail;
                    }
                    "cdr" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "cdr expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Cdr);
                        self.in_tail_position = saved_tail;
                    }
                    "list?" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "list? expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::IsList);
                        self.in_tail_position = saved_tail;
                    }

                    // String/Symbol operations
                    "string?" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "string? expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::IsString);
                        self.in_tail_position = saved_tail;
                    }
                    "symbol?" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "symbol? expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::IsSymbol);
                        self.in_tail_position = saved_tail;
                    }
                    "symbol->string" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "symbol->string expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::SymbolToString);
                        self.in_tail_position = saved_tail;
                    }
                    "string->symbol" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "string->symbol expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::StringToSymbol);
                        self.in_tail_position = saved_tail;
                    }

                    // User-defined function call or closure variable or macro
                    _ => {
                        // Check if it's a macro
                        if let Some(macro_def) = self.macros.get(operator).cloned() {
                            // It's a macro - expand it at compile time
                            let args: Vec<SourceExpr> = items[1..].to_vec();
                            let expanded = self.expand_macro(&macro_def, &args)?;
                            // Compile the expanded expression
                            self.compile_expr(&expanded)?;
                        } else {
                            // Check if operator is a variable (could be a closure)
                            let is_variable = self.local_bindings.contains_key(operator)
                                || self.pattern_bindings.contains_key(operator)
                                || self.param_names.contains(operator);

                            if is_variable {
                                // It's a variable - load it as a closure and use CallClosure
                                let saved_tail = self.in_tail_position;

                                // Closure and arguments are not in tail position
                                self.in_tail_position = false;
                                self.compile_variable_load(operator)?;

                                // Compile all arguments
                                let arg_count = items.len() - 1;
                                for i in 1..items.len() {
                                    self.compile_expr(&items[i])?;
                                }

                                // Call the closure
                                self.emit(Instruction::CallClosure(arg_count));

                                self.in_tail_position = saved_tail;
                            } else {
                                // It's a regular function call
                                let arg_count = items.len() - 1;
                                let is_tail_call = self.in_tail_position;

                                // Arguments are not in tail position
                                self.in_tail_position = false;
                                for i in 1..items.len() {
                                    self.compile_expr(&items[i])?;
                                }

                                // Emit TailCall if in tail position, otherwise Call
                                if is_tail_call {
                                    self.emit(Instruction::TailCall(operator.to_string(), arg_count));
                                } else {
                                    self.emit(Instruction::Call(operator.to_string(), arg_count));
                                }

                                // Restore tail position
                                self.in_tail_position = is_tail_call;
                            }
                        }
                    }
                }
            } else {
                // Non-symbol operator - should be a closure expression
                // Compile the operator expression (should produce a closure)
                self.compile_expr(&items[0])?;

                // Compile all arguments
                let arg_count = items.len() - 1;
                for i in 1..items.len() {
                    self.compile_expr(&items[i])?;
                }

                // Call the closure
                self.emit(Instruction::CallClosure(arg_count));
            }
            }
        }

        Ok(start_address)
    }

    // Convert a SourceExpr to a runtime Value (for quote)
    fn expr_to_value(&self, expr: &SourceExpr) -> Result<Value, CompileError> {
        match &expr.expr {
            LispExpr::Number(n) => Ok(Value::Integer(*n)),
            LispExpr::Boolean(b) => Ok(Value::Boolean(*b)),
            LispExpr::Symbol(s) => {
                // Symbols in quoted expressions become Symbol values
                Ok(Value::Symbol(s.clone()))
            }
            LispExpr::List(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.expr_to_value(item)?);
                }
                Ok(Value::List(values))
            }
            LispExpr::DottedList(items, rest) => {
                // '(a b . rest) - cons a and b onto rest
                let rest_value = self.expr_to_value(rest)?;

                // Rest must be a list
                if let Value::List(mut rest_list) = rest_value {
                    // Prepend items to rest_list
                    let mut result = Vec::new();
                    for item in items {
                        result.push(self.expr_to_value(item)?);
                    }
                    result.append(&mut rest_list);
                    Ok(Value::List(result))
                } else {
                    Err(CompileError::new(
                        "Rest of dotted list must be a list".to_string(),
                        rest.location.clone(),
                    ))
                }
            }
        }
    }

    fn compile_global_var(&mut self, expr: &SourceExpr, is_const: bool) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => {
                return Err(CompileError::new(
                    format!("{} expects a list", if is_const { "defconst" } else { "defvar" }),
                    expr.location.clone(),
                ));
            }
        };

        // Check length: (defvar name value)
        if items.len() != 3 {
            return Err(CompileError::new(
                format!("{} expects exactly: ({} name value)",
                    if is_const { "defconst" } else { "defvar" },
                    if is_const { "defconst" } else { "defvar" }),
                expr.location.clone(),
            ));
        }

        // Extract variable name
        let var_name = match &items[1].expr {
            LispExpr::Symbol(s) => s.clone(),
            _ => {
                return Err(CompileError::new(
                    "Variable name must be a symbol".to_string(),
                    items[1].location.clone(),
                ));
            }
        };

        // Register as global variable (is_const flag indicates if mutable)
        self.global_vars.insert(var_name.clone(), !is_const);

        // Compile the value expression
        self.compile_expr(&items[2])?;

        // Emit StoreGlobal to store the value
        self.emit(Instruction::StoreGlobal(var_name));

        Ok(())
    }

        fn compile_defun(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => {
                return Err(CompileError::new(
                    "defun expects a list".to_string(),
                    expr.location.clone(),
                ));
            }
        };

        // Check minimum length (defun name ...)
        if items.len() < 3 {
            return Err(CompileError::new(
                "defun expects at least: (defun name clause)".to_string(),
                expr.location.clone(),
            ));
        }

        // Check first element is "defun"
        match &items[0].expr {
            LispExpr::Symbol(s) if s == "defun" => {}
            _ => {
                return Err(CompileError::new(
                    "First element must be 'defun'".to_string(),
                    items[0].location.clone(),
                ));
            }
        }

        // Extract function name
        let fn_name = match &items[1].expr {
            LispExpr::Symbol(s) => s.clone(),
            _ => {
                return Err(CompileError::new(
                    "Function name must be a symbol".to_string(),
                    items[1].location.clone(),
                ));
            }
        };

        // Determine if this is old-style (single clause) or new-style (multi-clause)
        // Old: (defun name (params) body) - 4 elements, items[2] is a list of symbols
        // New: (defun name ((pattern) body) ...) - 3+ elements, items[2] is a list starting with list

        let is_old_style = items.len() == 4 &&
            matches!(&items[2].expr, LispExpr::List(params) if
                params.iter().all(|p| matches!(&p.expr, LispExpr::Symbol(_))));

        if is_old_style {
            // Old single-clause style: (defun name (params) body)
            self.compile_single_clause_defun(&fn_name, &items[2], &items[3])
        } else {
            // New multi-clause style: (defun name ((pattern1) body1) ((pattern2) body2) ...)
            let clauses: Vec<&SourceExpr> = items[2..].iter().collect();
            self.compile_multi_clause_defun(&fn_name, &clauses, &items[1].location)
        }
    }

    // Compile old-style single-clause defun
    fn compile_single_clause_defun(
        &mut self,
        fn_name: &str,
        params_expr: &SourceExpr,
        body_expr: &SourceExpr,
    ) -> Result<(), CompileError> {
        // Extract parameters
        let params = match &params_expr.expr {
            LispExpr::List(p) => p,
            _ => {
                return Err(CompileError::new(
                    "Parameters must be a list".to_string(),
                    params_expr.location.clone(),
                ));
            }
        };

        let param_names: Result<Vec<String>, CompileError> = params
            .iter()
            .map(|p| match &p.expr {
                LispExpr::Symbol(s) => Ok(s.clone()),
                _ => Err(CompileError::new(
                    "Parameter must be a symbol".to_string(),
                    p.location.clone(),
                )),
            })
            .collect();

        let param_names = param_names?;

        // Save current compilation context
        let saved_bytecode = std::mem::take(&mut self.bytecode);
        let saved_params = std::mem::take(&mut self.param_names);
        let saved_address = self.instruction_address;
        let saved_tail_position = self.in_tail_position;

        // Set up new context for function
        self.bytecode = Vec::new();
        self.param_names = param_names;
        self.instruction_address = 0;
        self.in_tail_position = true; // Function body is in tail position

        // Compile function body
        self.compile_expr(body_expr)?;

        // Emit return instruction
        self.emit(Instruction::Ret);

        // Store compiled function
        let fn_bytecode = std::mem::take(&mut self.bytecode);
        self.functions.insert(fn_name.to_string(), fn_bytecode);

        // Restore context
        self.bytecode = saved_bytecode;
        self.param_names = saved_params;
        self.instruction_address = saved_address;
        self.in_tail_position = saved_tail_position;

        Ok(())
    }

    // Compile new-style multi-clause defun
    fn compile_multi_clause_defun(
        &mut self,
        fn_name: &str,
        clauses: &[&SourceExpr],
        name_location: &Location,
    ) -> Result<(), CompileError> {
        if clauses.is_empty() {
            return Err(CompileError::new(
                "defun requires at least one clause".to_string(),
                name_location.clone(),
            ));
        }

        // Parse all clauses to extract patterns and bodies
        // We need to handle two cases:
        // 1. Pattern list: (defun foo ((a b) body)) - clause_items[0] is a List
        // 2. Single dotted pattern: (defun foo ((a . b) body)) - clause_items[0] is a DottedList

        // First collect owned pattern vecs for dotted list cases
        let mut owned_patterns: Vec<Vec<SourceExpr>> = Vec::new();

        for clause in clauses {
            let clause_items = match &clause.expr {
                LispExpr::List(items) => items,
                _ => {
                    return Err(CompileError::new(
                        "Each clause must be a list ((pattern...) body)".to_string(),
                        clause.location.clone(),
                    ));
                }
            };

            if clause_items.len() != 2 {
                return Err(CompileError::new(
                    "Each clause must have exactly 2 elements: (pattern body)".to_string(),
                    clause.location.clone(),
                ));
            }

            // If it's a dotted list, wrap it as a single-element pattern vec
            if matches!(&clause_items[0].expr, LispExpr::DottedList(_,_)) {
                owned_patterns.push(vec![clause_items[0].clone()]);
            }
        }

        // Now build parsed_clauses with proper references
        let mut dotted_idx = 0;
        let mut parsed_clauses: Vec<(&Vec<SourceExpr>, &SourceExpr)> = Vec::new();

        for clause in clauses {
            let clause_items = match &clause.expr {
                LispExpr::List(items) => items,
                _ => unreachable!(),
            };

            match &clause_items[0].expr {
                LispExpr::List(p) => {
                    parsed_clauses.push((p, &clause_items[1]));
                }
                LispExpr::DottedList(_,_) => {
                    parsed_clauses.push((&owned_patterns[dotted_idx], &clause_items[1]));
                    dotted_idx += 1;
                }
                _ => {
                    return Err(CompileError::new(
                        "Pattern tuple must be a list or dotted list".to_string(),
                        clause_items[0].location.clone(),
                    ));
                }
            }
        }

        // Find maximum arity among all clauses (for param_names setup)
        let max_arity = parsed_clauses.iter()
            .map(|(patterns, _)| patterns.len())
            .max()
            .unwrap_or(0);

        // Save current compilation context
        let saved_bytecode = std::mem::take(&mut self.bytecode);
        let saved_params = std::mem::take(&mut self.param_names);
        let saved_address = self.instruction_address;
        let saved_tail_position = self.in_tail_position;

        // Set up new context for function
        self.bytecode = Vec::new();
        self.param_names = (0..max_arity).map(|i| format!("__arg{}", i)).collect();
        self.instruction_address = 0;
        self.in_tail_position = true; // Pattern clause bodies are in tail position

        // Compile pattern matching dispatch
        self.compile_pattern_dispatch(&parsed_clauses)?;

        // Emit error if no clause matched
        // For now, just emit a halt (will panic at runtime)
        self.emit(Instruction::Halt);

        // Store compiled function
        let fn_bytecode = std::mem::take(&mut self.bytecode);
        self.functions.insert(fn_name.to_string(), fn_bytecode);

        // Restore context
        self.bytecode = saved_bytecode;
        self.param_names = saved_params;
        self.instruction_address = saved_address;
        self.in_tail_position = saved_tail_position;

        Ok(())
    }

    // Helper to compile a single pattern match
    // Emits code to test if the value at the given location matches the pattern
    // Returns bindings created by this pattern
    // If is_last_clause is false, emits JmpIfFalse instructions and returns their indices for patching
    fn compile_single_pattern(
        &mut self,
        pattern: &SourceExpr,
        value_location: ValueLocation,
        is_last_clause: bool,
        bindings: &mut Vec<(String, ValueLocation)>,
        jmp_indices: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        match &pattern.expr {
            // Literal patterns: must match exactly
            LispExpr::Number(n) => {
                value_location.emit_load(self);
                self.emit(Instruction::Push(Value::Integer(*n)));
                self.emit(Instruction::Eq);
                if !is_last_clause {
                    jmp_indices.push(self.bytecode.len());
                    self.emit(Instruction::JmpIfFalse(0));
                }
            }

            LispExpr::Boolean(b) => {
                value_location.emit_load(self);
                self.emit(Instruction::Push(Value::Boolean(*b)));
                self.emit(Instruction::Eq);
                if !is_last_clause {
                    jmp_indices.push(self.bytecode.len());
                    self.emit(Instruction::JmpIfFalse(0));
                }
            }

            // Variable patterns: always match, bind to name
            LispExpr::Symbol(s) if s != "_" => {
                bindings.push((s.clone(), value_location));
            }

            // Wildcard pattern: always match, don't bind
            LispExpr::Symbol(s) if s == "_" => {
                // No code needed
            }

            // List patterns
            LispExpr::List(items) => {
                // Check if this is a quoted expression (quote ...)
                if items.len() == 2 {
                    if let LispExpr::Symbol(s) = &items[0].expr {
                        if s == "quote" {
                            // Quoted pattern - match exact value
                            let quoted_value = self.expr_to_value(&items[1])?;
                            value_location.emit_load(self);
                            self.emit(Instruction::Push(quoted_value));
                            self.emit(Instruction::Eq);
                            if !is_last_clause {
                                jmp_indices.push(self.bytecode.len());
                                self.emit(Instruction::JmpIfFalse(0));
                            }
                            return Ok(());
                        }
                    }
                }

                // Fixed-length list pattern: (a b c)
                if !is_last_clause {
                    // Check that value is a list
                    value_location.emit_load(self);
                    self.emit(Instruction::IsList);
                    jmp_indices.push(self.bytecode.len());
                    self.emit(Instruction::JmpIfFalse(0));
                }

                // TODO: Check length matches expected length
                // For now, pattern matching will fail at runtime if lengths don't match

                // Extract and match each element
                for (i, item_pattern) in items.iter().enumerate() {
                    // Load list, extract i-th element using car/cdr
                    let elem_location = ValueLocation::ListElement(Box::new(value_location.clone()), i);
                    self.compile_single_pattern(
                        item_pattern,
                        elem_location,
                        is_last_clause,
                        bindings,
                        jmp_indices,
                    )?;
                }
            }

            // Dotted list pattern: (h . t)
            LispExpr::DottedList(items, rest) => {
                if !is_last_clause {
                    // Check that value is a list
                    value_location.emit_load(self);
                    self.emit(Instruction::IsList);
                    jmp_indices.push(self.bytecode.len());
                    self.emit(Instruction::JmpIfFalse(0));

                    // Check that list is not empty
                    value_location.emit_load(self);
                    self.emit(Instruction::Push(Value::List(vec![])));
                    self.emit(Instruction::Neq); // NOT equal to empty
                    jmp_indices.push(self.bytecode.len());
                    self.emit(Instruction::JmpIfFalse(0));
                }

                // Match head elements
                for (i, item_pattern) in items.iter().enumerate() {
                    let elem_location = ValueLocation::ListElement(Box::new(value_location.clone()), i);
                    self.compile_single_pattern(
                        item_pattern,
                        elem_location,
                        is_last_clause,
                        bindings,
                        jmp_indices,
                    )?;
                }

                // Match rest (cdr after skipping head items)
                let rest_location = ValueLocation::ListRest(Box::new(value_location.clone()), items.len());
                self.compile_single_pattern(
                    rest,
                    rest_location,
                    is_last_clause,
                    bindings,
                    jmp_indices,
                )?;
            }

            _ => {
                return Err(CompileError::new(
                    format!("Unsupported pattern type: {:?}", pattern.expr),
                    pattern.location.clone(),
                ));
            }
        }

        Ok(())
    }

    // Compile pattern matching dispatch for multiple clauses
    fn compile_pattern_dispatch(
        &mut self,
        clauses: &[(&Vec<SourceExpr>, &SourceExpr)],
    ) -> Result<(), CompileError> {
        for (i, (patterns, body)) in clauses.iter().enumerate() {
            let is_last_clause = i == clauses.len() - 1;

            // Emit arity check at the start of this clause
            let arity = patterns.len();
            let arity_check_idx = if !is_last_clause {
                let idx = self.bytecode.len();
                self.emit(Instruction::CheckArity(arity, 0)); // Placeholder jump address
                Some(idx)
            } else {
                // Last clause doesn't need arity check - if we get here, no other clause matched
                None
            };

            // For each pattern in this clause, generate matching code
            let mut bindings: Vec<(String, ValueLocation)> = Vec::new();
            let mut jmp_indices = Vec::new(); // Indices of JmpIfFalse to patch

            for (arg_idx, pattern) in patterns.iter().enumerate() {
                self.compile_single_pattern(
                    pattern,
                    ValueLocation::Arg(arg_idx),
                    is_last_clause,
                    &mut bindings,
                    &mut jmp_indices,
                )?;
            }

            // If all patterns matched, execute body with bindings
            let saved_bindings = self.pattern_bindings.clone();

            // Set up pattern bindings for this clause
            self.pattern_bindings.clear();
            for (var_name, location) in bindings {
                self.pattern_bindings.insert(var_name, location);
            }

            // Compile body
            self.compile_expr(body)?;
            self.emit(Instruction::Ret);

            // Restore bindings
            self.pattern_bindings = saved_bindings;

            // Patch jump instructions to jump to next clause
            if !is_last_clause {
                let next_clause_addr = self.bytecode.len();

                // Patch arity check jump
                if let Some(idx) = arity_check_idx {
                    self.bytecode[idx] = Instruction::CheckArity(arity, next_clause_addr);
                }

                // Patch pattern match jumps
                for jmp_idx in jmp_indices {
                    self.bytecode[jmp_idx] = Instruction::JmpIfFalse(next_clause_addr);
                }
            }
        }

        Ok(())
    }

    // Compile let expression: (let ((pattern value) ...) body)
    fn compile_let(
        &mut self,
        bindings_expr: &SourceExpr,
        body_expr: &SourceExpr,
    ) -> Result<(), CompileError> {
        // Parse bindings list
        let bindings = match &bindings_expr.expr {
            LispExpr::List(b) => b,
            _ => {
                return Err(CompileError::new(
                    "let bindings must be a list".to_string(),
                    bindings_expr.location.clone(),
                ));
            }
        };

        // Save current local bindings and stack depth
        let saved_bindings = self.local_bindings.clone();
        let saved_stack_depth = self.stack_depth;

        let mut num_bindings = 0;

        // Process each binding
        for binding in bindings {
            let binding_pair = match &binding.expr {
                LispExpr::List(pair) => pair,
                _ => {
                    return Err(CompileError::new(
                        "Each binding must be a list (pattern value)".to_string(),
                        binding.location.clone(),
                    ));
                }
            };

            if binding_pair.len() != 2 {
                return Err(CompileError::new(
                    "Each binding must have exactly 2 elements: (pattern value)".to_string(),
                    binding.location.clone(),
                ));
            }

            let pattern = &binding_pair[0];
            let value_expr = &binding_pair[1];

            // Save tail position and set to false for binding values
            let saved_tail = self.in_tail_position;
            self.in_tail_position = false;

            // Compile the value expression (pushes result onto stack)
            self.compile_expr(value_expr)?;

            // Restore tail position
            self.in_tail_position = saved_tail;

            // The value is now on the stack at position stack_depth
            let value_position = self.stack_depth;
            self.stack_depth += 1;
            num_bindings += 1;

            // Bind the pattern to this stack position
            self.bind_pattern_to_local(pattern, value_position)?;
        }

        // Compile body with bindings available (body inherits tail position from let)
        self.compile_expr(body_expr)?;

        // Clean up let bindings from stack
        // Stack state: [... bindings(num_bindings) result]
        // We want: [... result]
        if num_bindings > 0 {
            self.emit(Instruction::Slide(num_bindings));
        }

        // Restore binding context
        self.local_bindings = saved_bindings;
        self.stack_depth = saved_stack_depth;

        Ok(())
    }

    // Compile defmacro: (defmacro name (params) body)
    fn compile_defmacro(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => {
                return Err(CompileError::new(
                    "defmacro expects a list".to_string(),
                    expr.location.clone(),
                ));
            }
        };

        // Check format: (defmacro name (params) body)
        if items.len() != 4 {
            return Err(CompileError::new(
                "defmacro expects: (defmacro name (params) body)".to_string(),
                expr.location.clone(),
            ));
        }

        // Extract macro name
        let macro_name = match &items[1].expr {
            LispExpr::Symbol(s) => s.clone(),
            _ => {
                return Err(CompileError::new(
                    "Macro name must be a symbol".to_string(),
                    items[1].location.clone(),
                ));
            }
        };

        // Extract parameters
        let params = match &items[2].expr {
            LispExpr::List(p) => {
                let mut param_names = Vec::new();
                for param in p {
                    match &param.expr {
                        LispExpr::Symbol(s) => param_names.push(s.clone()),
                        _ => {
                            return Err(CompileError::new(
                                "Macro parameters must be symbols".to_string(),
                                param.location.clone(),
                            ));
                        }
                    }
                }
                param_names
            }
            _ => {
                return Err(CompileError::new(
                    "Macro parameters must be a list".to_string(),
                    items[2].location.clone(),
                ));
            }
        };

        // Store macro definition (body is unevaluated)
        let macro_def = MacroDef {
            params,
            body: items[3].clone(),
        };

        self.macros.insert(macro_name, macro_def);

        Ok(())
    }

    // Expand a macro call at compile time
    fn expand_macro(&mut self, macro_def: &MacroDef, args: &[SourceExpr]) -> Result<SourceExpr, CompileError> {
        // Check arity
        if args.len() != macro_def.params.len() {
            return Err(CompileError::new(
                format!("Macro arity mismatch: expected {}, got {}", macro_def.params.len(), args.len()),
                Location::unknown(),
            ));
        }

        // Create a new compiler for evaluating the macro
        let mut macro_compiler = Compiler::new();

        // Set up macro parameters as "arguments"
        macro_compiler.param_names = macro_def.params.clone();

        // Compile macro body
        macro_compiler.compile_expr(&macro_def.body)?;
        macro_compiler.emit(Instruction::Halt);

        let macro_bytecode = std::mem::take(&mut macro_compiler.bytecode);

        // Create a VM and run the macro
        let mut vm = VM::new();
        vm.current_bytecode = macro_bytecode;

        // Create a frame with the quoted arguments
        let mut arg_values = Vec::new();
        for arg_expr in args {
            arg_values.push(self.expr_to_value(arg_expr)?);
        }

        let frame = Frame {
            return_address: 0,
            locals: arg_values,
            return_bytecode: Vec::new(),
            function_name: "<macro>".to_string(),
            captured: Vec::new(),
        };
        vm.call_stack.push(frame);

        // Run the VM
        vm.run();

        // Get the result from the stack
        if vm.value_stack.is_empty() {
            return Err(CompileError::new(
                "Macro expansion produced no value".to_string(),
                Location::unknown(),
            ));
        }

        let result_value = vm.value_stack.pop().unwrap();

        // Convert the result back to a SourceExpr
        self.value_to_expr(&result_value)
    }

    // Convert a Value back to a SourceExpr (inverse of expr_to_value)
    fn value_to_expr(&self, value: &Value) -> Result<SourceExpr, CompileError> {
        match value {
            Value::Integer(n) => Ok(SourceExpr::unknown(LispExpr::Number(*n))),
            Value::Boolean(b) => Ok(SourceExpr::unknown(LispExpr::Boolean(*b))),
            Value::Symbol(s) => Ok(SourceExpr::unknown(LispExpr::Symbol(s.clone()))),
            Value::String(s) => {
                // Strings are represented as special symbols in the AST
                Ok(SourceExpr::unknown(LispExpr::Symbol(format!("__STRING__{}", s))))
            }
            Value::List(items) => {
                let mut exprs = Vec::new();
                for item in items {
                    exprs.push(self.value_to_expr(item)?);
                }
                Ok(SourceExpr::unknown(LispExpr::List(exprs)))
            }
            Value::Closure { .. } => {
                Err(CompileError::new(
                    "Cannot convert closure to expression in macro expansion".to_string(),
                    Location::unknown(),
                ))
            }
        }
    }

    // Helper to bind a pattern to a local stack position
    fn bind_pattern_to_local(
        &mut self,
        pattern: &SourceExpr,
        stack_pos: usize,
    ) -> Result<(), CompileError> {
        match &pattern.expr {
            // Simple variable binding
            LispExpr::Symbol(s) if s != "_" => {
                self.local_bindings.insert(s.clone(), ValueLocation::Local(stack_pos));
            }

            // Wildcard - no binding needed
            LispExpr::Symbol(s) if s == "_" => {
                // No binding
            }

            // Destructuring patterns - need to extract components
            LispExpr::List(items) => {
                // Check if this is a quoted pattern
                if items.len() == 2 {
                    if let LispExpr::Symbol(s) = &items[0].expr {
                        if s == "quote" {
                            // Quoted pattern - this would match a literal, not destructure
                            // For now, not supported in let (only in function patterns)
                            return Err(CompileError::new(
                                "Quoted literal patterns not supported in let bindings".to_string(),
                                pattern.location.clone(),
                            ));
                        }
                    }
                }

                // Fixed-length list pattern: (a b c)
                // Extract each element and bind
                for (i, item_pattern) in items.iter().enumerate() {
                    let elem_location = ValueLocation::ListElement(
                        Box::new(ValueLocation::Local(stack_pos)),
                        i
                    );
                    self.bind_pattern_element_to_location(item_pattern, elem_location)?;
                }
            }

            // Dotted list pattern: (h . t)
            LispExpr::DottedList(items, rest) => {
                // Bind head elements
                for (i, item_pattern) in items.iter().enumerate() {
                    let elem_location = ValueLocation::ListElement(
                        Box::new(ValueLocation::Local(stack_pos)),
                        i
                    );
                    self.bind_pattern_element_to_location(item_pattern, elem_location)?;
                }

                // Bind rest
                let rest_location = ValueLocation::ListRest(
                    Box::new(ValueLocation::Local(stack_pos)),
                    items.len()
                );
                self.bind_pattern_element_to_location(rest, rest_location)?;
            }

            _ => {
                return Err(CompileError::new(
                    format!("Unsupported pattern in let binding: {:?}", pattern.expr),
                    pattern.location.clone(),
                ));
            }
        }

        Ok(())
    }

    // Compile lambda expression: (lambda (params) body)
    fn compile_lambda(
        &mut self,
        params_expr: &SourceExpr,
        body_expr: &SourceExpr,
    ) -> Result<(), CompileError> {
        // Parse parameters
        let params = match &params_expr.expr {
            LispExpr::List(p) => {
                let mut param_names = Vec::new();
                for param in p {
                    match &param.expr {
                        LispExpr::Symbol(s) => param_names.push(s.clone()),
                        _ => {
                            return Err(CompileError::new(
                                "Lambda parameters must be symbols".to_string(),
                                param.location.clone(),
                            ));
                        }
                    }
                }
                param_names
            }
            _ => {
                return Err(CompileError::new(
                    "Lambda parameters must be a list".to_string(),
                    params_expr.location.clone(),
                ));
            }
        };

        // Find free variables in body (variables not in params)
        let free_vars = self.find_free_variables(body_expr, &params);

        // Save current compilation context
        let saved_bytecode = std::mem::take(&mut self.bytecode);
        let saved_params = std::mem::take(&mut self.param_names);
        let saved_local_bindings = self.local_bindings.clone();
        let saved_pattern_bindings = self.pattern_bindings.clone();
        let saved_address = self.instruction_address;
        let saved_stack_depth = self.stack_depth;
        let saved_tail_position = self.in_tail_position;

        // Set up new context for closure body
        self.bytecode = Vec::new();
        self.param_names = params.clone();
        self.instruction_address = 0;
        self.local_bindings.clear();
        self.pattern_bindings.clear();
        self.stack_depth = 0;
        self.in_tail_position = true; // Lambda body is in tail position

        // Set up captured variables as "LoadCaptured" locations
        for (i, var_name) in free_vars.iter().enumerate() {
            self.pattern_bindings.insert(var_name.clone(), ValueLocation::Captured(i));
        }

        // Compile body
        self.compile_expr(body_expr)?;
        self.emit(Instruction::Ret);

        // Get compiled body
        let body_bytecode = std::mem::take(&mut self.bytecode);

        // Restore context
        self.bytecode = saved_bytecode;
        self.param_names = saved_params;
        self.local_bindings = saved_local_bindings;
        self.pattern_bindings = saved_pattern_bindings;
        self.instruction_address = saved_address;
        self.stack_depth = saved_stack_depth;
        self.in_tail_position = saved_tail_position;

        // Emit code to push captured variable values onto stack
        for var_name in &free_vars {
            // Load the value of this free variable
            self.compile_variable_load(var_name)?;
        }

        // Emit MakeClosure instruction
        self.emit(Instruction::MakeClosure(params, body_bytecode, free_vars.len()));

        Ok(())
    }

    // Find free variables in an expression (variables not in bound_vars)
    fn find_free_variables(&self, expr: &SourceExpr, bound_vars: &[String]) -> Vec<String> {
        let mut free_vars = Vec::new();
        self.collect_free_variables(expr, bound_vars, &mut free_vars);
        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        free_vars.retain(|v| seen.insert(v.clone()));
        free_vars
    }

    // Helper to recursively collect free variables
    fn collect_free_variables(
        &self,
        expr: &SourceExpr,
        bound_vars: &[String],
        free_vars: &mut Vec<String>,
    ) {
        match &expr.expr {
            LispExpr::Symbol(s) => {
                // Check if it's a variable (not a string literal, not bound)
                if !s.starts_with("__STRING__") && !bound_vars.contains(s) {
                    // Check if it's available in current environment
                    if self.local_bindings.contains_key(s)
                        || self.pattern_bindings.contains_key(s)
                        || self.param_names.contains(s)
                    {
                        free_vars.push(s.clone());
                    }
                }
            }
            LispExpr::List(items) => {
                if items.is_empty() {
                    return;
                }

                // Check for special forms that introduce bindings
                if let LispExpr::Symbol(s) = &items[0].expr {
                    match s.as_str() {
                        "let" if items.len() == 3 => {
                            // let introduces new bindings
                            if let LispExpr::List(bindings) = &items[1].expr {
                                let mut new_bound = bound_vars.to_vec();
                                for binding in bindings {
                                    if let LispExpr::List(pair) = &binding.expr {
                                        if pair.len() == 2 {
                                            // Collect free vars from value expression first
                                            self.collect_free_variables(&pair[1], bound_vars, free_vars);
                                            // Then add binding var to bound list
                                            if let LispExpr::Symbol(var) = &pair[0].expr {
                                                new_bound.push(var.clone());
                                            }
                                        }
                                    }
                                }
                                // Collect from body with extended bindings
                                self.collect_free_variables(&items[2], &new_bound, free_vars);
                                return;
                            }
                        }
                        "lambda" if items.len() == 3 => {
                            // lambda introduces new parameters
                            if let LispExpr::List(params) = &items[1].expr {
                                let mut new_bound = bound_vars.to_vec();
                                for param in params {
                                    if let LispExpr::Symbol(p) = &param.expr {
                                        new_bound.push(p.clone());
                                    }
                                }
                                self.collect_free_variables(&items[2], &new_bound, free_vars);
                                return;
                            }
                        }
                        "quote" => {
                            // Quoted expressions don't have free variables
                            return;
                        }
                        _ => {}
                    }
                }

                // Default: recursively process all items
                for item in items {
                    self.collect_free_variables(item, bound_vars, free_vars);
                }
            }
            LispExpr::DottedList(items, rest) => {
                for item in items {
                    self.collect_free_variables(item, bound_vars, free_vars);
                }
                self.collect_free_variables(rest, bound_vars, free_vars);
            }
            _ => {}
        }
    }

    // Helper to load a variable (for capturing)
    fn compile_variable_load(&mut self, var_name: &str) -> Result<(), CompileError> {
        // Check local bindings first
        if let Some(location) = self.local_bindings.get(var_name) {
            location.clone().emit_load(self);
        } else if let Some(location) = self.pattern_bindings.get(var_name) {
            location.clone().emit_load(self);
        } else if let Some(idx) = self.param_names.iter().position(|p| p == var_name) {
            self.emit(Instruction::LoadArg(idx));
        } else {
            return Err(CompileError::new(
                format!("Variable '{}' not found for capture", var_name),
                Location::unknown(),
            ));
        }
        Ok(())
    }

    // Helper to bind a pattern element to a specific location
    fn bind_pattern_element_to_location(
        &mut self,
        pattern: &SourceExpr,
        location: ValueLocation,
    ) -> Result<(), CompileError> {
        match &pattern.expr {
            LispExpr::Symbol(s) if s != "_" => {
                self.local_bindings.insert(s.clone(), location);
            }
            LispExpr::Symbol(s) if s == "_" => {
                // Wildcard, no binding
            }
            // Could recursively handle nested patterns here
            _ => {
                return Err(CompileError::new(
                    "Only simple variables and wildcards supported in nested patterns".to_string(),
                    pattern.location.clone(),
                ));
            }
        }
        Ok(())
    }

    // Compile quasiquote expression
    // Quasiquote is like quote, but allows unquote (,) and unquote-splicing (,@)
    fn compile_quasiquote(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        match &expr.expr {
            // Check for unquote: (unquote expr)
            LispExpr::List(items) if items.len() == 2 => {
                if let LispExpr::Symbol(s) = &items[0].expr {
                    if s == "unquote" {
                        // Unquote: evaluate the expression
                        self.compile_expr(&items[1])?;
                        return Ok(());
                    }
                }
                // Not an unquote, process as normal list
                self.compile_quasiquote_list(items)?;
            }

            // Empty list or regular list
            LispExpr::List(items) => {
                self.compile_quasiquote_list(items)?;
            }

            // Dotted list
            LispExpr::DottedList(_items, _rest) => {
                // For dotted lists in quasiquote, we need to handle unquote-splicing specially
                // For now, just quote the whole thing (simplified implementation)
                let value = self.expr_to_value(expr)?;
                self.emit(Instruction::Push(value));
            }

            // Atoms: just quote them
            _ => {
                let value = self.expr_to_value(expr)?;
                self.emit(Instruction::Push(value));
            }
        }

        Ok(())
    }

    // Helper to compile a quasiquoted list
    // Handles unquote-splicing and builds the list at runtime
    fn compile_quasiquote_list(&mut self, items: &[SourceExpr]) -> Result<(), CompileError> {
        if items.is_empty() {
            // Empty list
            self.emit(Instruction::Push(Value::List(vec![])));
            return Ok(());
        }

        // Check if we have any unquotes or splicing - if not, we can just quote the whole thing
        let has_unquote_or_splice = items.iter().any(|item| {
            if let LispExpr::List(inner) = &item.expr {
                if inner.len() == 2 {
                    if let LispExpr::Symbol(s) = &inner[0].expr {
                        return s == "unquote" || s == "unquote-splicing";
                    }
                }
            }
            // Recursively check for unquotes in nested expressions
            self.contains_unquote(item)
        });

        if !has_unquote_or_splice {
            // No unquotes at all - just convert to a value and push it
            let value = self.expr_to_value(&SourceExpr::new(
                LispExpr::List(items.to_vec()),
                Location::unknown(),
            ))?;
            self.emit(Instruction::Push(value));
            return Ok(());
        }

        // We have unquotes - need to build the list at runtime
        // Strategy: collect all elements into a vector of code that pushes each element
        // Then build the list using cons operations

        // For simplicity, let's build forward using an explicit loop
        // We'll push all elements onto the stack, then build the list

        let mut elem_count = 0;

        // Check for splicing first
        let has_splicing = items.iter().any(|item| {
            if let LispExpr::List(inner) = &item.expr {
                if inner.len() == 2 {
                    if let LispExpr::Symbol(s) = &inner[0].expr {
                        return s == "unquote-splicing";
                    }
                }
            }
            false
        });

        if has_splicing {
            // Complex case with splicing
            // We'll build the list in forward order differently
            // Collect segments and splice them together

            // Build forward: start with list containing all non-splice elements and splice points
            self.emit(Instruction::Push(Value::List(vec![])));

            for item in items.iter() {
                if let LispExpr::List(inner) = &item.expr {
                    if inner.len() == 2 {
                        if let LispExpr::Symbol(s) = &inner[0].expr {
                            if s == "unquote-splicing" {
                                // Evaluate the list to splice and append it
                                self.compile_expr(&inner[1])?;
                                // Stack: [accumulator, splice_list]
                                // We want: [accumulator..., splice_list...]
                                self.emit_append()?;
                                continue;
                            } else if s == "unquote" {
                                // Regular unquote - cons the element
                                self.compile_expr(&inner[1])?;
                                // Stack: [accumulator, elem]
                                // We need to make a single-element list and append
                                self.emit(Instruction::MakeList(1));
                                self.emit_append()?;
                                continue;
                            }
                        }
                    }
                }
                // Regular element - quasiquote it and append as single-element list
                self.compile_quasiquote(item)?;
                self.emit(Instruction::MakeList(1));
                self.emit_append()?;
            }
        } else {
            // No splicing - simpler case
            // Push all elements onto stack, then use MakeList
            for item in items {
                if let LispExpr::List(inner) = &item.expr {
                    if inner.len() == 2 {
                        if let LispExpr::Symbol(s) = &inner[0].expr {
                            if s == "unquote" {
                                self.compile_expr(&inner[1])?;
                                elem_count += 1;
                                continue;
                            }
                        }
                    }
                }
                self.compile_quasiquote(item)?;
                elem_count += 1;
            }

            // Now create a list from the elements on the stack
            self.emit(Instruction::MakeList(elem_count));
        }

        Ok(())
    }

    // Helper to check if an expression contains unquote or unquote-splicing
    fn contains_unquote(&self, expr: &SourceExpr) -> bool {
        match &expr.expr {
            LispExpr::List(items) => {
                if items.len() == 2 {
                    if let LispExpr::Symbol(s) = &items[0].expr {
                        if s == "unquote" || s == "unquote-splicing" {
                            return true;
                        }
                    }
                }
                items.iter().any(|item| self.contains_unquote(item))
            }
            LispExpr::DottedList(items, rest) => {
                items.iter().any(|item| self.contains_unquote(item)) || self.contains_unquote(rest)
            }
            _ => false,
        }
    }

    // Emit code to append two lists (both on stack)
    // Stack before: [... list1 list2]
    // Stack after: [... (append list1 list2)]
    fn emit_append(&mut self) -> Result<(), CompileError> {
        self.emit(Instruction::Append);
        Ok(())
    }


    pub fn compile_program(&mut self, exprs: &[SourceExpr]) -> Result<(HashMap<String, Vec<Instruction>>, Vec<Instruction>), CompileError> {
        // First pass: compile all defun, defmacro, defvar, and defconst expressions
        for expr in exprs {
            if let LispExpr::List(items) = &expr.expr {
                if let Some(first) = items.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        if s == "defun" {
                            self.compile_defun(expr)?;
                        } else if s == "defmacro" {
                            self.compile_defmacro(expr)?;
                        } else if s == "defvar" {
                            self.compile_global_var(expr, false)?;
                        } else if s == "defconst" {
                            self.compile_global_var(expr, true)?;
                        }
                    }
                }
            }
        }

        // Second pass: compile non-definition expressions into main bytecode
        for expr in exprs {
            let is_definition = if let LispExpr::List(items) = &expr.expr {
                if let Some(first) = items.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        s == "defun" || s == "defmacro" || s == "defvar" || s == "defconst"
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if !is_definition {
                self.compile_expr(expr)?;
            }
        }

        // Emit Halt at end of main bytecode
        self.emit(Instruction::Halt);

        // Return (functions, main bytecode)
        Ok((self.functions.clone(), self.bytecode.clone()))
    }
}

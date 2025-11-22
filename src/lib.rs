use std::collections::HashMap;

pub mod parser;
pub mod bytecode;


#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
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
    Ret,
    LoadArg(usize),
    Print,
    Halt,
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
}

pub struct VM {
    pub instruction_pointer: usize,
    pub value_stack: Vec<Value>,
    pub call_stack: Vec<Frame>,
    pub functions: HashMap<String, Vec<Instruction>>,
    pub current_bytecode: Vec<Instruction>,
    pub halted: bool,
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
        }
    }


    fn execute_one_instruction(&mut self) {
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
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x == y));
                    }
                    _ => panic!("Type error: Eq expects integers"),
                }
                self.instruction_pointer += 1;
            }
            Instruction::Neq => {
                let b = self.value_stack.pop().expect("Stack underflow");
                let a = self.value_stack.pop().expect("Stack underflow");
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        self.value_stack.push(Value::Boolean(x != y));
                    }
                    _ => panic!("Type error: Neq expects integers"),
                }
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
            Instruction::Print => {
                let value = self.value_stack.pop().expect("Stack underflow");
                match value {
                    Value::Integer(n) => println!("{}", n),
                    Value::Boolean(b) => println!("{}", b),
                }
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
                };
                self.call_stack.push(frame);

                // Switch to function bytecode
                self.current_bytecode = fn_bytecode;
                self.instruction_pointer = 0;
            }
            Instruction::Halt => {
                self.halted = true;
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


pub struct Compiler {
    bytecode: Vec<Instruction>,
    functions: HashMap<String, Vec<Instruction>>,
    instruction_address: usize,
    param_names: Vec<String>, // Track parameter names for LoadArg
}

impl Compiler {

    pub fn new() -> Self {
        Compiler {
            bytecode: Vec::new(),
            functions: HashMap::new(),
            instruction_address: 0,
            param_names: Vec::new(),
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

            // Case: Symbol - check if it's a parameter
            LispExpr::Symbol(s) => {
                // Check if this symbol is a parameter
                if let Some(idx) = self.param_names.iter().position(|p| p == s) {
                    self.emit(Instruction::LoadArg(idx));
                } else {
                    return Err(CompileError::new(
                        format!("Symbol '{}' not found in parameters", s),
                        expr.location.clone(),
                    ));
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

                // Extract operator (first element should be a Symbol)
                let operator = match &items[0].expr {
                    LispExpr::Symbol(s) => s.as_str(),
                    _ => {
                        return Err(CompileError::new(
                            "First element of list must be a symbol (operator)".to_string(),
                            items[0].location.clone(),
                        ));
                    }
                };

                match operator {
                    // Arithmetic operators: +, -, *, /
                    "+" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "+ expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Add);
                    }
                    "-" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "- expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Sub);
                    }
                    "*" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "* expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Mul);
                    }
                    "/" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "/ expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Div);
                    }
                    "%" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "% expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Mod);
                    }
                    "neg" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "neg expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Neg);
                    }

                    // Comparison operators
                    "<=" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "<= expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Leq);
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

                        // Compile condition
                        self.compile_expr(&items[1])?;

                        // Emit JmpIfFalse with placeholder address
                        let jmp_if_false_index = self.bytecode.len();
                        self.emit(Instruction::JmpIfFalse(0)); // placeholder

                        // Compile then-branch
                        self.compile_expr(&items[2])?;

                        // Emit Jmp to skip else-branch, with placeholder address
                        let jmp_to_end_index = self.bytecode.len();
                        self.emit(Instruction::Jmp(0)); // placeholder

                        // Record else-branch start address
                        let else_addr = self.instruction_address;

                        // Patch the JmpIfFalse to jump here
                        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(else_addr);

                        // Compile else-branch
                        self.compile_expr(&items[3])?;

                        // Record end address
                        let end_addr = self.instruction_address;

                        // Patch the Jmp to jump to end
                        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);
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

                    // User-defined function call
                    _ => {
                        // Compile all arguments
                        let arg_count = items.len() - 1;
                        for i in 1..items.len() {
                            self.compile_expr(&items[i])?;
                        }
                        // Emit Call instruction
                        self.emit(Instruction::Call(operator.to_string(), arg_count));
                    }
                }
            }
        }

        Ok(start_address)
    }


    fn compile_defun(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        // Assert expr is (defun name (params...) body)
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => {
                return Err(CompileError::new(
                    "defun expects a list".to_string(),
                    expr.location.clone(),
                ));
            }
        };

        if items.len() != 4 {
            return Err(CompileError::new(
                "defun expects 4 elements: (defun name (params) body)".to_string(),
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

        // Extract parameters
        let params = match &items[2].expr {
            LispExpr::List(p) => p,
            _ => {
                return Err(CompileError::new(
                    "Parameters must be a list".to_string(),
                    items[2].location.clone(),
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

        // Set up new context for function
        self.bytecode = Vec::new();
        self.param_names = param_names;
        self.instruction_address = 0;

        // Compile function body
        self.compile_expr(&items[3])?;

        // Emit return instruction
        self.emit(Instruction::Ret);

        // Store function bytecode
        let fn_bytecode = std::mem::take(&mut self.bytecode);
        self.functions.insert(fn_name, fn_bytecode);

        // Restore context
        self.bytecode = saved_bytecode;
        self.param_names = saved_params;
        self.instruction_address = saved_address;

        Ok(())
    }


    pub fn compile_program(&mut self, exprs: &[SourceExpr]) -> Result<(HashMap<String, Vec<Instruction>>, Vec<Instruction>), CompileError> {
        // First pass: compile all defun expressions
        for expr in exprs {
            if let LispExpr::List(items) = &expr.expr {
                if let Some(first) = items.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        if s == "defun" {
                            self.compile_defun(expr)?;
                        }
                    }
                }
            }
        }

        // Second pass: compile non-defun expressions into main bytecode
        for expr in exprs {
            let is_defun = if let LispExpr::List(items) = &expr.expr {
                if let Some(first) = items.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        s == "defun"
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if !is_defun {
                self.compile_expr(expr)?;
            }
        }

        // Emit Halt at end of main bytecode
        self.emit(Instruction::Halt);

        // Return (functions, main bytecode)
        Ok((self.functions.clone(), self.bytecode.clone()))
    }
}

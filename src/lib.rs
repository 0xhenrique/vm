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
    Leq,
    JmpIfFalse(usize),
    Jmp(usize),
    Call(String, usize),
    Ret,
    LoadArg(usize),
    Print,
    Halt,
}


#[derive(Debug, Clone, PartialEq)]
pub enum LispExpr {
    Number(i64),
    Boolean(bool),
    Symbol(String),
    List(Vec<LispExpr>),
}

#[allow(dead_code)]
fn number(n: i64) -> LispExpr {
    LispExpr::Number(n)
}

#[allow(dead_code)]
fn boolean(b: bool) -> LispExpr {
    LispExpr::Boolean(b)
}

#[allow(dead_code)]
fn symbol(s: &str) -> LispExpr {
    LispExpr::Symbol(s.to_string())
}

#[allow(dead_code)]
fn list(items: Vec<LispExpr>) -> LispExpr {
    LispExpr::List(items)
}

#[derive(Debug)]
pub struct Frame {
    pub return_address: usize,
    pub locals: Vec<Value>,
    pub return_bytecode: Vec<Instruction>, // Bytecode to return to after function call
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

                // Create new frame with return bytecode
                let frame = Frame {
                    return_address: self.instruction_pointer + 1,
                    locals: args,
                    return_bytecode: self.current_bytecode.clone(),
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
    fn compile_expr(&mut self, expr: &LispExpr) -> usize {
        let start_address = self.instruction_address;

        match expr {
            // Case: Number or Boolean
            LispExpr::Number(n) => {
                self.emit(Instruction::Push(Value::Integer(*n)));
            }
            LispExpr::Boolean(b) => {
                self.emit(Instruction::Push(Value::Boolean(*b)));
            }

            // Case: Symbol
            LispExpr::Symbol(s) => {
                // Check if this symbol is a parameter
                if let Some(idx) = self.param_names.iter().position(|p| p == s) {
                    self.emit(Instruction::LoadArg(idx));
                } else {
                    panic!("Symbol '{}' not found in parameters", s);
                }
            }

            // Case: List (func call or special form)
            LispExpr::List(items) => {
                if items.is_empty() {
                    panic!("Empty list cannot be compiled");
                }

                // Extract operator (first element should be a Symbol)
                let operator = match &items[0] {
                    LispExpr::Symbol(s) => s.as_str(),
                    _ => panic!("First element of list must be a symbol (operator)"),
                };

                match operator {
                    // Arithmetic operators
                    "+" => {
                        if items.len() != 3 {
                            panic!("+ expects exactly 2 arguments");
                        }
                        self.compile_expr(&items[1]);
                        self.compile_expr(&items[2]);
                        self.emit(Instruction::Add);
                    }
                    "-" => {
                        if items.len() != 3 {
                            panic!("- expects exactly 2 arguments");
                        }
                        self.compile_expr(&items[1]);
                        self.compile_expr(&items[2]);
                        self.emit(Instruction::Sub);
                    }
                    "*" => {
                        if items.len() != 3 {
                            panic!("* expects exactly 2 arguments");
                        }
                        self.compile_expr(&items[1]);
                        self.compile_expr(&items[2]);
                        self.emit(Instruction::Mul);
                    }
                    "/" => {
                        if items.len() != 3 {
                            panic!("/ expects exactly 2 arguments");
                        }
                        self.compile_expr(&items[1]);
                        self.compile_expr(&items[2]);
                        self.emit(Instruction::Div);
                    }

                    // Comparison operator
                    "<=" => {
                        if items.len() != 3 {
                            panic!("<= expects exactly 2 arguments");
                        }
                        self.compile_expr(&items[1]);
                        self.compile_expr(&items[2]);
                        self.emit(Instruction::Leq);
                    }

                    // Conditional: (if condition then-branch else-branch)
                    "if" => {
                        if items.len() != 4 {
                            panic!("if expects exactly 3 arguments (condition, then, else)");
                        }

                        // Compile condition
                        self.compile_expr(&items[1]);

                        // Emit JmpIfFalse with placeholder address
                        let jmp_if_false_index = self.bytecode.len();
                        self.emit(Instruction::JmpIfFalse(0)); // placeholder

                        // Compile then-branch
                        self.compile_expr(&items[2]);

                        // Emit Jmp to skip else-branch, with placeholder address
                        let jmp_to_end_index = self.bytecode.len();
                        self.emit(Instruction::Jmp(0)); // placeholder

                        // Record else-branch start address
                        let else_addr = self.instruction_address;

                        // Patch the JmpIfFalse to jump here
                        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(else_addr);

                        // Compile else-branch
                        self.compile_expr(&items[3]);

                        // Record end address
                        let end_addr = self.instruction_address;

                        // Patch the Jmp to jump to end
                        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);
                    }

                    // Print: (print expr)
                    "print" => {
                        if items.len() != 2 {
                            panic!("print expects exactly 1 argument");
                        }
                        self.compile_expr(&items[1]);
                        self.emit(Instruction::Print);
                    }

                    // User-defined function call
                    _ => {
                        // Compile all arguments
                        let arg_count = items.len() - 1;
                        for i in 1..items.len() {
                            self.compile_expr(&items[i]);
                        }
                        // Emit Call instruction
                        self.emit(Instruction::Call(operator.to_string(), arg_count));
                    }
                }
            }
        }

        start_address
    }

    fn compile_defun(&mut self, expr: &LispExpr) {
        // Assert expr is (defun name (params...) body)
        let items = match expr {
            LispExpr::List(items) => items,
            _ => panic!("defun expects a list"),
        };

        if items.len() != 4 {
            panic!("defun expects 4 elements: (defun name (params) body)");
        }

        match &items[0] {
            LispExpr::Symbol(s) if s == "defun" => {}
            _ => panic!("First element must be 'defun'"),
        }

        let fn_name = match &items[1] {
            LispExpr::Symbol(s) => s.clone(),
            _ => panic!("Function name must be a symbol"),
        };

        let params = match &items[2] {
            LispExpr::List(p) => p,
            _ => panic!("Parameters must be a list"),
        };

        let param_names: Vec<String> = params
            .iter()
            .map(|p| match p {
                LispExpr::Symbol(s) => s.clone(),
                _ => panic!("Parameter must be a symbol"),
            })
            .collect();

        // Save current compilation context
        let saved_bytecode = std::mem::take(&mut self.bytecode);
        let saved_params = std::mem::take(&mut self.param_names);
        let saved_address = self.instruction_address;

        // Set up new context for function
        self.bytecode = Vec::new();
        self.param_names = param_names;
        self.instruction_address = 0;

        // Compile function body
        self.compile_expr(&items[3]);

        // Emit return instruction
        self.emit(Instruction::Ret);

        // Store function bytecode
        let fn_bytecode = std::mem::take(&mut self.bytecode);
        self.functions.insert(fn_name, fn_bytecode);

        // Restore context
        self.bytecode = saved_bytecode;
        self.param_names = saved_params;
        self.instruction_address = saved_address;
    }

    pub fn compile_program(&mut self, exprs: &[LispExpr]) -> (HashMap<String, Vec<Instruction>>, Vec<Instruction>) {
        // First pass: compile all defun expressions
        for expr in exprs {
            if let LispExpr::List(items) = expr {
                if let Some(LispExpr::Symbol(s)) = items.first() {
                    if s == "defun" {
                        self.compile_defun(expr);
                    }
                }
            }
        }

        // Second pass: compile non-defun expressions into main bytecode
        for expr in exprs {
            let is_defun = if let LispExpr::List(items) = expr {
                if let Some(LispExpr::Symbol(s)) = items.first() {
                    s == "defun"
                } else {
                    false
                }
            } else {
                false
            };

            if !is_defun {
                self.compile_expr(expr);
            }
        }

        // Emit Halt at end of main bytecode
        self.emit(Instruction::Halt);

        // Return (functions, main bytecode)
        (self.functions.clone(), self.bytecode.clone())
    }
}

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
enum Instruction {
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
enum LispExpr {
    Number(i64),
    Boolean(bool),
    Symbol(String),
    List(Vec<LispExpr>),
}

fn number(n: i64) -> LispExpr {
    LispExpr::Number(n)
}

fn boolean(b: bool) -> LispExpr {
    LispExpr::Boolean(b)
}

fn symbol(s: &str) -> LispExpr {
    LispExpr::Symbol(s.to_string())
}

fn list(items: Vec<LispExpr>) -> LispExpr {
    LispExpr::List(items)
}

#[derive(Debug)]
struct Frame {
    return_address: usize,
    locals: Vec<Value>,
}

struct VM {
    instruction_pointer: usize,
    value_stack: Vec<Value>,
    call_stack: Vec<Frame>,
    functions: HashMap<String, Vec<Instruction>>,
    current_bytecode: Vec<Instruction>,
    halted: bool,
}

impl VM {
    fn new() -> Self {
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
                self.instruction_pointer = frame.return_address;
            }
            Instruction::Call(fn_name, arg_count) => {
                let bytecode = self.functions.get(&fn_name)
                    .expect(&format!("Function '{}' not found", fn_name))
                    .clone();

                // Pop arguments from value stack in reverse order
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.value_stack.pop().expect("Stack underflow"));
                }
                args.reverse();

                let frame = Frame {
                    return_address: self.instruction_pointer + 1,
                    locals: args,
                };
                self.call_stack.push(frame);

                // Save current bytecode and switch to function bytecode
                // @TODO: in a full implementation, need to save and restore current_bytecode
                self.instruction_pointer = 0;
                // this is a simplification, will need to handle
                // switching between different bytecode sequences
            }
            Instruction::Halt => {
                self.halted = true;
            }
        }
    }

    fn run(&mut self) {
        while !self.halted {
            self.execute_one_instruction();
        }
    }
}

struct Compiler {
    bytecode: Vec<Instruction>,
    functions: HashMap<String, Vec<Instruction>>,
    instruction_address: usize,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            bytecode: Vec::new(),
            functions: HashMap::new(),
            instruction_address: 0,
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
            // Case: Number or Boolean - emit Push instruction
            LispExpr::Number(n) => {
                self.emit(Instruction::Push(Value::Integer(*n)));
            }
            LispExpr::Boolean(b) => {
                self.emit(Instruction::Push(Value::Boolean(*b)));
            }

            // Case: Symbol - WIP: error for now
            LispExpr::Symbol(s) => {
                panic!("Symbol '{}' not supported in this context yet", s);
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

                    _ => panic!("Operator '{}' not yet implemented", operator),
                }
            }
        }

        start_address
    }
}

fn main() {
    println!("=== Phase 1: VM Test ===");
    let mut vm = VM::new();

    // Hardcoded bytecode: [Push(5), Push(3), Add, Print, Halt]
    vm.current_bytecode = vec![
        Instruction::Push(Value::Integer(5)),
        Instruction::Push(Value::Integer(3)),
        Instruction::Add,
        Instruction::Print,
        Instruction::Halt,
    ];

    vm.run();

    println!("\n=== Phase 2: LispExpr Construction Test ===");

    // Example: (+ 5 3)
    let simple_add = list(vec![
        symbol("+"),
        number(5),
        number(3),
    ]);
    println!("Simple addition: {:?}", simple_add);

    // Example: (if (<= n 1) n (+ (fib (- n 1)) (fib (- n 2))))
    // Testing how to construct a more complex expression with recursion like Fibonacci
    let fib_body = list(vec![
        symbol("if"),
        list(vec![
            symbol("<="),
            symbol("n"),
            number(1),
        ]),
        symbol("n"),
        list(vec![
            symbol("+"),
            list(vec![
                symbol("fib"),
                list(vec![
                    symbol("-"),
                    symbol("n"),
                    number(1),
                ]),
            ]),
            list(vec![
                symbol("fib"),
                list(vec![
                    symbol("-"),
                    symbol("n"),
                    number(2),
                ]),
            ]),
        ]),
    ]);
    println!("\nFibonacci body: {:?}", fib_body);

    // Example: (defun fib (n) ...)
    let fib_defun = list(vec![
        symbol("defun"),
        symbol("fib"),
        list(vec![symbol("n")]),
        fib_body,
    ]);
    println!("\nFibonacci defun: {:?}", fib_defun);

    // Example: (fib 10)
    let fib_call = list(vec![
        symbol("fib"),
        number(10),
    ]);
    println!("\nFibonacci call: {:?}", fib_call);

    println!("\n=== Basic Compilation Test ===");

    // Test 1: Compile and run (+ 5 3)
    let mut compiler = Compiler::new();
    let expr = list(vec![symbol("+"), number(5), number(3)]);
    println!("\nCompiling: {:?}", expr);
    compiler.compile_expr(&expr);
    compiler.emit(Instruction::Print);
    compiler.emit(Instruction::Halt);

    println!("Generated bytecode:");
    for (i, instr) in compiler.bytecode.iter().enumerate() {
        println!("  {}: {:?}", i, instr);
    }

    let mut vm = VM::new();
    vm.current_bytecode = compiler.bytecode;
    print!("Output: ");
    vm.run();

    // Test 2: Compile and run (* (- 10 2) (+ 3 2))
    let mut compiler2 = Compiler::new();
    let expr2 = list(vec![
        symbol("*"),
        list(vec![symbol("-"), number(10), number(2)]),
        list(vec![symbol("+"), number(3), number(2)]),
    ]);
    println!("\nCompiling: {:?}", expr2);
    compiler2.compile_expr(&expr2);
    compiler2.emit(Instruction::Print);
    compiler2.emit(Instruction::Halt);

    println!("Generated bytecode:");
    for (i, instr) in compiler2.bytecode.iter().enumerate() {
        println!("  {}: {:?}", i, instr);
    }

    let mut vm2 = VM::new();
    vm2.current_bytecode = compiler2.bytecode;
    print!("Output: ");
    vm2.run();

    // Test 3: Compile and run (<= 5 10)
    let mut compiler3 = Compiler::new();
    let expr3 = list(vec![symbol("<="), number(5), number(10)]);
    println!("\nCompiling: {:?}", expr3);
    compiler3.compile_expr(&expr3);
    compiler3.emit(Instruction::Print);
    compiler3.emit(Instruction::Halt);

    let mut vm3 = VM::new();
    vm3.current_bytecode = compiler3.bytecode;
    print!("Output: ");
    vm3.run();
}

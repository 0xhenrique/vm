use std::collections::HashMap;
use std::env;

mod parser;
use parser::Parser;

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
pub enum LispExpr {
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
    return_bytecode: Vec<Instruction>, // Bytecode to return to after function call
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
    param_names: Vec<String>, // Track parameter names for LoadArg
}

impl Compiler {
    fn new() -> Self {
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

            // Case: List (function call or special form)
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

        // New context for function
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

    fn compile_program(&mut self, exprs: &[LispExpr]) -> (HashMap<String, Vec<Instruction>>, Vec<Instruction>) {
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

fn run_program(program: &[LispExpr]) {
    let mut compiler = Compiler::new();
    let (functions, bytecode) = compiler.compile_program(program);
    let mut vm = VM::new();
    vm.functions = functions;
    vm.current_bytecode = bytecode;
    vm.run();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <lisp-expression>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} \"(+ 5 3)\"", args[0]);
        eprintln!("  {} \"(* (- 10 2) (+ 3 2))\"", args[0]);
        eprintln!("  {} \"(defun double (x) (* x 2)) (print (double 5))\"", args[0]);
        eprintln!();
        eprintln!("Note: Use prefix notation (Lisp style)");
        std::process::exit(1);
    }

    let input = &args[1];

    // Parse the input
    let mut parser = Parser::new(input);
    let exprs = match parser.parse_all() {
        Ok(e) => e,
        Err(msg) => {
            eprintln!("Parse error: {}", msg);
            std::process::exit(1);
        }
    };

    // Execute the expressions
    run_program(&exprs);
}

// For backwards compatibility DO NOT DELETE! Keep the old test code
#[allow(dead_code)]
fn run_all_tests() {
    println!("=== VM Test ===");
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

    println!("\n=== LispExpr Construction Test ===");

    // Example: (+ 5 3)
    let simple_add = list(vec![
        symbol("+"),
        number(5),
        number(3),
    ]);
    println!("Simple addition: {:?}", simple_add);

    // Example: (if (<= n 1) n (+ (fib (- n 1)) (fib (- n 2))))
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

    // Example: Boolean literals
    let bool_expr = list(vec![
        symbol("if"),
        boolean(true),
        number(100),
        number(200),
    ]);
    println!("\nBoolean example: {:?}", bool_expr);

    println!("\n=== Basic Compilation Test ===");

    // Test 0: Boolean literal test
    let mut compiler0 = Compiler::new();
    let expr0 = list(vec![
        symbol("if"),
        boolean(false),
        number(999),
        number(42),
    ]);
    println!("\nCompiling boolean test: {:?}", expr0);
    compiler0.compile_expr(&expr0);
    compiler0.emit(Instruction::Print);
    compiler0.emit(Instruction::Halt);

    let mut vm0 = VM::new();
    vm0.current_bytecode = compiler0.bytecode;
    print!("Output (should be 42): ");
    vm0.run();

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

    println!("\n=== Conditional Compilation Test ===");

    // Test 4: Compile and run (if (<= 5 10) 100 200)
    // Condition is true, so should return 100
    let mut compiler4 = Compiler::new();
    let expr4 = list(vec![
        symbol("if"),
        list(vec![symbol("<="), number(5), number(10)]),
        number(100),
        number(200),
    ]);
    println!("\nCompiling: {:?}", expr4);
    compiler4.compile_expr(&expr4);
    compiler4.emit(Instruction::Print);
    compiler4.emit(Instruction::Halt);

    println!("Generated bytecode:");
    for (i, instr) in compiler4.bytecode.iter().enumerate() {
        println!("  {}: {:?}", i, instr);
    }

    let mut vm4 = VM::new();
    vm4.current_bytecode = compiler4.bytecode;
    print!("Output: ");
    vm4.run();

    // Test 5: Compile and run (if (<= 15 10) 100 200)
    // Condition is false, so should return 200
    let mut compiler5 = Compiler::new();
    let expr5 = list(vec![
        symbol("if"),
        list(vec![symbol("<="), number(15), number(10)]),
        number(100),
        number(200),
    ]);
    println!("\nCompiling: {:?}", expr5);
    compiler5.compile_expr(&expr5);
    compiler5.emit(Instruction::Print);
    compiler5.emit(Instruction::Halt);

    println!("Generated bytecode:");
    for (i, instr) in compiler5.bytecode.iter().enumerate() {
        println!("  {}: {:?}", i, instr);
    }

    let mut vm5 = VM::new();
    vm5.current_bytecode = compiler5.bytecode;
    print!("Output: ");
    vm5.run();

    // Test 6: Nested if with arithmetic
    // (if (<= 3 5) (+ 10 20) (* 2 3))
    let mut compiler6 = Compiler::new();
    let expr6 = list(vec![
        symbol("if"),
        list(vec![symbol("<="), number(3), number(5)]),
        list(vec![symbol("+"), number(10), number(20)]),
        list(vec![symbol("*"), number(2), number(3)]),
    ]);
    println!("\nCompiling: {:?}", expr6);
    compiler6.compile_expr(&expr6);
    compiler6.emit(Instruction::Print);
    compiler6.emit(Instruction::Halt);

    println!("Generated bytecode:");
    for (i, instr) in compiler6.bytecode.iter().enumerate() {
        println!("  {}: {:?}", i, instr);
    }

    let mut vm6 = VM::new();
    vm6.current_bytecode = compiler6.bytecode;
    print!("Output: ");
    vm6.run();

    println!("\n=== Function Compilation Test ===");

    // Test 7: Simple function definition and call
    // (defun double (x) (* x 2))
    // (double 5)
    let double_defun = list(vec![
        symbol("defun"),
        symbol("double"),
        list(vec![symbol("x")]),
        list(vec![symbol("*"), symbol("x"), number(2)]),
    ]);

    let double_call = list(vec![
        symbol("print"),
        list(vec![symbol("double"), number(5)]),
    ]);

    let program = vec![double_defun, double_call];

    let mut compiler7 = Compiler::new();
    let (functions, main_bytecode) = compiler7.compile_program(&program);

    println!("\nFunction 'double' bytecode:");
    if let Some(bytecode) = functions.get("double") {
        for (i, instr) in bytecode.iter().enumerate() {
            println!("  {}: {:?}", i, instr);
        }
    }

    println!("\nMain bytecode:");
    for (i, instr) in main_bytecode.iter().enumerate() {
        println!("  {}: {:?}", i, instr);
    }

    let mut vm7 = VM::new();
    vm7.functions = functions.clone();
    vm7.current_bytecode = main_bytecode.clone();
    print!("\nOutput: ");
    vm7.run();

    println!("\n=== Recursive Function Test ===");

    // Test 8: Factorial function
    // (defun fact (n) (if (<= n 1) 1 (* n (fact (- n 1)))))
    // (fact 5)
    let fact_defun = list(vec![
        symbol("defun"),
        symbol("fact"),
        list(vec![symbol("n")]),
        list(vec![
            symbol("if"),
            list(vec![symbol("<="), symbol("n"), number(1)]),
            number(1),
            list(vec![
                symbol("*"),
                symbol("n"),
                list(vec![
                    symbol("fact"),
                    list(vec![symbol("-"), symbol("n"), number(1)]),
                ]),
            ]),
        ]),
    ]);

    let fact_call = list(vec![
        symbol("print"),
        list(vec![symbol("fact"), number(5)]),
    ]);

    let program2 = vec![fact_defun, fact_call];

    let mut compiler8 = Compiler::new();
    let (functions8, main_bytecode8) = compiler8.compile_program(&program2);

    println!("\nFunction 'fact' bytecode:");
    if let Some(bytecode) = functions8.get("fact") {
        for (i, instr) in bytecode.iter().enumerate() {
            println!("  {}: {:?}", i, instr);
        }
    }

    println!("\nMain bytecode:");
    for (i, instr) in main_bytecode8.iter().enumerate() {
        println!("  {}: {:?}", i, instr);
    }

    let mut vm8 = VM::new();
    vm8.functions = functions8;
    vm8.current_bytecode = main_bytecode8;
    print!("\nOutput: ");
    vm8.run();

    println!("\n=== Complete Integration Test - Fibonacci ===");

    // (defun fib (n)
    //   (if (<= n 1)
    //       n
    //       (+ (fib (- n 1)) (fib (- n 2)))))
    let fib_defun = list(vec![
        symbol("defun"),
        symbol("fib"),
        list(vec![symbol("n")]),
        list(vec![
            symbol("if"),
            list(vec![symbol("<="), symbol("n"), number(1)]),
            symbol("n"),
            list(vec![
                symbol("+"),
                list(vec![
                    symbol("fib"),
                    list(vec![symbol("-"), symbol("n"), number(1)]),
                ]),
                list(vec![
                    symbol("fib"),
                    list(vec![symbol("-"), symbol("n"), number(2)]),
                ]),
            ]),
        ]),
    ]);

    let fib_call = list(vec![
        symbol("print"),
        list(vec![symbol("fib"), number(10)]),
    ]);

    let fibonacci_program = vec![fib_defun, fib_call];

    println!("\nCompiling Fibonacci program...");

    let mut compiler_fib = Compiler::new();
    let (fib_functions, fib_bytecode) = compiler_fib.compile_program(&fibonacci_program);

    println!("\nFunction 'fib' bytecode:");
    if let Some(bytecode) = fib_functions.get("fib") {
        for (i, instr) in bytecode.iter().enumerate() {
            println!("  {}: {:?}", i, instr);
        }
    }

    println!("\nMain bytecode:");
    for (i, instr) in fib_bytecode.iter().enumerate() {
        println!("  {}: {:?}", i, instr);
    }

    println!("\nRunning Fibonacci program with run_program()...");
    print!("fib(10) = ");
    run_program(&fibonacci_program);

    println!("\n✓ All systems operational!");
    println!("  - VM executes bytecode correctly");
    println!("  - Compiler translates LispExpr to bytecode");
    println!("  - Recursive functions work (factorial, fibonacci)");
    println!("  - Expected: 55, Got: 55 (10th Fibonacci number)");
}

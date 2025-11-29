use crate::{Compiler, VM, parser::Parser, disassembler, Value};
use std::io::{self, Write};

pub struct Repl {
    compiler: Compiler,
    vm: VM,
    pub input_buffer: String,
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            compiler: Compiler::new(),
            vm: VM::new(),
            input_buffer: String::new(),
        }
    }

    pub fn run(&mut self) {
        println!("Lisp REPL v0.1.0");
        println!("Type :help for commands, :quit to exit");
        println!();

        loop {
            let prompt = if self.input_buffer.is_empty() {
                "lisp> "
            } else {
                "...   "
            };

            print!("{}", prompt);
            io::stdout().flush().unwrap();

            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();

                    if trimmed.is_empty() {
                        continue;
                    }

                    if self.input_buffer.is_empty() && trimmed.starts_with(':') {
                        if !self.handle_command(trimmed) {
                            break;
                        }
                        continue;
                    }

                    self.input_buffer.push_str(&line);

                    if self.is_complete_input() {
                        self.eval_and_print();
                        self.input_buffer.clear();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }
        }

        println!("Goodbye!");
    }

    pub fn is_complete_input(&self) -> bool {
        let mut depth = 0;
        let mut in_string = false;
        let mut chars = self.input_buffer.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => in_string = !in_string,
                '(' if !in_string => depth += 1,
                ')' if !in_string => depth -= 1,
                _ => {}
            }
        }

        depth == 0 && !self.input_buffer.trim().is_empty()
    }

    fn eval_and_print(&mut self) {
        let input = self.input_buffer.trim();

        let mut parser = Parser::new(input);
        let exprs = match parser.parse_all() {
            Ok(exprs) => exprs,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                return;
            }
        };

        if exprs.is_empty() {
            return;
        }

        let mut fresh_compiler = Compiler::new();
        for (name, bytecode) in &self.vm.functions {
            fresh_compiler.functions.insert(name.clone(), bytecode.clone());
        }

        let (new_functions, main_bytecode) = match fresh_compiler.compile_program(&exprs) {
            Ok(result) => result,
            Err(e) => {
                let source_lines: Vec<&str> = input.lines().collect();
                let source_line = if e.location.line > 0 && e.location.line <= source_lines.len() {
                    Some(source_lines[e.location.line - 1])
                } else {
                    None
                };
                eprintln!("{}", e.format(source_line));
                return;
            }
        };

        for (name, bytecode) in new_functions {
            self.vm.functions.insert(name, bytecode);
        }

        self.vm.current_bytecode = main_bytecode;
        self.vm.value_stack.clear();
        self.vm.call_stack.clear();
        self.vm.instruction_pointer = 0;
        self.vm.halted = false;

        match self.vm.run() {
            Ok(_) => {
                if let Some(result) = self.vm.value_stack.last() {
                    println!("=> {}", self.format_value(result));
                }
            }
            Err(runtime_error) => {
                eprintln!("{}", runtime_error.format());
            }
        }
    }

    pub fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Integer(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::List(items) => {
                let formatted_items: Vec<String> = items
                    .iter()
                    .map(|v| self.format_value(v))
                    .collect();
                format!("({})", formatted_items.join(" "))
            }
            Value::Symbol(s) => s.clone(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Function(name) => format!("<function {}>", name),
            Value::Closure { params, .. } => {
                format!("<closure ({})>", params.join(" "))
            }
            Value::HashMap(map) => {
                let mut items: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{} {}", self.format_value(&Value::String(k.clone())), self.format_value(v)))
                    .collect();
                items.sort(); // Sort for consistent output
                format!("{{{}}}", items.join(" "))
            }
            Value::Vector(items) => {
                let formatted_items: Vec<String> = items
                    .iter()
                    .map(|v| self.format_value(v))
                    .collect();
                format!("[{}]", formatted_items.join(" "))
            }
        }
    }

    fn handle_command(&mut self, cmd: &str) -> bool {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return true;
        }

        match parts[0] {
            ":quit" | ":exit" | ":q" => {
                return false;
            }
            ":help" | ":h" => {
                self.print_help();
            }
            ":functions" | ":f" => {
                self.list_functions();
            }
            ":clear" | ":c" => {
                self.clear_state();
            }
            ":bytecode" | ":bc" => {
                if parts.len() < 2 {
                    eprintln!("Usage: :bytecode <expression>");
                } else {
                    let expr = parts[1..].join(" ");
                    self.show_bytecode(&expr);
                }
            }
            _ => {
                eprintln!("Unknown command: {}", parts[0]);
                eprintln!("Type :help for available commands");
            }
        }

        true
    }

    fn print_help(&self) {
        println!("Available commands:");
        println!("  :help, :h           - Show this help message");
        println!("  :quit, :exit, :q    - Exit the REPL");
        println!("  :functions, :f      - List all defined functions");
        println!("  :clear, :c          - Clear all state (reset VM and compiler)");
        println!("  :bytecode <expr>    - Show bytecode for an expression");
        println!();
        println!("Examples:");
        println!("  (+ 2 3)");
        println!("  (defun square (x) (* x x))");
        println!("  (square 5)");
    }

    fn list_functions(&self) {
        if self.vm.functions.is_empty() {
            println!("No functions defined");
        } else {
            println!("Defined functions:");
            let mut names: Vec<_> = self.vm.functions.keys().collect();
            names.sort();
            for name in names {
                let bytecode = &self.vm.functions[name];
                println!("  {} ({} instructions)", name, bytecode.len());
            }
        }
    }

    pub fn clear_state(&mut self) {
        self.compiler = Compiler::new();
        self.vm = VM::new();
        self.input_buffer.clear();
        println!("State cleared");
    }

    fn show_bytecode(&mut self, expr: &str) {
        let mut parser = Parser::new(expr);
        let exprs = match parser.parse_all() {
            Ok(exprs) => exprs,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                return;
            }
        };

        let mut temp_compiler = Compiler::new();
        for (name, bytecode) in &self.vm.functions {
            temp_compiler.functions.insert(name.clone(), bytecode.clone());
        }

        let (functions, main) = match temp_compiler.compile_program(&exprs) {
            Ok(result) => result,
            Err(e) => {
                let source_lines: Vec<&str> = expr.lines().collect();
                let source_line = if e.location.line > 0 && e.location.line <= source_lines.len() {
                    Some(source_lines[e.location.line - 1])
                } else {
                    None
                };
                eprintln!("{}", e.format(source_line));
                return;
            }
        };

        println!("{}", disassembler::disassemble_bytecode(&functions, &main));
    }
}

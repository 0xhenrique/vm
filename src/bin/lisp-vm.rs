use lisp_bytecode_vm::{VM, bytecode};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Lisp Bytecode VM");
        eprintln!();
        eprintln!("Usage: {} [--print-result] <bytecode-file>", args[0]);
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --print-result    Print the final value on the stack");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} program.bc", args[0]);
        eprintln!("  {} --print-result program.bc", args[0]);
        eprintln!();
        eprintln!("Note: Compile Lisp source files using 'bytecomp' first");
        std::process::exit(1);
    }

    // Parse flags
    let mut print_result = false;
    let mut bytecode_file = "";
    let mut vm_args = Vec::new();
    let mut i = 1;

    while i < args.len() {
        if args[i] == "--print-result" {
            print_result = true;
            i += 1;
        } else if bytecode_file.is_empty() {
            bytecode_file = &args[i];
            i += 1;
        } else {
            // Remaining args are for the VM
            vm_args.extend_from_slice(&args[i..]);
            break;
        }
    }

    if bytecode_file.is_empty() {
        eprintln!("Error: No bytecode file specified");
        std::process::exit(1);
    }

    // Load bytecode from file
    let (functions, main_bytecode) = match bytecode::load_bytecode_file(bytecode_file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error loading bytecode: {}", e);
            std::process::exit(1);
        }
    };

    // Execute bytecode on the VM
    let mut vm = VM::new();
    // Merge user-defined functions with builtins (don't overwrite builtins)
    for (name, bytecode) in functions {
        vm.functions.insert(name, bytecode);
    }
    vm.current_bytecode = main_bytecode;

    // Pass command-line arguments to the VM
    vm.args = vm_args;

    if let Err(runtime_error) = vm.run() {
        eprintln!("{}", runtime_error.format());
        std::process::exit(1);
    }

    // Print final result if requested
    if print_result {
        if let Some(value) = vm.value_stack.last() {
            println!("{}", format_value(value));
        }
    }
}

fn format_value(value: &lisp_bytecode_vm::Value) -> String {
    use lisp_bytecode_vm::Value;
    match value {
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => {
            // Always show at least one decimal place for whole numbers
            if f.fract() == 0.0 && !f.is_nan() && !f.is_infinite() {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        Value::Boolean(b) => b.to_string(),
        Value::String(s) => s.to_string(),
        Value::Symbol(s) => s.to_string(),
        Value::List(items) => {
            let formatted: Vec<String> = items.iter().map(|v| format_value(v)).collect();
            format!("({})", formatted.join(" "))
        }
        Value::Function(name) => format!("#<function:{}>", name),
        Value::Closure(closure_data) => {
            if closure_data.rest_param.is_some() {
                format!("#<closure/{} + rest>", closure_data.params.len())
            } else {
                format!("#<closure/{}>", closure_data.params.len())
            }
        }
        Value::HashMap(_) => "#<hashmap>".to_string(),
        Value::Vector(items) => {
            let formatted: Vec<String> = items.iter().map(|v| format_value(v)).collect();
            format!("[{}]", formatted.join(" "))
        }
        Value::TcpListener(_) => "#<tcp-listener>".to_string(),
        Value::TcpStream(_) => "#<tcp-stream>".to_string(),
        Value::SharedTcpListener(_) => "#<shared-tcp-listener>".to_string(),
    }
}

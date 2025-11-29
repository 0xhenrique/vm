use lisp_bytecode_vm::{VM, bytecode};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Lisp Bytecode VM");
        eprintln!();
        eprintln!("Usage: {} <bytecode-file>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} program.bc", args[0]);
        eprintln!();
        eprintln!("Note: Compile Lisp source files using 'bytecomp' first");
        std::process::exit(1);
    }

    let bytecode_file = &args[1];

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

    // Pass command-line arguments to the VM (skip program name and bytecode file)
    if args.len() > 2 {
        vm.args = args[2..].to_vec();
    }

    if let Err(runtime_error) = vm.run() {
        eprintln!("{}", runtime_error.format());
        std::process::exit(1);
    }
}

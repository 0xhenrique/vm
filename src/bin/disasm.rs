use lisp_bytecode_vm::{bytecode, disassembler};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Lisp Bytecode Disassembler");
        eprintln!();
        eprintln!("Usage: {} <bytecode-file>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} program.bc", args[0]);
        eprintln!("  {} factorial.lisp.bc", args[0]);
        std::process::exit(1);
    }

    let bytecode_file = &args[1];

    // Load bytecode from file
    let (functions, main_bytecode) = match bytecode::load_bytecode_file(bytecode_file) {
        Ok((f, m)) => (f, m),
        Err(e) => {
            eprintln!("Error loading bytecode file '{}': {}", bytecode_file, e);
            std::process::exit(1);
        }
    };

    // Disassemble and print
    let disassembly = disassembler::disassemble_bytecode(&functions, &main_bytecode);
    print!("{}", disassembly);
}

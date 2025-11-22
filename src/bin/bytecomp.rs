use lisp_bytecode_vm::{Compiler, bytecode, parser::Parser};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Lisp Bytecode Compiler");
        eprintln!();
        eprintln!("Usage: {} <input-file.lisp> [-o <output-file>]", args[0]);
        eprintln!();
        eprintln!("Options:");
        eprintln!("  -o <output>    Output bytecode file (default: <input>.bc)");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} program.lisp", args[0]);
        eprintln!("  {} program.lisp -o program.bc", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];

    // Determine output file
    let output_file = if args.len() >= 4 && args[2] == "-o" {
        args[3].clone()
    } else {
        format!("{}.bc", input_file.trim_end_matches(".lisp"))
    };

    // Read input file
    let source = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", input_file, e);
            std::process::exit(1);
        }
    };

    // Parse the source code
    let mut parser = Parser::new_with_file(&source, input_file.clone());
    let exprs = match parser.parse_all() {
        Ok(e) => e,
        Err(msg) => {
            eprintln!("Parse error: {}", msg);
            std::process::exit(1);
        }
    };

    // Compile to bytecode
    let mut compiler = Compiler::new();
    let (functions, main_bytecode) = match compiler.compile_program(&exprs) {
        Ok((f, m)) => (f, m),
        Err(compile_error) => {
            // Extract source line for context
            let source_lines: Vec<&str> = source.lines().collect();
            let source_line = if compile_error.location.line > 0 && compile_error.location.line <= source_lines.len() {
                Some(source_lines[compile_error.location.line - 1])
            } else {
                None
            };

            eprintln!("{}", compile_error.format(source_line));
            std::process::exit(1);
        }
    };

    // Save bytecode to file
    if let Err(e) = bytecode::save_bytecode_file(&output_file, &functions, &main_bytecode) {
        eprintln!("Error writing bytecode file: {}", e);
        std::process::exit(1);
    }

    println!("Compiled {} -> {}", input_file, output_file);
    println!("  {} function(s)", functions.len());
    println!("  {} main instruction(s)", main_bytecode.len());
}

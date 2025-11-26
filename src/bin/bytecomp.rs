use lisp_bytecode_vm::{Compiler, bytecode, parser::Parser, optimizer::Optimizer};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Lisp Bytecode Compiler");
        eprintln!();
        eprintln!("Usage: {} <input-file.lisp> [OPTIONS]", args[0]);
        eprintln!();
        eprintln!("Options:");
        eprintln!("  -o <output>    Output bytecode file (default: <input>.bc)");
        eprintln!("  --optimize     Enable bytecode optimizations");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} program.lisp", args[0]);
        eprintln!("  {} program.lisp -o program.bc", args[0]);
        eprintln!("  {} program.lisp --optimize", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];

    let mut output_file = format!("{}.bc", input_file.trim_end_matches(".lisp"));
    let mut optimize = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: -o requires an argument");
                    std::process::exit(1);
                }
            }
            "--optimize" => {
                optimize = true;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

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
    let (mut functions, mut main_bytecode) = match compiler.compile_program(&exprs) {
        Ok((f, m)) => (f, m),
        Err(compile_error) => {
            eprintln!("{}", compile_error.format(Some(&source)));
            std::process::exit(1);
        }
    };

    let original_main_count = main_bytecode.len();
    let original_function_count: usize = functions.values().map(|v| v.len()).sum();
    let original_total = original_main_count + original_function_count;

    // Apply optimizations if requested
    if optimize {
        let mut optimizer = Optimizer::new();

        main_bytecode = optimizer.optimize(main_bytecode);
        functions = optimizer.optimize_functions(functions);

        let stats = optimizer.get_stats();

        println!("Optimization results:");
        println!("  Constant folds: {}", stats.constant_folds);
        println!("  Dead code removed: {}", stats.dead_code_removed);
        println!("  Jump chains simplified: {}", stats.jump_chains_simplified);
        println!("  Original instructions: {}", original_total);
        println!("  Optimized instructions: {}", stats.optimized_instruction_count);
        println!("  Reduction: {:.1}%", stats.reduction_percentage());
        println!();
    }

    // Save bytecode to file
    if let Err(e) = bytecode::save_bytecode_file(&output_file, &functions, &main_bytecode) {
        eprintln!("Error writing bytecode file: {}", e);
        std::process::exit(1);
    }

    println!("Compiled {} -> {}", input_file, output_file);
    println!("  {} function(s)", functions.len());
    println!("  {} main instruction(s)", main_bytecode.len());
}

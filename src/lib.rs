// Core modules
pub mod vm;
pub mod compiler;

// Utility modules
pub mod parser;
pub mod disassembler;
pub mod repl;
pub mod optimizer;

// Re-export commonly used types for backward compatibility
pub use vm::{VM, Value, Instruction, List, FfiType};
pub use vm::errors::{CompileError, RuntimeError, Location};
pub use vm::stack::Frame;
pub use vm::bytecode;

pub use compiler::{Compiler, LispExpr, SourceExpr};

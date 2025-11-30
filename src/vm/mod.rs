// VM subsystem module
// This module contains all the runtime execution components

pub mod value;
pub mod instructions;
pub mod bytecode;
pub mod stack;
pub mod vm;
pub mod env;
pub mod builtins;
pub mod errors;
pub mod object;
pub mod ffi;

// Re-export commonly used types for convenience
pub use value::{Value, List};
pub use instructions::{Instruction, FfiType};
pub use vm::VM;
pub use ffi::FfiState;

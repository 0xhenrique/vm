use super::value::Value;
use super::instructions::Instruction;

#[derive(Debug)]
pub struct Frame {
    pub return_address: usize,
    pub locals: Vec<Value>,
    pub return_bytecode: Vec<Instruction>, // Bytecode to return to after function call
    pub function_name: String, // For stack traces
    pub captured: Vec<Value>, // Captured variables for closures
    pub stack_base: usize, // Base position of this function's locals on the value stack
    pub loop_start: Option<usize>, // Address of loop start for recur
    pub loop_bindings_start: Option<usize>, // Stack position where loop bindings start
    pub loop_bindings_count: Option<usize>, // Number of loop bindings
}

impl Frame {
    pub fn new(
        return_address: usize,
        return_bytecode: Vec<Instruction>,
        function_name: String,
        stack_base: usize,
    ) -> Self {
        Frame {
            return_address,
            locals: Vec::new(),
            return_bytecode,
            function_name,
            captured: Vec::new(),
            stack_base,
            loop_start: None,
            loop_bindings_start: None,
            loop_bindings_count: None,
        }
    }
}

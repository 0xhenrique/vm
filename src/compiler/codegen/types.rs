// Type definitions for the compiler

use crate::vm::value::Value;
use crate::vm::instructions::Instruction;
use super::Compiler;
use super::super::ast::SourceExpr;

// ==================== HELPER TYPES ====================

#[derive(Clone)]
pub(super) enum ValueLocation {
    Local(usize),                                  // Local variable on value stack
    Captured(usize),                               // Captured variable in closure
    ListElement(Box<ValueLocation>, usize),        // i-th element of a list
    ListRest(Box<ValueLocation>, usize),           // Rest after skipping n elements
}

impl ValueLocation {
    // Emit instructions to load the value at this location onto the stack
    pub(super) fn emit_load(&self, compiler: &mut Compiler) {
        match self {
            ValueLocation::Local(pos) => {
                compiler.emit(Instruction::GetLocal(*pos));
            }
            ValueLocation::Captured(idx) => {
                compiler.emit(Instruction::LoadCaptured(*idx));
            }
            ValueLocation::ListElement(list_loc, idx) => {
                // Load the list
                list_loc.emit_load(compiler);
                // Extract the i-th element using car/cdr
                for _ in 0..*idx {
                    compiler.emit(Instruction::Cdr);
                }
                compiler.emit(Instruction::Car);
            }
            ValueLocation::ListRest(list_loc, skip_count) => {
                // Load the list
                list_loc.emit_load(compiler);
                // Skip elements using cdr
                for _ in 0..*skip_count {
                    compiler.emit(Instruction::Cdr);
                }
            }
        }
    }
}

// Macro definition
#[derive(Debug, Clone)]
pub(super) struct MacroDef {
    pub params: Vec<String>,
    pub body: SourceExpr,
}

// Helper struct for parsed parameters (supports variadic syntax)
pub(super) struct ParsedParams {
    pub required: Vec<String>,
    pub rest: Option<String>,
}

// Pattern type for pattern matching in function definitions
#[derive(Debug, Clone)]
pub(super) enum Pattern {
    Variable(String),           // Matches anything, binds to name
    Wildcard,                   // Matches anything, no binding
    Literal(Value),             // Matches specific value (number, boolean)
    QuotedSymbol(String),       // Matches quoted symbol: 'foo
    EmptyList,                  // Matches empty list: '()
    List(Vec<Pattern>),         // Matches fixed-length list: (a b c)
    DottedList(Vec<Pattern>, Box<Pattern>), // Matches cons pattern: (h . t)
}

// A single clause in a multi-clause function definition
#[derive(Debug)]
pub(super) struct FunctionClause {
    pub patterns: Vec<Pattern>,     // Patterns for each argument
    pub body: SourceExpr,           // Body to execute if patterns match
}

// Submodules
mod types;
mod modules;
mod utils;
mod special_forms;
mod macros;

use std::collections::HashMap;
use std::sync::Arc;

use crate::vm::value::{Value, List};
use crate::vm::instructions::{Instruction, FfiType};
use crate::vm::ffi::parse_ffi_type;
use crate::vm::errors::{CompileError, Location};
use super::ast::{LispExpr, SourceExpr};

// Re-export types used internally
pub(self) use types::{ValueLocation, MacroDef, ParsedParams, Pattern, FunctionClause};

// ==================== COMPILER STRUCT ====================

pub struct Compiler {
    bytecode: Vec<Instruction>,
    pub functions: HashMap<String, Vec<Instruction>>,
    macros: HashMap<String, MacroDef>, // Macro definitions
    global_vars: HashMap<String, bool>, // Track global variables (value is mutable flag)
    known_functions: std::collections::HashSet<String>, // Functions known from runtime context (for eval)
    known_globals: std::collections::HashSet<String>, // Globals known from runtime context (for eval)
    instruction_address: usize,
    param_names: Vec<String>, // Track parameter names for LoadArg
    pattern_bindings: HashMap<String, ValueLocation>, // Track pattern match bindings
    local_bindings: HashMap<String, ValueLocation>, // Track let-bound variables
    stack_depth: usize, // Track current stack depth for let bindings
    in_tail_position: bool, // Track if current expression is in tail position (for TCO)
    pattern_match_jumps: Vec<usize>, // Temporary storage for pattern match jump indices
    // Module system fields
    current_module: Option<String>,                              // Current module being compiled (None = top-level)
    pub module_exports: HashMap<String, std::collections::HashSet<String>>, // Module name -> exported symbols
    imported_symbols: HashMap<String, String>,                   // Alias -> qualified name (e.g., "add" -> "math/add")
    module_functions: std::collections::HashSet<String>,         // Functions declared in current module (for forward references)
}

impl Compiler {

    pub fn new() -> Self {
        Compiler {
            bytecode: Vec::new(),
            functions: HashMap::new(),
            macros: HashMap::new(),
            global_vars: HashMap::new(),
            known_functions: std::collections::HashSet::new(),
            known_globals: std::collections::HashSet::new(),
            instruction_address: 0,
            param_names: Vec::new(),
            pattern_bindings: HashMap::new(),
            local_bindings: HashMap::new(),
            stack_depth: 0,
            in_tail_position: false,
            pattern_match_jumps: Vec::new(),
            // Module system fields
            current_module: None,
            module_exports: HashMap::new(),
            imported_symbols: HashMap::new(),
            module_functions: std::collections::HashSet::new(),
        }
    }

    // Inject known function names from runtime context (for eval)
    // This allows eval'd code to reference functions defined in the parent context
    pub fn with_known_functions<'a, I>(&mut self, function_names: I)
    where
        I: Iterator<Item = &'a String>,
    {
        for name in function_names {
            self.known_functions.insert(name.clone());
        }
    }

    // Inject known global variable names from runtime context (for eval)
    // This allows eval'd code to reference globals defined in the parent context
    pub fn with_known_globals<'a, I>(&mut self, global_names: I)
    where
        I: Iterator<Item = &'a String>,
    {
        for name in global_names {
            self.known_globals.insert(name.clone());
        }
    }

    // Clear main bytecode (used after loading stdlib to avoid accumulating bytecode)
    pub fn clear_main_bytecode(&mut self) {
        self.bytecode.clear();
        self.instruction_address = 0;
    }

    fn emit(&mut self, instruction: Instruction) {
        self.bytecode.push(instruction);
        self.instruction_address += 1;
    }

    // ==================== EXPRESSION COMPILATION ====================

    // Returns the starting address of compiled bytecode
    fn compile_expr(&mut self, expr: &SourceExpr) -> Result<usize, CompileError> {
        let start_address = self.instruction_address;

        match &expr.expr {
            // Case: Number or Boolean - emit Push instruction
            LispExpr::Number(n) => {
                self.emit(Instruction::Push(Value::Integer(*n)));
            }
            LispExpr::Float(f) => {
                self.emit(Instruction::Push(Value::Float(*f)));
            }
            LispExpr::Boolean(b) => {
                self.emit(Instruction::Push(Value::Boolean(*b)));
            }

            // Case: DottedList - only valid in patterns
            LispExpr::DottedList(_, _) => {
                return Err(CompileError::with_suggestion(
                    "Dotted lists can only be used in patterns, not in expressions".to_string(),
                    expr.location.clone(),
                    "Dotted lists like (a . b) are only valid in pattern matching (defun, match). To create a cons cell, use (cons a b) instead.".to_string(),
                ));
            }

            // Case: Symbol - check if it's a parameter or string literal
            LispExpr::Symbol(s) => {
                // Check if it's a string literal (hack from parser)
                if s.starts_with("__STRING__") {
                    let string_content = s["__STRING__".len()..].to_string();
                    self.emit(Instruction::Push(Value::String(Arc::new(string_content))));
                } else {
                    // Check local bindings first (let bindings)
                    if let Some(location) = self.local_bindings.get(s) {
                        let loc = location.clone();
                        loc.emit_load(self);
                    } else if let Some(location) = self.pattern_bindings.get(s) {
                        // Check pattern bindings (for nested pattern matches)
                        let loc = location.clone();
                        loc.emit_load(self);
                    } else if let Some(idx) = self.param_names.iter().position(|p| p == s) {
                        // Check if this symbol is a parameter
                        self.emit(Instruction::LoadArg(idx));
                    } else {
                        // Resolve the symbol name (handles imports and module context)
                        let resolved = self.resolve_global_name(s);

                        if self.global_vars.contains_key(&resolved) || self.known_globals.contains(&resolved)
                            || self.global_vars.contains_key(s) || self.known_globals.contains(s) {
                            // Check if this is a global variable (defined or known from context)
                            // Use resolved name if we found it, otherwise original
                            let load_name = if self.global_vars.contains_key(&resolved) || self.known_globals.contains(&resolved) {
                                resolved
                            } else {
                                s.clone()
                            };
                            self.emit(Instruction::LoadGlobal(load_name));
                        } else if self.functions.contains_key(&resolved) || self.known_functions.contains(&resolved)
                            || self.functions.contains_key(s) || self.known_functions.contains(s) || Self::is_builtin_function(s) {
                            // Check if this is a function name (user-defined, known from context, or builtin)
                            // Push it as a Function value so it can be passed around
                            let fn_name = if self.functions.contains_key(&resolved) || self.known_functions.contains(&resolved) {
                                resolved
                            } else {
                                s.clone()
                            };
                            self.emit(Instruction::Push(Value::Function(Arc::new(fn_name))));
                        } else {
                            // Generate helpful suggestion for undefined variable
                            let suggestion = self.suggest_similar_name(s);
                            return Err(CompileError::with_suggestion(
                                format!("Undefined variable '{}'", s),
                                expr.location.clone(),
                                suggestion,
                            ));
                        }
                    }
                }
            }

            // Case: List (function call or special form)
            LispExpr::List(items) => {
                if items.is_empty() {
                    return Err(CompileError::with_suggestion(
                        "Empty list cannot be compiled".to_string(),
                        expr.location.clone(),
                        "Empty lists '()' are only valid as data (nil). Use '(quote ())' for an empty list value, or check if you meant to call a function.".to_string(),
                    ));
                }

                // Check if operator is a symbol
                if let LispExpr::Symbol(operator) = &items[0].expr {
                    // Operator is a symbol - might be special form, built-in, or function call
                    match operator.as_str() {
                    // Arithmetic operators: +, -, *, /
                    "+" => {
                        if items.len() < 3 {
                            return Err(CompileError::with_suggestion(
                                "+ expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                                "The + operator requires at least 2 numbers to add. Example: (+ 1 2) or (+ 1 2 3 4)".to_string(),
                            ));
                        }
                        // Arguments to + are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Add
                        // This transforms (+ 1 2 3 4) into (+ 1 (+ 2 (+ 3 4)))
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Add);
                        }

                        // Restore tail position
                        self.in_tail_position = saved_tail;
                    }
                    "-" => {
                        if items.len() < 3 {
                            return Err(CompileError::with_suggestion(
                                "- expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                                "The - operator requires at least 2 numbers. Example: (- 10 3) subtracts 3 from 10. For negation, use (- 0 x) or the 'neg' function.".to_string(),
                            ));
                        }
                        // Arguments are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Sub
                        // This does left-associative subtraction: (- 10 2 3) = (- (- 10 2) 3) = 5
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Sub);
                        }

                        self.in_tail_position = saved_tail;
                    }
                    "*" => {
                        if items.len() < 3 {
                            return Err(CompileError::new(
                                "* expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Arguments are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Mul
                        // This transforms (* 2 3 4) into (* 2 (* 3 4)) = (* 2 12) = 24
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Mul);
                        }

                        self.in_tail_position = saved_tail;
                    }
                    "/" => {
                        if items.len() < 3 {
                            return Err(CompileError::new(
                                "/ expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Arguments are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Div
                        // This transforms (/ 20 2 2) into (/ (/ 20 2) 2) = (/ 10 2) = 5
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Div);
                        }

                        self.in_tail_position = saved_tail;
                    }
                    "%" => {
                        if items.len() < 3 {
                            return Err(CompileError::new(
                                "% expects at least 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Arguments are not in tail position
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;

                        // Compile first argument
                        self.compile_expr(&items[1])?;

                        // For each remaining argument, compile it and emit Mod
                        // This transforms (% 10 3 2) into (% (% 10 3) 2) = (% 1 2) = 1
                        for i in 2..items.len() {
                            self.compile_expr(&items[i])?;
                            self.emit(Instruction::Mod);
                        }

                        self.in_tail_position = saved_tail;
                    }
                    "neg" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "neg expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Neg);
                        self.in_tail_position = saved_tail;
                    }

                    // Comparison operators
                    "<=" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "<= expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Leq);
                        self.in_tail_position = saved_tail;
                    }
                    "<" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "< expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Lt);
                    }
                    ">" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "> expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Gt);
                    }
                    ">=" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                ">= expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Gte);
                    }
                    "==" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "== expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Eq);
                    }
                    "!=" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "!= expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Neq);
                    }

                    // Conditional: (if condition then-branch else-branch)
                    "if" => {
                        if items.len() != 4 {
                            return Err(CompileError::new(
                                "if expects exactly 3 arguments (condition, then, else)".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        // Save tail position for branches (they inherit from if)
                        let saved_tail = self.in_tail_position;

                        // Compile condition (not in tail position)
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;

                        // Emit JmpIfFalse with placeholder address
                        let jmp_if_false_index = self.bytecode.len();
                        self.emit(Instruction::JmpIfFalse(0)); // placeholder

                        // Compile then-branch (inherits tail position from if)
                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[2])?;

                        // Emit Jmp to skip else-branch, with placeholder address
                        let jmp_to_end_index = self.bytecode.len();
                        self.emit(Instruction::Jmp(0)); // placeholder

                        // Record else-branch start address
                        let else_addr = self.instruction_address;

                        // Patch the JmpIfFalse to jump here
                        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(else_addr);

                        // Compile else-branch (inherits tail position from if)
                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[3])?;

                        // Record end address
                        let end_addr = self.instruction_address;

                        // Patch the Jmp to jump to end
                        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);

                        // Restore tail position
                        self.in_tail_position = saved_tail;
                    }

                    // Logical and: (and expr1 expr2 ...) - short-circuit on false
                    "and" => {
                        if items.len() < 2 {
                            return Err(CompileError::new(
                                "and expects at least 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        if items.len() == 2 {
                            // Single expression: just compile it
                            self.compile_expr(&items[1])?;
                        } else {
                            // Multiple expressions: compile as nested if expressions
                            // (and a b c) => (if a (if b c false) false)
                            self.compile_and_helper(&items[1..], expr)?;
                        }
                    }

                    // Logical or: (or expr1 expr2 ...) - short-circuit on true
                    "or" => {
                        if items.len() < 2 {
                            return Err(CompileError::new(
                                "or expects at least 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        if items.len() == 2 {
                            // Single expression: just compile it
                            self.compile_expr(&items[1])?;
                        } else {
                            // Multiple expressions: compile as nested if expressions
                            // (or a b c) => (if a true (if b true c))
                            self.compile_or_helper(&items[1..], expr)?;
                        }
                    }

                    // Cond: (cond (test1 expr1) (test2 expr2) ... (else default))
                    "cond" => {
                        if items.len() < 2 {
                            return Err(CompileError::new(
                                "cond expects at least 1 clause".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        self.compile_cond(&items[1..], expr)?;
                    }

                    // When: (when test expr) - syntactic sugar for (if test expr false)
                    "when" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "when expects exactly 2 arguments (test, expr)".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        let saved_tail = self.in_tail_position;

                        // Compile test (not in tail position)
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;

                        // Emit JmpIfFalse with placeholder
                        let jmp_if_false_index = self.bytecode.len();
                        self.emit(Instruction::JmpIfFalse(0));

                        // Compile then-branch (inherits tail position)
                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[2])?;

                        // Emit Jmp to skip else-branch
                        let jmp_to_end_index = self.bytecode.len();
                        self.emit(Instruction::Jmp(0));

                        // Else branch (push false)
                        let else_addr = self.instruction_address;
                        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(else_addr);
                        self.emit(Instruction::Push(Value::Boolean(false)));

                        // End
                        let end_addr = self.instruction_address;
                        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);

                        self.in_tail_position = saved_tail;
                    }

                    // Unless: (unless test expr) - syntactic sugar for (if test false expr)
                    "unless" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "unless expects exactly 2 arguments (test, expr)".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        let saved_tail = self.in_tail_position;

                        // Compile test (not in tail position)
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;

                        // Emit JmpIfFalse with placeholder
                        let jmp_if_false_index = self.bytecode.len();
                        self.emit(Instruction::JmpIfFalse(0));

                        // Then branch (push false)
                        self.emit(Instruction::Push(Value::Boolean(false)));

                        // Emit Jmp to skip else-branch
                        let jmp_to_end_index = self.bytecode.len();
                        self.emit(Instruction::Jmp(0));

                        // Else branch (compile expr)
                        let else_addr = self.instruction_address;
                        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(else_addr);

                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[2])?;

                        // End
                        let end_addr = self.instruction_address;
                        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);

                        self.in_tail_position = saved_tail;
                    }

                    // Do/Begin: (do expr1 expr2 ... exprN) or (begin expr1 expr2 ... exprN)
                    // Sequences side effects - evaluates all expressions, returns last value
                    "do" | "begin" => {
                        if items.len() < 2 {
                            return Err(CompileError::new(
                                "do/begin expects at least 1 expression".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        let saved_tail = self.in_tail_position;

                        // Compile all expressions except the last
                        for expr in &items[1..items.len()-1] {
                            self.in_tail_position = false;
                            self.compile_expr(expr)?;
                            // Pop the result since we don't need it (side effects only)
                            self.emit(Instruction::PopN(1));
                        }

                        // Compile the last expression (inherits tail position)
                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[items.len()-1])?;

                        self.in_tail_position = saved_tail;
                    }

                    // Print: (print expr)
                    "print" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "print expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Print);
                    }

                    // Quote: (quote expr) - return expr unevaluated as a list
                    "quote" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "quote expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Convert the quoted expression to a runtime Value
                        let value = self.expr_to_value(&items[1])?;
                        self.emit(Instruction::Push(value));
                    }

                    // Macroexpand: (macroexpand '(macro-call ...)) - expand macro once
                    "macroexpand" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "macroexpand expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        // Extract the form to expand (handling quoted forms)
                        let form_to_expand = &items[1];
                        let actual_form = if let LispExpr::List(quoted_items) = &form_to_expand.expr {
                            // Check if it's a quoted form: (quote expr)
                            if quoted_items.len() == 2 {
                                if let LispExpr::Symbol(s) = &quoted_items[0].expr {
                                    if s == "quote" {
                                        // Extract the quoted expression
                                        &quoted_items[1]
                                    } else {
                                        form_to_expand
                                    }
                                } else {
                                    form_to_expand
                                }
                            } else {
                                form_to_expand
                            }
                        } else {
                            form_to_expand
                        };

                        // Now check if actual_form is a macro call
                        if let LispExpr::List(form_items) = &actual_form.expr {
                            if let Some(first) = form_items.first() {
                                if let LispExpr::Symbol(name) = &first.expr {
                                    if let Some(macro_def) = self.macros.get(name).cloned() {
                                        // It's a macro - expand it
                                        let args = &form_items[1..];
                                        let expanded = self.expand_macro(&macro_def, args)?;
                                        // Return the expanded form as a value
                                        let value = self.expr_to_value(&expanded)?;
                                        self.emit(Instruction::Push(value));
                                    } else {
                                        // Not a macro - return the original form
                                        let value = self.expr_to_value(actual_form)?;
                                        self.emit(Instruction::Push(value));
                                    }
                                } else {
                                    // First element is not a symbol - return as is
                                    let value = self.expr_to_value(actual_form)?;
                                    self.emit(Instruction::Push(value));
                                }
                            } else {
                                // Empty list - return as is
                                let value = self.expr_to_value(actual_form)?;
                                self.emit(Instruction::Push(value));
                            }
                        } else {
                            // Not a list - return as is
                            let value = self.expr_to_value(actual_form)?;
                            self.emit(Instruction::Push(value));
                        }
                    }

                    "list" => {
                        // list is variadic - compile all arguments and use MakeList
                        let arg_count = items.len() - 1; // Exclude 'list' itself
                        for arg in &items[1..] {
                            self.compile_expr(arg)?;
                        }
                        self.emit(Instruction::MakeList(arg_count));
                    }

                    "hash-map" => {
                        // hash-map expects key-value pairs: (hash-map "key1" val1 "key2" val2 ...)
                        let arg_count = items.len() - 1; // Exclude 'hash-map' itself
                        if arg_count % 2 != 0 {
                            return Err(CompileError::new(
                                "hash-map expects an even number of arguments (key-value pairs)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        // Compile all key-value pairs
                        for arg in &items[1..] {
                            self.compile_expr(arg)?;
                        }
                        self.emit(Instruction::MakeHashMap(arg_count / 2));
                    }

                    "vector" => {
                        // vector is variadic - compile all arguments and use MakeVector
                        let arg_count = items.len() - 1; // Exclude 'vector' itself
                        for arg in &items[1..] {
                            self.compile_expr(arg)?;
                        }
                        self.emit(Instruction::MakeVector(arg_count));
                    }

                    // FFI call: (ffi-call func-ptr (arg-types...) return-type arg1 arg2 ...)
                    "ffi-call" => {
                        if items.len() < 4 {
                            return Err(CompileError::new(
                                "ffi-call expects at least 3 arguments: func-ptr, (arg-types...), return-type, [args...]".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        // Parse arg types from second argument (must be a list)
                        let arg_types = self.parse_ffi_arg_types(&items[2])?;

                        // Parse return type from third argument
                        let return_type = self.parse_ffi_type(&items[3])?;

                        // Check argument count matches
                        let actual_args = items.len() - 4; // Exclude ffi-call, func-ptr, arg-types, return-type
                        if actual_args != arg_types.len() {
                            return Err(CompileError::new(
                                format!(
                                    "ffi-call: expected {} arguments based on arg-types, got {}",
                                    arg_types.len(),
                                    actual_args
                                ),
                                expr.location.clone(),
                            ));
                        }

                        // Compile function pointer expression
                        self.compile_expr(&items[1])?;

                        // Compile all arguments
                        for arg in &items[4..] {
                            self.compile_expr(arg)?;
                        }

                        // Emit FFI call instruction with type info
                        self.emit(Instruction::FfiCall(arg_types, return_type));
                    }

                    // Quasiquote: (quasiquote expr) - like quote but allows unquote and unquote-splicing
                    "quasiquote" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "quasiquote expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_quasiquote(&items[1])?;
                    }

                    // Let: (let ((var val) ...) body)
                    "let" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "let expects exactly 2 arguments: bindings and body".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        self.compile_let(&items[1], &items[2])?;
                    }

                    // Loop: (loop [bindings] body)
                    "loop" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "loop expects exactly 2 arguments: bindings and body".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        self.compile_loop(&items[1], &items[2])?;
                    }

                    // Recur: (recur new-values...)
                    "recur" => {
                        if items.len() < 1 {
                            return Err(CompileError::new(
                                "recur expects at least 0 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }

                        self.compile_recur(&items[1..])?;
                    }

                    // Lambda: (lambda (params) body)
                    "lambda" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "lambda expects exactly 2 arguments: parameters and body".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.compile_lambda(&items[1], &items[2])?;
                    }

                    // List operations
                    "cons" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "cons expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::Cons);
                        self.in_tail_position = saved_tail;
                    }
                    "car" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "car expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Car);
                        self.in_tail_position = saved_tail;
                    }
                    "cdr" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "cdr expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::Cdr);
                        self.in_tail_position = saved_tail;
                    }
                    "list?" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "list? expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::IsList);
                        self.in_tail_position = saved_tail;
                    }

                    // String/Symbol operations
                    "string?" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "string? expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::IsString);
                        self.in_tail_position = saved_tail;
                    }
                    "symbol?" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "symbol? expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::IsSymbol);
                        self.in_tail_position = saved_tail;
                    }
                    "symbol->string" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "symbol->string expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::SymbolToString);
                        self.in_tail_position = saved_tail;
                    }
                    "string->symbol" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "string->symbol expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::StringToSymbol);
                        self.in_tail_position = saved_tail;
                    }
                    "string-length" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "string-length expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::StringLength);
                        self.in_tail_position = saved_tail;
                    }
                    "substring" => {
                        if items.len() != 4 {
                            return Err(CompileError::new(
                                "substring expects exactly 3 arguments (string, start, end)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?; // string
                        self.compile_expr(&items[2])?; // start
                        self.compile_expr(&items[3])?; // end
                        self.emit(Instruction::Substring);
                        self.in_tail_position = saved_tail;
                    }
                    "string-append" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "string-append expects exactly 2 arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.compile_expr(&items[2])?;
                        self.emit(Instruction::StringAppend);
                        self.in_tail_position = saved_tail;
                    }
                    "string->list" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "string->list expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::StringToList);
                        self.in_tail_position = saved_tail;
                    }
                    "list->string" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "list->string expects exactly 1 argument".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::ListToString);
                        self.in_tail_position = saved_tail;
                    }

                    // File I/O operations
                    "read-file" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "read-file expects exactly 1 argument (path)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::ReadFile);
                        self.in_tail_position = saved_tail;
                    }
                    "write-file" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "write-file expects exactly 2 arguments (path, content)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?; // path
                        self.compile_expr(&items[2])?; // content
                        self.emit(Instruction::WriteFile);
                        self.in_tail_position = saved_tail;
                    }
                    "file-exists?" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "file-exists? expects exactly 1 argument (path)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::FileExists);
                        self.in_tail_position = saved_tail;
                    }

                    // Command-line arguments
                    "get-args" => {
                        if items.len() != 1 {
                            return Err(CompileError::new(
                                "get-args expects no arguments".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        self.emit(Instruction::GetArgs);
                    }
                    "write-binary-file" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "write-binary-file expects exactly 2 arguments (path, bytes)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?; // path
                        self.compile_expr(&items[2])?; // bytes list
                        self.emit(Instruction::WriteBinaryFile);
                        self.in_tail_position = saved_tail;
                    }
                    "char-code" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "char-code expects exactly 1 argument (single-char string)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::CharCode);
                        self.in_tail_position = saved_tail;
                    }

                    // List operations
                    "list-ref" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "list-ref expects exactly 2 arguments (list, index)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?; // list
                        self.compile_expr(&items[2])?; // index
                        self.emit(Instruction::ListRef);
                        self.in_tail_position = saved_tail;
                    }
                    "list-length" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "list-length expects exactly 1 argument (list)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::ListLength);
                        self.in_tail_position = saved_tail;
                    }
                    "append" => {
                        if items.len() != 3 {
                            return Err(CompileError::new(
                                "append expects exactly 2 arguments (list1, list2)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?; // first list
                        self.compile_expr(&items[2])?; // second list
                        self.emit(Instruction::Append);
                        self.in_tail_position = saved_tail;
                    }

                    // Number operations
                    "number->string" => {
                        if items.len() != 2 {
                            return Err(CompileError::new(
                                "number->string expects exactly 1 argument (integer)".to_string(),
                                expr.location.clone(),
                            ));
                        }
                        let saved_tail = self.in_tail_position;
                        self.in_tail_position = false;
                        self.compile_expr(&items[1])?;
                        self.emit(Instruction::NumberToString);
                        self.in_tail_position = saved_tail;
                    }

                    // User-defined function call or closure variable or macro
                    _ => {
                        // Check if it's a macro
                        if let Some(macro_def) = self.macros.get(operator).cloned() {
                            // It's a macro - expand it at compile time
                            let args: Vec<SourceExpr> = items[1..].to_vec();
                            let expanded = self.expand_macro(&macro_def, &args)?;
                            // Compile the expanded expression
                            self.compile_expr(&expanded)?;
                        } else {
                            // Check if operator is a variable (could be a closure)
                            let is_variable = self.local_bindings.contains_key(operator)
                                || self.pattern_bindings.contains_key(operator)
                                || self.param_names.contains(operator);

                            if is_variable {
                                // It's a variable - load it as a closure and use CallClosure
                                let saved_tail = self.in_tail_position;

                                // Closure and arguments are not in tail position
                                self.in_tail_position = false;
                                self.compile_variable_load(operator)?;

                                // Compile all arguments
                                let arg_count = items.len() - 1;
                                for i in 1..items.len() {
                                    self.compile_expr(&items[i])?;
                                }

                                // Call the closure
                                self.emit(Instruction::CallClosure(arg_count));

                                self.in_tail_position = saved_tail;
                            } else {
                                // It's a regular function call
                                let arg_count = items.len() - 1;
                                let is_tail_call = self.in_tail_position;

                                // Arguments are not in tail position
                                self.in_tail_position = false;
                                for i in 1..items.len() {
                                    self.compile_expr(&items[i])?;
                                }

                                // Resolve the function name:
                                // 1. Check for imported symbol alias
                                // 2. If in a module and no "/" in name, try module-local first
                                // 3. Otherwise use the operator as-is (may be qualified like "math/add")
                                let resolved_name = self.resolve_function_name(operator);

                                // Emit TailCall if in tail position, otherwise Call
                                if is_tail_call {
                                    self.emit(Instruction::TailCall(resolved_name, arg_count));
                                } else {
                                    self.emit(Instruction::Call(resolved_name, arg_count));
                                }

                                // Restore tail position
                                self.in_tail_position = is_tail_call;
                            }
                        }
                    }
                }
            } else {
                // Non-symbol operator - should be a closure expression
                // Compile the operator expression (should produce a closure)
                self.compile_expr(&items[0])?;

                // Compile all arguments
                let arg_count = items.len() - 1;
                for i in 1..items.len() {
                    self.compile_expr(&items[i])?;
                }

                // Call the closure
                self.emit(Instruction::CallClosure(arg_count));
            }
            }
        }

        Ok(start_address)
    }

    // Convert a SourceExpr to a runtime Value (for quote)
    fn expr_to_value(&self, expr: &SourceExpr) -> Result<Value, CompileError> {
        match &expr.expr {
            LispExpr::Number(n) => Ok(Value::Integer(*n)),
            LispExpr::Float(f) => Ok(Value::Float(*f)),
            LispExpr::Boolean(b) => Ok(Value::Boolean(*b)),
            LispExpr::Symbol(s) => {
                // Symbols in quoted expressions become Symbol values
                Ok(Value::Symbol(Arc::new(s.clone())))
            }
            LispExpr::List(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.expr_to_value(item)?);
                }
                Ok(Value::List(List::from_vec(values)))
            }
            LispExpr::DottedList(items, rest) => {
                // '(a b . rest) - cons a and b onto rest
                let rest_value = self.expr_to_value(rest)?;

                // Rest must be a list
                if let Value::List(rest_list) = rest_value {
                    // Prepend items to rest_list (from back to front)
                    let mut result = rest_list;
                    for item in items.iter().rev() {
                        result = List::cons(self.expr_to_value(item)?, result);
                    }
                    Ok(Value::List(result))
                } else {
                    Err(CompileError::new(
                        "Rest of dotted list must be a list".to_string(),
                        rest.location.clone(),
                    ))
                }
            }
        }
    }

    fn compile_def(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => {
                return Err(CompileError::new(
                    "def expects a list".to_string(),
                    expr.location.clone(),
                ));
            }
        };

        // Check length: (def name value)
        if items.len() != 3 {
            return Err(CompileError::new(
                "def expects exactly: (def name value)".to_string(),
                expr.location.clone(),
            ));
        }

// ==================== COMPILER CORE ====================


        // Extract variable name
        let var_name = match &items[1].expr {
            LispExpr::Symbol(s) => s.clone(),
            _ => {
                return Err(CompileError::new(
                    "Variable name must be a symbol".to_string(),
                    items[1].location.clone(),
                ));
            }
        };

        // Qualify with module name if in a module
        let qualified_name = self.qualify_name(&var_name);

        // Enforce immutability - no redefinition allowed
        if self.global_vars.contains_key(&qualified_name) {
            return Err(CompileError::new(
                format!("Cannot redefine constant '{}' - all bindings are immutable", qualified_name),
                items[1].location.clone(),
            ));
        }

        // Register as immutable global variable (false = immutable)
        self.global_vars.insert(qualified_name.clone(), false);

        // Compile the value expression
        self.compile_expr(&items[2])?;

        // Emit StoreGlobal to store the value
        self.emit(Instruction::StoreGlobal(qualified_name));

        Ok(())
    }

    fn compile_defun(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => {
                return Err(CompileError::new(
                    "defun expects a list".to_string(),
                    expr.location.clone(),
                ));
            }
        };

        // Check minimum length (defun name ...)
        if items.len() < 3 {
            return Err(CompileError::new(
                "defun expects at least: (defun name clause)".to_string(),
                expr.location.clone(),
            ));
        }

        // Check first element is "defun"
        match &items[0].expr {
            LispExpr::Symbol(s) if s == "defun" => {}
            _ => {
                return Err(CompileError::new(
                    "First element must be 'defun'".to_string(),
                    items[0].location.clone(),
                ));
            }
        }

        // Extract function name
        let fn_name = match &items[1].expr {
            LispExpr::Symbol(s) => s.clone(),
            _ => {
                return Err(CompileError::new(
                    "Function name must be a symbol".to_string(),
                    items[1].location.clone(),
                ));
            }
        };

        // Determine if this is a multi-clause or single-clause defun
        // Multi-clause: (defun name ((pattern) body) ((pattern) body) ...)
        // Single-clause: (defun name (params) body)
        //
        // Heuristic: if items.len() == 4 and items[2] is a list/dotted-list that looks
        // like parameters (only contains symbols), it's single-clause.
        // Otherwise, if items[2] looks like a clause (a list starting with a list), it's multi-clause.

        if items.len() == 4 && self.looks_like_param_list(&items[2]) {
            // Single-clause defun: (defun name (params) body)
            self.compile_single_clause_defun(&fn_name, &items[2], &items[3])
        } else {
            // Multi-clause defun: (defun name clause1 clause2 ...)
            let clauses = &items[2..];
            self.compile_multi_clause_defun(&fn_name, clauses, &items[0].location)
        }
    }

    // Check if an expression looks like a parameter list (only contains symbols)
    fn looks_like_param_list(&self, expr: &SourceExpr) -> bool {
        match &expr.expr {
            LispExpr::List(params) => {
                // Check for special case (. rest) - this is a valid param list
                if params.len() == 2 {
                    if let LispExpr::Symbol(s) = &params[0].expr {
                        if s == "." {
                            return true;
                        }
                    }
                }
                // Regular param list: all symbols
                params.iter().all(|p| matches!(&p.expr, LispExpr::Symbol(_)))
            }
            LispExpr::DottedList(head, tail) => {
                // Dotted list: (a b . rest) - all should be symbols
                head.iter().all(|p| matches!(&p.expr, LispExpr::Symbol(_)))
                    && matches!(&tail.expr, LispExpr::Symbol(_))
            }
            _ => false,
        }
    }

    // Parse parameter list, detecting variadic syntax (a b . rest)
    fn parse_params(params_expr: &SourceExpr) -> Result<ParsedParams, CompileError> {
        match &params_expr.expr {
            // Dotted list: (a b . rest) - parser already separated them for us
            LispExpr::DottedList(required_params, rest_param) => {
                let mut required = Vec::new();
                for param in required_params {
                    match &param.expr {
                        LispExpr::Symbol(s) => required.push(s.clone()),
                        _ => {
                            return Err(CompileError::new(
                                "Parameter must be a symbol".to_string(),
                                param.location.clone(),
                            ));
                        }
                    }
                }

                // Extract rest parameter name
                let rest = match &rest_param.expr {
                    LispExpr::Symbol(s) => Some(s.clone()),
                    _ => {
                        return Err(CompileError::new(
                            "Rest parameter must be a symbol".to_string(),
                            rest_param.location.clone(),
                        ));
                    }
                };

                Ok(ParsedParams { required, rest })
            }

            // Regular list: (a b c) or special case (. rest) for zero required params
            LispExpr::List(params) => {
                // Special case: (. rest) for zero required parameters
                if params.len() == 2 {
                    if let LispExpr::Symbol(s) = &params[0].expr {
                        if s == "." {
                            // This is (. rest) syntax
                            match &params[1].expr {
                                LispExpr::Symbol(rest_name) => {
                                    return Ok(ParsedParams {
                                        required: Vec::new(),
                                        rest: Some(rest_name.clone()),
                                    });
                                }
                                _ => {
                                    return Err(CompileError::new(
                                        "Rest parameter must be a symbol".to_string(),
                                        params[1].location.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }

                // Regular parameter list
                let mut required = Vec::new();
                for param in params {
                    match &param.expr {
                        LispExpr::Symbol(s) => required.push(s.clone()),
                        _ => {
                            return Err(CompileError::new(
                                "Parameter must be a symbol".to_string(),
                                param.location.clone(),
                            ));
                        }
                    }
                }

                Ok(ParsedParams { required, rest: None })
            }

            _ => {
                Err(CompileError::new(
                    "Parameters must be a list".to_string(),
                    params_expr.location.clone(),
                ))
            }
        }
    }

    // Compile old-style single-clause defun
    fn compile_single_clause_defun(
        &mut self,
        fn_name: &str,
        params_expr: &SourceExpr,
        body_expr: &SourceExpr,
    ) -> Result<(), CompileError> {
        // Parse parameters (handles both regular and variadic)
        let parsed_params = Self::parse_params(params_expr)?;

        // Build complete param list for compilation context
        let mut all_params = parsed_params.required.clone();
        if let Some(ref rest_name) = parsed_params.rest {
            all_params.push(rest_name.clone());
        }

        // Save current compilation context
        let saved_bytecode = std::mem::take(&mut self.bytecode);
        let saved_params = std::mem::take(&mut self.param_names);
        let saved_address = self.instruction_address;
        let saved_tail_position = self.in_tail_position;

        // Set up new context for function
        self.bytecode = Vec::new();
        self.param_names = all_params;
        self.instruction_address = 0;
        self.in_tail_position = true; // Function body is in tail position

        // If variadic, emit PackRestArgs at the start of function
        if parsed_params.rest.is_some() {
            self.emit(Instruction::PackRestArgs(parsed_params.required.len()));
        }

        // Compile function body
        self.compile_expr(body_expr)?;

        // Emit return instruction
        self.emit(Instruction::Ret);

        // Store compiled function (qualified with module name if in a module)
        let fn_bytecode = std::mem::take(&mut self.bytecode);
        let qualified_name = self.qualify_name(fn_name);
        self.functions.insert(qualified_name, fn_bytecode);

        // Restore context
        self.bytecode = saved_bytecode;
        self.param_names = saved_params;
        self.instruction_address = saved_address;
        self.in_tail_position = saved_tail_position;

        Ok(())
    }

// ==================== PATTERN MATCHING FOR DEFUN ====================

    // Compile multi-clause defun with pattern matching
    fn compile_multi_clause_defun(
        &mut self,
        fn_name: &str,
        clauses: &[SourceExpr],
        location: &Location,
    ) -> Result<(), CompileError> {
        // Parse all clauses first to validate them
        let parsed_clauses: Vec<FunctionClause> = clauses
            .iter()
            .map(|c| self.parse_clause(c))
            .collect::<Result<Vec<_>, _>>()?;

        if parsed_clauses.is_empty() {
            return Err(CompileError::new(
                "defun requires at least one clause".to_string(),
                location.clone(),
            ));
        }

        // Support multi-arity functions: find the maximum arity across all clauses
        // The function will accept up to max_arity arguments, and each clause
        // will check if the actual argument count matches its expected arity
        let max_arity = parsed_clauses.iter()
            .map(|clause| clause.patterns.len())
            .max()
            .unwrap_or(0);

        // Collect variable names from all patterns to build param_names
        // For pattern matching, we use synthetic parameter names based on position
        let param_names: Vec<String> = (0..max_arity).map(|i| format!("__arg{}", i)).collect();

        // Save current compilation context
        let saved_bytecode = std::mem::take(&mut self.bytecode);
        let saved_params = std::mem::take(&mut self.param_names);
        let saved_address = self.instruction_address;
        let saved_tail_position = self.in_tail_position;
        let saved_local_bindings = std::mem::take(&mut self.local_bindings);
        let saved_stack_depth = self.stack_depth;

        // Set up new context for function
        self.bytecode = Vec::new();
        self.param_names = param_names;
        self.instruction_address = 0;
        self.stack_depth = 0;

        // Compile pattern matching dispatch
        // Structure:
        // clause_0:
        //   CheckArity(expected_arity_0, clause_1)  # Jump if arg count doesn't match
        //   <pattern checks for clause 0>
        //   JmpIfFalse(clause_1)
        //   <bind variables for clause 0>
        //   <body for clause 0>
        //   Ret
        // clause_1:
        //   CheckArity(expected_arity_1, clause_2)  # Jump if arg count doesn't match
        //   <pattern checks for clause 1>
        //   JmpIfFalse(clause_2)
        //   ...
        // no_match:
        //   <error: no matching clause>

        let num_clauses = parsed_clauses.len();
        let mut clause_addresses: Vec<usize> = Vec::with_capacity(num_clauses + 1);

        for (clause_idx, clause) in parsed_clauses.iter().enumerate() {
            clause_addresses.push(self.instruction_address);

            // Save bindings for this clause
            self.local_bindings.clear();
            self.stack_depth = 0;

            // Get the arity for this specific clause
            let clause_arity = clause.patterns.len();

            // Emit CheckArity instruction: if argument count doesn't match, jump to next clause
            // We'll patch this jump address after we know where the next clause starts
            let arity_check_idx = self.bytecode.len();
            self.emit(Instruction::CheckArity(clause_arity, 0)); // placeholder jump address

            // Compile pattern checks for this clause
            // If any pattern fails, jump to next clause
            self.pattern_match_jumps.clear();
            let _jump_count = self.compile_pattern_checks(&clause.patterns, clause_arity)?;

            // Save the jump indices to patch later (both arity check and pattern checks)
            let mut jumps_to_patch: Vec<usize> = vec![arity_check_idx];
            jumps_to_patch.extend(self.pattern_match_jumps.clone());

            // All patterns matched! Bind variables from patterns
            self.bind_pattern_variables(&clause.patterns, clause_arity)?;

            // Compile the body in tail position
            self.in_tail_position = true;
            self.compile_expr(&clause.body)?;

            // Clean up any stack values from pattern bindings
            if self.stack_depth > 0 {
                self.emit(Instruction::Slide(self.stack_depth));
            }

            // Return
            self.emit(Instruction::Ret);

            // Patch all jump addresses to point to the next clause (or error)
            let target = self.instruction_address;
            for jump_idx in jumps_to_patch {
                self.patch_jump(jump_idx, target);
            }

            // If this is the last clause, emit error handler
            if clause_idx == num_clauses - 1 {
                // Emit error for no matching clause
                self.emit(Instruction::Push(Value::String(Arc::new(
                    format!("No matching clause in function '{}'", fn_name)
                ))));
                self.emit(Instruction::Print);
                self.emit(Instruction::Halt);
            }
        }

        // Store compiled function (qualified with module name if in a module)
        let fn_bytecode = std::mem::take(&mut self.bytecode);
        let qualified_name = self.qualify_name(fn_name);
        self.functions.insert(qualified_name, fn_bytecode);

        // Restore context
        self.bytecode = saved_bytecode;
        self.param_names = saved_params;
        self.instruction_address = saved_address;
        self.in_tail_position = saved_tail_position;
        self.local_bindings = saved_local_bindings;
        self.stack_depth = saved_stack_depth;

        Ok(())
    }

    // Compile pattern checks for all patterns in a clause
    // Returns the number of patterns checked
    fn compile_pattern_checks(&mut self, patterns: &[Pattern], _arity: usize) -> Result<usize, CompileError> {
        // For each pattern, generate check code
        // If any check fails, jump to the next clause
        // We'll collect all jump addresses and patch them to point to the same target

        for (arg_idx, pattern) in patterns.iter().enumerate() {
            // Compile check for this argument's pattern
            // This will emit JmpIfFalse instructions for failures
            self.compile_pattern_check_for_arg(pattern, arg_idx)?;
        }

        Ok(patterns.len())
    }

    // Compile check for a pattern against a specific argument
    // Emits JmpIfFalse for failure conditions, which get collected in pattern_match_jumps
    fn compile_pattern_check_for_arg(&mut self, pattern: &Pattern, arg_idx: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(_) | Pattern::Wildcard => {
                // Always matches - no check needed
            }
            Pattern::Literal(value) => {
                // Load argument and check equality
                self.emit(Instruction::LoadArg(arg_idx));
                self.emit(Instruction::Push(value.clone()));
                self.emit(Instruction::Eq);
                let jump_idx = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(jump_idx);
            }
            Pattern::QuotedSymbol(s) => {
                // Load argument and check equality with symbol
                self.emit(Instruction::LoadArg(arg_idx));
                self.emit(Instruction::Push(Value::Symbol(Arc::new(s.clone()))));
                self.emit(Instruction::Eq);
                let jump_idx = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(jump_idx);
            }
            Pattern::EmptyList => {
                // Load argument and check equality with empty list
                self.emit(Instruction::LoadArg(arg_idx));
                self.emit(Instruction::Push(Value::List(List::Nil)));
                self.emit(Instruction::Eq);
                let jump_idx = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(jump_idx);
            }
            Pattern::List(sub_patterns) => {
                // Check if it's a list with the right length
                // 1. Check it's a list
                self.emit(Instruction::LoadArg(arg_idx));
                self.emit(Instruction::IsList);
                let not_list_jump = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(not_list_jump);

                // 2. Check length
                self.emit(Instruction::LoadArg(arg_idx));
                self.emit(Instruction::ListLength);
                self.emit(Instruction::Push(Value::Integer(sub_patterns.len() as i64)));
                self.emit(Instruction::Eq);
                let wrong_len_jump = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(wrong_len_jump);

                // 3. Check each element
                for (elem_idx, sub_pattern) in sub_patterns.iter().enumerate() {
                    self.compile_pattern_check_for_list_element(sub_pattern, arg_idx, elem_idx)?;
                }
            }
            Pattern::DottedList(head_patterns, tail_pattern) => {
                // Check if it's a list with at least head_patterns.len() elements
                // 1. Check it's a list
                self.emit(Instruction::LoadArg(arg_idx));
                self.emit(Instruction::IsList);
                let not_list_jump = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(not_list_jump);

                // 2. Check length is at least head_patterns.len()
                if !head_patterns.is_empty() {
                    self.emit(Instruction::LoadArg(arg_idx));
                    self.emit(Instruction::ListLength);
                    self.emit(Instruction::Push(Value::Integer(head_patterns.len() as i64)));
                    self.emit(Instruction::Gte); // length >= required
                    let too_short_jump = self.instruction_address;
                    self.emit(Instruction::JmpIfFalse(0));
                    self.pattern_match_jumps.push(too_short_jump);
                }

                // 3. Check each head element
                for (elem_idx, sub_pattern) in head_patterns.iter().enumerate() {
                    self.compile_pattern_check_for_list_element(sub_pattern, arg_idx, elem_idx)?;
                }

                // 4. Check the tail pattern (rest of the list)
                self.compile_pattern_check_for_list_tail(tail_pattern, arg_idx, head_patterns.len())?;
            }
        }
        Ok(())
    }

    // Compile check for a pattern against a list element
    fn compile_pattern_check_for_list_element(&mut self, pattern: &Pattern, arg_idx: usize, elem_idx: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(_) | Pattern::Wildcard => {
                // Always matches - no check needed
            }
            Pattern::Literal(value) => {
                // Load element and check equality
                self.emit(Instruction::LoadArg(arg_idx));
                for _ in 0..elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                self.emit(Instruction::Push(value.clone()));
                self.emit(Instruction::Eq);
                let jump_idx = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(jump_idx);
            }
            Pattern::QuotedSymbol(s) => {
                self.emit(Instruction::LoadArg(arg_idx));
                for _ in 0..elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                self.emit(Instruction::Push(Value::Symbol(Arc::new(s.clone()))));
                self.emit(Instruction::Eq);
                let jump_idx = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(jump_idx);
            }
            Pattern::EmptyList => {
                self.emit(Instruction::LoadArg(arg_idx));
                for _ in 0..elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                self.emit(Instruction::Push(Value::List(List::Nil)));
                self.emit(Instruction::Eq);
                let jump_idx = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(jump_idx);
            }
            Pattern::List(_) | Pattern::DottedList(_, _) => {
                // Nested complex patterns - would need recursive handling
                // For now, skip the check (will always succeed structurally)
                // The binding phase will still extract the values
            }
        }
        Ok(())
    }

    // Compile check for a pattern against a list tail (rest after skipping elements)
    fn compile_pattern_check_for_list_tail(&mut self, pattern: &Pattern, arg_idx: usize, skip_count: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(_) | Pattern::Wildcard => {
                // Always matches - no check needed
            }
            Pattern::EmptyList => {
                // Check that the rest is empty
                self.emit(Instruction::LoadArg(arg_idx));
                for _ in 0..skip_count {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Push(Value::List(List::Nil)));
                self.emit(Instruction::Eq);
                let jump_idx = self.instruction_address;
                self.emit(Instruction::JmpIfFalse(0));
                self.pattern_match_jumps.push(jump_idx);
            }
            _ => {
                // Other patterns as tail - skip for now
            }
        }
        Ok(())
    }

    // Patch a JmpIfFalse, Jmp, or CheckArity instruction with the correct target address
    fn patch_jump(&mut self, idx: usize, target: usize) {
        match &mut self.bytecode[idx] {
            Instruction::JmpIfFalse(addr) => *addr = target,
            Instruction::Jmp(addr) => *addr = target,
            Instruction::CheckArity(_, addr) => *addr = target,
            _ => panic!("Expected jump instruction at index {}", idx),
        }
    }

    // Bind variables from patterns to their locations
    fn bind_pattern_variables(&mut self, patterns: &[Pattern], _arity: usize) -> Result<(), CompileError> {
        for (arg_idx, pattern) in patterns.iter().enumerate() {
            self.bind_pattern_variable(pattern, arg_idx)?;
        }
        Ok(())
    }

    // Bind variables from a single pattern
    fn bind_pattern_variable(&mut self, pattern: &Pattern, arg_idx: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(name) => {
                // Bind variable to argument position
                // We'll track this in param_names and use LoadArg
                // Actually, param_names is for positional params
                // For pattern variables, we need to use local_bindings

                // Load the argument onto the stack and bind the variable to that stack position
                self.emit(Instruction::LoadArg(arg_idx));
                let stack_pos = self.stack_depth;
                self.stack_depth += 1;
                self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
            }
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::QuotedSymbol(_) | Pattern::EmptyList => {
                // No binding needed
            }
            Pattern::List(sub_patterns) => {
                // Bind sub-pattern variables
                // Each sub-pattern corresponds to an element of the list
                for (elem_idx, sub_pattern) in sub_patterns.iter().enumerate() {
                    self.bind_nested_pattern_variable(sub_pattern, arg_idx, elem_idx)?;
                }
            }
            Pattern::DottedList(head_patterns, tail_pattern) => {
                // Bind head pattern variables
                for (elem_idx, sub_pattern) in head_patterns.iter().enumerate() {
                    self.bind_nested_pattern_variable(sub_pattern, arg_idx, elem_idx)?;
                }
                // Bind tail pattern variable
                self.bind_tail_pattern_variable(tail_pattern, arg_idx, head_patterns.len())?;
            }
        }
        Ok(())
    }

    // Bind a variable from a nested pattern (element of a list)
    fn bind_nested_pattern_variable(&mut self, pattern: &Pattern, arg_idx: usize, elem_idx: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(name) => {
                // Load the argument, extract the element, and bind
                self.emit(Instruction::LoadArg(arg_idx));
                // Navigate to element using cdr/car
                for _ in 0..elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                let stack_pos = self.stack_depth;
                self.stack_depth += 1;
                self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
            }
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::QuotedSymbol(_) | Pattern::EmptyList => {
                // No binding needed
            }
            Pattern::List(sub_patterns) => {
                // Nested list pattern - recursively bind variables
                // For each sub-pattern in the nested list, we need to bind its variables
                for (sub_elem_idx, sub_pattern) in sub_patterns.iter().enumerate() {
                    self.bind_deeply_nested_pattern_variable(sub_pattern, arg_idx, elem_idx, sub_elem_idx)?;
                }
            }
            Pattern::DottedList(head_patterns, tail_pattern) => {
                // Nested dotted list pattern - recursively bind variables
                // Bind head pattern variables
                for (sub_elem_idx, sub_pattern) in head_patterns.iter().enumerate() {
                    self.bind_deeply_nested_pattern_variable(sub_pattern, arg_idx, elem_idx, sub_elem_idx)?;
                }
                // Bind tail pattern variable
                self.bind_deeply_nested_tail_pattern_variable(tail_pattern, arg_idx, elem_idx, head_patterns.len())?;
            }
        }
        Ok(())
    }

    // Bind the tail of a dotted list pattern
    fn bind_tail_pattern_variable(&mut self, pattern: &Pattern, arg_idx: usize, skip_count: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(name) => {
                // Load the argument, skip elements, bind the rest
                self.emit(Instruction::LoadArg(arg_idx));
                for _ in 0..skip_count {
                    self.emit(Instruction::Cdr);
                }
                let stack_pos = self.stack_depth;
                self.stack_depth += 1;
                self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
            }
            Pattern::Wildcard | Pattern::EmptyList => {
                // No binding needed
            }
            Pattern::List(sub_patterns) => {
                // Nested list pattern as tail - recursively bind variables
                // For each sub-pattern, emit full navigation path
                for (sub_elem_idx, sub_pattern) in sub_patterns.iter().enumerate() {
                    match sub_pattern {
                        Pattern::Variable(name) => {
                            self.emit(Instruction::LoadArg(arg_idx));
                            for _ in 0..skip_count {
                                self.emit(Instruction::Cdr);
                            }
                            // Navigate to element in nested list
                            for _ in 0..sub_elem_idx {
                                self.emit(Instruction::Cdr);
                            }
                            self.emit(Instruction::Car);
                            let stack_pos = self.stack_depth;
                            self.stack_depth += 1;
                            self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
                        }
                        _ => {
                            // More complex nested patterns - skip for now
                        }
                    }
                }
            }
            Pattern::DottedList(head_patterns, tail_pattern) => {
                // Nested dotted list pattern as tail - recursively bind variables
                // Bind head patterns
                for (sub_elem_idx, sub_pattern) in head_patterns.iter().enumerate() {
                    match sub_pattern {
                        Pattern::Variable(name) => {
                            self.emit(Instruction::LoadArg(arg_idx));
                            for _ in 0..skip_count {
                                self.emit(Instruction::Cdr);
                            }
                            // Navigate to element in nested list
                            for _ in 0..sub_elem_idx {
                                self.emit(Instruction::Cdr);
                            }
                            self.emit(Instruction::Car);
                            let stack_pos = self.stack_depth;
                            self.stack_depth += 1;
                            self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
                        }
                        _ => {
                            // More complex nested patterns - skip for now
                        }
                    }
                }

                // Bind tail pattern
                match tail_pattern.as_ref() {
                    Pattern::Variable(name) => {
                        self.emit(Instruction::LoadArg(arg_idx));
                        for _ in 0..skip_count {
                            self.emit(Instruction::Cdr);
                        }
                        // Skip head elements
                        for _ in 0..head_patterns.len() {
                            self.emit(Instruction::Cdr);
                        }
                        let stack_pos = self.stack_depth;
                        self.stack_depth += 1;
                        self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
                    }
                    _ => {
                        // Complex tail patterns - skip for now
                    }
                }
            }
            _ => {
                // Other patterns as tail - literals, quoted symbols
                // No binding needed
            }
        }
        Ok(())
    }

    // Bind a deeply nested pattern variable (pattern inside a pattern inside a list element)
    // Example: for ((((x . _) . _)) ...), we need to navigate multiple levels deep
    fn bind_deeply_nested_pattern_variable(&mut self, pattern: &Pattern, arg_idx: usize, elem_idx: usize, sub_elem_idx: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(name) => {
                // Load the argument, navigate to outer element, then to inner element
                self.emit(Instruction::LoadArg(arg_idx));
                // Navigate to outer element
                for _ in 0..elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                // Navigate to inner element
                for _ in 0..sub_elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                let stack_pos = self.stack_depth;
                self.stack_depth += 1;
                self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
            }
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::QuotedSymbol(_) | Pattern::EmptyList => {
                // No binding needed
            }
            Pattern::List(sub_patterns) => {
                // Even more deeply nested - need to go another level
                for (sub_sub_elem_idx, sub_pattern) in sub_patterns.iter().enumerate() {
                    self.bind_triple_nested_pattern_variable(sub_pattern, arg_idx, elem_idx, sub_elem_idx, sub_sub_elem_idx)?;
                }
            }
            Pattern::DottedList(head_patterns, tail_pattern) => {
                // Even more deeply nested dotted list
                for (sub_sub_elem_idx, sub_pattern) in head_patterns.iter().enumerate() {
                    self.bind_triple_nested_pattern_variable(sub_pattern, arg_idx, elem_idx, sub_elem_idx, sub_sub_elem_idx)?;
                }
                self.bind_triple_nested_tail_pattern_variable(tail_pattern, arg_idx, elem_idx, sub_elem_idx, head_patterns.len())?;
            }
        }
        Ok(())
    }

    // Bind the tail of a deeply nested dotted list pattern
    fn bind_deeply_nested_tail_pattern_variable(&mut self, pattern: &Pattern, arg_idx: usize, elem_idx: usize, skip_count: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(name) => {
                // Load the argument, navigate to outer element, then skip inner elements
                self.emit(Instruction::LoadArg(arg_idx));
                // Navigate to outer element
                for _ in 0..elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                // Skip inner elements
                for _ in 0..skip_count {
                    self.emit(Instruction::Cdr);
                }
                let stack_pos = self.stack_depth;
                self.stack_depth += 1;
                self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
            }
            Pattern::Wildcard | Pattern::EmptyList => {
                // No binding needed
            }
            _ => {
                // Complex nested tail patterns - would need even more recursion
                // Skip for now
            }
        }
        Ok(())
    }

    // Bind a triple nested pattern variable (for very deep nesting)
    fn bind_triple_nested_pattern_variable(&mut self, pattern: &Pattern, arg_idx: usize, elem_idx: usize, sub_elem_idx: usize, sub_sub_elem_idx: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(name) => {
                // Load the argument and navigate through three levels
                self.emit(Instruction::LoadArg(arg_idx));
                // Navigate to first level
                for _ in 0..elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                // Navigate to second level
                for _ in 0..sub_elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                // Navigate to third level
                for _ in 0..sub_sub_elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                let stack_pos = self.stack_depth;
                self.stack_depth += 1;
                self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
            }
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::QuotedSymbol(_) | Pattern::EmptyList => {
                // No binding needed
            }
            _ => {
                // Even deeper nesting - we'll stop here for practical purposes
                // Most real-world code won't need more than 3 levels of nesting
            }
        }
        Ok(())
    }

    // Bind the tail of a triple nested dotted list pattern
    fn bind_triple_nested_tail_pattern_variable(&mut self, pattern: &Pattern, arg_idx: usize, elem_idx: usize, sub_elem_idx: usize, skip_count: usize) -> Result<(), CompileError> {
        match pattern {
            Pattern::Variable(name) => {
                // Load the argument and navigate through levels, then skip
                self.emit(Instruction::LoadArg(arg_idx));
                // Navigate to first level
                for _ in 0..elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                // Navigate to second level
                for _ in 0..sub_elem_idx {
                    self.emit(Instruction::Cdr);
                }
                self.emit(Instruction::Car);
                // Skip elements at third level
                for _ in 0..skip_count {
                    self.emit(Instruction::Cdr);
                }
                let stack_pos = self.stack_depth;
                self.stack_depth += 1;
                self.local_bindings.insert(name.clone(), ValueLocation::Local(stack_pos));
            }
            _ => {
                // No binding needed for other patterns
            }
        }
        Ok(())
    }

    // Parse a clause: ((pattern1 pattern2 ...) body)
    fn parse_clause(&self, expr: &SourceExpr) -> Result<FunctionClause, CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => {
                return Err(CompileError::new(
                    "Clause must be a list: ((patterns...) body)".to_string(),
                    expr.location.clone(),
                ));
            }
        };

        if items.len() != 2 {
            return Err(CompileError::new(
                "Clause must have exactly 2 elements: ((patterns...) body)".to_string(),
                expr.location.clone(),
            ));
        }

        // Parse patterns from first element
        let patterns = self.parse_patterns(&items[0])?;
        let body = items[1].clone();

        Ok(FunctionClause { patterns, body })
    }

    // Parse patterns list: (pattern1 pattern2 ...)
    fn parse_patterns(&self, expr: &SourceExpr) -> Result<Vec<Pattern>, CompileError> {
        match &expr.expr {
            LispExpr::List(items) => {
                items.iter().map(|p| self.parse_pattern(p)).collect()
            }
            LispExpr::DottedList(head, tail) => {
                // Dotted list like (a b . rest) - parse head patterns and tail as rest pattern
                let patterns: Vec<Pattern> = head.iter().map(|p| self.parse_pattern(p)).collect::<Result<Vec<_>, _>>()?;
                let rest_pattern = self.parse_pattern(tail)?;
                // This represents a variadic clause - we need to handle this specially
                // For now, we'll create a DottedList pattern for the whole thing
                if patterns.is_empty() {
                    // (. rest) - just a rest parameter
                    Ok(vec![rest_pattern])
                } else {
                    // (a b . rest) - this is tricky. For now, disallow in multi-clause
                    Err(CompileError::new(
                        "Variadic patterns (a b . rest) not yet supported in multi-clause defun. Use single-clause defun with variadic parameters.".to_string(),
                        expr.location.clone(),
                    ))
                }
            }
            _ => {
                Err(CompileError::new(
                    "Patterns must be a list".to_string(),
                    expr.location.clone(),
                ))
            }
        }
    }

    // Parse a single pattern
    fn parse_pattern(&self, expr: &SourceExpr) -> Result<Pattern, CompileError> {
        match &expr.expr {
            // Simple symbol: variable or wildcard
            LispExpr::Symbol(s) => {
                if s == "_" {
                    Ok(Pattern::Wildcard)
                } else {
                    Ok(Pattern::Variable(s.clone()))
                }
            }
            // Number literal
            LispExpr::Number(n) => {
                Ok(Pattern::Literal(Value::Integer(*n)))
            }
            // Float literal
            LispExpr::Float(f) => {
                Ok(Pattern::Literal(Value::Float(*f)))
            }
            // Boolean literal
            LispExpr::Boolean(b) => {
                Ok(Pattern::Literal(Value::Boolean(*b)))
            }
            // List pattern: could be a list pattern or quoted expression
            LispExpr::List(items) => {
                // Check for quote: '() or 'symbol
                if items.len() == 2 {
                    if let LispExpr::Symbol(s) = &items[0].expr {
                        if s == "quote" {
                            return self.parse_quoted_pattern(&items[1]);
                        }
                    }
                }
                // Regular list pattern: (a b c)
                let sub_patterns: Vec<Pattern> = items
                    .iter()
                    .map(|p| self.parse_pattern(p))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Pattern::List(sub_patterns))
            }
            // Dotted list pattern: (h . t)
            LispExpr::DottedList(head, tail) => {
                let head_patterns: Vec<Pattern> = head
                    .iter()
                    .map(|p| self.parse_pattern(p))
                    .collect::<Result<Vec<_>, _>>()?;
                let tail_pattern = self.parse_pattern(tail)?;
                Ok(Pattern::DottedList(head_patterns, Box::new(tail_pattern)))
            }
        }
    }

    // Parse a quoted pattern: 'symbol or '()
    fn parse_quoted_pattern(&self, expr: &SourceExpr) -> Result<Pattern, CompileError> {
        match &expr.expr {
            // 'symbol - quoted symbol
            LispExpr::Symbol(s) => {
                Ok(Pattern::QuotedSymbol(s.clone()))
            }
            // '() - empty list
            LispExpr::List(items) if items.is_empty() => {
                Ok(Pattern::EmptyList)
            }
            // '(1 2 3) - quoted list (treat as literal list pattern)
            LispExpr::List(items) => {
                let sub_patterns: Vec<Pattern> = items
                    .iter()
                    .map(|p| self.parse_quoted_list_element(p))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Pattern::List(sub_patterns))
            }
            _ => {
                Err(CompileError::new(
                    format!("Unsupported quoted pattern: {:?}", expr.expr),
                    expr.location.clone(),
                ))
            }
        }
    }

    // Parse elements within a quoted list (they're all literals)
    fn parse_quoted_list_element(&self, expr: &SourceExpr) -> Result<Pattern, CompileError> {
        match &expr.expr {
            LispExpr::Symbol(s) => Ok(Pattern::QuotedSymbol(s.clone())),
            LispExpr::Number(n) => Ok(Pattern::Literal(Value::Integer(*n))),
            LispExpr::Float(f) => Ok(Pattern::Literal(Value::Float(*f))),
            LispExpr::Boolean(b) => Ok(Pattern::Literal(Value::Boolean(*b))),
            LispExpr::List(items) if items.is_empty() => Ok(Pattern::EmptyList),
            LispExpr::List(items) => {
                let sub_patterns: Vec<Pattern> = items
                    .iter()
                    .map(|p| self.parse_quoted_list_element(p))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Pattern::List(sub_patterns))
            }
            LispExpr::DottedList(head, tail) => {
                let head_patterns: Vec<Pattern> = head
                    .iter()
                    .map(|p| self.parse_quoted_list_element(p))
                    .collect::<Result<Vec<_>, _>>()?;
                let tail_pattern = self.parse_quoted_list_element(tail)?;
                Ok(Pattern::DottedList(head_patterns, Box::new(tail_pattern)))
            }
        }
    }

    // Helper to bind a pattern to a local stack position
    fn bind_pattern_to_local(
        &mut self,
        pattern: &SourceExpr,
        stack_pos: usize,
    ) -> Result<(), CompileError> {
        match &pattern.expr {
            // Simple variable binding
            LispExpr::Symbol(s) if s != "_" => {
                self.local_bindings.insert(s.clone(), ValueLocation::Local(stack_pos));
            }

            // Wildcard - no binding needed
            LispExpr::Symbol(s) if s == "_" => {
                // No binding
            }

            // Destructuring patterns - need to extract components
            LispExpr::List(items) => {
                // Check if this is a quoted pattern
                if items.len() == 2 {
                    if let LispExpr::Symbol(s) = &items[0].expr {
                        if s == "quote" {
                            // Quoted pattern - this would match a literal, not destructure
                            // For now, not supported in let (only in function patterns)
                            return Err(CompileError::new(
                                "Quoted literal patterns not supported in let bindings".to_string(),
                                pattern.location.clone(),
                            ));
                        }
                    }
                }

                // Fixed-length list pattern: (a b c)
                // Extract each element and bind
                for (i, item_pattern) in items.iter().enumerate() {
                    let elem_location = ValueLocation::ListElement(
                        Box::new(ValueLocation::Local(stack_pos)),
                        i
                    );
                    self.bind_pattern_element_to_location(item_pattern, elem_location)?;
                }
            }

            // Dotted list pattern: (h . t)
            LispExpr::DottedList(items, rest) => {
                // Bind head elements
                for (i, item_pattern) in items.iter().enumerate() {
                    let elem_location = ValueLocation::ListElement(
                        Box::new(ValueLocation::Local(stack_pos)),
                        i
                    );
                    self.bind_pattern_element_to_location(item_pattern, elem_location)?;
                }

                // Bind rest
                let rest_location = ValueLocation::ListRest(
                    Box::new(ValueLocation::Local(stack_pos)),
                    items.len()
                );
                self.bind_pattern_element_to_location(rest, rest_location)?;
            }

            _ => {
                return Err(CompileError::new(
                    format!("Unsupported pattern in let binding: {:?}", pattern.expr),
                    pattern.location.clone(),
                ));
            }

// ==================== FUNCTION/LAMBDA COMPILATION ====================

        }

        Ok(())
    }

    // Compile lambda expression: (lambda (params) body)
    fn compile_lambda(
        &mut self,
        params_expr: &SourceExpr,
        body_expr: &SourceExpr,
    ) -> Result<(), CompileError> {
        // Parse parameters (handles both regular and variadic)
        let parsed_params = Self::parse_params(params_expr)?;

        // Build complete param list for compilation context
        let mut all_params = parsed_params.required.clone();
        if let Some(ref rest_name) = parsed_params.rest {
            all_params.push(rest_name.clone());
        }

        // Find free variables in body (variables not in all_params)
        let free_vars = self.find_free_variables(body_expr, &all_params);

        // Save current compilation context
        let saved_bytecode = std::mem::take(&mut self.bytecode);
        let saved_params = std::mem::take(&mut self.param_names);
        let saved_local_bindings = self.local_bindings.clone();
        let saved_pattern_bindings = self.pattern_bindings.clone();
        let saved_address = self.instruction_address;
        let saved_stack_depth = self.stack_depth;
        let saved_tail_position = self.in_tail_position;

        // Set up new context for closure body
        self.bytecode = Vec::new();
        self.param_names = all_params.clone();
        self.instruction_address = 0;
        self.local_bindings.clear();
        self.pattern_bindings.clear();
        self.stack_depth = 0;
        self.in_tail_position = true; // Lambda body is in tail position

        // Set up captured variables as "LoadCaptured" locations
        for (i, var_name) in free_vars.iter().enumerate() {
            self.pattern_bindings.insert(var_name.clone(), ValueLocation::Captured(i));
        }

        // Compile body
        self.compile_expr(body_expr)?;
        self.emit(Instruction::Ret);

        // Get compiled body
        let body_bytecode = std::mem::take(&mut self.bytecode);

        // Restore context
        self.bytecode = saved_bytecode;
        self.param_names = saved_params;
        self.local_bindings = saved_local_bindings;
        self.pattern_bindings = saved_pattern_bindings;
        self.instruction_address = saved_address;
        self.stack_depth = saved_stack_depth;
        self.in_tail_position = saved_tail_position;

        // Emit code to push captured variable values onto stack
        for var_name in &free_vars {
            // Load the value of this free variable
            self.compile_variable_load(var_name)?;
        }

        // Emit appropriate closure instruction based on whether it's variadic
        match parsed_params.rest {
            None => {
                // Regular closure
                self.emit(Instruction::MakeClosure(parsed_params.required, body_bytecode, free_vars.len()));
            }
            Some(rest_name) => {
                // Variadic closure
                self.emit(Instruction::MakeVariadicClosure(parsed_params.required, rest_name, body_bytecode, free_vars.len()));
            }
        }

        Ok(())
    }

    // Find free variables in an expression (variables not in bound_vars)
    fn find_free_variables(&self, expr: &SourceExpr, bound_vars: &[String]) -> Vec<String> {
        let mut free_vars = Vec::new();
        self.collect_free_variables(expr, bound_vars, &mut free_vars);
        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        free_vars.retain(|v| seen.insert(v.clone()));
        free_vars
    }

    // Helper to recursively collect free variables
    fn collect_free_variables(
        &self,
        expr: &SourceExpr,
        bound_vars: &[String],
        free_vars: &mut Vec<String>,
    ) {
        match &expr.expr {
            LispExpr::Symbol(s) => {
                // Check if it's a variable (not a string literal, not bound)
                if !s.starts_with("__STRING__") && !bound_vars.contains(s) {
                    // Check if it's available in current environment
                    if self.local_bindings.contains_key(s)
                        || self.pattern_bindings.contains_key(s)
                        || self.param_names.contains(s)
                    {
                        free_vars.push(s.clone());
                    }
                }
            }
            LispExpr::List(items) => {
                if items.is_empty() {
                    return;
                }

                // Check for special forms that introduce bindings
                if let LispExpr::Symbol(s) = &items[0].expr {
                    match s.as_str() {
                        "let" if items.len() == 3 => {
                            // let introduces new bindings
                            if let LispExpr::List(bindings) = &items[1].expr {
                                let mut new_bound = bound_vars.to_vec();
                                for binding in bindings {
                                    if let LispExpr::List(pair) = &binding.expr {
                                        if pair.len() == 2 {
                                            // Collect free vars from value expression first
                                            self.collect_free_variables(&pair[1], bound_vars, free_vars);
                                            // Then add binding var to bound list
                                            if let LispExpr::Symbol(var) = &pair[0].expr {
                                                new_bound.push(var.clone());
                                            }
                                        }
                                    }
                                }
                                // Collect from body with extended bindings
                                self.collect_free_variables(&items[2], &new_bound, free_vars);
                                return;
                            }
                        }
                        "lambda" if items.len() == 3 => {
                            // lambda introduces new parameters
                            if let LispExpr::List(params) = &items[1].expr {
                                let mut new_bound = bound_vars.to_vec();
                                for param in params {
                                    if let LispExpr::Symbol(p) = &param.expr {
                                        new_bound.push(p.clone());
                                    }
                                }
                                self.collect_free_variables(&items[2], &new_bound, free_vars);
                                return;
                            }
                        }
                        "quote" => {
                            // Quoted expressions don't have free variables
                            return;
                        }
                        _ => {}
                    }
                }

                // Default: recursively process all items
                for item in items {
                    self.collect_free_variables(item, bound_vars, free_vars);
                }
            }
            LispExpr::DottedList(items, rest) => {
                for item in items {
                    self.collect_free_variables(item, bound_vars, free_vars);
                }
                self.collect_free_variables(rest, bound_vars, free_vars);
            }
            _ => {}
        }
    }

    // Helper to load a variable (for capturing)
    fn compile_variable_load(&mut self, var_name: &str) -> Result<(), CompileError> {
        // Check local bindings first
        if let Some(location) = self.local_bindings.get(var_name) {
            let loc = location.clone();
                        loc.emit_load(self);
        } else if let Some(location) = self.pattern_bindings.get(var_name) {
            let loc = location.clone();
                        loc.emit_load(self);
        } else if let Some(idx) = self.param_names.iter().position(|p| p == var_name) {
            self.emit(Instruction::LoadArg(idx));
        } else {
            return Err(CompileError::new(
                format!("Variable '{}' not found for capture", var_name),
                Location::unknown(),
            ));
        }
        Ok(())
    }

    // Helper to bind a pattern element to a specific location
    fn bind_pattern_element_to_location(
        &mut self,
        pattern: &SourceExpr,
        location: ValueLocation,
    ) -> Result<(), CompileError> {
        match &pattern.expr {
            LispExpr::Symbol(s) if s != "_" => {
                self.local_bindings.insert(s.clone(), location);
            }
            LispExpr::Symbol(s) if s == "_" => {

// ==================== QUASIQUOTE COMPILATION ====================

                // Wildcard, no binding
            }
            // Could recursively handle nested patterns here
            _ => {
                return Err(CompileError::new(
                    "Only simple variables and wildcards supported in nested patterns".to_string(),
                    pattern.location.clone(),
                ));
            }
        }
        Ok(())
    }

    // Compile quasiquote expression
    // Quasiquote is like quote, but allows unquote (,) and unquote-splicing (,@)
    fn compile_quasiquote(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        match &expr.expr {
            // Check for unquote: (unquote expr)
            LispExpr::List(items) if items.len() == 2 => {
                if let LispExpr::Symbol(s) = &items[0].expr {
                    if s == "unquote" {
                        // Unquote: evaluate the expression
                        self.compile_expr(&items[1])?;
                        return Ok(());
                    }
                }
                // Not an unquote, process as normal list
                self.compile_quasiquote_list(items)?;
            }

            // Empty list or regular list
            LispExpr::List(items) => {
                self.compile_quasiquote_list(items)?;
            }

            // Dotted list
            LispExpr::DottedList(_items, _rest) => {
                // For dotted lists in quasiquote, we need to handle unquote-splicing specially
                // For now, just quote the whole thing (simplified implementation)
                let value = self.expr_to_value(expr)?;
                self.emit(Instruction::Push(value));
            }

            // Atoms: just quote them
            _ => {
                let value = self.expr_to_value(expr)?;
                self.emit(Instruction::Push(value));
            }
        }

        Ok(())
    }

    // Helper to compile a quasiquoted list
    // Handles unquote-splicing and builds the list at runtime
    fn compile_quasiquote_list(&mut self, items: &[SourceExpr]) -> Result<(), CompileError> {
        if items.is_empty() {
            // Empty list
            self.emit(Instruction::Push(Value::List(List::Nil)));
            return Ok(());
        }

        // Check if we have any unquotes or splicing - if not, we can just quote the whole thing
        let has_unquote_or_splice = items.iter().any(|item| {
            if let LispExpr::List(inner) = &item.expr {
                if inner.len() == 2 {
                    if let LispExpr::Symbol(s) = &inner[0].expr {
                        return s == "unquote" || s == "unquote-splicing";
                    }
                }
            }
            // Recursively check for unquotes in nested expressions
            self.contains_unquote(item)
        });

        if !has_unquote_or_splice {
            // No unquotes at all - just convert to a value and push it
            let value = self.expr_to_value(&SourceExpr::new(
                LispExpr::List(items.to_vec()),
                Location::unknown(),
            ))?;
            self.emit(Instruction::Push(value));
            return Ok(());
        }

        // We have unquotes - need to build the list at runtime
        // Strategy: collect all elements into a vector of code that pushes each element
        // Then build the list using cons operations

        // For simplicity, let's build forward using an explicit loop
        // We'll push all elements onto the stack, then build the list

        let mut elem_count = 0;

        // Check for splicing first
        let has_splicing = items.iter().any(|item| {
            if let LispExpr::List(inner) = &item.expr {
                if inner.len() == 2 {
                    if let LispExpr::Symbol(s) = &inner[0].expr {
                        return s == "unquote-splicing";
                    }
                }
            }
            false
        });

        if has_splicing {
            // Complex case with splicing
            // We'll build the list in forward order differently
            // Collect segments and splice them together

            // Build forward: start with list containing all non-splice elements and splice points
            self.emit(Instruction::Push(Value::List(List::Nil)));

            for item in items.iter() {
                if let LispExpr::List(inner) = &item.expr {
                    if inner.len() == 2 {
                        if let LispExpr::Symbol(s) = &inner[0].expr {
                            if s == "unquote-splicing" {
                                // Evaluate the list to splice and append it
                                self.compile_expr(&inner[1])?;
                                // Stack: [accumulator, splice_list]
                                // We want: [accumulator..., splice_list...]
                                self.emit_append()?;
                                continue;
                            } else if s == "unquote" {
                                // Regular unquote - cons the element
                                self.compile_expr(&inner[1])?;
                                // Stack: [accumulator, elem]
                                // We need to make a single-element list and append
                                self.emit(Instruction::MakeList(1));
                                self.emit_append()?;
                                continue;
                            }
                        }
                    }
                }
                // Regular element - quasiquote it and append as single-element list
                self.compile_quasiquote(item)?;
                self.emit(Instruction::MakeList(1));
                self.emit_append()?;
            }
        } else {
            // No splicing - simpler case
            // Push all elements onto stack, then use MakeList
            for item in items {
                if let LispExpr::List(inner) = &item.expr {
                    if inner.len() == 2 {
                        if let LispExpr::Symbol(s) = &inner[0].expr {
                            if s == "unquote" {
                                self.compile_expr(&inner[1])?;
                                elem_count += 1;
                                continue;
                            }
                        }
                    }
                }
                self.compile_quasiquote(item)?;
                elem_count += 1;
            }

            // Now create a list from the elements on the stack
            self.emit(Instruction::MakeList(elem_count));
        }

        Ok(())
    }

    // Helper to check if an expression contains unquote or unquote-splicing
    fn contains_unquote(&self, expr: &SourceExpr) -> bool {
        match &expr.expr {
            LispExpr::List(items) => {
                if items.len() == 2 {
                    if let LispExpr::Symbol(s) = &items[0].expr {
                        if s == "unquote" || s == "unquote-splicing" {
                            return true;
                        }
                    }
                }
                items.iter().any(|item| self.contains_unquote(item))
            }
            LispExpr::DottedList(items, rest) => {
                items.iter().any(|item| self.contains_unquote(item)) || self.contains_unquote(rest)
            }
            _ => false,
        }
    }

    // Emit code to append two lists (both on stack)
    // Stack before: [... list1 list2]
    // Stack after: [... (append list1 list2)]
    fn emit_append(&mut self) -> Result<(), CompileError> {
        self.emit(Instruction::Append);
        Ok(())
    }


    pub fn compile_program(&mut self, exprs: &[SourceExpr]) -> Result<(HashMap<String, Vec<Instruction>>, Vec<Instruction>), CompileError> {
        // First pass: compile all defun, defmacro, def, module, and import expressions
        for expr in exprs {
            if let LispExpr::List(items) = &expr.expr {
                if let Some(first) = items.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        if s == "defun" {
                            self.compile_defun(expr)?;
                        } else if s == "defmacro" {
                            self.compile_defmacro(expr)?;
                        } else if s == "def" {
                            self.compile_def(expr)?;
                        } else if s == "module" {
                            self.compile_module(expr)?;
                        } else if s == "import" {
                            self.compile_import(expr)?;
                        } else if s == "defvar" {
                            // defvar has been removed - provide helpful error
                            return Err(CompileError::new(
                                "'defvar' has been removed - use 'def' for immutable bindings".to_string(),
                                expr.location.clone(),
                            ));
                        } else if s == "defconst" {
                            // defconst has been renamed - provide helpful error
                            return Err(CompileError::new(
                                "'defconst' has been renamed to 'def'".to_string(),
                                expr.location.clone(),
                            ));
                        }
                    }
                }
            }
        }

        // Second pass: compile non-definition expressions into main bytecode
        // We need to track and pop intermediate results
        let non_def_exprs: Vec<_> = exprs.iter().enumerate().filter_map(|(i, expr)| {
            let is_definition = if let LispExpr::List(items) = &expr.expr {
                if let Some(first) = items.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        s == "defun" || s == "defmacro" || s == "def" || s == "module" || s == "import"
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if !is_definition {
                Some((i, expr))
            } else {
                None
            }
        }).collect();

        let num_non_defs = non_def_exprs.len();
        for (idx, (_, expr)) in non_def_exprs.iter().enumerate() {
            self.compile_expr(expr)?;
            // Pop the result if it's not the last expression
            // This prevents values from accumulating on the stack between top-level expressions
            if idx < num_non_defs - 1 {
                self.emit(Instruction::PopN(1));
            }
        }

        // Emit Halt at end of main bytecode
        self.emit(Instruction::Halt);

        // Return (functions, main bytecode)
        Ok((self.functions.clone(), self.bytecode.clone()))
    }

    // ==================== FFI TYPE PARSING ====================

    /// Parse FFI argument types from a list expression
    fn parse_ffi_arg_types(&self, expr: &SourceExpr) -> Result<Vec<FfiType>, CompileError> {
        match &expr.expr {
            LispExpr::List(items) => {
                let mut types = Vec::new();
                for item in items {
                    types.push(self.parse_ffi_type(item)?);
                }
                Ok(types)
            }
            _ => Err(CompileError::new(
                "ffi-call: argument types must be a list".to_string(),
                expr.location.clone(),
            )),
        }
    }

    /// Parse a single FFI type from an expression (symbol like :int, :string, etc.)
    fn parse_ffi_type(&self, expr: &SourceExpr) -> Result<FfiType, CompileError> {
        match &expr.expr {
            LispExpr::Symbol(s) => {
                parse_ffi_type(s).map_err(|e| CompileError::new(e, expr.location.clone()))
            }
            _ => Err(CompileError::new(
                "ffi-call: type must be a symbol (e.g., :int, :string, :pointer)".to_string(),
                expr.location.clone(),
            )),
        }
    }
}

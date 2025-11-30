// Macro system: defmacro, expand_macro, value_to_expr

use crate::vm::value::Value;
use crate::vm::instructions::Instruction;
use crate::vm::errors::{CompileError, Location};
use crate::vm::vm::VM;
use crate::vm::stack::Frame;
use super::Compiler;
use super::types::MacroDef;
use super::super::ast::{LispExpr, SourceExpr};

// ==================== MACRO SYSTEM ====================

impl Compiler {
    // Compile defmacro: (defmacro name (params) body)
    pub(super) fn compile_defmacro(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => {
                return Err(CompileError::new(
                    "defmacro expects a list".to_string(),
                    expr.location.clone(),
                ));
            }
        };

        // Check format: (defmacro name (params) body)
        if items.len() != 4 {
            return Err(CompileError::new(
                "defmacro expects: (defmacro name (params) body)".to_string(),
                expr.location.clone(),
            ));
        }

        // Extract macro name
        let macro_name = match &items[1].expr {
            LispExpr::Symbol(s) => s.clone(),
            _ => {
                return Err(CompileError::new(
                    "Macro name must be a symbol".to_string(),
                    items[1].location.clone(),
                ));
            }
        };

        // Extract parameters
        let params = match &items[2].expr {
            LispExpr::List(p) => {
                let mut param_names = Vec::new();
                for param in p {
                    match &param.expr {
                        LispExpr::Symbol(s) => param_names.push(s.clone()),
                        _ => {
                            return Err(CompileError::new(
                                "Macro parameters must be symbols".to_string(),
                                param.location.clone(),
                            ));
                        }
                    }
                }
                param_names
            }
            _ => {
                return Err(CompileError::new(
                    "Macro parameters must be a list".to_string(),
                    items[2].location.clone(),
                ));
            }
        };

        // Store macro definition (body is unevaluated)
        let macro_def = MacroDef {
            params,
            body: items[3].clone(),
        };

        self.macros.insert(macro_name, macro_def);

        Ok(())
    }

    // Expand a macro call at compile time
    pub(super) fn expand_macro(&mut self, macro_def: &MacroDef, args: &[SourceExpr]) -> Result<SourceExpr, CompileError> {
        // Check arity
        if args.len() != macro_def.params.len() {
            return Err(CompileError::new(
                format!("Macro arity mismatch: expected {}, got {}", macro_def.params.len(), args.len()),
                Location::unknown(),
            ));
        }

        // Create a new compiler for evaluating the macro
        let mut macro_compiler = Compiler::new();

        // Set up macro parameters as "arguments"
        macro_compiler.param_names = macro_def.params.clone();

        // Compile macro body
        macro_compiler.compile_expr(&macro_def.body)?;
        macro_compiler.emit(Instruction::Halt);

        let macro_bytecode = std::mem::take(&mut macro_compiler.bytecode);

        // Create a VM and run the macro
        let mut vm = VM::new();
        vm.current_bytecode = macro_bytecode;

        // Create a frame with the quoted arguments
        let mut arg_values = Vec::new();
        for arg_expr in args {
            arg_values.push(self.expr_to_value(arg_expr)?);
        }

        let frame = Frame {
            return_address: 0,
            locals: arg_values,
            return_bytecode: Vec::new(),
            function_name: "<macro>".to_string(),
            captured: Vec::new(),
            stack_base: 0, // Macro expansion uses a fresh VM
            loop_start: None,
            loop_bindings_start: None,
            loop_bindings_count: None,
        };
        vm.call_stack.push(frame);

        // Run the VM
        if let Err(runtime_error) = vm.run() {
            return Err(CompileError::new(
                format!("Macro expansion failed: {}", runtime_error.message),
                Location::unknown(),
            ));
        }

        // Get the result from the stack
        if vm.value_stack.is_empty() {
            return Err(CompileError::new(
                "Macro expansion produced no value".to_string(),
                Location::unknown(),
            ));
        }

        let result_value = vm.value_stack.pop().unwrap();

        // Convert the result back to a SourceExpr
        self.value_to_expr(&result_value)
    }

    // Convert a Value back to a SourceExpr (inverse of expr_to_value)
    pub(super) fn value_to_expr(&self, value: &Value) -> Result<SourceExpr, CompileError> {
        match value {
            Value::Integer(n) => Ok(SourceExpr::unknown(LispExpr::Number(*n))),
            Value::Float(f) => Ok(SourceExpr::unknown(LispExpr::Float(*f))),
            Value::Boolean(b) => Ok(SourceExpr::unknown(LispExpr::Boolean(*b))),
            Value::Symbol(s) => Ok(SourceExpr::unknown(LispExpr::Symbol(s.to_string()))),
            Value::String(s) => {
                // Strings are represented as special symbols in the AST
                Ok(SourceExpr::unknown(LispExpr::Symbol(format!("__STRING__{}", s))))
            }
            Value::List(items) => {
                let mut exprs = Vec::new();
                for item in items.iter() {
                    exprs.push(self.value_to_expr(item)?);
                }
                Ok(SourceExpr::unknown(LispExpr::List(exprs)))
            }
            Value::Function(name) => {
                // Functions become symbols in the macro expansion
                Ok(SourceExpr::unknown(LispExpr::Symbol(name.to_string())))
            }
            Value::Closure(_) => {
                Err(CompileError::new(
                    "Cannot convert closure to expression in macro expansion".to_string(),
                    Location::unknown(),
                ))
            }
            Value::HashMap(_) => {
                Err(CompileError::new(
                    "Cannot convert hashmap to expression in macro expansion".to_string(),
                    Location::unknown(),
                ))
            }
            Value::Vector(_) => {
                Err(CompileError::new(
                    "Cannot convert vector to expression in macro expansion".to_string(),
                    Location::unknown(),
                ))
            }
            Value::TcpListener(_) => {
                Err(CompileError::new(
                    "Cannot convert tcp-listener to expression in macro expansion".to_string(),
                    Location::unknown(),
                ))
            }
            Value::TcpStream(_) => {
                Err(CompileError::new(
                    "Cannot convert tcp-stream to expression in macro expansion".to_string(),
                    Location::unknown(),
                ))
            }
            Value::SharedTcpListener(_) => {
                Err(CompileError::new(
                    "Cannot convert shared-tcp-listener to expression in macro expansion".to_string(),
                    Location::unknown(),
                ))
            }
        }
    }
}

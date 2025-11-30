// Special forms: let, loop, recur, cond, and, or

use crate::vm::value::Value;
use crate::vm::instructions::Instruction;
use crate::vm::errors::CompileError;
use super::Compiler;
use super::types::ValueLocation;
use super::super::ast::{LispExpr, SourceExpr};

// ==================== SPECIAL FORMS (LET, LOOP, RECUR, COND, AND, OR) ====================

impl Compiler {
    // Compile let expression: (let ((pattern value) ...) body)
    pub(super) fn compile_let(
        &mut self,
        bindings_expr: &SourceExpr,
        body_expr: &SourceExpr,
    ) -> Result<(), CompileError> {
        // Parse bindings list
        let bindings = match &bindings_expr.expr {
            LispExpr::List(b) => b,
            _ => {
                return Err(CompileError::new(
                    "let bindings must be a list".to_string(),
                    bindings_expr.location.clone(),
                ));
            }
        };

        // Save current local bindings and stack depth
        let saved_bindings = self.local_bindings.clone();
        let saved_stack_depth = self.stack_depth;

        let mut num_bindings = 0;

        // Process each binding
        for binding in bindings {
            let binding_pair = match &binding.expr {
                LispExpr::List(pair) => pair,
                _ => {
                    return Err(CompileError::new(
                        "Each binding must be a list (pattern value)".to_string(),
                        binding.location.clone(),
                    ));
                }
            };

            if binding_pair.len() != 2 {
                return Err(CompileError::new(
                    "Each binding must have exactly 2 elements: (pattern value)".to_string(),
                    binding.location.clone(),
                ));
            }

            let pattern = &binding_pair[0];
            let value_expr = &binding_pair[1];

            // Save tail position and set to false for binding values
            let saved_tail = self.in_tail_position;
            self.in_tail_position = false;

            // Compile the value expression (pushes result onto stack)
            self.compile_expr(value_expr)?;

            // Restore tail position
            self.in_tail_position = saved_tail;

            // The value is now on the stack at position stack_depth
            let value_position = self.stack_depth;
            self.stack_depth += 1;
            num_bindings += 1;

            // Bind the pattern to this stack position
            self.bind_pattern_to_local(pattern, value_position)?;
        }

        // Compile body with bindings available (body inherits tail position from let)
        self.compile_expr(body_expr)?;

        // Clean up let bindings from stack
        // Stack state: [... bindings(num_bindings) result]
        // We want: [... result]
        if num_bindings > 0 {
            self.emit(Instruction::Slide(num_bindings));
        }

        // Restore binding context
        self.local_bindings = saved_bindings;

        self.stack_depth = saved_stack_depth;

        Ok(())
    }

    pub(super) fn compile_loop(
        &mut self,
        bindings_expr: &SourceExpr,
        body_expr: &SourceExpr,
    ) -> Result<(), CompileError> {
        // Parse bindings list (similar to let)
        let bindings = match &bindings_expr.expr {
            LispExpr::List(b) => b,
            _ => {
                return Err(CompileError::new(
                    "loop bindings must be a list".to_string(),
                    bindings_expr.location.clone(),
                ));
            }
        };

        // Save current local bindings and stack depth
        let saved_bindings = self.local_bindings.clone();
        let saved_stack_depth = self.stack_depth;

        let mut num_bindings = 0;
        let mut _binding_names = Vec::new();

        // Process each binding
        for binding in bindings {
            let binding_pair = match &binding.expr {
                LispExpr::List(pair) => pair,
                _ => {
                    return Err(CompileError::new(
                        "Each binding must be a list (name value)".to_string(),
                        binding.location.clone(),
                    ));
                }
            };

            if binding_pair.len() != 2 {
                return Err(CompileError::new(
                    "Each binding must have exactly 2 elements: (name value)".to_string(),
                    binding.location.clone(),
                ));
            }

            let name_expr = &binding_pair[0];
            let value_expr = &binding_pair[1];

            // Get binding name (must be a symbol)
            let name = match &name_expr.expr {
                LispExpr::Symbol(s) => s.clone(),
                _ => {
                    return Err(CompileError::new(
                        "loop binding name must be a symbol".to_string(),
                        name_expr.location.clone(),
                    ));
                }
            };

            // Compile the value expression (pushes result onto stack)
            let saved_tail = self.in_tail_position;
            self.in_tail_position = false;
            self.compile_expr(value_expr)?;
            self.in_tail_position = saved_tail;

            // The value is now on the stack at position stack_depth
            let value_position = self.stack_depth;
            self.stack_depth += 1;
            num_bindings += 1;

            // Create local binding
            self.local_bindings.insert(name.clone(), ValueLocation::Local(value_position));
            _binding_names.push(name);
        }

        // Emit BeginLoop instruction to mark loop start
        self.emit(Instruction::BeginLoop(num_bindings));

        // Compile body (body is in tail position - recur will jump back)
        let saved_tail = self.in_tail_position;
        self.in_tail_position = true; // Body of loop is in tail position for recur
        self.compile_expr(body_expr)?;
        self.in_tail_position = saved_tail;

        // Clean up loop bindings from stack (only executed if body returns without recur)
        if num_bindings > 0 {
            self.emit(Instruction::Slide(num_bindings));
        }

        // Restore binding context
        self.local_bindings = saved_bindings;
        self.stack_depth = saved_stack_depth;

        Ok(())
    }

    pub(super) fn compile_recur(&mut self, args: &[SourceExpr]) -> Result<(), CompileError> {
        // Compile arguments (similar to function call)
        let saved_tail = self.in_tail_position;
        self.in_tail_position = false;

        for arg in args {
            self.compile_expr(arg)?;
        }

        self.in_tail_position = saved_tail;

        // Emit Recur instruction
        self.emit(Instruction::Recur(args.len()));

        Ok(())
    }

    // Helper for compiling and: (and a b c) => (if a (if b c false) false)
    pub(super) fn compile_and_helper(&mut self, exprs: &[SourceExpr], context: &SourceExpr) -> Result<(), CompileError> {
        if exprs.is_empty() {
            // Empty and is true
            self.emit(Instruction::Push(Value::Boolean(true)));
            return Ok(());
        }

        if exprs.len() == 1 {
            // Last expression - just compile it
            self.compile_expr(&exprs[0])?;
            return Ok(());
        }

        // Multiple expressions: if first then (and rest...) else false
        let saved_tail = self.in_tail_position;

        // Compile first expression (not in tail position)
        self.in_tail_position = false;
        self.compile_expr(&exprs[0])?;

        // Emit JmpIfFalse with placeholder
        let jmp_if_false_index = self.bytecode.len();
        self.emit(Instruction::JmpIfFalse(0));

        // Compile rest (inherits tail position)
        self.in_tail_position = saved_tail;
        self.compile_and_helper(&exprs[1..], context)?;

        // Emit Jmp to skip false branch
        let jmp_to_end_index = self.bytecode.len();
        self.emit(Instruction::Jmp(0));

        // False branch
        let false_addr = self.instruction_address;
        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(false_addr);
        self.emit(Instruction::Push(Value::Boolean(false)));

        // End
        let end_addr = self.instruction_address;
        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);

        self.in_tail_position = saved_tail;
        Ok(())
    }

    // Helper for compiling or: (or a b c) => (if a true (if b true c))
    pub(super) fn compile_or_helper(&mut self, exprs: &[SourceExpr], context: &SourceExpr) -> Result<(), CompileError> {
        if exprs.is_empty() {
            // Empty or is false
            self.emit(Instruction::Push(Value::Boolean(false)));
            return Ok(());
        }

        if exprs.len() == 1 {
            // Last expression - just compile it
            self.compile_expr(&exprs[0])?;
            return Ok(());
        }

        // Multiple expressions: if first then true else (or rest...)
        let saved_tail = self.in_tail_position;

        // Compile first expression (not in tail position)
        self.in_tail_position = false;
        self.compile_expr(&exprs[0])?;

        // Emit JmpIfFalse with placeholder
        let jmp_if_false_index = self.bytecode.len();
        self.emit(Instruction::JmpIfFalse(0));

        // True branch
        self.emit(Instruction::Push(Value::Boolean(true)));

        // Emit Jmp to skip rest
        let jmp_to_end_index = self.bytecode.len();
        self.emit(Instruction::Jmp(0));

        // Rest branch
        let rest_addr = self.instruction_address;
        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(rest_addr);

        self.in_tail_position = saved_tail;
        self.compile_or_helper(&exprs[1..], context)?;

        // End
        let end_addr = self.instruction_address;
        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);

        self.in_tail_position = saved_tail;
        Ok(())
    }

    // Helper for compiling cond: (cond (test1 expr1) (test2 expr2) ... (else default))
    pub(super) fn compile_cond(&mut self, clauses: &[SourceExpr], context: &SourceExpr) -> Result<(), CompileError> {
        if clauses.is_empty() {
            // No clauses - push false
            self.emit(Instruction::Push(Value::Boolean(false)));
            return Ok(());
        }

        let saved_tail = self.in_tail_position;

        for (i, clause) in clauses.iter().enumerate() {
            let is_last = i == clauses.len() - 1;

            match &clause.expr {
                LispExpr::List(items) if items.len() == 2 => {
                    // Check if this is an else clause
                    let is_else = match &items[0].expr {
                        LispExpr::Symbol(s) if s == "else" => true,
                        _ => false,
                    };

                    if is_else {
                        // Else clause - just compile the expression
                        if !is_last {
                            return Err(CompileError::new(
                                "else clause must be the last clause in cond".to_string(),
                                clause.location.clone(),
                            ));
                        }
                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[1])?;
                    } else {
                        // Regular clause: (test expr)
                        // Compile test (not in tail position)
                        self.in_tail_position = false;
                        self.compile_expr(&items[0])?;

                        // Emit JmpIfFalse with placeholder
                        let jmp_if_false_index = self.bytecode.len();
                        self.emit(Instruction::JmpIfFalse(0));

                        // Compile then branch (inherits tail position)
                        self.in_tail_position = saved_tail;
                        self.compile_expr(&items[1])?;

                        // Emit Jmp to end
                        let jmp_to_end_index = self.bytecode.len();
                        self.emit(Instruction::Jmp(0));

                        // Record else/next branch address
                        let else_addr = self.instruction_address;
                        self.bytecode[jmp_if_false_index] = Instruction::JmpIfFalse(else_addr);

                        // If this is the last clause and not else, compile remaining clauses
                        if !is_last {
                            self.compile_cond(&clauses[i + 1..], context)?;
                        } else {
                            // Last clause without else - push false
                            self.emit(Instruction::Push(Value::Boolean(false)));
                        }

                        // Patch jump to end
                        let end_addr = self.instruction_address;
                        self.bytecode[jmp_to_end_index] = Instruction::Jmp(end_addr);

                        // Important: break after processing to avoid double-processing
                        break;
                    }
                }
                _ => {
                    return Err(CompileError::new(
                        "cond clause must be a list of (test expr)".to_string(),
                        clause.location.clone(),
                    ));
                }
            }
        }

        self.in_tail_position = saved_tail;
        Ok(())
    }
}

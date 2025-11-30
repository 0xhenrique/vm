// Module system: module definitions, exports, imports

use crate::vm::instructions::Instruction;
use crate::vm::errors::CompileError;
use super::Compiler;
use super::super::ast::{LispExpr, SourceExpr};

// ==================== MODULE SYSTEM ====================

impl Compiler {
    /// Qualify a name with the current module prefix (if any)
    /// e.g., in module "math", "add" becomes "math/add"
    pub(super) fn qualify_name(&self, name: &str) -> String {
        match &self.current_module {
            Some(module) => format!("{}/{}", module, name),
            None => name.to_string(),
        }
    }

    /// Inject known module exports from runtime context (for import)
    pub fn with_known_module_exports(&mut self, module: &str, exports: &std::collections::HashSet<String>) {
        self.module_exports.insert(module.to_string(), exports.clone());
    }

    /// Compile a module declaration
    /// (module name (export sym1 sym2 ...) body...)
    pub(super) fn compile_module(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => return Err(CompileError::new(
                "module expects a list".to_string(),
                expr.location.clone(),
            )),
        };

        // Minimum: (module name body)
        if items.len() < 3 {
            return Err(CompileError::with_suggestion(
                "module expects at least: (module name body...)".to_string(),
                expr.location.clone(),
                "Use: (module math (export add sub) (defun add (x y) (+ x y)))".to_string(),
            ));
        }

        // Extract module name
        let module_name = match &items[1].expr {
            LispExpr::Symbol(s) => s.clone(),
            _ => return Err(CompileError::new(
                "Module name must be a symbol".to_string(),
                items[1].location.clone(),
            )),
        };

        // Check for nested modules
        if self.current_module.is_some() {
            return Err(CompileError::new(
                format!("Cannot define module '{}' inside another module", module_name),
                expr.location.clone(),
            ));
        }

        // Initialize exports for this module
        self.module_exports.insert(module_name.clone(), std::collections::HashSet::new());

        // Set current module context
        self.current_module = Some(module_name.clone());

        // Pre-scan: collect all function names declared in this module
        // This enables forward references and recursive calls
        self.module_functions.clear();
        for item in items.iter().skip(2) {
            if let LispExpr::List(inner) = &item.expr {
                if let Some(first) = inner.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        if s == "defun" && inner.len() >= 2 {
                            if let LispExpr::Symbol(fn_name) = &inner[1].expr {
                                self.module_functions.insert(fn_name.clone());
                            }
                        }
                    }
                }
            }
        }

        // First pass: process imports (need to happen before defuns for resolution)
        for item in items.iter().skip(2) {
            if let LispExpr::List(inner) = &item.expr {
                if let Some(first) = inner.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        if s == "import" {
                            self.compile_import(item)?;
                        }
                    }
                }
            }
        }

        // Second pass: process exports
        for item in items.iter().skip(2) {
            if let LispExpr::List(inner) = &item.expr {
                if let Some(first) = inner.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        if s == "export" {
                            self.compile_export(item, &module_name)?;
                        }
                    }
                }
            }
        }

        // Third pass: process all other module body items
        for item in items.iter().skip(2) {
            if let LispExpr::List(inner) = &item.expr {
                if let Some(first) = inner.first() {
                    if let LispExpr::Symbol(s) = &first.expr {
                        match s.as_str() {
                            "export" | "import" => {
                                // Already processed
                            }
                            "defun" => self.compile_defun(item)?,
                            "defmacro" => self.compile_defmacro(item)?,
                            "def" => self.compile_def(item)?,
                            _ => {
                                // Other expressions in module body - compile as main code
                                self.compile_expr(item)?;
                                self.emit(Instruction::PopN(1));
                            }
                        }
                        continue;
                    }
                }
            }
            // Compile other expressions in module
            self.compile_expr(item)?;
            self.emit(Instruction::PopN(1));
        }

        // Exit module context and clear module-local state
        self.current_module = None;
        self.module_functions.clear();

        Ok(())
    }

    /// Compile an export declaration
    /// (export sym1 sym2 ...) or (export (sym1 sym2 ...))
    pub(super) fn compile_export(&mut self, expr: &SourceExpr, module_name: &str) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => return Err(CompileError::new(
                "export expects a list".to_string(),
                expr.location.clone(),
            )),
        };

        if items.len() < 2 {
            return Err(CompileError::with_suggestion(
                "export expects at least one symbol".to_string(),
                expr.location.clone(),
                "Use: (export add subtract) or (export (add subtract))".to_string(),
            ));
        }

        // Check if exports are wrapped in a list: (export (sym1 sym2 ...))
        let exports_list = if items.len() == 2 {
            if let LispExpr::List(inner) = &items[1].expr {
                inner.as_slice()
            } else {
                // Single symbol export: (export sym)
                &items[1..2]
            }
        } else {
            // Multiple symbols: (export sym1 sym2 ...)
            &items[1..]
        };

        for export_expr in exports_list {
            let export_name = match &export_expr.expr {
                LispExpr::Symbol(s) => s.clone(),
                _ => return Err(CompileError::new(
                    "Export items must be symbols".to_string(),
                    export_expr.location.clone(),
                )),
            };

            if let Some(exports) = self.module_exports.get_mut(module_name) {
                exports.insert(export_name);
            }
        }

        Ok(())
    }

    /// Compile an import declaration
    /// (import module-name) - import all exports with qualified names
    /// (import module-name sym1 sym2 ...) - import specific symbols
    pub(super) fn compile_import(&mut self, expr: &SourceExpr) -> Result<(), CompileError> {
        let items = match &expr.expr {
            LispExpr::List(items) => items,
            _ => return Err(CompileError::new(
                "import expects a list".to_string(),
                expr.location.clone(),
            )),
        };

        if items.len() < 2 {
            return Err(CompileError::with_suggestion(
                "import expects a module name".to_string(),
                expr.location.clone(),
                "Use: (import math) or (import math add subtract)".to_string(),
            ));
        }

        // Extract module name
        let module_name = match &items[1].expr {
            LispExpr::Symbol(s) => s.clone(),
            _ => return Err(CompileError::new(
                "Module name must be a symbol".to_string(),
                items[1].location.clone(),
            )),
        };

        // Check if we have exports for this module
        let exports = self.module_exports.get(&module_name).cloned();

        if items.len() == 2 {
            // Import all exports with qualified names (no aliases needed, use module/func syntax)
            // This just validates the module exists
            if exports.is_none() {
                return Err(CompileError::with_suggestion(
                    format!("Unknown module '{}'", module_name),
                    items[1].location.clone(),
                    format!("Make sure module '{}' is defined before importing it.", module_name),
                ));
            }
        } else {
            // Import specific symbols as unqualified aliases
            for item in items.iter().skip(2) {
                let sym_name = match &item.expr {
                    LispExpr::Symbol(s) => s.clone(),
                    _ => return Err(CompileError::new(
                        "Import items must be symbols".to_string(),
                        item.location.clone(),
                    )),
                };

                // Check if the symbol is exported by the module
                if let Some(ref exports) = exports {
                    if !exports.contains(&sym_name) {
                        return Err(CompileError::with_suggestion(
                            format!("Symbol '{}' is not exported by module '{}'", sym_name, module_name),
                            item.location.clone(),
                            format!("Available exports: {:?}", exports),
                        ));
                    }
                }

                // Create alias: sym_name -> module_name/sym_name
                let qualified_name = format!("{}/{}", module_name, sym_name);
                self.imported_symbols.insert(sym_name, qualified_name);
            }
        }

        Ok(())
    }

    /// Resolve a function name for a call:
    /// 1. Check if it's an imported alias
    /// 2. If we're in a module and name doesn't have "/", check for module-local function
    /// 3. Otherwise return as-is (may be qualified like "math/add" or global like "+")
    pub(super) fn resolve_function_name(&self, name: &str) -> String {
        // First check if this is an imported alias
        if let Some(qualified) = self.imported_symbols.get(name) {
            return qualified.clone();
        }

        // If the name already contains "/" it's already qualified, use as-is
        if name.contains('/') {
            return name.to_string();
        }

        // If we're in a module, check if it's a module-local function
        if let Some(ref module) = self.current_module {
            let qualified = format!("{}/{}", module, name);
            // Check if it's declared in the current module (for forward refs/recursion)
            // or if we've already compiled it
            if self.module_functions.contains(name) || self.functions.contains_key(&qualified) {
                return qualified;
            }
        }

        // Return original name (builtin or global function)
        name.to_string()
    }

    /// Resolve a global variable/function name:
    /// 1. Check if it's an imported alias
    /// 2. If we're in a module, check for module-local definition
    /// 3. If name contains "/", use as-is (already qualified)
    /// 4. Otherwise return original name
    pub(super) fn resolve_global_name(&self, name: &str) -> String {
        // First check if this is an imported alias
        if let Some(qualified) = self.imported_symbols.get(name) {
            return qualified.clone();
        }

        // If the name already contains "/" it's already qualified, use as-is
        if name.contains('/') {
            return name.to_string();
        }

        // If we're in a module, check if it's a module-local definition
        if let Some(ref module) = self.current_module {
            let qualified = format!("{}/{}", module, name);
            // Check if it's declared in the module (for forward refs) or already defined
            if self.module_functions.contains(name) || self.global_vars.contains_key(&qualified) || self.functions.contains_key(&qualified) {
                return qualified;
            }
        }

        // Return original name
        name.to_string()
    }
}

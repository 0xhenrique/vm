use crate::{Instruction, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub original_instruction_count: usize,
    pub optimized_instruction_count: usize,
    pub constant_folds: usize,
    pub dead_code_removed: usize,
    pub jump_chains_simplified: usize,
    pub peephole_optimizations: usize,
    pub strength_reductions: usize,
}

impl OptimizationStats {
    pub fn new() -> Self {
        OptimizationStats {
            original_instruction_count: 0,
            optimized_instruction_count: 0,
            constant_folds: 0,
            dead_code_removed: 0,
            jump_chains_simplified: 0,
            peephole_optimizations: 0,
            strength_reductions: 0,
        }
    }

    pub fn reduction_percentage(&self) -> f64 {
        if self.original_instruction_count == 0 {
            0.0
        } else {
            let reduction = self.original_instruction_count - self.optimized_instruction_count;
            (reduction as f64 / self.original_instruction_count as f64) * 100.0
        }
    }
}

pub struct Optimizer {
    stats: OptimizationStats,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer {
            stats: OptimizationStats::new(),
        }
    }

    pub fn optimize(&mut self, bytecode: Vec<Instruction>) -> Vec<Instruction> {
        self.stats.original_instruction_count += bytecode.len();

        let mut optimized = bytecode;

        optimized = self.constant_folding_pass(optimized);
        optimized = self.peephole_optimization_pass(optimized);
        optimized = self.jump_to_jump_elimination_pass(optimized);
        optimized = self.dead_code_elimination_pass(optimized);

        self.stats.optimized_instruction_count += optimized.len();

        optimized
    }

    pub fn optimize_functions(&mut self, functions: HashMap<String, Vec<Instruction>>)
        -> HashMap<String, Vec<Instruction>> {
        functions.into_iter()
            .map(|(name, bytecode)| {
                let optimized = self.optimize(bytecode);
                (name, optimized)
            })
            .collect()
    }

    pub fn get_stats(&self) -> &OptimizationStats {
        &self.stats
    }

    fn constant_folding_pass(&mut self, bytecode: Vec<Instruction>) -> Vec<Instruction> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            // Binary operations on two values
            if i + 2 < bytecode.len() {
                let folded = match (&bytecode[i], &bytecode[i + 1], &bytecode[i + 2]) {
                    // Integer + Integer
                    (Instruction::Push(Value::Integer(a)), Instruction::Push(Value::Integer(b)), op) => {
                        self.fold_int_int(*a, *b, op)
                    }
                    // Float + Float
                    (Instruction::Push(Value::Float(a)), Instruction::Push(Value::Float(b)), op) => {
                        self.fold_float_float(*a, *b, op)
                    }
                    // Integer + Float (coerce to float)
                    (Instruction::Push(Value::Integer(a)), Instruction::Push(Value::Float(b)), op) => {
                        self.fold_float_float(*a as f64, *b, op)
                    }
                    // Float + Integer (coerce to float)
                    (Instruction::Push(Value::Float(a)), Instruction::Push(Value::Integer(b)), op) => {
                        self.fold_float_float(*a, *b as f64, op)
                    }
                    _ => None,
                };

                if let Some(value) = folded {
                    result.push(Instruction::Push(value));
                    self.stats.constant_folds += 1;
                    i += 3;
                    continue;
                }
            }

            // Unary operations
            if i + 1 < bytecode.len() {
                let folded = match (&bytecode[i], &bytecode[i + 1]) {
                    (Instruction::Push(Value::Integer(n)), Instruction::Neg) => {
                        Some(Value::Integer(-n))
                    }
                    (Instruction::Push(Value::Float(f)), Instruction::Neg) => {
                        Some(Value::Float(-f))
                    }
                    _ => None,
                };

                if let Some(value) = folded {
                    result.push(Instruction::Push(value));
                    self.stats.constant_folds += 1;
                    i += 2;
                    continue;
                }
            }

            result.push(bytecode[i].clone());
            i += 1;
        }

        result
    }

    fn fold_int_int(&self, a: i64, b: i64, op: &Instruction) -> Option<Value> {
        match op {
            Instruction::Add => Some(Value::Integer(a + b)),
            Instruction::Sub => Some(Value::Integer(a - b)),
            Instruction::Mul => Some(Value::Integer(a * b)),
            Instruction::Div if b != 0 => Some(Value::Integer(a / b)),
            Instruction::Mod if b != 0 => Some(Value::Integer(a % b)),
            Instruction::Leq => Some(Value::Boolean(a <= b)),
            Instruction::Lt => Some(Value::Boolean(a < b)),
            Instruction::Gt => Some(Value::Boolean(a > b)),
            Instruction::Gte => Some(Value::Boolean(a >= b)),
            Instruction::Eq => Some(Value::Boolean(a == b)),
            Instruction::Neq => Some(Value::Boolean(a != b)),
            _ => None,
        }
    }

    fn fold_float_float(&self, a: f64, b: f64, op: &Instruction) -> Option<Value> {
        match op {
            Instruction::Add => Some(Value::Float(a + b)),
            Instruction::Sub => Some(Value::Float(a - b)),
            Instruction::Mul => Some(Value::Float(a * b)),
            Instruction::Div if b != 0.0 => Some(Value::Float(a / b)),
            Instruction::Mod if b != 0.0 => Some(Value::Float(a % b)),
            Instruction::Leq => Some(Value::Boolean(a <= b)),
            Instruction::Lt => Some(Value::Boolean(a < b)),
            Instruction::Gt => Some(Value::Boolean(a > b)),
            Instruction::Gte => Some(Value::Boolean(a >= b)),
            Instruction::Eq => {
                // Handle NaN correctly: NaN != NaN
                if a.is_nan() || b.is_nan() {
                    Some(Value::Boolean(false))
                } else {
                    Some(Value::Boolean(a == b))
                }
            }
            Instruction::Neq => {
                // Handle NaN correctly: NaN != NaN is true
                if a.is_nan() || b.is_nan() {
                    Some(Value::Boolean(true))
                } else {
                    Some(Value::Boolean(a != b))
                }
            }
            _ => None,
        }
    }

    fn peephole_optimization_pass(&mut self, bytecode: Vec<Instruction>) -> Vec<Instruction> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            // Try 3-instruction patterns (for strength reduction)
            if i + 2 < bytecode.len() {
                let optimized = self.try_strength_reduction(&bytecode[i], &bytecode[i + 1], &bytecode[i + 2]);
                if let Some(instructions) = optimized {
                    result.extend(instructions);
                    self.stats.strength_reductions += 1;
                    i += 3;
                    continue;
                }
            }

            // Try 2-instruction patterns
            if i + 1 < bytecode.len() {
                let optimized = self.try_peephole_2(&bytecode[i], &bytecode[i + 1]);
                if let Some(instructions) = optimized {
                    result.extend(instructions);
                    self.stats.peephole_optimizations += 1;
                    i += 2;
                    continue;
                }
            }

            // No optimization found, keep the instruction
            result.push(bytecode[i].clone());
            i += 1;
        }

        result
    }

    fn try_strength_reduction(&self, instr1: &Instruction, instr2: &Instruction, instr3: &Instruction) -> Option<Vec<Instruction>> {
        match (instr1, instr2, instr3) {
            // x * -1 → -x (replace multiplication with negation)
            (_, Instruction::Push(Value::Integer(-1)), Instruction::Mul) => {
                Some(vec![
                    instr1.clone(),
                    Instruction::Neg,
                ])
            }
            (_, Instruction::Push(Value::Float(f)), Instruction::Mul) if *f == -1.0 => {
                Some(vec![
                    instr1.clone(),
                    Instruction::Neg,
                ])
            }

            // x * 0 → 0 (eliminate multiplication, just pop x and push 0)
            (_, Instruction::Push(Value::Integer(0)), Instruction::Mul) => {
                Some(vec![
                    instr1.clone(),
                    Instruction::PopN(1),
                    Instruction::Push(Value::Integer(0)),
                ])
            }
            (_, Instruction::Push(Value::Float(f)), Instruction::Mul) if *f == 0.0 => {
                Some(vec![
                    instr1.clone(),
                    Instruction::PopN(1),
                    Instruction::Push(Value::Float(0.0)),
                ])
            }

            _ => None,
        }
    }

    fn try_peephole_2(&self, instr1: &Instruction, instr2: &Instruction) -> Option<Vec<Instruction>> {
        match (instr1, instr2) {
            // Algebraic simplifications: x + 0 = x
            (Instruction::Push(Value::Integer(0)), Instruction::Add) => {
                Some(vec![]) // Remove both: pushing 0 and adding it is a no-op
            }
            (Instruction::Push(Value::Float(f)), Instruction::Add) if *f == 0.0 => {
                Some(vec![]) // Remove both: pushing 0.0 and adding it is a no-op
            }

            // Algebraic simplifications: x - 0 = x
            (Instruction::Push(Value::Integer(0)), Instruction::Sub) => {
                Some(vec![]) // Remove both: subtracting 0 is a no-op
            }
            (Instruction::Push(Value::Float(f)), Instruction::Sub) if *f == 0.0 => {
                Some(vec![]) // Remove both: subtracting 0.0 is a no-op
            }

            // Algebraic simplifications: x * 1 = x
            (Instruction::Push(Value::Integer(1)), Instruction::Mul) => {
                Some(vec![]) // Remove both: multiplying by 1 is a no-op
            }
            (Instruction::Push(Value::Float(f)), Instruction::Mul) if *f == 1.0 => {
                Some(vec![]) // Remove both: multiplying by 1.0 is a no-op
            }

            // Algebraic simplifications: x / 1 = x
            (Instruction::Push(Value::Integer(1)), Instruction::Div) => {
                Some(vec![]) // Remove both: dividing by 1 is a no-op
            }
            (Instruction::Push(Value::Float(f)), Instruction::Div) if *f == 1.0 => {
                Some(vec![]) // Remove both: dividing by 1.0 is a no-op
            }

            // Double negation: -(-(x)) = x
            (Instruction::Neg, Instruction::Neg) => {
                Some(vec![]) // Remove both negations
            }

            _ => None,
        }
    }

    fn dead_code_elimination_pass(&mut self, bytecode: Vec<Instruction>) -> Vec<Instruction> {
        let reachable = self.compute_reachable(&bytecode);

        let mut result = Vec::new();
        for (i, instr) in bytecode.into_iter().enumerate() {
            if reachable.contains(&i) {
                result.push(instr);
            } else {
                self.stats.dead_code_removed += 1;
            }
        }

        result
    }

    fn compute_reachable(&self, bytecode: &[Instruction]) -> HashSet<usize> {
        let mut reachable = HashSet::new();
        let mut to_visit = vec![0];

        while let Some(addr) = to_visit.pop() {
            if addr >= bytecode.len() || reachable.contains(&addr) {
                continue;
            }

            reachable.insert(addr);

            match &bytecode[addr] {
                Instruction::Jmp(target) => {
                    to_visit.push(*target);
                }
                Instruction::JmpIfFalse(target) => {
                    to_visit.push(*target);
                    if addr + 1 < bytecode.len() {
                        to_visit.push(addr + 1);
                    }
                }
                Instruction::Halt | Instruction::Ret => {
                }
                _ => {
                    if addr + 1 < bytecode.len() {
                        to_visit.push(addr + 1);
                    }
                }
            }
        }

        reachable
    }

    fn jump_to_jump_elimination_pass(&mut self, bytecode: Vec<Instruction>) -> Vec<Instruction> {
        let jump_targets = self.resolve_jump_chains(&bytecode);

        bytecode.into_iter().map(|instr| {
            match instr {
                Instruction::Jmp(target) => {
                    if let Some(&final_target) = jump_targets.get(&target) {
                        if final_target != target {
                            self.stats.jump_chains_simplified += 1;
                        }
                        Instruction::Jmp(final_target)
                    } else {
                        Instruction::Jmp(target)
                    }
                }
                Instruction::JmpIfFalse(target) => {
                    if let Some(&final_target) = jump_targets.get(&target) {
                        if final_target != target {
                            self.stats.jump_chains_simplified += 1;
                        }
                        Instruction::JmpIfFalse(final_target)
                    } else {
                        Instruction::JmpIfFalse(target)
                    }
                }
                other => other,
            }
        }).collect()
    }

    fn resolve_jump_chains(&self, bytecode: &[Instruction]) -> HashMap<usize, usize> {
        let mut resolved = HashMap::new();

        for (i, instr) in bytecode.iter().enumerate() {
            if let Instruction::Jmp(target) = instr {
                let final_target = self.follow_jump_chain(bytecode, *target, 100);
                resolved.insert(i, final_target);
            } else if let Instruction::JmpIfFalse(target) = instr {
                let final_target = self.follow_jump_chain(bytecode, *target, 100);
                resolved.insert(i, final_target);
            }
        }

        resolved
    }

    fn follow_jump_chain(&self, bytecode: &[Instruction], mut target: usize, max_depth: usize) -> usize {
        let mut depth = 0;
        let mut visited = HashSet::new();

        while depth < max_depth && target < bytecode.len() {
            if visited.contains(&target) {
                break;
            }
            visited.insert(target);

            if let Instruction::Jmp(next_target) = &bytecode[target] {
                target = *next_target;
                depth += 1;
            } else {
                break;
            }
        }

        target
    }
}

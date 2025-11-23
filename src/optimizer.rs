use crate::{Instruction, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub original_instruction_count: usize,
    pub optimized_instruction_count: usize,
    pub constant_folds: usize,
    pub dead_code_removed: usize,
    pub jump_chains_simplified: usize,
}

impl OptimizationStats {
    pub fn new() -> Self {
        OptimizationStats {
            original_instruction_count: 0,
            optimized_instruction_count: 0,
            constant_folds: 0,
            dead_code_removed: 0,
            jump_chains_simplified: 0,
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
            if i + 2 < bytecode.len() {
                if let (
                    Instruction::Push(Value::Integer(a)),
                    Instruction::Push(Value::Integer(b)),
                    op
                ) = (&bytecode[i], &bytecode[i + 1], &bytecode[i + 2]) {
                    let folded = match op {
                        Instruction::Add => Some(Value::Integer(a + b)),
                        Instruction::Sub => Some(Value::Integer(a - b)),
                        Instruction::Mul => Some(Value::Integer(a * b)),
                        Instruction::Div if *b != 0 => Some(Value::Integer(a / b)),
                        Instruction::Mod if *b != 0 => Some(Value::Integer(a % b)),
                        Instruction::Leq => Some(Value::Boolean(a <= b)),
                        Instruction::Lt => Some(Value::Boolean(a < b)),
                        Instruction::Gt => Some(Value::Boolean(a > b)),
                        Instruction::Gte => Some(Value::Boolean(a >= b)),
                        Instruction::Eq => Some(Value::Boolean(a == b)),
                        Instruction::Neq => Some(Value::Boolean(a != b)),
                        _ => None,
                    };

                    if let Some(value) = folded {
                        result.push(Instruction::Push(value));
                        self.stats.constant_folds += 1;
                        i += 3;
                        continue;
                    }
                }
            }

            if i + 1 < bytecode.len() {
                if let (Instruction::Push(Value::Integer(n)), Instruction::Neg) =
                    (&bytecode[i], &bytecode[i + 1]) {
                    result.push(Instruction::Push(Value::Integer(-n)));
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

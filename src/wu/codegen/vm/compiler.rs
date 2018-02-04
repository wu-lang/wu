
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use super::vm::{Machine, Instruction};
use super::value::Value;

use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct JumpPatch(usize);

#[derive(Clone, Copy)]
pub struct BranchTarget(usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Block<'b> {
    pub name:        &'b str,
    pub code:        Rc<[Instruction]>,
    pub constants:   Rc<[Value<'b>]>,
    pub local_names: Rc<[&'b str]>,
}

pub struct Compiler<'c> {
    locals:    HashMap<&'c str, u16>,
    constants: Vec<Value<'c>>,
    code:      Vec<Instruction>,
    vm:        &'c mut Machine<'c>,
}

impl<'c> Compiler<'c> {
    pub fn new(vm: &'c mut Machine<'c>) -> Self {
        Compiler {
            locals:    HashMap::new(),
            code:      Vec::new(),
            constants: Vec::new(),
            vm,
        }
    }

    pub fn declare_local(&mut self, name: &'c str) -> u16 {
        if self.locals.len() > (u16::max_value() as usize) {
            panic!("local overflow")
        } else {
            let index = self.locals.len() as u16;
            let entry = self.locals.entry(name);

            match entry {
                Entry::Vacant(value) => {
                    value.insert(index);
                    index
                },

                _ => panic!("entry not vacant")
            }
        }
    }

    pub fn fetch_local(&mut self, name: &str) -> u16 {
        self.locals.get(name).map(|i| *i).unwrap()
    }

    pub fn emit(&mut self, instruction: Instruction) {
        self.code.push(instruction);
    }

    pub fn emit_load_const(&mut self, value: Value<'c>) {
        let idx = self.constants.len();
        if idx < (u16::max_value() as usize) {
            let idx = idx as u16;

            self.constants.push(value);
            self.emit(Instruction::LoadConst(idx))
        }
    }

    pub fn emit_branch_false(&mut self) -> JumpPatch {
        let jump = JumpPatch(self.code.len());
        self.emit(Instruction::BranchFalse(0));

        jump
    }

    pub fn emit_branch_true(&mut self) -> JumpPatch {
        let jump = JumpPatch(self.code.len());
        self.emit(Instruction::BranchTrue(0));

        jump
    }

    pub fn emit_jump(&mut self) -> JumpPatch {
        let jump = JumpPatch(self.code.len());
        self.emit(Instruction::Jump(0));

        jump
    }

    pub fn save_branch_target(&self) -> BranchTarget {
        BranchTarget(self.code.len())
    }

    pub fn patch_jump(&mut self, patch: JumpPatch) {
        let cur = self.code.len();
        let branch_loc = patch.0;
        let diff = (cur as isize) - (branch_loc as isize);

        if diff < (i16::max_value() as isize) || diff < (i16::min_value() as isize) {
            let diff = diff as i16;

            match self.code[branch_loc] {
                Instruction::Jump(_)        => self.code[branch_loc] = Instruction::Jump(diff),
                Instruction::BranchTrue(_)  => self.code[branch_loc] = Instruction::BranchTrue(diff),
                Instruction::BranchFalse(_) => self.code[branch_loc] = Instruction::BranchFalse(diff),
                _                           => unreachable!(),
            }
        }
    }

    pub fn emit_jump_to_target(&mut self, target: BranchTarget) {
        let cur = self.code.len();
        let BranchTarget(target) = target;
        let diff = (target as isize) - (cur as isize);

        if !(diff > (i16::max_value() as isize) || diff < (i16::min_value() as isize)) {
            let diff = diff as i16;
            self.emit(Instruction::Jump(diff))
        }
    }
}
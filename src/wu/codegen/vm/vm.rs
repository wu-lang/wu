use super::value::*;
use super::compiler::*;

use std::ptr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    
    Neg,
    
    Lt,
    LtEqual,
    Gt,
    GtEqual,
    Equal,
    NEqual,
    
    LoadConst(u16),
    LoadLocal(u16),
    StoreLocal(u16),
    
    BranchTrue(i16),
    BranchFalse(i16),
    Jump(i16),
    
    Pop,
    Return,
    Print,
}

pub struct CallInfo<'c> {
    locals: Box<[Value<'c>]>,
    pc: usize,
    func: *const Block<'c>,
}

pub struct Machine<'m> {
    value_stack: Vec<Value<'m>>,
    next_object: *mut Object<'m>,
    call_stack:  Vec<CallInfo<'m>>,
}

impl<'m> Machine<'m> {
    pub fn new() -> Self {
        Machine {
            value_stack: Vec::new(),
            next_object: ptr::null_mut(),
            call_stack:  Vec::new(),
        }
    }

    pub fn alloc(&mut self, kind: ObjectType<'m>) -> Value<'m> {
        let object = Box::into_raw(
            Box::new(
                Object {
                    next: self.next_object,
                    kind,
                }
            )
        );
        
        self.next_object = object;

        Value::Object(object)
    }
}
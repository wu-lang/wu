use std::hash::{ Hash, Hasher };
use std::mem;

use super::compiler::Block;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType<'o> {
    Lambda(*mut Block<'o>),
    Array(*mut Vec<Value<'o>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object<'o> {
    pub next: *mut Object<'o>,
    pub kind: ObjectType<'o>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'v> {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    LongInt(i128),
    Str(&'v str),
    Object(*mut Object<'v>)
}

impl<'v> Default for Value<'v> {
    fn default() -> Value<'v> {
        Value::Nil
    }
}

impl<'v> Hash for Value<'v> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Value::Nil => state.write_u8(0),

            Value::Bool(b) => {
                state.write_u8(1);
                state.write_u8(b as u8)
            },

            Value::Int(n) => {
                state.write_u8(2);
                state.write_u64(
                    unsafe {
                        mem::transmute(n)
                    }
                )
            },

            Value::Float(n) => {
                state.write_u8(3);
                state.write_u64(
                    unsafe {
                        mem::transmute(n)
                    }
                )
            },

            Value::LongInt(n) => {
                state.write_u8(4);
                state.write_u128(
                    unsafe {
                        mem::transmute(n)
                    }
                )
            },

            Value::Str(n) => {
                state.write_u8(5);
                state.write_u128(
                    unsafe {
                        mem::transmute(n)
                    }
                )
            },

            Value::Object(n) => {
                state.write_u8(5);
                state.write_usize(
                    unsafe {
                        mem::transmute(n)
                    }
                )
            },
        }
    }
}

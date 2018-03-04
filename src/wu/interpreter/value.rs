use std::hash::{ Hash, Hasher };
use std::mem;

use super::CompiledBlock;

pub enum HeapObjectType {
  Str(Box<str>),
  Array(Vec<Value>),
  Function(CompiledBlock)
}

pub struct HeapObject {
  pub next: *mut HeapObject,
  pub kind: HeapObjectType
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Bool(bool),
  Int(i64),
  Float(f64),
  Char(char),
  HeapObject(*mut HeapObject),
  Nil,
}

impl Value {
  pub fn is_truthy(&self) -> bool {
    use self::Value::*;

    match *self {
      Bool(n) => n,
      _       => false,
    }
  }
}

impl Hash for Value {
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

      Value::Char(n) => {
        state.write_u8(4);
        state.write_u32(
          unsafe {
            mem::transmute(n)
          }
        )
      },

      Value::HeapObject(n) => {
        state.write_u8(5);
        state.write_u64(
          unsafe {
            mem::transmute(n)
          }
        )
      },
    }
  }
}

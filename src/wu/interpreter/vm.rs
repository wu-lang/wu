use super::{ HeapObjectType, CompiledBlock, };
use std::ptr;

use super::super::error::Response::Wrong;
use super::value::HeapObject;
use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Code {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Neg,
  Lt,
  LtEq,
  Gt,
  GtEq,
  Eq,
  NEq,

  LoadConst(u16),
  LoadLocal(u16),
  StoreLocal(u16),

  BranchTrue(i16),
  BranchFalse(i16),
  Jump(i16),

  Pop,
  Return,
}

pub struct Machine {
  stack: Vec<Value>,
  next:  *mut HeapObject,
}

impl Machine {
  pub fn new() -> Self {
    Machine {
      stack: Vec::new(),
      next:  ptr::null_mut(),
    }
  }

  pub fn execute(&mut self, entry: &mut CompiledBlock) -> Result<(), ()> {
    use self::Code::*;
    use self::Value::*;

    let mut pointer = 0;
    let mut locals  = vec![Nil; entry.local_names.len()].into_boxed_slice();

    macro_rules! match_binop {
      ($($pat:pat => $block:block)+) => {{
        let _a = self.stack.pop().unwrap();
        let _b = self.stack.pop().unwrap();

        let _result = match (&_b, &_a) {
          $($pat => $block)+,
          _      => return Err(
            response!(
              Wrong(format!("invalid operation `{:?}` `{:?}`", _a, _b))
            )
          ),
        };

        self.stack.push(_result);
      }}
    }

    loop {
      let op = &entry.code[pointer];

      match *op {
        LoadConst(index)  => self.stack.push(entry.constants[index as usize].clone()),
        LoadLocal(index)  => self.stack.push(locals[index as usize].clone()),
        StoreLocal(index) => locals[index as usize] = self.stack.pop().unwrap(),

        BranchTrue(dif) => {
          if self.stack.pop().unwrap().is_truthy() {
            pointer = pointer.wrapping_add(dif as usize)
          } else {
            pointer = pointer.wrapping_add(1)
          }

          continue
        },

        BranchFalse(dif) => {
          if !self.stack.pop().unwrap().is_truthy() {
            pointer = pointer.wrapping_add(dif as usize)
          } else {
            pointer = pointer.wrapping_add(1)
          }

          continue
        },

        Jump(dif) => {
          pointer = pointer.wrapping_add(dif as usize);
          continue
        },

        Pop => { self.stack.pop().unwrap(); },

        Eq  => {
          let a = self.stack.pop();
          let b = self.stack.pop();

          self.stack.push(Bool(a == b))
        },

        Lt  => match_binop! {
          (&Int(a),   &Int(b))   => { Bool(a < b) }
          (&Int(a),   &Float(b)) => { Bool(a < b as i64) }
          (&Float(a), &Float(b)) => { Bool(a < b) }
          (&Float(a), &Int(b))   => { Bool(a < b as f64) }
        },

        Gt  => match_binop! {
          (&Int(a),   &Int(b))   => { Bool(a > b) }
          (&Int(a),   &Float(b)) => { Bool(a > b as i64) }
          (&Float(a), &Float(b)) => { Bool(a > b) }
          (&Float(a), &Int(b))   => { Bool(a > b as f64) }
        },

        NEq  => {
          let a = self.stack.pop();
          let b = self.stack.pop();

          self.stack.push(Bool(a != b))
        },

        Neg => match self.stack.pop().unwrap() {
          Int(n)   => self.stack.push(Int(-n)),
          Float(n) => self.stack.push(Float(-n)),

          ref c => return Err(
            response!(
              Wrong(format!("invalid negation: `{:?}`", c))
            )
          )
        },

        Add => match_binop! {
          (&Int(a),   &Int(b))   => { Int(a + b) }
          (&Float(a), &Float(b)) => { Float(a + b) }
        },

        Sub => match_binop! {
          (&Int(a),   &Int(b))   => { Int(a - b) }
          (&Float(a), &Float(b)) => { Float(a - b) }
        },

        Mul => match_binop! {
          (&Int(a),   &Int(b))   => { Int(a * b) }
          (&Float(a), &Float(b)) => { Float(a * b) }
        },

        Div => match_binop! {
          (&Int(a),   &Int(b))   => { Int(a / b) }
          (&Float(a), &Float(b)) => { Float(a / b) }
        },

        Mod => match_binop! {
          (&Int(a),   &Int(b))   => { Int(a % b) }
          (&Float(a), &Float(b)) => { Float(a % b) }
        },

        Return => break,

        _ => (),
      }

      pointer = pointer.wrapping_add(1)
    }

    Ok(())
  }



  pub fn alloc(&mut self, kind: HeapObjectType) -> Value {
    let object = Box::into_raw(
      Box::new(
        HeapObject {
          next: self.next,
          kind,
        }
      )
    );
    
    self.next = object;

    Value::HeapObject(object)
  }
}
use super::{ HeapObjectType, CompiledBlock, };

use std::ptr;
use std::mem;

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

  StoreArray(u16),

  Call(u16),

  Pop,
  Return,
}



pub struct CallInfo {
  locals:  Box<[Value]>,
  pointer: usize,
  func:    *const CompiledBlock,
}



pub struct Machine {
  pub stack: Vec<Value>,
  pub next:  *mut HeapObject,
  pub calls: Vec<CallInfo>,
}

impl Machine {
  pub fn new() -> Self {
    Machine {
      stack: Vec::new(),
      next:  ptr::null_mut(),
      calls: Vec::new(),
    }
  }

  pub fn execute(&mut self, entry: *const CompiledBlock) -> Result<(), ()> {
    use self::Code::*;
    use self::Value::*;

    let mut pointer = 0;

    let mut func   = unsafe { &*entry };
    let mut locals = vec![Nil; func.local_names.len()].into_boxed_slice();

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
      let op = &func.code[pointer];

      match *op {
        LoadConst(index)  => self.stack.push(func.constants[index as usize].clone()),
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
          (&Float(a), &Int(b))   => { Float(a + b as f64) }
        },

        Sub => match_binop! {
          (&Int(a),   &Int(b))   => { Int(a - b) }
          (&Float(a), &Float(b)) => { Float(a - b) }
          (&Float(a), &Int(b))   => { Float(a - b as f64) }
        },

        Mul => match_binop! {
          (&Int(a),   &Int(b))   => { Int(a * b) }
          (&Int(a),   &Float(b)) => { Float(a as f64 * b) }
          (&Float(a), &Int(b))   => { Float(a * b as f64) }
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

        Return => {
          if let Some(call_info) = self.calls.pop() {
            func    = unsafe { &*call_info.func };

            locals  = call_info.locals;
            pointer = call_info.pointer;
          } else {
            break
          }
        },

        StoreArray(ref len) => {
          let mut content = Vec::new();

          for _ in 0 .. *len {
            content.push(self.stack.pop().unwrap())
          }

          let array = self.alloc(HeapObjectType::Array(content));

          self.stack.push(array)
        },

        Call(args) => {
          let func_index = self.stack.len() - args as usize - 1;
          let func_value = self.stack[func_index].clone();

          let backup_func = func;

          func = if let HeapObject(object) = func_value {
            let object = unsafe { &*object };

            if let HeapObjectType::Function(ref func) = object.kind {
              func
            } else {
              return Err(
                response!(
                  Wrong(format!("calling non-callable: `{:?}`", func_value))
                )
              )
            }
          } else {
            return Err(
              response!(
                Wrong(format!("calling non-callable: `{:?}`", func_value))
              )
            )
          };

          let mut new_locals = vec![Nil; func.local_names.len()].into_boxed_slice();

          for index in 0 .. args as usize {
            new_locals[index] = self.stack[func_index + 1 + index].clone()
          }

          for _ in 0 .. args as usize + 1 {
            self.stack.pop();
          }

          let old_locals = mem::replace(&mut locals, new_locals);

          self.calls.push(
            CallInfo {
              pointer,
              locals: old_locals,
              func:   backup_func,
            }
          );

          pointer = 0;

          continue
        }

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
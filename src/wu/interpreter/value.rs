use std::hash::{ Hash, Hasher };
use std::mem;

#[derive(Debug, Clone)]
pub enum Value<'v> {
  Bool(bool),
  Int(i64),
  Float(f64),
  Str(&'v str),
  Nil,
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


      Value::Str(n) => {
        state.write_u8(5);
        state.write_u128(
          unsafe {
            mem::transmute(n)
          }
        )
      },
    }
  }
}

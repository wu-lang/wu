use super::value::*;

#[derive(Debug, Clone)]
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



pub struct Machine<'m> {
  stack: Vec<Value<'m>>,
}

impl<'m> Machine<'m> {
  pub fn new() -> Self {
    Machine {
      stack: Vec::new(),
    }
  }
}
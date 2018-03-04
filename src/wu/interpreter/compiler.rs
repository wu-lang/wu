use super::*;
use std::mem;

pub enum Value {
  Bool(bool),
  Int(i32),
  Float(f32),
  Char(char),
  Nil,
}



pub struct Compiler {
  pub bytecode: Vec<u8>,
}

impl Compiler {
  pub fn new() -> Self {
    Compiler {
      bytecode: Vec::new(),
    }
  }



  pub fn compile(&mut self, ast: &Vec<Statement>) -> Result<(), ()> {
    for statement in ast {
      self.compile_statement(statement)?
    }

    Ok(())
  }

  fn compile_statement(&mut self, statement: &Statement) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.compile_expression(expression)?,
      _ => (),
    }

    Ok(())
  }

  fn compile_expression(&mut self, expression: &Expression) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Int(ref n) => {
        self.emit(Instruction::PUSH as u8);
        self.emit(4);
        self.emit_bytes(
          unsafe {
            &mem::transmute::<i32, [u8; mem::size_of::<i32>()]>(*n)
          }
        );
      },

      _ => (),
    }

    Ok(())
  }



  fn emit(&mut self, byte: u8) {
    self.bytecode.push(byte)
  }

  fn emit_bytes(&mut self, bytes: &[u8]) {
    self.bytecode.extend(bytes)
  }
}
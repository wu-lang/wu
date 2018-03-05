#[macro_use()]
use super::*;
use std::mem;



#[macro_export]
macro_rules! to_bytes {
    ($value:expr => $t:ty) => {{
        unsafe { mem::transmute::<_,[u8;mem::size_of::<$t>()]>($value) }
    }}
}



pub struct Compiler<'c> {
  pub bytecode: Vec<u8>,
  pub visitor:  &'c mut Visitor<'c>,
  
  offset: u32,
}

impl<'c> Compiler<'c> {
  pub fn new(visitor: &'c mut Visitor<'c>) -> Self {
    Compiler {
      bytecode: Vec::new(),
      visitor,
      offset: 0,
    }
  }



  pub fn compile(&mut self, ast: &'c Vec<Statement>) -> Result<(), ()> {

    for statement in ast {
      self.compile_statement(statement)?
    }

    self.emit(Instruction::Halt as u8);

    Ok(())
  }

  fn compile_statement(&mut self, statement: &'c Statement) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.compile_expression(expression)?,
      Constant(ref t, _, ref right) => {
        self.compile_expression(right)?;
        self.emit(Instruction::Pop as u8);

        let right_type = self.visitor.type_expression(right)?;
        let size       = right_type.node.size_bytes();

        self.emit(right_type.node.size_bytes());

        let address = &to_bytes!(self.offset => u32);

        self.emit_bytes(address);

        self.offset += size as u32;
      }
      _ => (),
    }

    Ok(())
  }

  fn compile_expression(&mut self, expression: &Expression) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Int(ref n) => {
        self.emit(Instruction::Push as u8);
        self.emit(mem::size_of::<i32>() as u8);
        self.emit_bytes(
          unsafe {
            &mem::transmute::<i32, [u8; mem::size_of::<i32>()]>(*n)
          }
        );
      },

      Float(ref n) => {
        self.emit(Instruction::Push as u8);
        self.emit(mem::size_of::<f32>() as u8);
        self.emit_bytes(
          unsafe {
            &mem::transmute::<f32, [u8; mem::size_of::<f32>()]>(*n)
          }
        );
      },

      Char(ref n) => {
        self.emit(Instruction::Push as u8);
        self.emit(mem::size_of::<char>() as u8);
        self.emit(*n as u8)
      },

      String(ref n) => {
        self.emit(Instruction::Push as u8);
        self.emit(n.len() as u8);
        self.emit_bytes(n.as_bytes());
      },

      Bool(ref n) => {
        self.emit(Instruction::Push as u8);
        self.emit(mem::size_of::<u8>() as u8);
        self.emit(*n as u8)
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
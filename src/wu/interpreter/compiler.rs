#[macro_use]
use super::*;

use super::super::error::Response::Wrong;
use std::mem;



pub struct Compiler<'c> {
  pub bytecode: Vec<u8>,
  pub visitor:  &'c mut Visitor<'c>,
}

impl<'c> Compiler<'c> {
  pub fn new(visitor: &'c mut Visitor<'c>) -> Self {
    Compiler {
      bytecode: Vec::new(),
      visitor,
    }
  }



  pub fn compile(&mut self, ast: &'c Vec<Statement>) -> Result<(), ()> {
    println!();

    for statement in ast {
      self.compile_statement(statement)?
    }

    self.emit(Instruction::Halt);

    println!();

    Ok(())
  }

  fn compile_statement(&mut self, statement: &'c Statement) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.compile_expression(expression)?,
      Constant(ref t, ref left, ref right) => {
        if let ExpressionNode::Identifier(_) = left.node {
          self.assign(t, left, right)?
        }
      }

      Variable(ref t, ref left, ref right) => {
        if let ExpressionNode::Identifier(_) = left.node {
          if let &Some(ref right) = right {
            self.assign(t, left, right)?
          }
        } else if let ExpressionNode::Set(_) = left.node {
          return Err(
            response!(
              Wrong("set declaration compilation is unimplemented"),
              statement.pos
            )
          )
        }
      }

      _ => (),
    }

    Ok(())
  }

  fn compile_expression(&mut self, expression: &'c Expression) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Int(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(mem::size_of::<u128>() as u8);
        self.emit_bytes(
          unsafe {
            &mem::transmute::<u128, [u8; mem::size_of::<u128>()]>(*n)
          }
        );
      },

      Float(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(mem::size_of::<f64>() as u8);
        self.emit_bytes(
          unsafe {
            &mem::transmute::<f64, [u8; mem::size_of::<f64>()]>(*n)
          }
        );
      },

      Char(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(mem::size_of::<char>() as u8);
        self.emit_byte(*n as u8)
      },

      String(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(n.len() as u8);
        self.emit_bytes(n.as_bytes());
      },

      Bool(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(mem::size_of::<u8>() as u8);
        self.emit_byte(*n as u8)
      },

      Identifier(ref name) => {
        let (index, env_index) = self.visitor.symtab.get_name(name).unwrap();
        let offset             = self.visitor.typetab.get_offset(index, env_index).unwrap();
        let size               = self.visitor.typetab.get_type(index, env_index).unwrap().node.byte_size();

        self.emit(Instruction::PushDeref);
        self.emit_byte(size as u8);
        self.emit_bytes(&to_bytes!(offset => u32));
      },

      Binary(ref left, ref op, ref right) => {
        use self::Operator::*;

        let left_type = self.visitor.type_expression(left)?;

        match *op {
          Add => {
            self.compile_expression(left)?;
            self.compile_expression(right)?;

            if left_type.node.is_int() {
              self.emit(Instruction::AddI)
            } else if left_type.node.is_float() {
              self.emit(Instruction::AddF)
            }

            self.emit_byte(left_type.node.byte_size())
          },

          Eq | Lt | Gt | NEq | LtEq | GtEq => {
            self.compile_expression(left)?;

            let left_type = self.visitor.type_expression(left)?;

            if left_type.node.is_float() {
              if left_type.node != TypeNode::F64 {
                self.emit(Instruction::ConvFF);

                self.emit_byte(left_type.node.byte_size());
                self.emit_byte(8);
              }
            } else if left_type.node.is_int() {
              if left_type.node != TypeNode::I128 {
                self.emit(Instruction::ConvII);

                self.emit_byte(left_type.node.byte_size());
                self.emit_byte(16);
              }
            }

            self.compile_expression(right)?;

            let right_type = self.visitor.type_expression(right)?;

            if right_type.node.is_float() {
              if right_type.node != TypeNode::F64 {
                self.emit(Instruction::ConvFF);

                self.emit_byte(right_type.node.byte_size());
                self.emit_byte(8);
              }
            } else if right_type.node.is_int() {
              if right_type.node != TypeNode::I128 {
                self.emit(Instruction::ConvII);

                self.emit_byte(right_type.node.byte_size());
                self.emit_byte(16);
              }
            }

            match *op {
              Eq => if left_type.node.is_int() {
                self.emit(Instruction::EqI)
              } else if left_type.node.is_float() {
                self.emit(Instruction::EqF)
              },

              Lt => if left_type.node.is_int() {
                self.emit(Instruction::LtI)
              } else if left_type.node.is_float() {
                self.emit(Instruction::LtF)
              },

              Gt => if left_type.node.is_int() {
                self.emit(Instruction::GtI)
              } else if left_type.node.is_float() {
                self.emit(Instruction::GtF)
              },

              _ => (),
            }
          },

          _ => (),
        }
      }

      Cast(ref expression, ref t) => {
        use self::TypeNode::*;

        let size = self.visitor.type_expression(&expression)?.node.byte_size();

        self.compile_expression(expression)?;

        let mut sign_a = false;
        let mut sign_b = false;

        match (self.visitor.type_expression(expression)?.node, &t.node) {
          (ref a, ref b) if b.is_float() => match a {
            _ if a.is_int() => {
              if !a.is_uint() {
                sign_a = true
              }

              self.emit(Instruction::ConvIF);
            },

            _ if a.is_float() => self.emit(Instruction::ConvFF),

            c => return Err(
              response!(
                Wrong(format!("can't cast from `{}`", c)),
                expression.pos
              )
            )
          },

          (ref a, ref b) if b.is_int() => match a {
            _ if a.is_float() => {
              if !b.is_uint() {
                sign_b = true
              }

              self.emit(Instruction::ConvFI);
            }

            _ if a.is_int() => self.emit(Instruction::ConvII),

            other => return Err(
              response!(
                Wrong(format!("can't cast from `{}`", other)),
                expression.pos
              )
            )
          },

          (_, ref node) => return Err(response!(Wrong(format!("can't cast to `{}`", node))))
        }

        self.emit_byte(if sign_a { -(size as i8) as u8 } else { size as u8 });
        self.emit_byte(if sign_b { -(t.node.byte_size() as i8) as u8 } else { t.node.byte_size() as u8 })
      },

      If(ref condition, ref body, ref elses) => {

      },

      _ => (),
    }

    Ok(())
  }



  fn assign(&mut self, t: &Type, left: &'c Expression<'c>, right: &'c Expression<'c>) -> Result<(), ()> {
    use self::TypeNode::*;

    if let ExpressionNode::Identifier(ref name) = left.node {
      self.compile_expression(right)?;

      let right_type = self.visitor.type_expression(right)?;

      if right_type.node != t.node {
        match &t.node {
          c if c.is_int() => match &right_type.node {
            &I128 => {
              self.emit(Instruction::ConvII);
              self.emit_byte(right_type.node.byte_size() as u8);
              self.emit_byte(t.node.byte_size() as u8)
            },

            _ => (),
          },

          &F32 | &F64 => match &right_type.node {
            &I128 => {
              self.emit(Instruction::ConvIF);
              self.emit_byte(-(right_type.node.byte_size() as i8) as u8);
              self.emit_byte(t.node.byte_size() as u8)
            },

            &U128 => {
              self.emit(Instruction::ConvIF);
              self.emit_byte(right_type.node.byte_size() as u8);
              self.emit_byte(t.node.byte_size() as u8)
            },

            c if c.is_float() => {
              self.emit(Instruction::ConvFF);
              self.emit_byte(right_type.node.byte_size() as u8);
              self.emit_byte(t.node.byte_size() as u8)
            },

            _ => (),
          }

          _ => (),
        }
      }

      self.emit(Instruction::Pop);

      if t.node != Nil {
        self.emit_byte(t.node.byte_size() as u8);      
      } else {
        self.emit_byte(right_type.node.byte_size() as u8);
      }

      let (index, env_index) = self.visitor.symtab.get_name(name).unwrap();
      let offset             = self.visitor.typetab.get_offset(index, env_index).unwrap();

      let address = &to_bytes!(offset => u32);

      self.emit_bytes(address);
    }

    Ok(())
  }



  fn emit(&mut self, code: Instruction) {
    print!("\n{}", code);
    self.bytecode.push(code as u8)
  }

  fn emit_byte(&mut self, byte: u8) {
    print!("\t{} ", byte);
    self.bytecode.push(byte)
  }

  fn emit_bytes(&mut self, bytes: &[u8]) {
    print!("\t{:?} ", bytes);
    self.bytecode.extend(bytes)
  }
}
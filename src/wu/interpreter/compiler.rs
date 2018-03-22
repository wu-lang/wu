#[macro_use]
use super::*;

use super::super::error::Response::Wrong;
use std::mem;

use colored::Colorize;



pub struct Compiler<'c> {
  pub bytecode: Vec<u8>,
  pub visitor:  &'c mut Visitor<'c>,

  frame_index: usize,
}

impl<'c> Compiler<'c> {
  pub fn new(visitor: &'c mut Visitor<'c>) -> Self {
    /*let tabs: Vec<(SymTab, TypeTab)> = visitor.tab_frames.iter().map(|x| x.to_owned()).rev().collect();*/

    visitor.tabs = vec!(visitor.tab_frames.pop().unwrap());

    Compiler {
      bytecode: Vec::new(),
      visitor,
      frame_index: 0,
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

  fn compile_expression(&mut self, expression: &'c Expression<'c>) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Block(ref content) => {
        self.emit(Instruction::PushF);

        self.visitor.tabs.push(self.visitor.tab_frames.pop().unwrap());

        for element in content {
          self.compile_statement(element)?
        }

        self.visitor.tabs.pop();

        self.emit(Instruction::PopF);
      },

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
        let (index, env_index) = self.visitor.current_tab().0.get_name(name).unwrap();
        let offset             = self.visitor.current_tab().1.get_offset(index, env_index).unwrap();
        let size               = self.visitor.current_tab().1.get_type(index, env_index).unwrap().node.byte_size();

        self.emit(Instruction::PushDeref);
        self.emit_byte(size as u8);
        self.emit_bytes(&to_bytes!(offset => u32));
      },

      Function(ref params, ref return_type, ref body) => {
        use self::StatementNode::*;

        println!("\nfunc -> {}:", return_type);

        self.emit(Instruction::Jmp);

        let jump = self.bytecode.len();        // reference, for changing tmp address
        self.emit_bytes(&to_bytes!(0 => u32)); // the temporary adress

        let function_address = &to_bytes!(self.bytecode.len() as u32 => u32);

        self.visitor.tabs.push(self.visitor.tab_frames.pop().unwrap());

        for param in params {
          match param.node {
            Variable(ref t, ref left, ref right) => if right.is_none() {
              self.emit(Instruction::Pop);
              self.emit_byte(t.node.byte_size() as u8);

              if let ExpressionNode::Identifier(ref name) = left.node {
                let (index, env_index) = self.visitor.current_tab().0.get_name(name).unwrap();
                let offset             = self.visitor.current_tab().1.get_offset(index, env_index).unwrap();

                let address = &to_bytes!(offset => u32);

                self.emit_bytes(address);
              } else {
                unreachable!()
              }
            } else {
              panic!("this argument kind ain't implemented yet")
            },

            _ => panic!("this argument kind ain't implemented yet")
          }
        }

        self.compile_expression(body)?;

        self.visitor.tabs.pop(); // grr

        self.emit(Instruction::Ret);

        let address = to_bytes!(self.bytecode.len() as u32 => u32);

        for (offset, byte) in address.iter().enumerate() {
          self.bytecode[jump + offset] = *byte
        }

        println!("\n\t{} -> {:?}", "[altered last jmp]".bold(), address);

        self.emit(Instruction::Push);
        self.emit_byte(4);
        self.emit_bytes(function_address);

        println!()
      },

      Call(ref caller, ref args) => {
        for arg in args.iter().rev() {
          self.compile_expression(arg)?
        }

        self.compile_expression(caller)?;
        self.emit(Instruction::Call);
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
            
            self.emit_byte(left_type.node.byte_size() as u8)
          },

          Sub => {
            self.compile_expression(left)?;
            self.compile_expression(right)?;

            if left_type.node.is_int() {
              self.emit(Instruction::SubI)
            } else if left_type.node.is_float() {
              self.emit(Instruction::SubF)
            }

            self.emit_byte(left_type.node.byte_size() as u8)
          },

          Mul => {
            self.compile_expression(left)?;
            self.compile_expression(right)?;

            if left_type.node.is_int() {
              self.emit(Instruction::MulI)
            } else if left_type.node.is_float() {
              self.emit(Instruction::MulF)
            }

            self.emit_byte(left_type.node.byte_size() as u8)
          },

          Eq | Lt | Gt | NEq | LtEq | GtEq => {
            self.compile_expression(left)?;

            let left_type = self.visitor.type_expression(left)?;

            if left_type.node.is_float() {
              if left_type.node != TypeNode::F64 {
                self.emit(Instruction::ConvFF);

                self.emit_byte(left_type.node.byte_size() as u8);
                self.emit_byte(8);
              }
            } else if left_type.node.is_int() {
              if left_type.node != TypeNode::I128 {
                self.emit(Instruction::ConvII);

                self.emit_byte(left_type.node.byte_size() as u8);
                self.emit_byte(16);
              }
            }

            self.compile_expression(right)?;

            let right_type = self.visitor.type_expression(right)?;

            if right_type.node.is_float() {
              if right_type.node != TypeNode::F64 {
                self.emit(Instruction::ConvFF);

                self.emit_byte(right_type.node.byte_size() as u8);
                self.emit_byte(8);
              }
            } else if right_type.node.is_int() {
              if right_type.node != TypeNode::I128 {
                self.emit(Instruction::ConvII);

                self.emit_byte(right_type.node.byte_size() as u8);
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
        println!("\n\nif:");

        let mut jumps = Vec::new();            // TODO: reimplement jump buffer

        self.compile_expression(condition)?;

        self.emit(Instruction::JmpF);

        jumps.push(self.bytecode.len());       // reference, for changing tmp address
        self.emit_bytes(&to_bytes!(0 => u32)); // the temporary adress

        self.compile_expression(body)?;

        self.emit(Instruction::Jmp);
        jumps.insert(0, self.bytecode.len());

        self.emit_bytes(&to_bytes!(0 => u32));


        if let &Some(ref elses) = elses {
          for (index, &(ref maybe_condition, ref body, _)) in elses.iter().enumerate() {
            if let &Some(ref condition) = maybe_condition {
              println!("\n\nelif:");

              self.emit(Instruction::Dump);
              self.emit_byte(16);

              let address = to_bytes!(self.bytecode.len() as u32 => u32);

              let jump = jumps.pop().unwrap();

              for (offset, byte) in address.iter().enumerate() {
                self.bytecode[jump + offset] = *byte
              }

              println!("\n\n\t{} -> {:?}", "[altered last jmpf]".bold(), address);

              self.compile_expression(condition)?;

              if elses.len() - 1 > index {
                self.emit(Instruction::JmpF);

                jumps.push(self.bytecode.len());       // reference, for changing tmp address
                self.emit_bytes(&to_bytes!(0 => u32)); // the temporary adress
              }

              self.compile_expression(body)?;

              self.emit(Instruction::Jmp);
              jumps.insert(0, self.bytecode.len());

              self.emit_bytes(&to_bytes!(0 => u32));

            } else {
              println!("\n\nelse:");
              let address = to_bytes!(self.bytecode.len() as u32 => u32);

              let jump = jumps.pop().unwrap();

              for (offset, byte) in address.iter().enumerate() {
                self.bytecode[jump + offset] = *byte
              }

              println!("\n\t{} -> {:?}", "[altered last jmpf]".bold(), address);
    
              self.compile_expression(body)?;
            }
          }
        } else {
          let address = to_bytes!(self.bytecode.len() as u32 => u32);

          for (offset, byte) in address.iter().enumerate() {
            self.bytecode[jumps.last().unwrap() + offset] = *byte;
          }

          println!("\n\t{} -> {:?}", "[altered last jmpf]".bold(), address);
        }

        for jump in jumps { // take care of last jmps
          let address = to_bytes!(self.bytecode.len() as u32 => u32);

          for (offset, byte) in address.iter().enumerate() {
            self.bytecode[jump + offset] = *byte
          }

          println!("\n\t{} -> {:?}", "[altered jmp]".bold(), address);
        }
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

      let (index, env_index) = self.visitor.current_tab().0.get_name(name).unwrap();
      let offset             = self.visitor.current_tab().1.get_offset(index, env_index).unwrap();

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
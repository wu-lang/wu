use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::mem;

use super::{ Machine, Code, Value, HeapObjectType, };
use super::super::error::Response::Wrong;

use super::ast::*;
use super::TypeNode;

#[derive(Clone, Copy)]
pub struct JumpPatch(usize);

#[derive(Clone, Copy)]
pub struct BranchTarget(usize);



#[derive(Debug, Clone, PartialEq)]
pub struct CompiledBlock {
  pub name:        String,
  pub code:        Box<[Code]>,
  pub constants:   Box<[Value]>,
  pub local_names: Box<[String]>,
}



pub struct Compiler<'c> {
  pub locals:    HashMap<&'c str, u16>,
  pub constants: Vec<Value>,
  pub code:      Vec<Code>,
  pub vm:        &'c mut Machine,
}

impl<'c> Compiler<'c> {
  pub fn new(vm: &'c mut Machine) -> Self {
    Compiler {
      locals:    HashMap::new(),
      code:      Vec::new(),
      constants: Vec::new(),
      vm,
    }
  }

  pub fn declare_local(&mut self, name: &'c str) -> Result<u16, ()> {
    if self.locals.len() > (u16::max_value() as usize) {
      Err(
        response!(
          Wrong(format!("local overflow at name `{}`", name))
        )
      )
    } else {
      let index = self.locals.len() as u16;
      let entry = self.locals.entry(name);

      match entry {
        Entry::Vacant(value) => {
          value.insert(index);
          Ok(index)
        },

        _ => Err(
          response!(
            Wrong(format!("non-vacant entry `{}`", name))
          )
        )
      }
    }
  }

  pub fn fetch_local(&mut self, name: &str) -> u16 {
    self.locals.get(name).map(|i| *i).unwrap()
  }

  pub fn emit(&mut self, code: Code) {
    self.code.push(code);
  }

  pub fn emit_load_const(&mut self, value: Value) {
    let idx = self.constants.len();
    if idx < (u16::max_value() as usize) {
      let idx = idx as u16;

      self.constants.push(value);
      self.emit(Code::LoadConst(idx))
    }
  }

  pub fn emit_branch_false(&mut self) -> JumpPatch {
    let jump = JumpPatch(self.code.len());
    self.emit(Code::BranchFalse(0));

    jump
  }

  pub fn emit_branch_true(&mut self) -> JumpPatch {
    let jump = JumpPatch(self.code.len());
    self.emit(Code::BranchTrue(0));

    jump
  }

  pub fn emit_jump(&mut self) -> JumpPatch {
    let jump = JumpPatch(self.code.len());
    self.emit(Code::Jump(0));

    jump
  }

  pub fn save_branch_target(&self) -> BranchTarget {
    BranchTarget(self.code.len())
  }

  pub fn patch_jump(&mut self, patch: JumpPatch) {
    use self::Code::*;

    let cur = self.code.len();
    let branch_loc = patch.0;
    let diff = (cur as isize) - (branch_loc as isize);

    if diff < (i16::max_value() as isize) || diff < (i16::min_value() as isize) {
      let diff = diff as i16;

      match self.code[branch_loc] {
        Jump(_)        => self.code[branch_loc] = Jump(diff),
        BranchTrue(_)  => self.code[branch_loc] = BranchTrue(diff),
        BranchFalse(_) => self.code[branch_loc] = BranchFalse(diff),
        _              => unreachable!(),
      }
    }
  }

  pub fn emit_jump_to_target(&mut self, target: BranchTarget) {
    let cur = self.code.len();
    let BranchTarget(target) = target;
    let diff = (target as isize) - (cur as isize);

    if !(diff > (i16::max_value() as isize) || diff < (i16::min_value() as isize)) {
      let diff = diff as i16;
      self.emit(Code::Jump(diff))
    }
  }



  fn compile_statement(&mut self, statement: &'c Statement<'c>) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.compile_expression(expression)?,
      Constant(_, ref left, ref right) => match left.node {
        ExpressionNode::Identifier(ref name) => {
          self.compile_expression(right)?;

          let index = self.declare_local(name)?;

          self.emit(Code::StoreLocal(index))
        },

        ExpressionNode::Set(ref content) => {
          self.compile_expression(right)?; // push content of tuple set onto stack

          for element in content.iter() {
            match element.node {
              ExpressionNode::Identifier(ref name) => {
                let index = self.declare_local(name)?;

                self.emit(Code::StoreLocal(index)) // pop and assign
              },

              _ => unreachable!(),
            }
          }
        }

        _ => unreachable!(),
      },

      Variable(_, ref left, ref right) => {
        match left.node {
          ExpressionNode::Identifier(ref name) => if let &Some(ref right) = right {
            self.compile_expression(right)?;

            let index = self.declare_local(name)?;

            self.emit(Code::StoreLocal(index))
          } else {
            self.declare_local(name)?;
          },

          ExpressionNode::Set(ref content) => if let &Some(ref right) = right {
            self.compile_expression(right)?; // push content of tuple set onto stack

            for element in content.iter() {
              match element.node {
                ExpressionNode::Identifier(ref name) => {
                  let index = self.declare_local(name)?;

                  self.emit(Code::StoreLocal(index)) // pop and assign
                },

                _ => unreachable!(),
              }
            }
          } else {
            for element in content {
              match element.node {
                ExpressionNode::Identifier(ref name) => {
                  self.declare_local(name)?;
                },

                _ => unreachable!()
              }
            }
          }

          _ => unreachable!()
        }
      }
      _ => (),
    }

    Ok(())
  }

  fn compile_expression(&mut self, expression: &'c Expression<'c>) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Int(ref n)    => self.emit_load_const(Value::Int(n.clone())),
      Float(ref n)  => self.emit_load_const(Value::Float(n.clone())),
      Bool(ref n)   => self.emit_load_const(Value::Bool(n.clone())),
      Char(ref n)   => self.emit_load_const(Value::Char(*n)),
      String(ref n) => {
        let value = self.vm.alloc(HeapObjectType::Str(n.clone().into_boxed_str()));
        self.emit_load_const(value)
      },

      Block(ref statements) => for statement in statements {
        self.compile_statement(statement)?
      }

      Cast(ref expression, ref t) => match (&expression.node, &t.node) {
        (&Int(ref n), &TypeNode::Float)   => self.emit_load_const(Value::Float(*n as f64)),
        (&Int(ref n), &TypeNode::Int)     => self.emit_load_const(Value::Int(*n)),
        (&Float(ref n), &TypeNode::Float) => self.emit_load_const(Value::Float(*n)),
        _ => self.compile_expression(expression)?
      }

      Binary(ref left, ref op, ref right) => {
        use self::Operator::*;

        self.compile_expression(left)?;
        self.compile_expression(right)?;

        match *op {
          Add => self.emit(Code::Add),
          Sub => self.emit(Code::Sub),
          Mul => self.emit(Code::Mul),
          Div => self.emit(Code::Div),
          _   => (),
        }
      }

      Identifier(ref name) => {
        let index = self.fetch_local(name);
        self.emit(Code::LoadLocal(index))
      },

      Set(ref content) => for element in content {
        self.compile_expression(element)?
      },

      Array(ref content) => {
        self.vm.alloc(HeapObjectType::Array(Vec::new())); // newarray; reference to heap allocation stored in `self.vm.next`

        for expression in content {
          self.compile_expression(expression)? // push content
        }

        self.emit(Code::StoreArray(content.len() as u16)) // pops content values, store those in allocated array slot, pushes array reference
      },

      Function(ref params, _, ref body) => {
        use self::ExpressionNode::*;
        use self::StatementNode::*;

        let function = {
          let mut locals = HashMap::new();

          for (index, param) in params.iter().enumerate() {
            match param.node {
              Constant(_, ref name, _) | Variable(_, ref name, _) => if let Identifier(ref name) = name.node {
                locals.insert(name.as_str(), index as u16);
              } else {
                unreachable!()
              },

              _ => unreachable!()
            }
          }

          let mut compiler = Compiler {
            locals,
            code: Vec::new(),
            constants: Vec::new(),
            vm: self.vm,
          };

          compiler.compile_entry(&body, "<fun>")?
        };

        let value = self.vm.alloc(HeapObjectType::Function(function));

        self.emit_load_const(value)
      },

      Call(ref caller, ref args) => {
        self.compile_expression(caller)?;

        for arg in args {
          self.compile_expression(arg)?
        }

        self.emit(Code::Call(args.len() as u16))
      }

      _ => (),
    }

    Ok(())
  }



  pub fn compile_entry(&mut self, block: &'c Expression<'c>, name: &'c str) -> Result<CompiledBlock, ()> {
    self.compile_expression(&block)?;

    self.emit_load_const(Value::Nil);

    self.code.push(Code::Return);

    let mut local_names = vec![String::new(); self.locals.len()];

    for (name, index) in self.locals.drain() {
      local_names[index as usize] = name.to_owned();
    }

    Ok(
      CompiledBlock {
        name:        name.to_string(),
        code:        mem::replace(&mut self.code, Vec::new()).into_boxed_slice(),
        constants:   mem::replace(&mut self.constants, Vec::new()).into_boxed_slice(),
        local_names: local_names.into_boxed_slice(),
      }
    )
  }
}
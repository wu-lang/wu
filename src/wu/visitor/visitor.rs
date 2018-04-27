use super::*;
use super::super::parser::Parser;
use super::super::error::Response::Wrong;

use std::fmt::{ self, Formatter, Write, Display };

use std::rc::Rc;
use std::mem;



#[derive(Debug, Clone)]
pub enum TypeNode {
  I08,
  I32,
  I64,
  I128,

  F64,
  F32,

  U08,
  U32,
  U64,
  U128,

  Bool,
  Str,
  Char,
  Nil,
  Id(String),
  Set(Vec<Type>),
  Array(Rc<Type>, u32),
  Func(Vec<Type>, Rc<Type>),
}

impl TypeNode {
  pub fn byte_size(&self) -> i8 {
    use self::TypeNode::*;

    match *self {
      I08  => mem::size_of::<i8>()   as i8,
      I32  => mem::size_of::<i32>()  as i8,
      I64  => mem::size_of::<i64>()  as i8,
      I128 => mem::size_of::<i128>() as i8,

      F32  => mem::size_of::<f32>() as i8,
      F64  => mem::size_of::<f64>() as i8,

      U08  => -(mem::size_of::<u8>()   as i8),
      U32  => -(mem::size_of::<u32>()  as i8),
      U64  => -(mem::size_of::<u64>()  as i8),
      U128 => -(mem::size_of::<u128>() as i8),

      Char => mem::size_of::<char>() as i8,
      Bool => mem::size_of::<bool>() as i8,

      Func(..) => 4, // address size

      Array(ref t, ref len) => t.node.byte_size(),
      ref other             => panic!("no type size: {:?}", other),
    }
  }

  pub fn check_expression(&self, other: &ExpressionNode) -> bool {
    use self::TypeNode::*;

    match *other {
      ExpressionNode::Int(_) => match *self {
        I08 | I32 | I64 | I128 | F32 | F64 | U08 | U32 | U64 | U128 => true,
        _ => false,
      },

      ExpressionNode::Float(_) => match *self {
        F32 | F64 => true,
        _         => false,
      },

      ExpressionNode::Array(ref content) => {
        for element in content {
          if let &Array(ref content, _) = self {
            if !content.node.check_expression(&element.node) {
              println!("{:?} checking {:?}", self, element.node);
              return false
            }
          }
        }

        true
      },

      _ => false
    }
  }

  pub fn is_int(&self) -> bool {
    use self::TypeNode::*;

    [I08, I32, I64, I128, U08, U32, U64, U128].contains(&self)
  }

  pub fn is_uint(&self) -> bool {
    use self::TypeNode::*;

    [U08, U32, U64, U128].contains(&self)
  }

  pub fn is_float(&self) -> bool {
     use self::TypeNode::*;

    [F32, F64].contains(&self)
  }
}

impl PartialEq for TypeNode {
  fn eq(&self, other: &TypeNode) -> bool {
    use self::TypeNode::*;

    match (self, other) {
      (&I08,  &I08)  => true,
      (&I32,  &I32)  => true,
      (&I64,  &I64)  => true,
      (&I128, &I128) => true,

      (&F32,  &F32) => true,
      (&F64,  &F64) => true,

      (&U08,  &U08)  => true,
      (&U32,  &U32)  => true,
      (&U64,  &U64)  => true,
      (&U128, &U128) => true,

      (&Bool, &Bool) => true,
      (&Str,  &Str)  => true,
      (&Char, &Char) => true,
      (&Nil,  &Nil)  => true,

      (&Array(ref a, _), &Array(ref b, _)) => a == b,
      (&Id(ref a), &Id(ref b))             => a == b,
      (&Set(ref a), &Set(ref b))           => a == b,

      _                                    => false,
    }
  }
}



#[derive(Debug, Clone)]
pub enum TypeMode {
  Undeclared,
  Immutable,
  Optional,
  Regular,
}

impl Display for TypeNode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::TypeNode::*;

    match *self {
      I08                 => write!(f, "i8"),
      I32                 => write!(f, "i32"),
      I64                 => write!(f, "i64"),
      I128                => write!(f, "i128"),

      F64                 => write!(f, "f64"),
      F32                 => write!(f, "f32"),

      U08                 => write!(f, "u8"),
      U32                 => write!(f, "u32"),
      U64                 => write!(f, "u64"),
      U128                => write!(f, "u128"),

      Bool                => write!(f, "bool"),
      Str                 => write!(f, "str"),
      Char                => write!(f, "char"),
      Nil                 => write!(f, "nil"),
      Array(ref n, ref l) => write!(f, "[{}; {}]", n, l),
      Id(ref n)           => write!(f, "{}", n),
      Set(ref content)    => {
        write!(f, "(");

        for (index, element) in content.iter().enumerate() {
          if index < content.len() - 1 {
            write!(f, "{}, ", element)?          
          } else {
            write!(f, "{}", element)?
          }
        }

        write!(f, ")")
      },
      Func(ref params, ref return_type) => {
        write!(f, "(");

        for (index, element) in params.iter().enumerate() {
          if index < params.len() - 1 {
            write!(f, "{}, ", element)?          
          } else {
            write!(f, "{}", element)?
          }
        }

        write!(f, ") {}", return_type)
      },
    }
  }
}



impl TypeMode {
  pub fn check(&self, other: &TypeMode) -> bool {
    use self::TypeMode::{ Optional, Immutable, Regular, Undeclared, };

    match (self, other) {
      (&Regular,       &Regular)    => true,
      (&Immutable,     &Immutable)  => true,
      (&Undeclared,    &Undeclared) => true,
      (&Optional,      &Optional)   => true,
      _                             => false,
    }
  }
}

impl Display for TypeMode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::TypeMode::*;

    match *self {
      Regular    => Ok(()),
      Immutable  => write!(f, "constant "),
      Undeclared => write!(f, "undeclared "),
      Optional   => write!(f, "optional "),
    }
  }
}

impl PartialEq for TypeMode {
  fn eq(&self, other: &TypeMode) -> bool {
    use self::TypeMode::*;

    match (self, other) {
      (&Regular,    &Regular)    => true,
      (&Regular,    &Immutable)  => true,
      (&Immutable,  &Immutable)  => true,
      (&Immutable,  &Regular)    => true,
      (_,           &Optional)   => true,
      (&Optional,   _)           => true,
      (&Undeclared, _)           => false,
      (_,           &Undeclared) => false,
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Type {
  pub node: TypeNode,
  pub mode: TypeMode,
}

impl Type {
  pub fn new(node: TypeNode, mode: TypeMode) -> Self {
    Type {
      node, mode,
    }
  }

  pub fn id(id: &str) -> Type {
    Type::new(TypeNode::Id(id.to_owned()), TypeMode::Regular)
  }

  pub fn from(node: TypeNode) -> Type {
    Type::new(node, TypeMode::Regular)
  }

  pub fn set(content: Vec<Type>) -> Type {
    Type::new(TypeNode::Set(content), TypeMode::Regular)
  }

  pub fn array(t: Type, len: u32) -> Type {
    Type::new(TypeNode::Array(Rc::new(t), len), TypeMode::Regular)
  }

  pub fn function(params: Vec<Type>, return_type: Type) -> Type {
    Type::new(TypeNode::Func(params, Rc::new(return_type)), TypeMode::Regular)
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}{}", self.mode, self.node)
  }
}



pub struct Visitor<'v> {
  pub tabs:       Vec<(SymTab, TypeTab)>,
  pub tab_frames: Vec<(SymTab, TypeTab)>,

  pub source:  &'v Source,
  pub ast:     &'v Vec<Statement<'v>>,

  pub offsets: Vec<u32>,
  pub depth:   u32,
}

impl<'v> Visitor<'v> {
  pub fn new(source: &'v Source, ast: &'v Vec<Statement<'v>>) -> Self {
    Visitor {
      tabs:       vec!((SymTab::global(), TypeTab::global())),
      tab_frames: Vec::new(), // very intelligent hack

      source,
      ast,
      offsets: vec!(0),
      depth:   0,
    }
  }

  pub fn visit(&mut self) -> Result<(), ()> {
    for statement in self.ast {
      self.visit_statement(&statement)?
    }

    self.tab_frames.push(self.tabs.last().unwrap().clone());

    Ok(())
  }

  pub fn visit_statement(&mut self, statement: &'v Statement<'v>) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.visit_expression(expression),

      Variable(_, ref left, _) => match left.node {
        ExpressionNode::Identifier(_) | ExpressionNode::Set(_) => {
          self.visit_variable(&statement.node)
        },
        _ => Ok(())
      },

      Constant(_, ref left, _) => match left.node {
        ExpressionNode::Identifier(_) | ExpressionNode::Set(_) => self.visit_constant(&statement.node),
        _ => Ok(())
      },
    }
  }

  fn visit_expression(&mut self, expression: &'v Expression<'v>) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Identifier(ref name) => if self.current_tab().0.get_name(name).is_none() {
        Err(
          response!(
            Wrong(format!("no such value `{}` in this scope", name)),
            self.source.file,
            expression.pos
          )
        )
      } else {
        Ok(())
      },

      Set(ref content) => {
        for expression in content {
          self.visit_expression(expression)?
        }

        Ok(())
      },

      Block(ref statements) => {
        self.push_scope();

        for statement in statements {
          self.visit_statement(statement)?
        }

        self.pop_scope();

        Ok(())
      },

      Loop(ref body) => self.visit_expression(body),

      If(ref condition, ref body, ref elses) => {
        self.visit_expression(&*condition)?;

        let condition_type = self.type_expression(&*condition)?.node;

        if condition_type == TypeNode::Bool {

          self.push_scope();

          self.visit_expression(body)?;
          let body_type = self.type_expression(body)?;

          self.pop_scope();

          if let &Some(ref elses) = elses {
            for &(ref maybe_condition, ref body, _) in elses {
              if let Some(ref condition) = *maybe_condition {
                let condition_type = self.type_expression(condition)?.node;

                if condition_type != TypeNode::Bool {
                  return Err(
                    response!(
                      Wrong(format!("mismatched condition, must be `bool` got `{}`", condition_type)),
                      self.source.file,
                      condition.pos
                    )
                  )
                }
              }

              self.push_scope();

              self.visit_expression(body)?;
              let else_body_type = self.type_expression(body)?;

              self.pop_scope();

              if body_type != else_body_type {
                return Err(
                  response!(
                    Wrong(format!("mismatched types, expected `{}` got `{}`", body_type, else_body_type)),
                    self.source.file,
                    body.pos
                  )
                )
              }
            }
          }

          Ok(())

        } else {
          return Err(
            response!(
              Wrong(format!("mismatched condition, must be `bool` got `{}`", condition_type)),
              self.source.file,
              expression.pos
            )
          )
        }
      },

      Call(ref expression, ref args) => {
        self.visit_expression(expression)?;

        let expression_type = self.type_expression(expression)?.node;

        if let TypeNode::Func(ref params, ..) = expression_type {
          for (index, param) in params.iter().enumerate() {
            let arg_type = self.type_expression(&args[index])?;

            if !param.node.check_expression(&args[index].node) && param != &arg_type {
              return Err(
                response!(
                  Wrong(format!("mismatched argument, expected `{}` got `{}`", expression_type, arg_type)),
                  self.source.file,
                  expression.pos
                )
              )
            }
          }
        } else {
          return Err(
            response!(
              Wrong(format!("expected function, found `{}`", expression_type)),
              self.source.file,
              expression.pos
            )
          )
        }

        Ok(())
      },

      Function(ref params, ref return_type, ref body) => {
        use self::ExpressionNode::*;
        use self::StatementNode::*;

        let mut param_names = Vec::new();
        let mut param_types = Vec::new();

        // simply pushing without pushing
        self.depth += 1;

        for param in params {
          match param.node {
            Constant(ref t, ref name, _) | Variable(ref t, ref name, _) => if let Identifier(ref name) = name.node {
              param_names.push(name.clone());

              let offset = *self.offsets.last().unwrap();

              param_types.push((t.clone(), offset, self.depth));

              let len = self.offsets.len();
              self.offsets[len - 1] += t.node.byte_size() as u32
            } else {
              return Err(
                response!(
                  Wrong("set parameters are work-in-progress"),
                  self.source.file,
                  param.pos
                )
              )
            },

            _ => unreachable!()
          }
        }

        self.offsets.push(0);

        let parent = self.current_tab().clone();

        self.tabs.push(
          (
            SymTab::new(Rc::new(parent.0), &param_names),
            TypeTab::new(Rc::new(parent.1), &param_types)
          )
        );


        self.visit_expression(body)?;
        let body_type = self.type_expression(body)?;

        self.pop_scope();

        if return_type != &body_type {
          Err(
            response!(
              Wrong(format!("mismatched return type, expected `{}` got `{}`", return_type, body_type)),
              self.source.file,
              expression.pos
            )
          )
        } else {
          Ok(())
        }
      },

      Array(ref content) => {
        let t = self.type_expression(content.first().unwrap())?;

        for element in content {
          let element_type = self.type_expression(element)?;

          if !t.node.check_expression(&Parser::fold_expression(element)?.node) && t.node != element_type.node {
            return Err(
              response!(
                Wrong(format!("mismatched types in array, expected `{}` got `{}`", t, element_type)),
                self.source.file,
                element.pos
              )
            )
          }
        }

        Ok(())
      },

      Index(ref left, ref index) => {
        let left_type = self.type_expression(left)?;

        if let TypeNode::Array(_, ref len) = left_type.node {
          let index_type = self.type_expression(index)?;

          if !index_type.node.is_uint() || !TypeNode::U32.check_expression(&index.node) {
            if let Int(ref index) = Parser::fold_expression(&**index)?.node {
              if *index as u32 > *len {
                return Err(
                  response!(
                    Wrong(format!("static index out of bounds `{}` in `{}`", index, len)),
                    self.source.file,
                    left.pos
                  )
                )
              }
            }
          } else {
            return Err(
              response!(
                Wrong(format!("can't index with `{}`, must be unsigned integer", index_type)),
                self.source.file,
                left.pos
              )
            )
          }

        } else {
          return Err(
            response!(
              Wrong(format!("can't index `{}`", left_type)),
              self.source.file,
              left.pos
            )
          )
        }

        Ok(())
      },

      _ => Ok(())
    }
  }

  fn visit_variable(&mut self, variable: &'v StatementNode) -> Result<(), ()> {
    use self::ExpressionNode::{Identifier, Set};

    if let &StatementNode::Variable(ref variable_type, ref left, ref right) = variable {
      match left.node {
        Identifier(ref name) => {
          let index = if let Some((index, _)) = self.current_tab().0.get_name(name) {
            index
          } else {
            self.current_tab().0.add_name(name)
          };

          self.current_tab().1.grow();

          if let &Some(ref right) = right {
            self.visit_expression(&right)?;

            let right_type = self.type_expression(&right)?;

            if variable_type.node != TypeNode::Nil {
              if !variable_type.node.check_expression(&Parser::fold_expression(right)?.node) && variable_type.node != right_type.node {
                return Err(
                  response!(
                    Wrong(format!("mismatched types, expected type `{}` got `{}`", variable_type.node, right_type)),
                    self.source.file,
                    right.pos
                  )
                )
              } else {
                let offset = *self.offsets.last().unwrap();
                let depth  = self.depth;
                
                self.current_tab().1.set_type(index, 0, (variable_type.to_owned(), offset, depth))?;

                let len = self.offsets.len();

                self.offsets[len - 1] += variable_type.node.byte_size().abs() as u32
              }

            } else {
              let size = right_type.node.byte_size().abs() as u32;

              let offset = *self.offsets.last().unwrap();
              let depth  = self.depth;
              
              self.current_tab().1.set_type(index, 0, (right_type, offset, depth))?;

              let len = self.offsets.len();

              self.offsets[len - 1] += size;
            }

          } else {
            let offset = *self.offsets.last().unwrap();
            let depth  = self.depth;
            

            self.current_tab().1.set_type(index, 0, (variable_type.to_owned(), offset, depth))?;

            let len = self.offsets.len();

            self.offsets[len - 1] += variable_type.node.byte_size() as u32
          }
        },

        Set(ref names) => {
          if let &Some(ref right) = right {
            let right_content = match right.node {
              Set(ref content) => content,

              _ => return Err(
                response!(
                  Wrong("can't assign set to non-set"),
                  self.source.file,
                  left.pos
                )
              )
            };          

            for (content_index, expression) in names.iter().enumerate() {            
              if let Identifier(ref name) = expression.node {
                let index = if let Some((index, _)) = self.current_tab().0.get_name(name) {
                  index
                } else {
                  self.current_tab().1.grow();
                  self.current_tab().0.add_name(name)
                };

                if let Some(right) = right_content.get(content_index) {                
                  self.visit_expression(&right)?;

                  let right_type = self.type_expression(right)?;

                  if variable_type.node != TypeNode::Nil {                  
                    if let TypeNode::Set(ref type_content) = variable_type.node {
                      if type_content[content_index] != right_type {
                        return Err(
                          response!(
                            Wrong(format!("mismatched types, expected type `{}` got `{}`", type_content[content_index], right_type)),
                            self.source.file,
                            right.pos
                          )
                        )
                      } else {
                        let offset = *self.offsets.last().unwrap();
                        let depth  = self.depth;

                        self.current_tab().1.set_type(index, 0, (variable_type.to_owned(), offset, depth))?;
                        
                        let len = self.offsets.len();

                        self.offsets[len - 1] += variable_type.node.byte_size() as u32
                      }
                    } else {
                      return Err(
                        response!(
                          Wrong(format!("mismatched types of set declaration got `{}`", variable_type.node)),
                          self.source.file,
                          left.pos
                        )
                      )
                    }
                  } else {                  
                    let size   = right_type.node.byte_size() as u32;
                    let offset = *self.offsets.last().unwrap();
                    let depth  = self.depth;
                    

                    self.current_tab().1.set_type(index, 0, (right_type, offset, depth))?;
                    
                    let len = self.offsets.len();

                    self.offsets[len - 1] += size
                  }
                } else {
                  return Err(
                    response!(
                      Wrong("missing"),
                      self.source.file,
                      right.pos
                    )
                  )
                }
              }
            }
          } else {
            for expression in names {            
              if let Identifier(ref name) = expression.node {
                let index = if let Some((index, _)) = self.current_tab().0.get_name(name) {
                  index
                } else {
                  self.current_tab().1.grow();
                  self.current_tab().0.add_name(name)
                };

                let offset = *self.offsets.last().unwrap();
                let depth  = self.depth;

                self.current_tab().1.set_type(index, 0, (variable_type.to_owned(), offset, depth))?;
                
                let len = self.offsets.len();

                self.offsets[len - 1] += variable_type.node.byte_size() as u32
              }
            }
          }
        }

        _ => return Err(
          response!(
            Wrong("unexpected variable declaration"),
            self.source.file,
            left.pos
          )
        )
      }

      Ok(())
    } else {
      unreachable!()
    }
  }

  fn visit_constant(&mut self, constant: &'v StatementNode) -> Result<(), ()> {
    use self::ExpressionNode::*;

    if let &StatementNode::Constant(ref constant_type, ref left, ref right) = constant {
      match left.node {
        Identifier(ref name) => {
          let index = if let Some((index, _)) = self.current_tab().0.get_name(name) {
            index
          } else {
            self.current_tab().0.add_name(name)
          };

          self.current_tab().1.grow();

          match right.node {
            Function(..) | Block(_) => (),
            _                       => self.visit_expression(right)?,
          }

          let right_type = self.type_expression(right)?;

          if constant_type.node != TypeNode::Nil {
            if !constant_type.node.check_expression(&Parser::fold_expression(right)?.node) && constant_type != &right_type {
              return Err(
                response!(
                  Wrong(format!("mismatched types, expected type `{}` got `{}`", constant_type.node, right_type)),
                  self.source.file,
                  right.pos
                )
              )
            } else {
              let offset = *self.offsets.last().unwrap();
              let depth  = self.depth;

              self.current_tab().1.set_type(index, 0, (constant_type.to_owned(), offset, depth))?;

              let len = self.offsets.len();

              self.offsets[len - 1] += constant_type.node.byte_size() as u32
            }
          } else {
            let size   = right_type.node.byte_size().abs() as u32;
            let offset = *self.offsets.last().unwrap();
            let depth  = self.depth;

            self.current_tab().1.set_type(index, 0, (right_type, offset, depth))?;

            let len = self.offsets.len();

            self.offsets[len - 1] += size
          }

          match right.node {
            Function(..) | Block(_) => self.visit_expression(right)?,
            _                       => (),
          }
        },

        Set(ref names) => {          
          let right_content = match right.node {
            Set(ref content) => content,

            _ => return Err(
              response!(
                Wrong("can't assign set to non-set"),
                self.source.file,
                left.pos
              )
            )
          };          

          for (content_index, expression) in names.iter().enumerate() {            
            if let Identifier(ref name) = expression.node {
              let index = if let Some((index, _)) = self.current_tab().0.get_name(name) {
                index
              } else {
                self.current_tab().1.grow();
                self.current_tab().0.add_name(name)
              };

              if let Some(right) = right_content.get(content_index) {                
                self.visit_expression(&right)?;

                let right_type = self.type_expression(right)?;

                if constant_type.node != TypeNode::Nil {                  
                  if let TypeNode::Set(ref type_content) = constant_type.node {
                    if type_content[content_index] != right_type {                      
                      return Err(
                        response!(
                          Wrong(format!("mismatched types, expected type `{}` got `{}`", type_content[content_index], right_type)),
                          self.source.file,
                          right.pos
                        )
                      )
                    } else {
                      let offset = *self.offsets.last().unwrap();
                      let depth  = self.depth;

                      self.current_tab().1.set_type(index, 0, (constant_type.to_owned(), offset, depth))?;

                      let len = self.offsets.len();

                      self.offsets[len - 1] += constant_type.node.byte_size() as u32
                    }
                  } else {
                    return Err(
                      response!(
                        Wrong(format!("mismatched types of set declaration got `{}`", constant_type.node)),
                        self.source.file,
                        left.pos
                      )
                    )
                  }
                } else {                  
                  let size = right_type.node.byte_size() as u32;

                  let offset = *self.offsets.last().unwrap();
                  let depth  = self.depth;

                  self.current_tab().1.set_type(index, 0, (right_type, offset, depth))?;

                  let len = self.offsets.len();

                  self.offsets[len - 1] += size
                }
              } else {
                return Err(
                  response!(
                    Wrong("missing"),
                    self.source.file,
                    right.pos
                  )
                )
              }
            }
          }
        }

        _ => return Err(
          response!(
            Wrong("unexpected constant declaration"),
            self.source.file,
            left.pos
          )
        )
      }

      Ok(())
    } else {
      unreachable!()
    }
  }



  pub fn type_statement(&mut self, statement: &'v Statement<'v>) -> Result<Type, ()> {
    use self::StatementNode::*;

    let t = match statement.node {
      Expression(ref expression) => self.type_expression(expression)?,
      _                          => Type::from(TypeNode::Nil)
    };

    Ok(t)
  }



  pub fn type_expression(&mut self, expression: &'v Expression<'v>) -> Result<Type, ()> {
    use self::ExpressionNode::*;

    let t = match expression.node {
      Identifier(ref name) => if let Some((index, env_index)) = self.current_tab().0.get_name(name) {
        self.current_tab().1.get_type(index, env_index)?
      } else {
        return Err(
          response!(
            Wrong(format!("no such value `{}` in this scope", name)),
            self.source.file,
            expression.pos
          )
        )
      },

      String(_) => Type::from(TypeNode::Str),
      Char(_)   => Type::from(TypeNode::Char),
      Bool(_)   => Type::from(TypeNode::Bool),
      Int(_)    => Type::from(TypeNode::I128),
      Float(_)  => Type::from(TypeNode::F64),

      Call(ref expression, _) => {
        if let TypeNode::Func(_, ref return_type) = self.type_expression(expression)?.node {
          (**return_type).clone()
        } else {
          unreachable!()
        }
      },

      Index(ref array, _) => if let TypeNode::Array(ref t, _) = self.type_expression(array)?.node {
        (**t).clone()
      } else {
        unreachable!()
      },

      Loop(ref expression)     |
      If(_, ref expression, _) => self.type_expression(expression)?,

      Array(ref content) => Type::array(self.type_expression(content.first().unwrap())?, content.len() as u32),

      Cast(_, ref t) => Type::from(t.node.clone()),

      Binary(ref left, ref op, ref right) => {
        use self::Operator::*;

        match (self.type_expression(left)?.node, op, self.type_expression(right)?.node) {
          (ref a, ref op, ref b) => match **op {
            Add | Sub | Mul | Div => if a == b {
              Type::from(a.to_owned())
            } else {
              return Err(
                response!(
                  Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                  self.source.file,
                  expression.pos
                )
              )
            },

            Eq | Lt | Gt | NEq | LtEq | GtEq => if a == b {
              Type::from(TypeNode::Bool)
            } else {
              return Err(
                response!(
                  Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                  self.source.file,
                  expression.pos
                )
              )
            },

            _ => return Err(
              response!(
                Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                self.source.file,
                expression.pos
              )
            )
          },

          _ => Type::from(TypeNode::Nil),
        }
      },

      Function(ref params, ref return_type, _) => {
        use self::StatementNode::*;

        let mut param_types = Vec::new();

        for param in params {
          match param.node {
            Variable(ref t, ..) | Constant(ref t, ..) => param_types.push(t.clone()),
            _ => unreachable!(),
          }
        }

        Type::function(param_types, return_type.clone())
      },

      Set(ref content) => {
        let mut type_content = Vec::new();

        for expression in content {
          type_content.push(self.type_expression(expression)?)
        }

        Type::set(type_content)
      },

      Block(ref statements) => self.type_statement(statements.last().unwrap())?, // temporary

      _ => Type::from(TypeNode::Nil)
    };

    Ok(t)
  }



  pub fn current_tab(&mut self) -> &mut (SymTab, TypeTab) {
    let len = self.tabs.len() - 1;

    &mut self.tabs[len]
  }



  pub fn push_scope(&mut self) {
    self.offsets.push(0);

    let local_symtab  = SymTab::new(Rc::new(self.current_tab().0.clone()), &[]);
    let local_typetab = TypeTab::new(Rc::new(self.current_tab().1.clone()), &[]);

    self.tabs.push((local_symtab.clone(), local_typetab.clone()));

    self.depth += 1
  }

  pub fn pop_scope(&mut self) {
    self.offsets.pop();

    self.tab_frames.push(self.tabs.pop().unwrap());

    self.depth -= 1
  }
}
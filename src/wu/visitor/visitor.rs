use super::*;
use super::super::error::Response::Wrong;

use std::fmt::{ self, Formatter, Write, Display };

use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum TypeNode {
  Int,
  Float,
  Number,
  Bool,
  Str,
  Char,
  Nil,
  Id(String),
  Set(Vec<Type>),
  Array(Rc<Type>),
}

impl PartialEq for TypeNode {
  fn eq(&self, other: &TypeNode) -> bool {
    use self::TypeNode::*;

    match (self, other) {
      (&Int, &Int)       => true,
      (&Int, &Number)    => true,
      (&Number, &Int)    => true,
      (&Float, &Float)   => true,
      (&Float, &Number)  => true,
      (&Number, &Float)  => true,
      (&Number, &Number) => true,
      (&Bool, &Bool)     => true,
      (&Str, &Str)       => true,
      (&Char, &Char)     => true,
      (&Nil, &Nil)       => true,
      (&Id(ref a), &Id(ref b))   => a == b,
      (&Set(ref a), &Set(ref b)) => a == b,
      _                          => false,
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
      Number       => write!(f, "number"),
      Int          => write!(f, "int"),
      Float        => write!(f, "float"),
      Bool         => write!(f, "bool"),
      Str          => write!(f, "string"),
      Char         => write!(f, "char"),
      Nil          => write!(f, "nil"),
      Array(ref n) => write!(f, "[{}]", n),
      Id(ref n)    => write!(f, "{}", n),
      Set(ref content) => {
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

  pub fn int() -> Type {
    Type::new(TypeNode::Int, TypeMode::Regular)
  }

  pub fn float() -> Type {
    Type::new(TypeNode::Float, TypeMode::Regular)
  }

  pub fn string() -> Type {
    Type::new(TypeNode::Str, TypeMode::Regular)
  }

  pub fn char() -> Type {
    Type::new(TypeNode::Char, TypeMode::Regular)
  }

  pub fn bool() -> Type {
    Type::new(TypeNode::Bool, TypeMode::Regular)
  }

  pub fn nil() -> Type {
    Type::new(TypeNode::Nil, TypeMode::Regular)
  }

  pub fn set(content: Vec<Type>) -> Type {
    Type::new(TypeNode::Set(content), TypeMode::Regular)
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}", self.mode, self.node)
    }
}



pub struct Visitor<'v> {
  pub symtab:  SymTab<'v>,
  pub typetab: TypeTab<'v>,
  pub source:  &'v Source,
  pub ast:     &'v Vec<Statement<'v>>,
}

impl<'v> Visitor<'v> {
  pub fn new(source: &'v Source, ast: &'v Vec<Statement<'v>>) -> Self {
    Visitor {
      symtab:  SymTab::global(),
      typetab: TypeTab::global(),
      source,
      ast,
    }
  }

  pub fn visit(&mut self) -> Result<(), ()> {
    for statement in self.ast {
      self.visit_statement(&statement)?
    }

    Ok(())
  }

  pub fn visit_statement(&mut self, statement: &'v Statement<'v>) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.visit_expression(expression),

      Variable(_, ref left, _) => match left.node {
        ExpressionNode::Identifier(_) | ExpressionNode::Set(_) => self.visit_variable(&statement.node),
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
      Identifier(ref name) => if self.symtab.get_name(name).is_none() {
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
      }

      Block(ref statements) => {
        for statement in statements {
          self.visit_statement(statement)?
        }

        Ok(())
      }

      _ => Ok(())
    }
  }

  fn visit_variable(&mut self, variable: &'v StatementNode) -> Result<(), ()> {
    use self::ExpressionNode::{Identifier, Set};

    if let &StatementNode::Variable(ref variable_type, ref left, ref right) = variable {
      match left.node {
        Identifier(ref name) => {
          let index = if let Some((index, _)) = self.symtab.get_name(name) {
            index
          } else {
            self.symtab.add_name(name)
          };

          self.typetab.grow();

          if let &Some(ref right) = right {
            self.visit_expression(&right)?;

            let right_type = self.type_expression(&right)?;

            if variable_type.node != TypeNode::Nil {
              if variable_type != &right_type {
                return Err(
                  response!(
                    Wrong(format!("mismatched types, expected type `{}` got `{}`", variable_type.node, right_type)),
                    self.source.file,
                    right.pos
                  )
                )
              } else {
                self.typetab.set_type(index, 0, variable_type.to_owned())?
              }
            } else {
              self.typetab.set_type(index, 0, right_type)?
            }
          } else {
            self.typetab.set_type(index, 0, variable_type.to_owned())?
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
                let index = if let Some((index, _)) = self.symtab.get_name(name) {
                  index
                } else {
                  self.typetab.grow();
                  self.symtab.add_name(name)
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
                        self.typetab.set_type(index, 0, variable_type.to_owned())?
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
                    self.typetab.set_type(index, 0, right_type)?
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
                let index = if let Some((index, _)) = self.symtab.get_name(name) {
                  index
                } else {
                  self.typetab.grow();
                  self.symtab.add_name(name)
                };

                self.typetab.set_type(index, 0, variable_type.to_owned())?
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
    use self::ExpressionNode::{Identifier, Set};

    if let &StatementNode::Constant(ref constant_type, ref left, ref right) = constant {
      match left.node {
        Identifier(ref name) => {
          let index = if let Some((index, _)) = self.symtab.get_name(name) {
            index
          } else {
            self.symtab.add_name(name)
          };

          self.typetab.grow();

          self.visit_expression(&right)?;

          let right_type = self.type_expression(right)?;

          if constant_type.node != TypeNode::Nil {
            if constant_type != &right_type {              
              return Err(
                response!(
                  Wrong(format!("mismatched types, expected type `{}` got `{}`", constant_type.node, right_type)),
                  self.source.file,
                  right.pos
                )
              )
            } else {
              self.typetab.set_type(index, 0, constant_type.to_owned())?
            }
          } else {
            self.typetab.set_type(index, 0, right_type)?
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
              let index = if let Some((index, _)) = self.symtab.get_name(name) {
                index
              } else {
                self.typetab.grow();
                self.symtab.add_name(name)
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
                      self.typetab.set_type(index, 0, constant_type.to_owned())?
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
                  self.typetab.set_type(index, 0, right_type)?
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



  fn type_expression(&mut self, expression: &'v Expression<'v>) -> Result<Type, ()> {
    use self::ExpressionNode::*;

    let t = match expression.node {
      Identifier(ref name) => if let Some((index, env_index)) = self.symtab.get_name(name) {
        self.typetab.get_type(index, env_index)?
      } else {
        unreachable!()
      },

      String(_) => Type::string(),
      Char(_)   => Type::char(),
      Bool(_)   => Type::bool(),
      Int(_)    => Type::int(),
      Float(_)  => Type::float(),

      Cast(ref expression, ref t) => match (self.type_expression(expression)?.node, &t.node) {
        (TypeNode::Int, &TypeNode::Float) => Type::float(),
        (TypeNode::Int, &TypeNode::Int)   => Type::int(),

        (a, b) => return Err(
          response!(
            Wrong(format!("can't cast from {} to {}", a, b)),
            self.source.file,
            expression.pos
          )
        )
      }

      Binary(ref left, ref op, ref right) => {
        use self::Operator::*;
        use self::TypeNode::*;

        match (self.type_expression(left)?.node, op, self.type_expression(right)?.node) {
          (ref a, ref op, ref b) => match **op {
            Add | Sub | Mul | Div => match (a, b) {
              (&Int,   &Int)   => Type::int(),
              (&Float, &Int)   => Type::float(),
              (&Float, &Float) => Type::float(),
              _                => return Err(
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
          _ => Type::nil(),
        }
      },

      Set(ref content) => {
        let mut type_content = Vec::new();

        for expression in content {
          type_content.push(self.type_expression(expression)?)
        }

        Type::set(type_content)
      },

      _ => Type::nil()
    };

    Ok(t)
  }



  fn fold_expression(&self, expression: &Expression<'v>) -> Result<Expression<'v>, ()> {
    use self::ExpressionNode::*;
    use self::Operator::*;

    let node = match expression.node {
      Binary(ref left, ref op, ref right) => {
        let node = match (&self.fold_expression(&*left)?.node, op, &self.fold_expression(&*right)?.node) {
          (&Int(ref a),   &Add, &Int(ref b))   => Int(a + b),
          (&Float(ref a), &Add, &Float(ref b)) => Float(a + b),
          (&Int(ref a),   &Sub, &Int(ref b))   => Int(a - b),
          (&Float(ref a), &Sub, &Float(ref b)) => Float(a - b),
          (&Int(ref a),   &Mul, &Int(ref b))   => Int(a * b),
          (&Float(ref a), &Mul, &Float(ref b)) => Float(a * b),
          (&Int(ref a),   &Div, &Int(ref b))   => Int(a / b),
          (&Float(ref a), &Div, &Float(ref b)) => Float(a / b),
          
          _ => return Err(()),
        };

        Expression::new(
          node,
          expression.pos.clone()
        )
      },

      _ => expression.clone()
    };

    Ok(node)
  }
}
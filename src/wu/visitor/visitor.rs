use super::*;
use super::super::error::Response::Wrong;

use std::fmt::{ self, Write, Formatter, Display, };

pub enum TypeNode {
  Int,
  Float,
  Number,
  Bool,
  Str,
  Nil,
  Id(String)
}

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
      Number    => write!(f, "number"),
      Int       => write!(f, "int"),
      Float     => write!(f, "float"),
      Bool      => write!(f, "bool"),
      Str       => write!(f, "string"),
      Nil       => write!(f, "nil"),
      Id(ref n) => write!(f, "`{}`", n),
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
      Immutable  => write!(f, "constant"),
      Undeclared => write!(f, "undeclared"),
      Optional   => write!(f, "optional"),
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

  pub fn number() -> Type {
    Type::new(TypeNode::Number, TypeMode::Regular)
  }

  pub fn int() -> Type {
    Type::new(TypeNode::Int, TypeMode::Regular)
  }

  pub fn float() -> Type {
    Type::new(TypeNode::Float, TypeMode::Regular)
  }

  pub fn bool() -> Type {
    Type::new(TypeNode::Bool, TypeMode::Regular)
  }

  pub fn nil() -> Type {
    Type::new(TypeNode::Nil, TypeMode::Regular)
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.mode, self.node)
    }
}



pub struct Visitor<'v> {
  pub symtab: SymTab<'v>,
  pub source: &'v Source,
}

impl<'v> Visitor<'v> {
  pub fn new(source: &'v Source) -> Self {
    Visitor {
      symtab: SymTab::global(),
      source,
    }
  }

  pub fn visit_statement(&mut self, statement: &Statement<'v>) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.visit_expression(expression),
      _ => Ok(())
    }
  }

  fn visit_expression(&mut self, expression: &Expression<'v>) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Identifier(ref name) => if let Some((index, env_index)) = self.symtab.get_name(name) {
        Ok(())
      } else {
        Err(
          response!(
            Wrong(format!("no such value `{}` in this scope", name)),
            self.source.file,
            expression.pos
          )
        )
      }
      _ => Ok(())
    }
  }
}
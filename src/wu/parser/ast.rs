use std::rc::Rc;
use std::fmt;

use std::collections::HashMap;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode<'s> {
  Expression(Expression<'s>),
  Variable(Type<'s>, String, Option<Expression<'s>>),
  Assignment(Expression<'s>, Expression<'s>),
  Return(Option<Rc<Expression<'s>>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement<'s> {
  pub node: StatementNode<'s>,
  pub pos:  TokenElement<'s>,
}

impl<'s> Statement<'s> {
  pub fn new(node: StatementNode<'s>, pos: TokenElement<'s>) -> Self {
    Statement {
      node,
      pos,
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode<'e> {
  Int(u64),
  Float(f64),
  Str(String),
  Char(char),
  Bool(bool),
  Unwrap(Rc<Expression<'e>>),
  Identifier(String),
  Binary(Rc<Expression<'e>>, Operator, Rc<Expression<'e>>),
  Block(Vec<Statement<'e>>),
  Cast(Rc<Expression<'e>>, Type<'e>),
  Array(Vec<Expression<'e>>),
  Index(Rc<Expression<'e>>, Rc<Expression<'e>>),
  Function(Vec<(String, Type<'e>)>, Type<'e>, Rc<Expression<'e>>, Option<Vec<String>>),
  Call(Rc<Expression<'e>>, Vec<Expression<'e>>),
  If(Rc<Expression<'e>>, Rc<Expression<'e>>, Option<Vec<(Option<Expression<'e>>, Expression<'e>, TokenElement<'e>)>>),
  Module(Rc<Expression<'e>>),
  EOF,
  Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression<'e> {
  pub node: ExpressionNode<'e>,
  pub pos:  TokenElement<'e>,
}

impl<'e> Expression<'e> {
  pub fn new(node: ExpressionNode<'e>, pos: TokenElement<'e>) -> Self {
    Expression {
      node,
      pos,
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
  Add, Sub, Mul, Div, Mod, Pow, Concat, Eq, Lt, Gt, NEq, LtEq, GtEq,
}

impl Operator {
  pub fn from_str(operator: &str) -> Option<(Operator, u8)> {
    use self::Operator::*;

    let op_prec = match operator {
      "==" => (Eq,     1),
      "<"  => (Lt,     1),
      ">"  => (Gt,     1),
      "!=" => (NEq,    1),
      "<=" => (LtEq,   1),
      ">=" => (GtEq,   1),
      "+"  => (Add,    2),
      "-"  => (Sub,    2),
      "++" => (Concat, 2),
      "*"  => (Mul,    3),
      "/"  => (Div,    3),
      "%"  => (Mod,    3),
      "^"  => (Pow,    4),
      _    => return None,
    };

    Some(op_prec)
  }

  pub fn as_str(&self) -> &str {
    use self::Operator::*;
    
    match *self {
      Add    => "+",
      Sub    => "-",
      Concat => "++",
      Pow    => "^",
      Mul    => "*",
      Div    => "/",
      Mod    => "%",
      Eq     => "==",
      Lt     => "<",
      Gt     => ">",
      NEq    => "!=",
      LtEq   => "<=",
      GtEq   => ">=",
    }
  }
}

impl fmt::Display for Operator {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
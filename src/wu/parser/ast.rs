use std::rc::Rc;
use std::fmt;

use std::collections::HashMap;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode {
  Expression(Expression),
  Variable(Type, String, Option<Expression>),
  Assignment(Expression, Expression),
  Return(Option<Rc<Expression>>),
  Import(String, Vec<String>),

  Break,
  Skip,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
  pub node: StatementNode,
  pub pos:  TokenElement,
}

impl Statement {
  pub fn new(node: StatementNode, pos: TokenElement) -> Self {
    Statement {
      node,
      pos,
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode {
  Int(u64),
  Float(f64),
  Str(String),
  Char(char),
  Bool(bool),
  Unwrap(Rc<Expression>),
  Neg(Rc<Expression>),
  Identifier(String),
  Binary(Rc<Expression>, Operator, Rc<Expression>),
  Block(Vec<Statement>),
  Cast(Rc<Expression>, Type),
  Array(Vec<Expression>),
  Index(Rc<Expression>, Rc<Expression>),
  Function(Vec<(String, Type)>, Type, Rc<Expression>, Vec<String>),
  Call(Rc<Expression>, Vec<Expression>),
  If(Rc<Expression>, Rc<Expression>, Option<Vec<(Option<Expression>, Expression, TokenElement)>>),
  Module(Rc<Expression>),
  While(Rc<Expression>, Rc<Expression>),
  Struct(String, Vec<(String, Type)>, Vec<String>),
  Initialization(Rc<Expression>, Vec<(String, Expression)>),
  Extern(Type, Option<String>),
  EOF,
  Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
  pub node: ExpressionNode,
  pub pos:  TokenElement
}

impl Expression {
  pub fn new(node: ExpressionNode, pos: TokenElement) -> Self {
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
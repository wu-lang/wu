use std::convert::TryInto;
use std::fmt;
use std::rc::Rc;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode {
  Expression(Expression),
  Variable(Type, String, Option<Expression>),
  Assignment(Expression, Expression),
  Return(Option<Rc<Expression>>),
  Implement(Expression, Expression, Option<Expression>),
  Import(String, Vec<String>),
  Skip,
  Break,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
  pub node: StatementNode,
  pub pos: Pos,
}

impl Statement {
  pub fn new(node: StatementNode, pos: Pos) -> Self {
    Statement { node, pos }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode {
  Int(u64),
  Float(f64),
  Str(String),
  Char(char),
  Bool(bool),
  UnwrapSplat(Rc<Expression>),
  Unwrap(Rc<Expression>),

  Neg(Rc<Expression>),
  Not(Rc<Expression>),

  Identifier(String),
  Binary(Rc<Expression>, Operator, Rc<Expression>),
  Array(Vec<Expression>),

  Call(Rc<Expression>, Vec<Expression>),
  Index(Rc<Expression>, Rc<Expression>, bool), // whether_index_is_an_array_index: bool

  Cast(Rc<Expression>, Type),
  Block(Vec<Statement>),

  Function(Vec<(String, Type)>, Type, Rc<Expression>, bool), // is_method: bool
  If(
    Rc<Expression>,
    Rc<Expression>,
    Option<Vec<(Option<Expression>, Expression, Pos)>>,
  ),

  While(Rc<Expression>, Rc<Expression>),
  Module(Rc<Expression>),
  Extern(Type, Option<String>),
  Struct(String, Vec<(String, Type)>, String),
  Trait(String, Vec<(String, Type)>),
  Initialization(Rc<Expression>, Vec<(String, Expression)>),

  Empty,
  EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
  pub node: ExpressionNode,
  pub pos: Pos,
}
impl Expression {
  pub fn identifier_ref(&self) -> Result<&String, &Self> {
    match &self.node {
      ExpressionNode::Identifier(id) => Ok(id),
      _ => Err(&self),
    }
  }

  //Function(Vec<(String, Type)>, Type, Rc<Expression>, bool), // is_method: bool
  pub fn function_ref(
    &self,
  ) -> Result<(&Vec<(String, Type)>, &Type, &Rc<Expression>, &bool), &Self> {
    match &self.node {
      ExpressionNode::Function(args, return_type, block, is_method) => {
        Ok((args, return_type, block, is_method))
      }
      _ => Err(&self),
    }
  }

  pub fn float(&self) -> Result<f64, &Self> {
    match &self.node {
      ExpressionNode::Float(f) => Ok(*f),
      _ => Err(&self),
    }
  }

  pub fn int(&self) -> Result<u64, &Self> {
    match &self.node {
      ExpressionNode::Int(i) => Ok(*i),
      _ => Err(&self),
    }
  }

  pub fn block(self) -> Result<Vec<Statement>, Self> {
    match self.node {
      ExpressionNode::Block(ast) => Ok(ast),
      _ => Err(self),
    }
  }

  /*pub fn string(self) -> Result<String, Self> {
      match self.node {
          ExpressionNode::Str(id) => Ok(id),
          _ => Err(self),
      }
  }*/

  pub fn string_ref(&self) -> Result<&String, &Self> {
    match &self.node {
      ExpressionNode::Str(id) => Ok(id),
      _ => Err(&self),
    }
  }

  pub fn try_str(&self) -> Option<&str> {
    self.string_ref().map(|x| x.as_str()).ok()
  }
}

impl Expression {
  pub fn new(node: ExpressionNode, pos: Pos) -> Self {
    Expression { node, pos }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Pow,
  Concat,
  Eq,
  Lt,
  Gt,
  NEq,
  LtEq,
  GtEq,
  Or,
  And,
}

impl Operator {
  pub fn from_str(operator: &str) -> Option<(Operator, u8)> {
    use self::Operator::*;

    let op_prec = match operator {
      "or" => (Or, 0),
      "and" => (And, 0),
      "==" => (Eq, 1),
      "<" => (Lt, 1),
      ">" => (Gt, 1),
      "!=" => (NEq, 1),
      "<=" => (LtEq, 1),
      ">=" => (GtEq, 1),
      "+" => (Add, 2),
      "-" => (Sub, 2),
      "++" => (Concat, 2),
      "*" => (Mul, 3),
      "/" => (Div, 3),
      "%" => (Mod, 3),
      "^" => (Pow, 4),
      _ => return None,
    };

    Some(op_prec)
  }

  pub fn as_str(&self) -> &str {
    use self::Operator::*;

    match *self {
      Add => "+",
      Sub => "-",
      Concat => "++",
      Pow => "^",
      Mul => "*",
      Div => "/",
      Mod => "%",
      Eq => "==",
      Lt => "<",
      Gt => ">",
      NEq => "!=",
      LtEq => "<=",
      GtEq => ">=",
      Or => "or",
      And => "and",
    }
  }

  pub fn is_compoundable(operator: &str) -> bool {
    ["+", "-", "*", "/", "++", "%", "^", "not", "or", "and"].contains(&operator)
  }
}

impl fmt::Display for Operator {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

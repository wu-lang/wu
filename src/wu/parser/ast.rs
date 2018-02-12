use std::rc::Rc;

use super::*;

#[derive(Debug)]
pub enum StatementNode<'s> {
  Expression(Expression<'s>)
}

#[derive(Debug)]
pub struct Statement<'s> {
  pub node: StatementNode<'s>,
  pub pos:  &'s TokenElement<'s>,
}



#[derive(Debug)]
pub enum ExpressionNode<'e> {
  Number(f64),
  String(&'e str),
  Char(char),
  Bool(bool),
  Binary(Rc<Expression<'e>>, Operator, Rc<Expression<'e>>),
  Unary(Operator, Rc<Expression<'e>>),
}

#[derive(Debug)]
pub struct Expression<'e> {
  pub node: ExpressionNode<'e>,
  pub pos:  &'e TokenElement<'e>,
}


#[derive(Debug)]
pub enum Operator {
  Add, Sub, Mul, Div, Mod, Pow, Concat,
}

impl Operator {
  pub fn from_str(operator: &str) -> Option<(Operator, u8)> {
    use self::Operator::*;

    let op_prec = match operator {
      "*"  => (Mul, 0),
      "/"  => (Div, 0),
      "%"  => (Mod, 0),
      "^"  => (Pow, 0),
      "+"  => (Add, 1),
      "-"  => (Sub, 1),
      "++" => (Concat, 1),
      _    => return None,
    };

    Some(op_prec)
  }
}
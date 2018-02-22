use super::*;
use super::super::error::Response::Wrong;

use std::rc::Rc;

pub struct Parser<'p> {
  index:  usize,
  tokens: Vec<Token<'p>>,
  source: &'p Source,
}

impl<'p> Parser<'p> {
  pub fn new(tokens: Vec<Token<'p>>, source: &'p Source) -> Self {
    Parser {
      tokens,
      source,
      index: 0,
    }
  }



  pub fn parse(&mut self) -> Result<Vec<Statement<'p>>, ()> {
    let mut ast = Vec::new();

    while self.remaining() > 0 {
      ast.push(self.parse_statement()?)
    }

    Ok(ast)
  }

  fn parse_statement(&mut self) -> Result<Statement<'p>, ()> {
    use self::TokenType::*;

    while self.current_type() == &EOL && self.remaining() != 0 {
      self.next()?
    }

    let statement = match *self.current_type() {
      _ => {
        use self::ExpressionNode::*;

        let expression = self.parse_expression()?;

        if let Identifier(_) = expression.node {
          if self.current_type() == &TokenType::Symbol {
            let expression = match self.current_lexeme().as_str() {
              ":"   => return self.parse_declaration(expression),
              ref c => return Err(
                response!(
                  Wrong(format!("unexpected symbol `{}`", c)),
                  self.source.file,
                  TokenElement::Ref(self.current())
                )
              )
            };
          } else {
            Statement::new(
              StatementNode::Expression(expression),
              self.current_position(),
            )
          }
        } else {
          Statement::new(
            StatementNode::Expression(expression),
            self.current_position(),
          )
        }
      }
    };

    Ok(statement)
  }

  fn parse_expression(&mut self) -> Result<Expression<'p>, ()> {
    let atom = self.parse_atom()?;

    if self.current_type() == &TokenType::Operator {
      self.parse_binary(atom)
    } else {
      Ok(atom)
    }
  }

  fn parse_atom(&mut self) -> Result<Expression<'p>, ()> {
    use self::TokenType::*;

    if self.remaining() == 0 {
      Ok(
        Expression::new(
          ExpressionNode::EOF,
          self.current_position()
        )
      )
    } else {
      let token_type = self.current_type().clone();
      let position   = self.current_position();

      let expression = match token_type {
        Number => Expression::new(
          ExpressionNode::Number(self.eat()?.parse::<f64>().unwrap()),
          position
        ),

        Char => Expression::new(
          ExpressionNode::Char(self.eat()?.chars().last().unwrap()),
          position
        ),

        String => Expression::new(
          ExpressionNode::String(self.eat()?),
          position
        ),

        Identifier => Expression::new(
          ExpressionNode::Identifier(self.eat()?),
          position
        ),

        Bool => Expression::new(
          ExpressionNode::Bool(self.eat()? == "true"),
          position
        ),

        ref token_type => return Err(
          response!(
            Wrong(format!("unexpected token `{}`", token_type)),
            self.source.file,
            TokenElement::Ref(self.current())
          )
        )
      };
    
      Ok(expression)
    }
  }

  // basic precedence climbing
  fn parse_binary(&mut self, left: Expression<'p>) -> Result<Expression<'p>, ()> {
    let mut expression_stack = vec!(left);
    let mut operator_stack   = vec!(Operator::from_str(&self.eat()?).unwrap());

    expression_stack.push(self.parse_atom()?);

    while operator_stack.len() > 0 {
      while self.current_type() == &TokenType::Operator {
        let position               = self.current_position();
        let (operator, precedence) = Operator::from_str(&self.eat()?).unwrap();

        if precedence < operator_stack.last().unwrap().1 {
          let right = expression_stack.pop().unwrap();
          let left  = expression_stack.pop().unwrap();

          expression_stack.push(
            Expression::new(
              ExpressionNode::Binary(Rc::new(left), operator_stack.pop().unwrap().0, Rc::new(right)),
              self.current_position(),
            )
          );

          if self.remaining() > 0 {
            expression_stack.push(self.parse_atom()?);
            operator_stack.push((operator, precedence))
          } else {
            return Err(
              response!(
                Wrong("reached EOF in operation"),
                self.source.file,
                position
              )
            )
          }
        } else {
          expression_stack.push(self.parse_atom()?);
          operator_stack.push((operator, precedence))
        }
      }

      let right = expression_stack.pop().unwrap();
      let left  = expression_stack.pop().unwrap();

      expression_stack.push(
        Expression::new(
          ExpressionNode::Binary(Rc::new(left), operator_stack.pop().unwrap().0, Rc::new(right)),
          self.current_position(),
        )
      );
    }

    Ok(expression_stack.pop().unwrap())
  }

  fn parse_declaration(&mut self, left: Expression<'p>) -> Result<Statement<'p>, ()> {
    match self.current_lexeme().as_str() {
      ":" => {
        self.next()?;

        let position = left.pos.clone();

        if self.current_type() == &TokenType::Symbol {
          match self.current_lexeme().as_str() {
            ":" => {
              self.next()?;

              let right    = Some(self.parse_expression()?);

              Ok(
                Statement::new(
                  StatementNode::Constant(
                    Type::new(TypeNode::Nil, TypeMode::Immutable),
                    left,
                    right,
                  ),

                  position,
                )
              )
            },

            "=" => {
              self.next()?;

              let right    = Some(self.parse_expression()?);
              let position = left.pos.clone();

              Ok(
                Statement::new(
                  StatementNode::Variable(
                    Type::nil(),
                    left,
                    right,
                  ),

                  position,
                )
              )
            },

            ref c => Err(
              response!(
                Wrong(format!("unexpected symbol `{}`", c)),
                self.source.file,
                self.current_position()
              )
            )
          }
        } else {

          let t = self.parse_type()?;

          if self.current_type() == &TokenType::Symbol {
            match self.current_lexeme().as_str() {
              ":" => {
                self.next()?;

                let right    = Some(self.parse_expression()?);

                Ok(
                  Statement::new(
                    StatementNode::Constant(
                      Type::new(t.node, TypeMode::Immutable),
                      left,
                      right,
                    ),

                    position,
                  )
                )
              },

              "=" => {
                self.next()?;

                let right    = Some(self.parse_expression()?);
                let position = left.pos.clone();

                Ok(
                  Statement::new(
                    StatementNode::Variable(
                      t,
                      left,
                      right,
                    ),

                    position,
                  )
                )
              },

              ref c => Err(
                response!(
                  Wrong(format!("unexpected symbol `{}`", c)),
                  self.source.file,
                  self.current_position()
                )
              )
            }
          } else {
            Ok(
              Statement::new(
                StatementNode::Variable(
                  t,
                  left,
                  None,
                ),

                position,
              )
            )
          }
        }
      },

      _ => Err(
        response!(
          Wrong("invalid declaration without `:`"),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn parse_type(&mut self) -> Result<Type, ()> {
    use self::TokenType::*;

    let t = match *self.current_type() {
      Identifier => Type::id(&self.eat()?),

      _ => return Err(
        response!(
          Wrong(format!("expected type found `{}`", self.current_lexeme())),
          self.source.file,
          self.current_position()
        )
      )
    };

    Ok(t)
  }



  fn next(&mut self) -> Result<(), ()> {
    if self.index <= self.tokens.len() {
      self.index += 1;
      Ok(())
    } else {
      Err(
        response!(
          Wrong("moving outside token stack"),
          self.source.file
        )
      )
    }
  }

  fn remaining(&self) -> usize {
    self.tokens.len() - self.index
  }

  fn current_position(&self) -> TokenElement<'p> {
    let current = self.current();

    TokenElement::Pos(
      current.line,
      current.slice
    )
  }

  fn current(&self) -> &Token<'p> {
    if self.index > self.tokens.len() - 1 {
      &self.tokens[self.tokens.len() - 1]
    } else {
      &self.tokens[self.index]
    }
  }

  fn eat(&mut self) -> Result<String, ()> {
    let lexeme = self.current().lexeme.clone();
    self.next()?;

    Ok(lexeme)
  }

  fn current_lexeme(&self) -> String {
    self.current().lexeme.clone()
  }

  fn current_type(&self) -> &TokenType {
    &self.current().token_type
  }

  fn expect_type(&self, token_type: TokenType) -> Result<(), ()> {
    if self.current_type() == &token_type {
      Ok(())
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file
        )
      )
    }
  }
}
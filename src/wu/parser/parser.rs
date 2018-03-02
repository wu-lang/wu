use super::*;
use super::super::error::Response::Wrong;

use std::rc::Rc;

pub struct Parser<'p> {
  index:  usize,
  tokens: Vec<&'p Token<'p>>,
  source: &'p Source,
}

impl<'p> Parser<'p> {
  pub fn new(tokens: Vec<&'p Token<'p>>, source: &'p Source) -> Self {
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

        match expression.node {
          Identifier(_) | Set(_) => {
            if self.remaining() > 0 {
              if self.current_type() == &TokenType::Symbol {
                let statement = match self.current_lexeme().as_str() {
                  ":"   => self.parse_declaration(expression)?,
                  ref c => return Err(
                    response!(
                      Wrong(format!("unexpected symbol `{}`", c)),
                      self.source.file,
                      TokenElement::Ref(self.current())
                    )
                  )
                };

                statement
              } else {
                let position = expression.pos.clone();

                Statement::new(
                  StatementNode::Expression(expression),
                  position
                )
              }
            } else {
              let position = expression.pos.clone();

              Statement::new(
                StatementNode::Expression(expression),
                position,
              )
            }
          },

          _ => {
            let position = expression.pos.clone();

            Statement::new(
              StatementNode::Expression(expression),
              position,
            )
          },
        }
      }
    };

    self.newline()?;

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
        Int => Expression::new(
          ExpressionNode::Int(self.eat()?.parse::<i64>().unwrap()),
          position
        ),

        Float => Expression::new(
          ExpressionNode::Float(self.eat()?.parse::<f64>().unwrap()),
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

        Symbol => match self.current_lexeme().as_str() {
          "{" => Expression::new(
            ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
            position
          ),

          "(" => {
            let content = self.parse_block_of(("(", ")"), &Self::_parse_expression_comma)?;

            if content.len() == 1 {
              content[0].clone()
            } else if content.len() > 1 {
              Expression::new(
                ExpressionNode::Set(content),
                position
              )
            } else {
              return Err(
                response!(
                  Wrong("unhandled empty clause `()`"),
                  self.source.file,
                  TokenElement::Ref(self.current())
                )
              )
            }
          },

          ref c => return Err(
            response!(
              Wrong(format!("unexpected symbol `{}`", c)),
              self.source.file,
              TokenElement::Ref(self.current())
            )
          )
        }

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
    let left_position = left.pos.clone();

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

    let expression = expression_stack.pop().unwrap();

    let position = match left_position {
      TokenElement::Pos(ref line, ref slice) => if let TokenElement::Pos(_, ref slice2) = self.current_position() {
        TokenElement::Pos(*line, (slice.0, if slice2.1 < line.1.len() { slice2.1 } else { line.1.len() } ))
      } else {
        left_position.clone()
      },

      _ => left_position.clone(),
    };

    Ok(
      Expression::new(
        expression.node,
        position
      )
    )
  }

  fn parse_declaration(&mut self, left: Expression<'p>) -> Result<Statement<'p>, ()> {
    match self.current_lexeme().as_str() {
      ":" => {
        self.next()?;

        let position = left.pos.clone();

        match self.current_lexeme().as_str() {
          ":" => {
            self.next()?;

            let right    = self.parse_expression()?;

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

          _ => {
            let t = self.parse_type()?;

            match self.current_lexeme().as_str() {
              ":" => {
                self.next()?;

                let right = self.parse_expression()?;

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

              _ => Ok(
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
      Identifier => match self.eat()?.as_str() {
        "str"   => Type::string(),
        "char"  => Type::char(),
        "float" => Type::float(),
        "int"   => Type::int(),
        "bool"  => Type::bool(),
        id      => Type::id(id),
      },

      Symbol => match self.current_lexeme().as_str() {
        "(" => {
          let content = self.parse_block_of(("(", ")"), &Self::_parse_type_comma)?;

          if content.len() == 1 {
            content[0].clone()
          } else {
            Type::set(content)
          }
        }

        _   => return Err(
          response!(
            Wrong(format!("unexpected symbol `{}` in type", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      }

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

  fn parse_block_of<B>(&mut self, delimeters: (&str, &str), parse_with: &Fn(&mut Self) -> Result<Option<B>, ()>) -> Result<Vec<B>, ()> {
    self.eat_lexeme(delimeters.0)?;

    let mut block_tokens = Vec::new();
    let mut nest_count   = 1;

    while nest_count > 0 {
      if self.current_lexeme() == delimeters.1 {
        nest_count -= 1
      } else if self.current_lexeme() == delimeters.0 {
        nest_count += 1
      }

      if nest_count == 0 {
        break
      } else {
        block_tokens.push(self.current());

        self.next()?
      }
    }

    self.eat_lexeme(delimeters.1)?;

    if !block_tokens.is_empty() {
      let mut parser = Parser::new(block_tokens, self.source);
      let mut block  = Vec::new();

      while let Some(element) = parse_with(&mut parser)? {
        block.push(element)
      }

      Ok(block)
    } else {
      Ok(Vec::new())
    }
  }



  fn _parse_statement(self: &mut Self) -> Result<Option<Statement<'p>>, ()> {
    if self.remaining() > 0 {
      Ok(Some(self.parse_statement()?))
    } else {
      Ok(None)
    }
  }

  fn _parse_expression(self: &mut Self) -> Result<Option<Expression<'p>>, ()> {
    let expression = self.parse_expression()?;

    match expression.node {
      ExpressionNode::EOF => Ok(None),
      _                   => Ok(Some(expression)),
    }
  }

  fn _parse_expression_comma(self: &mut Self) -> Result<Option<Expression<'p>>, ()> {
    let expression = Self::_parse_expression(self);

    if self.remaining() > 0 {
      self.eat_lexeme(",")?;
    }

    expression
  }

  fn _parse_type_comma(self: &mut Self) -> Result<Option<Type>, ()> {
    if self.remaining() == 0 {
      Ok(None)
    } else {
      let t = self.parse_type()?;

      if self.remaining() > 0 {
        self.eat_lexeme(",")?;
      }

      Ok(Some(t))
    }
  }



  fn newline(&mut self) -> Result<(), ()> {
    if self.remaining() > 0 {
      match self.current_lexeme().as_str() {
        "\n" => self.next(),
        _    => Err(
          response!(
            Wrong(format!("expected new line found: `{}`", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      }
    } else {
      Ok(())
    }
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
    self.tokens.len().saturating_sub(self.index)
  }

  fn current_position(&self) -> TokenElement<'p> {
    let current = self.current();

    TokenElement::Pos(
      current.line,
      current.slice
    )
  }

  fn current(&self) -> &'p Token<'p> {
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

  fn eat_lexeme(&mut self, lexeme: &str) -> Result<String, ()> {
    if self.current_lexeme() == lexeme {
      let lexeme = self.current().lexeme.clone();
      self.next()?;

      Ok(lexeme)
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", lexeme, self.current_lexeme())),
          self.source.file,
          self.current_position()
        )
      )
    }
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
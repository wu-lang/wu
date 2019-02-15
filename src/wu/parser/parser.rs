use super::*;
use super::super::error::Response::Wrong;

use std::rc::Rc;

pub struct Parser<'p> {
  index:  usize,
  tokens: Vec<Token>,
  source: &'p Source,
}

impl<'p> Parser<'p> {
  pub fn new(tokens: Vec<Token>, source: &'p Source) -> Self {
    Parser {
      tokens,
      source,
      index: 0,
    }
  }



  pub fn parse(&mut self) -> Result<Vec<Statement>, ()> {
    let mut ast = Vec::new();

    while self.remaining() > 0 {
      ast.push(self.parse_statement()?)
    }

    Ok(ast)
  }



  fn parse_statement(&mut self) -> Result<Statement, ()> {
    use self::TokenType::*;

    while self.current_type() == EOL && self.remaining() != 0 {
      self.next()?
    }

    let position = self.current_position();

    let statement = match self.current_type() {
      Identifier => {
        let backup_index = self.index;
        let position     = self.current_position();
        let name         = self.eat_type(&Identifier)?;

        match self.current_lexeme().as_str() {
          ":" => {
            self.next()?;

            let position = self.current_position();
            let backup   = self.index;

            if let Some(right) = self.parse_right_hand(name.clone())? {
              Statement::new(
                StatementNode::Variable(
                  Type::from(TypeNode::Nil),
                  name,
                  Some(right)
                ),
                self.span_from(position)
              )
            } else {
              self.index = backup;

              let kind = if self.current_lexeme() == "=" {
                Type::from(TypeNode::Nil)
              } else {
                self.parse_type()?
              };

              if self.current_lexeme() == "=" {
                self.next()?;

                Statement::new(
                  StatementNode::Variable(
                    kind,
                    name,
                    Some(self.parse_expression()?)
                  ),
                  self.span_from(position)
                )
              } else {
                Statement::new(
                  StatementNode::Variable(
                    kind,
                    name,
                    None
                  ),
                  self.span_from(position)
                )
              }
            }
          },

          "=" => {
            self.next()?;

            Statement::new(
              StatementNode::Assignment(
                Expression::new(
                  ExpressionNode::Identifier(name),
                  position.clone()
                ),

                self.parse_expression()?
              ),
              position,
            )
          },

          c => {
            let expression = Expression::new(
              ExpressionNode::Identifier(name),
              position.clone()
            );

            if let Some(result) = self.try_parse_compound(&expression)? {
              result
            } else {
              self.index = backup_index;

              let expression = self.parse_expression()?;
              let position   = expression.pos.clone();

              if let Some(result) = self.try_parse_compound(&expression)? {
                result
              } else {

                if self.current_lexeme() == "=" {
                  self.next()?;

                  Statement::new(
                    StatementNode::Assignment(expression, self.parse_expression()?),
                    position
                  )
                } else {
                  Statement::new(
                    StatementNode::Expression(expression),
                    position,
                  )
                }
              }
            }
          },
        }
      },

      Keyword => match self.current_lexeme().as_str() {
        "return" => {
          self.next()?;

          if ["}", "\n"].contains(&self.current_lexeme().as_str()) {
            Statement::new(
              StatementNode::Return(None),
              position
            )
          } else {
            Statement::new(
              StatementNode::Return(Some(Rc::new(self.parse_expression()?))),
              self.span_from(position)
            )
          }
        },

        "break" => {
          self.next()?;

          Statement::new(
            StatementNode::Break,
            position
          )
        },

        "skip" => {
          self.next()?;

          Statement::new(
            StatementNode::Skip,
            position
          )
        },

        "import" => {
          self.next()?;

          let path = self.eat_type(&Identifier)?;

          let specifics = if self.current_lexeme() == "{" {
            self.parse_block_of(("{", "}"), &Self::_parse_name_comma)?
          } else {
            Vec::new()
          };

          Statement::new(
            StatementNode::Import(path, specifics),
            self.span_from(position)
          )
        },

        "implement" => {
          let pos = self.span_from(position);

          self.next()?;
          self.next_newline()?;

          self.expect_type(TokenType::Identifier)?;

          let start = self.index;

          while !["{", ":"].contains(&self.current_lexeme().as_str()) && self.remaining() > 1 {
            self.next()?;
          }

          let end = self.index;

          let mut name_parser = Parser::new(self.tokens[start .. end].to_vec(), self.source);

          let name = name_parser.parse_expression()?;

          self.next_newline()?;

          let mut parent = None;
          if self.current_lexeme() == ":" {
            self.next()?;

            parent = Some(self.parse_expression()?);

            self.next_newline()?
          }

          self.expect_lexeme("{")?;

          let body = self.parse_expression()?;

          Statement::new(
            StatementNode::Implement(
              name,
              body,
              parent
            ),
            pos
          )
        },

        _ => {
          let expression = self.parse_expression()?;

          let position = expression.pos.clone();

          Statement::new(
            StatementNode::Expression(expression),
            position,
          )
        },
      },

      _ => {
        let expression = self.parse_expression()?;
        let position   = expression.pos.clone();

        if let Some(result) = self.try_parse_compound(&expression)? {
          result
        } else {
          if self.current_lexeme() == "=" {
            self.next()?;

            Statement::new(
              StatementNode::Assignment(expression, self.parse_expression()?),
              position
            )
          } else {
            Statement::new(
              StatementNode::Expression(expression),
              position,
            )
          }
        }
      },
    };

    self.new_line()?;

    Ok(statement)
  }



  fn try_parse_compound(&mut self, left: &Expression) -> Result<Option<Statement>, ()> {
    if self.current_type() != TokenType::Operator {
      return Ok(None)
    }

    let backup_index = self.index;

    let c = self.eat_type(&TokenType::Operator)?;

    let mut result = None;
    
    if self::Operator::is_compoundable(&c) {

      let op = self::Operator::from_str(&c).unwrap().0;

      let position = self.current_position();

      if self.current_lexeme() == "=" {
        self.next()?;

        let right = self.parse_expression()?;
        let ass   = Statement::new(
          StatementNode::Assignment(
            left.clone(),

            Expression::new(
              ExpressionNode::Binary(
                Rc::new(left.clone()),
                op,
                Rc::new(right)
              ),

              self.span_from(position.clone())
            )
          ),

          self.span_from(position)
        );   

        result = Some(ass)           
      } else {
        self.index = backup_index
      }
    }

    Ok(result)
  }



  fn parse_right_hand(&mut self, name: String) -> Result<Option<Expression>, ()> {
    let declaration = match self.current_lexeme().as_str() {
      "fun" => Some(self.parse_function()?),

      "struct" => {
        let mut position = self.current_position();
        
        self.next()?;
        self.next_newline()?;

        position = self.span_from(position);

        self.expect_lexeme("{")?;

        let params = self.parse_block_of(("{", "}"), &Self::_parse_struct_param_comma)?;

        Some(
          Expression::new(
            ExpressionNode::Struct(
              name,
              params,
              format!("{}{}", self.source.file, position),
            ),

            position
          )
        )
      },

      "trait" => {
        let position = self.current_position();

        self.next()?;

        let body = self.parse_block_of(("{", "}"), &Self::_parse_param_comma)?;

        Some(
          Expression::new(
            ExpressionNode::Trait(
              name,
              body
            ),

            position
          )
        )
      }

      "module" => {
        let position = self.current_position();

        self.next()?;
        self.next_newline()?;

        self.next_newline()?;

        self.expect_lexeme("{")?;

        Some(
          Expression::new(
            ExpressionNode::Module(
              Rc::new(self.parse_expression()?),
            ),

            position
          )
        )
      },

      "extern" => {
        let position = self.current_position();

        self.next()?;

        let kind = self.parse_type()?;

        let lua = if self.current_lexeme() == "=" {
          self.next()?;

          Some(self.eat_type(&TokenType::Str)?)
        } else {
          None
        };

        Some(
          Expression::new(
            ExpressionNode::Extern(kind, lua),
            self.span_from(position)
          )
        )
      },

      _ => None
    };

    Ok(declaration)
  }



  fn parse_function(&mut self) -> Result<Expression, ()> {
    let mut position = self.current_position();

    self.next()?;
    self.next_newline()?;

    let mut params = if self.current_lexeme() == "(" {
      self.parse_block_of(("(", ")"), &Self::_parse_param_comma)?
    } else {
      Vec::new()
    };

    let mut is_method = false;

    if params.len() > 0 {
      if params[0].1.node.strong_cmp(&TypeNode::This) {

        params.remove(0);

        is_method = true
      }
    }

    let retty = if self.current_lexeme() == "->" {
      self.next()?;

      self.parse_type()?
    } else {
      Type::from(TypeNode::Nil)
    };

    position = self.span_from(position);

    self.next_newline()?;

    self.expect_lexeme("{")?;

    Ok(
      Expression::new(
        ExpressionNode::Function(
          params,
          retty,
          Rc::new(self.parse_expression()?),
          is_method
        ),

        position
      )
    )
  }



  fn parse_expression(&mut self) -> Result<Expression, ()> {
    let atom = self.parse_atom()?;

    if self.current_type() == TokenType::Operator {
      self.parse_binary(atom)
    } else {
      Ok(atom)
    }
  }



  fn parse_atom(&mut self) -> Result<Expression, ()> {
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
          ExpressionNode::Int(self.eat()?.parse::<u64>().unwrap()),
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

        Str => Expression::new(
          ExpressionNode::Str(self.eat()?),
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

        Operator => match self.current_lexeme().as_str() {
          "*" => {
            self.next()?;

            Expression::new(
              ExpressionNode::UnwrapSplat(
                Rc::new(self.parse_expression()?)
              ),

              self.span_from(position)
            )
          },

          "-" => {
            self.next()?;

            Expression::new(
              ExpressionNode::Neg(
                Rc::new(self.parse_expression()?)
              ),

              self.span_from(position)
            )
          },

          "not" => {
            self.next()?;

            Expression::new(
              ExpressionNode::Not(
                Rc::new(self.parse_expression()?)
              ),

              self.span_from(position)
            )
          },

          ref symbol => return Err(
            response!(
              Wrong(format!("unexpected operator `{}`", symbol)),
              self.source.file,
              self.current_position()
            )
          )
        },

        Symbol => match self.current_lexeme().as_str() {
          "{" => Expression::new(
            ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
            position
          ),

          "[" => Expression::new(
            ExpressionNode::Array(self.parse_block_of(("[", "]"), &Self::_parse_expression_comma)?),
            self.span_from(position)
          ),

          "(" => {
            self.next()?;
            self.next_newline()?;

            if self.current_lexeme() == ")" && self.current_type() == TokenType::Symbol {
              self.next()?;

              Expression::new(
                ExpressionNode::Empty,
                self.span_from(position)
              )
            } else {
              let expression = self.parse_expression()?;

              self.eat_lexeme(")")?;

              expression
            }
          },

          ref symbol => panic!(), /*return Err(
            response!(
              Wrong(format!("unexpected symbol `{}`", symbol)),
              self.source.file,
              self.current_position()
            )
          )*/
        },

        Keyword => match self.current_lexeme().as_str() {
          "fun" => self.parse_function()?,
          "nil" => {
            self.next()?;

            Expression::new(
              ExpressionNode::Empty,
              self.span_from(position)
            )
          },

          "if" => {
            self.next()?;

            let condition   = Rc::new(self.parse_expression()?);
            let if_position = self.span_from(position.clone());

            let body        = Rc::new(
              Expression::new(
                ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
                position
              )
            );

            let mut elses = Vec::new();

            loop {
              let branch_position = self.current_position();

              match self.current_lexeme().as_str() {
                "elif" => {
                  self.next()?;

                  let condition = self.parse_expression()?;
                  let position  = self.current_position();
                  let body      = Expression::new(
                    ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
                    position
                  );

                  elses.push((Some(condition), body, branch_position))
                },

                "else" => {
                  self.next()?;

                  let position  = self.current_position();
                  let body      = Expression::new(
                    ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
                    position
                  );

                  elses.push((None, body, branch_position))
                },

                _ => break,
              }
            }

            Expression::new(
              ExpressionNode::If(condition, body, if elses.len() > 0 { Some(elses) } else { None }),
              if_position
            )
          },

          "while" => {
            self.next()?;

            self.next_newline()?;

            let condition = self.parse_expression()?;

            self.next_newline()?;

            self.expect_lexeme("{")?;

            let position = self.span_from(position);

            Expression::new(
              ExpressionNode::While(
                Rc::new(condition),
                Rc::new(self.parse_expression()?)
              ),
              position
            )
          },

          "new" => {
            self.next()?;
            self.next_newline()?;

            let expression = self.parse_expression()?;

            let position = self.span_from(position);

            self.next_newline()?;

            let args = self.parse_block_of(("{", "}"), &Self::_parse_definition_comma)?;

            Expression::new(
              ExpressionNode::Initialization(Rc::new(expression), args),
              position
            )
          }

          ref symbol => return Err(
            response!(
              Wrong(format!("unexpected keyword `{}`", symbol)),
              self.source.file,
              self.current_position()
            )
          )
        },

        ref token_type => return Err(
          response!(
            Wrong(format!("unexpected token `{}`", token_type)),
            self.source.file,
            self.current_position()
          )
        )
      };

      if self.remaining() > 0 {
        self.parse_postfix(expression)
      } else {
        Ok(expression)
      }
    }
  }



  fn parse_postfix(&mut self, expression: Expression) -> Result<Expression, ()> {
    if self.remaining() == 0 {
      return Ok(expression)
    }

    match self.current_type() {
      TokenType::Symbol => match self.current_lexeme().as_str() {
        "(" => {
          let args = self.parse_block_of(("(", ")"), &Self::_parse_expression_comma)?;

          let position = expression.pos.clone();

          let call = Expression::new(
            ExpressionNode::Call(Rc::new(expression), args),
            self.span_from(position)
          );

          self.parse_postfix(call)
        },

        "[" => {
          self.next()?;

          let expr = self.parse_expression()?;

          self.eat_lexeme("]")?;

          let position = expression.pos.clone();

          let index = Expression::new(
            ExpressionNode::Index(Rc::new(expression), Rc::new(expr)),
            self.span_from(position)
          );

          self.parse_postfix(index)
        },

        "!" => {
          self.next()?;

          let position = expression.pos.clone();

          let question = Expression::new(
            ExpressionNode::Unwrap(
              Rc::new(expression)
            ),
            self.span_from(position)
          );

          self.parse_postfix(question)
        }

        _ => Ok(expression)
      },

      TokenType::Keyword => match self.current_lexeme().as_str() {
        "as" => {
          self.next()?;

          let t        = self.parse_type()?;
          let position = expression.pos.clone();

          self.parse_postfix(
            Expression::new(
              ExpressionNode::Cast(Rc::new(expression), t),
              position
            )
          )
        },

        _ => Ok(expression)
      },

      TokenType::Identifier => {
        let position = self.current_position();

        let id = Expression::new(
          ExpressionNode::Identifier(
            self.eat()?
          ),
          position
        );

        let position = expression.pos.clone();

        let index = Expression::new(
          ExpressionNode::Index(
            Rc::new(expression),
            Rc::new(id)
          ),
          self.span_from(position)
        );

        self.parse_postfix(index)
      },

      _ => Ok(expression)
    }
  }



  fn parse_binary(&mut self, left: Expression) -> Result<Expression, ()> {
    let left_position = left.pos.clone();

    let mut expression_stack = vec!(left);
    let mut operator_stack   = vec!(Operator::from_str(&self.eat()?).unwrap());

    expression_stack.push(self.parse_atom()?);

    while operator_stack.len() > 0 {
      while self.current_type() == TokenType::Operator {
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

    Ok(
      Expression::new(
        expression.node,
        self.span_from(left_position)
      )
    )
  }



  fn parse_type(&mut self) -> Result<Type, ()> {
    use self::TokenType::*;

    let mut t = match self.current_type() {
      Identifier => match self.eat()?.as_str() {
        "str"   => Type::from(TypeNode::Str),
        "char"  => Type::from(TypeNode::Char),
        "int"   => Type::from(TypeNode::Int),
        "float" => Type::from(TypeNode::Float),
        "any"   => Type::from(TypeNode::Any),
        "bool"  => Type::from(TypeNode::Bool),
        "self"  => Type::from(TypeNode::This),

        _ => {
          self.index -= 1; // lol
          Type::id(Rc::new(self.parse_expression()?))
        }
      },

      Keyword => match self.current_lexeme().as_str() {
        "fun" => {
          self.next()?;

          let mut params = if self.current_lexeme() == "(" {
            self.parse_block_of(("(", ")"), &Self::_parse_type_comma)?
          } else {
            Vec::new()
          };

          let mut is_method = false;

          if params.len() > 0 {
            if params[0].node.strong_cmp(&TypeNode::This) {
              params.remove(0);

              is_method = true
            }
          }

          let return_type = if self.current_lexeme() == "->" {
            self.next()?;

            self.parse_type()?
          } else {
            Type::from(TypeNode::Nil)
          };

          Type::from(TypeNode::Func(params, Rc::new(return_type), None, is_method))
        },

        _ => return Err(
          response!(
            Wrong(format!("unexpected keyword `{}` in type", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      },

      Symbol => match self.current_lexeme().as_str() {
        "[" => {
          self.next()?;
          self.next_newline()?;

          let t = self.parse_type()?;

          self.next_newline()?;

          let mut len = None;

          if self.current_lexeme() == ";" {
            self.eat_lexeme(";")?;

            self.next_newline()?;

            let expression = self.parse_expression()?;

            len = if let ExpressionNode::Int(ref len) = Self::fold_expression(&expression)?.node {
              Some(*len as usize)
            } else {
              return Err(
                response!(
                  Wrong(format!("length of array can be nothing but int")),
                  self.source.file,
                  expression.pos
                )
              )
            };
          }

          self.eat_lexeme("]")?;

          Type::array(t, len)
        },

        "..." => {
          self.next()?;

          let splatted = if ([")", "=", "?"].contains(&self.current_lexeme().as_str()) && self.current_type() == TokenType::Symbol)
              || self.remaining() == 0 || self.current_lexeme() == "\n" {

            Type::from(TypeNode::Any)
          } else {
            self.parse_type()?
          };

          Type::new(splatted.node, TypeMode::Splat(None))
        },

        _ => return Err(
          response!(
            Wrong(format!("unexpected symbol `{}` in type", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      },

      _ => return Err(
        response!(
          Wrong(format!("expected type found `{}`", self.current_lexeme())),
          self.source.file,
          self.current_position()
        )
      )
    };

    if self.current_lexeme() == "?" {
      self.next()?;

      let inner = t.node.clone();

      t.node = TypeNode::Optional(Rc::new(inner));
    }

    Ok(t)
  }



  fn new_line(&mut self) -> Result<(), ()> {
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



  fn next_newline(&mut self) -> Result<(), ()> {
    while self.current_lexeme() == "\n" && self.remaining() > 0 {
      self.next()?
    }

    Ok(())
  }



  fn next(&mut self) -> Result<(), ()> {
    if self.index <= self.tokens.len() {
      self.index += 1;
      Ok(())
    } else {
      Err(
        response!(
          Wrong("moving outside token stack"),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn remaining(&self) -> usize {
    self.tokens.len().saturating_sub(self.index)
  }

  fn current_position(&self) -> Pos {
    let current = self.current();

    Pos(
      current.line.clone(),
      current.slice
    )
  }

  fn span_from(&self, left_position: Pos) -> Pos {
    let Pos(ref line, ref slice) = left_position;
    let Pos(_, ref slice2)       = self.current_position();

    Pos(line.clone(), (slice.0, if slice2.1 < line.1.len() { slice2.1 } else { line.1.len() } ))
  }

  fn current(&self) -> Token {
    if self.index > self.tokens.len() - 1 {
      self.tokens[self.tokens.len() - 1].clone()
    } else {
      self.tokens[self.index].clone()
    }
  }

  fn eat(&mut self) -> Result<String, ()> {
    let lexeme = self.current().lexeme;
    self.next()?;

    Ok(lexeme)
  }

  fn eat_lexeme(&mut self, lexeme: &str) -> Result<String, ()> {
    if self.current_lexeme() == lexeme {
      let lexeme = self.current().lexeme;
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

  fn eat_type(&mut self, token_type: &TokenType) -> Result<String, ()> {
    if self.current_type() == *token_type {
      let lexeme = self.current().lexeme.clone();
      self.next()?;

      Ok(lexeme)
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn current_lexeme(&self) -> String {
    self.current().lexeme.clone()
  }

  fn current_type(&self) -> TokenType {
    self.current().token_type
  }

  fn expect_type(&self, token_type: TokenType) -> Result<(), ()> {
    if self.current_type() == token_type {
      Ok(())
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn expect_lexeme(&self, lexeme: &str) -> Result<(), ()> {
    if self.current_lexeme() == lexeme {
      Ok(())
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



  // A helper method for parsing sequences defined by provided static methods,
  // for as long as given static method returns Some(B)
  fn parse_block_of<B>(&mut self, delimeters: (&str, &str), parse_with: &Fn(&mut Self) -> Result<Option<B>, ()>) -> Result<Vec<B>, ()> {
    self.eat_lexeme(delimeters.0)?;

    let mut block_tokens = Vec::new();
    let mut nest_count   = 1;

    while nest_count > 0 {
      if self.current_lexeme() == delimeters.1 && self.current_type() == TokenType::Symbol {
        nest_count -= 1
      } else if self.current_lexeme() == delimeters.0 && self.current_type() == TokenType::Symbol {
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



  fn _parse_statement(self: &mut Self) -> Result<Option<Statement>, ()> {
    if self.remaining() > 0 {
      Ok(Some(self.parse_statement()?))
    } else {
      Ok(None)
    }
  }



  fn _parse_expression(self: &mut Self) -> Result<Option<Expression>, ()> {
    let expression = self.parse_expression()?;

    match expression.node {
      ExpressionNode::EOF => Ok(None),
      _                   => Ok(Some(expression)),
    }
  }



  fn _parse_name_comma(self: &mut Self) -> Result<Option<String>, ()> {
    if self.remaining() == 0 {
      Ok(None)
    } else {
      if self.remaining() > 0 && self.current_lexeme() == "\n" {
        self.next()?
      }

      let t = self.eat_type(&TokenType::Identifier)?;

      if self.remaining() > 0 {
        if ![",", "\n"].contains(&self.current_lexeme().as_str()) {
          return Err(
            response!(
              Wrong(format!("expected `,` or newline, found `{}`", self.current_lexeme())),
              self.source.file,
              self.current_position()
            )
          )
        } else {
          self.next()?;
        }

        if self.remaining() > 0 && self.current_lexeme() == "\n" {
          self.next()?
        }
      }

      Ok(Some(t))
    }
  }



  // Static method for parsing sequence `expr* ,* \n*` - for things like [1, 2, 3, 4,]
  fn _parse_expression_comma(self: &mut Self) -> Result<Option<Expression>, ()> {
    if self.remaining() > 0 && self.current_lexeme() == "\n" {
      self.next()?
    }

    let expression = Self::_parse_expression(self);

    if self.remaining() > 0 && self.current_lexeme() == "\n" {
      self.next()?
    }

    if self.remaining() > 0 {
      self.eat_lexeme(",")?;

      if self.remaining() > 0 && self.current_lexeme() == "\n" {
        self.next()?
      }
    }

    expression
  }



  fn _parse_param_comma(self: &mut Self) -> Result<Option<(String, Type)>, ()> {
    if self.remaining() > 0 && self.current_lexeme() == "\n" {
      self.next()?
    }

    if self.remaining() == 0 {
      return Ok(None)
    }

    let mut splat    = false;
    let mut optional = false;
    let position     = self.current_position();

    if self.current_lexeme() == "..." {
      splat = true;

      self.next()?;
      self.next_newline()?;
    }

    if self.current_lexeme() == "?" {
      optional = true;

      self.next()?;
      self.next_newline()?;
    }

    let name = self.eat_type(&TokenType::Identifier)?;

    let mut kind = if name == "self" {
      if splat {
        return Err(
          response!(
            Wrong("can't splat `self`"),
            self.source.file,
            position
          )
        )
      }

      Type::new(
        TypeNode::This,
        TypeMode::Regular,
      )
    } else {
      let mut kind = Type::from(TypeNode::Any);

      if self.current_lexeme() == ":" {
        self.eat_lexeme(":")?;

        kind = self.parse_type()?;

        if splat {
          kind.mode = TypeMode::Splat(None)
        }
      }

      kind
    };

    if optional {
      let inner = kind.node.clone();

      kind.node = TypeNode::Optional(Rc::new(inner));
    }

    let param = Some((name, kind));

    if self.remaining() > 0 {
      if ![",", "\n"].contains(&self.current_lexeme().as_str()) {
        return Err(
          response!(
            Wrong(format!("expected `,` or newline, found `{}`", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      } else {
        self.next()?;
      }

      if self.remaining() > 0 && self.current_lexeme() == "\n" {
        self.next()?
      }
    }

    Ok(param)
  }



  fn _parse_definition_comma(self: &mut Self) -> Result<Option<(String, Expression)>, ()> {
    if self.remaining() > 0 && self.current_lexeme() == "\n" {
      self.next()?
    }

    if self.remaining() == 0 {
      return Ok(None)
    }

    let position = self.current_position();

    let name = self.eat_type(&TokenType::Identifier)?;
    
    self.eat_lexeme(":")?;

    let mut value = self.parse_expression()?;

    value.pos = position;

    let param = Some((name, value));

    if self.remaining() > 0 {
      if ![",", "\n"].contains(&self.current_lexeme().as_str()) {
        return Err(
          response!(
            Wrong(format!("expected `,` or newline, found `{}`", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      } else {
        self.next()?;
      }

      if self.remaining() > 0 && self.current_lexeme() == "\n" {
        self.next()?
      }
    }

    Ok(param)
  }

  fn _parse_struct_param_comma(self: &mut Self) -> Result<Option<(String, Type)>, ()> {
    if self.remaining() > 0 && self.current_lexeme() == "\n" {
      self.next()?
    }

    if self.remaining() == 0 {
      return Ok(None)
    }

    let name = self.eat_type(&TokenType::Identifier)?;

    self.eat_lexeme(":")?;

    let value = self.parse_type()?;
    let param = Some((name, value));

    if self.remaining() > 0 {
      if ![",", "\n"].contains(&self.current_lexeme().as_str()) {
        return Err(
          response!(
            Wrong(format!("expected `,` or newline, found `{}`", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      } else {
        self.next()?;
      }

      if self.remaining() > 0 && self.current_lexeme() == "\n" {
        self.next()?
      }
    }

    Ok(param)
  }



  fn _parse_type_comma(self: &mut Self) -> Result<Option<Type>, ()> {
    if self.remaining() == 0 {
      Ok(None)
    } else {
      let t = self.parse_type()?;

      if self.remaining() > 0 {
        self.eat_lexeme(",")?;

        if self.remaining() > 0 && self.current_lexeme() == "\n" {
          self.next()?
        }
      }

      Ok(Some(t))
    }
  }



  pub fn fold_expression(expression: &Expression) -> Result<Expression, ()> {
    use self::ExpressionNode::*;
    use self::Operator::*;

    let node = match expression.node {
      Binary(ref left, ref op, ref right) => {
        let node = match (&Self::fold_expression(&*left)?.node, op, &Self::fold_expression(&*right)?.node) {
          (&Int(ref a),   &Add, &Int(ref b))   => Int(a + b),
          (&Float(ref a), &Add, &Float(ref b)) => Float(a + b),
          (&Int(ref a),   &Sub, &Int(ref b))   => Int(a - b),
          (&Float(ref a), &Sub, &Float(ref b)) => Float(a - b),
          (&Int(ref a),   &Mul, &Int(ref b))   => Int(a * b),
          (&Float(ref a), &Mul, &Float(ref b)) => Float(a * b),
          (&Int(ref a),   &Div, &Int(ref b))   => Int(a / b),
          (&Float(ref a), &Div, &Float(ref b)) => Float(a / b),

          _ => expression.node.clone()
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
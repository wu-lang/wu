use super::*;
use super::super::error::Response::Wrong;

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

    while self.remaining() > 1 {
      ast.push(self.parse_statement()?)
    }

    Ok(ast)
  }

  fn parse_statement(&mut self) -> Result<Statement<'p>, ()> {
    use self::TokenType::*;

    while self.current_type() == &EOL {
      self.next()?
    }

    match *self.current_type() {
      ref token_type => Err(
        response!(
          Wrong(format!("unexpected token `{}`", token_type)),
          self.source.file,
          TokenElement::Ref(self.current())
        )
      )
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
    self.tokens.len() - self.index
  }

  pub fn current(&self) -> &Token {
    if self.index > self.tokens.len() - 1 {
      return &self.tokens[self.tokens.len() - 1];
    }

    &self.tokens[self.index]
  }

  pub fn current_lexeme(&self) -> String {
    self.current().lexeme.clone()
  }

  pub fn current_type(&self) -> &TokenType {
    &self.current().token_type
  }
}
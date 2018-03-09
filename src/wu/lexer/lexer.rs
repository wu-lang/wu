use super::*;

use super::super::error::Response::Wrong;

use std::rc::Rc;

pub struct Lexer<'l> {
  tokenizer: Tokenizer<'l>,
  matchers:  Vec<Rc<Matcher<'l>>>,
  source:    &'l Source,
}

impl<'l> Lexer<'l> {
  pub fn new(tokenizer: Tokenizer<'l>, source: &'l Source) -> Self {
    Lexer {
      tokenizer,
      matchers: Vec::new(),
      source,
    }
  }

  pub fn default(data: Vec<char>, source: &'l Source) -> Self {
    use self::TokenType::*;

    let tokenizer = Tokenizer::new(data, source);
    let mut lexer = Self::new(tokenizer, source);

    lexer.matchers.push(Rc::new(CommentMatcher));  

    lexer.matchers.push(Rc::new(EOLMatcher));
    lexer.matchers.push(Rc::new(WhitespaceMatcher));
    lexer.matchers.push(Rc::new(StringLiteralMatcher));

    lexer.matchers.push(
      Rc::new(
        KeyMatcher::new(Keyword, &[
          "as", "->", "loop", "if", "else", "elif",
        ])
      )
    );

    lexer.matchers.push(
      Rc::new(
        KeyMatcher::new(Bool, &["true", "false"])
      )
    );

    lexer.matchers.push(Rc::new(IdentifierMatcher));
    lexer.matchers.push(Rc::new(NumberLiteralMatcher));

    lexer.matchers.push(
      Rc::new(
        ConstantStringMatcher::new(Operator, &["+", "-", "*", "/", "<", ">", "==", "!=", "<=", ">="])
      )
    );

    lexer.matchers.push(
      Rc::new(
        ConstantCharMatcher::new(Symbol, &['(', ')', '[', ']', '{', '}', ',', ':', ';', '=', '.', '|'])
      )
    );

    lexer
  }

  pub fn match_token(&mut self) -> Result<Option<Token<'l>>, ()> {
    for matcher in &mut self.matchers {
      match self.tokenizer.try_match_token(matcher.as_ref())? {
        Some(t) => return Ok(Some(t)),
        None    => continue,
      }
    }

    Ok(None)
  }
}

impl<'l> Iterator for Lexer<'l> {
  type Item = Result<Token<'l>, ()>;

  fn next(&mut self) -> Option<Result<Token<'l>, ()>> {
    let token = match self.match_token() {
      Ok(hmm) => match hmm {
        Some(n) => n,
        None    => {
          let pos = self.tokenizer.pos;

          return Some(
            Err(
              response!(
                Wrong("bumped into weird character"),
                self.source.file,
                TokenElement::Pos(
                  (pos.0, &self.source.lines.get(pos.0.saturating_sub(1)).unwrap_or(self.source.lines.last().unwrap_or(&String::new()))),
                  (pos.1 + 1, pos.1 + 1),
                )
              )
            )
          )
        },
      },

      Err(_) => return Some(Err(())),
    };

    match token.token_type {
      TokenType::EOF        => None,
      TokenType::Whitespace => self.next(),
      _                     => Some(Ok(token)),
    }
  }
}
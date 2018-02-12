use super::*;
use super::super::error::Response::*;


macro_rules! token {
  ($tokenizer:expr, $token_type:ident, $accum:expr) => {{
    token!($tokenizer, TokenType::$token_type, $accum)
  }};
  ($tokenizer:expr, $token_type:expr, $accum:expr) => {{
    let tokenizer  = $tokenizer  as &$crate::wu::lexer::tokenizer::Tokenizer<'t>;
    let token_type = $token_type as $crate::wu::lexer::token::TokenType;

    let accum: String = $accum;
    let pos           = tokenizer.last_position();

    Token::new(token_type, (pos.0, &tokenizer.source.lines[pos.0]), (pos.1, accum.len()), &accum)
  }};
}



pub trait Matcher<'t> {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()>;
}



pub struct CommentMatcher;

impl<'t> Matcher<'t> for CommentMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    if tokenizer.peek_range(2).unwrap_or_else(String::new) == "--" {
      while !tokenizer.end() && tokenizer.peek().unwrap() != '\n' {
        tokenizer.advance();
      }

      Ok(Some(token!(tokenizer, EOL, "\n".into())))
    } else {
      Ok(None)
    }
  }
}



pub struct ConstantCharMatcher {
  token_type: TokenType,
  constants: &'static [char],
}

impl ConstantCharMatcher {
  pub fn new(token_type: TokenType, constants: &'static [char]) -> Self {
    ConstantCharMatcher {
      token_type,
      constants,
    }
  }
}

impl<'t> Matcher<'t> for ConstantCharMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    let c = tokenizer.peek().unwrap();
    
    for constant in self.constants {
      if c == *constant {
        tokenizer.advance();
        return Ok(Some(token!(tokenizer, self.token_type.clone(), constant.to_string())))
      }
    }
    Ok(None)
  }
}



pub struct WhitespaceMatcher;

impl<'t> Matcher<'t> for WhitespaceMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    let string = tokenizer.collect_while(|c| c.is_whitespace() && c != '\n');

    if !string.is_empty() {
      Ok(Some(token!(tokenizer, Whitespace, string)))
    } else {
      Ok(None)
    }
  }
}



pub struct StringLiteralMatcher;

impl<'t> Matcher<'t> for StringLiteralMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    let mut raw_marker = false;

    let mut pos = tokenizer.pos;

    let delimeter  = match tokenizer.peek().unwrap() {
      '"'  => '"',
      '\'' => '\'',
      'r' => {
        if tokenizer.peek_n(1) == Some('"') {
          raw_marker = true;
          tokenizer.advance();

          pos = tokenizer.pos;

          '"'
        } else {
          return Ok(None)
        }
      },
      _ => return Ok(None),
    };

    tokenizer.advance();

    let mut string     = String::new();
    let mut found_escape = false;

    loop {
      if tokenizer.end() {
        return Err(
          response!(
            Wrong(format!("missing closing delimeter ´{}´ to close literal here", delimeter)),
            tokenizer.source.file,
            TokenElement::Pos(
              (pos.0 + 1, &tokenizer.source.lines[pos.0 + 1]),
              (pos.1 - 1, pos.1),
            )
          )
        )
      }

      if raw_marker {
        if tokenizer.peek().unwrap() == '"' {
          break
        }

        string.push(tokenizer.next().unwrap())
      } else if found_escape {
        string.push(
          match tokenizer.next().unwrap() {
            c @ '\\' | c @ '\'' | c @ '"' => c,
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            escaped => return Err(
              response!(
                Wrong(format!("unexpected escape character: {}", escaped)),
                tokenizer.source.file,
                TokenElement::Pos(
                  (tokenizer.pos.0, &tokenizer.source.lines[tokenizer.pos.0 + 1]),
                  (tokenizer.pos.1 - 1, tokenizer.pos.1),
                )
              )
            ),
          }
        );
        found_escape = false
      } else {
        match tokenizer.peek().unwrap() {
          '\\' => {
            tokenizer.next();
            found_escape = true
          },
          c if c == delimeter => break,
          _ => string.push(tokenizer.next().unwrap()),
        }
      }
    }

    tokenizer.advance();
    
    if delimeter == '"' {
      Ok(Some(token!(tokenizer, String, string)))
    } else {
      if string.len() > 1 {
        let pos = tokenizer.last_position();

        Err(
          response!(
            Wrong(format!("character literal may not contain more than one codepoint: '{}'", string)),
            tokenizer.source.file,
            TokenElement::Pos(
              (pos.0, &tokenizer.source.lines[pos.0 + 1]),
              (pos.1 - 1, pos.1 + string.len() + 1),
            )
          )
        )
      } else {
        Ok(Some(token!(tokenizer, Char, string)))
      }
    }
  }
}



pub struct IdentifierMatcher;

impl<'t> Matcher<'t> for IdentifierMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
        if !tokenizer.peek().unwrap().is_alphabetic() && !(tokenizer.peek().unwrap() == '_') {
            return Ok(None)
        }

        let accum = tokenizer.collect_while(|c| c.is_alphanumeric() || "_!?".contains(c));

        if accum.is_empty() {
            Ok(None)
        } else {
            Ok(Some(token!(tokenizer, Identifier, accum)))
        }
    }
}
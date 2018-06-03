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


    let line = tokenizer.source.lines.get(pos.0.saturating_sub(1)).unwrap_or(tokenizer.source.lines.last().unwrap());

    if TokenType::String == token_type || TokenType::Char == token_type {
      Token::new(token_type, (pos.0, &line), (pos.1 + 1, pos.1 + accum.len() + 2), &accum) // delimeters
    } else {
      Token::new(token_type, (pos.0, &line), (pos.1 + 1, pos.1 + accum.len()), &accum)
    }
  }};
}



pub trait Matcher<'t> {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()>;
}



pub struct CommentMatcher;

impl<'t> Matcher<'t> for CommentMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    if tokenizer.peek_range(3).unwrap_or_else(String::new) == "---" {
      tokenizer.advance_n(3);

      while !tokenizer.end() {
        if tokenizer.peek_range(3).unwrap_or_else(String::new) == "---" {
          tokenizer.advance_n(3);
          break
        }

        tokenizer.advance()
      }

      Ok(Some(token!(tokenizer, EOL, "\n".into())))

    } else if tokenizer.peek_range(2).unwrap_or_else(String::new) == "--" {
      while !tokenizer.end() && tokenizer.peek() != Some('\n') {
        tokenizer.advance()
      }

      Ok(Some(token!(tokenizer, EOL, "\n".into())))
    } else {
      Ok(None)
    }
  }
}



pub struct ConstantStringMatcher {
  token_type: TokenType,
  constants: &'static [&'static str],
}

impl ConstantStringMatcher {
  pub fn new(token_type: TokenType, constants: &'static [&'static str]) -> Self {
    ConstantStringMatcher {
      token_type,
      constants,
    }
  }
}

impl<'t> Matcher<'t> for ConstantStringMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    for constant in self.constants {
      let len = constant.len();
      let c   = match tokenizer.peek_range(len) {
        Some(len) => len,
        _         => return Ok(None),
      };

      if c == *constant {
        tokenizer.advance_n(len);

        let token = token!(tokenizer, self.token_type.clone(), constant.to_string());

        if c == "\n" {
          tokenizer.pos.0 += 1;
          tokenizer.pos.1 = 0;
        }

        return Ok(Some(token))
      }
    }
    Ok(None)
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
      constants
    }
  }
}

impl<'t> Matcher<'t> for ConstantCharMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    let c = tokenizer.peek().unwrap();
    
    for constant in self.constants {
      if c == *constant {
        tokenizer.advance();

        let token = token!(tokenizer, self.token_type.clone(), constant.to_string());

        if c == '\n' {
          tokenizer.pos.0 += 1;
          tokenizer.pos.1 = 0;
        }

        return Ok(Some(token))
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
        } else if tokenizer.peek_n(1) == Some('\'') {
          return Err(
            response!(
              Wrong("no such thing as a raw character literal"),
              tokenizer.source.file,
              TokenElement::Pos(
                (pos.0, &tokenizer.source.lines.get(pos.0.saturating_sub(1)).unwrap_or(tokenizer.source.lines.last().unwrap())),
                (pos.1 - 1, pos.1),
              )
            )
          )
        } else {
          return Ok(None)
        }
      },
      _ => return Ok(None),
    };

    tokenizer.advance();

    let mut string       = String::new();
    let mut found_escape = false;

    loop {
      if tokenizer.end() {
        return Err(
          response!(
            Wrong(format!("unterminated delimeter `{}`", delimeter)),
            tokenizer.source.file,
            TokenElement::Pos(
              (pos.0 + 1, &tokenizer.source.lines.get(pos.0.saturating_sub(1)).unwrap_or(tokenizer.source.lines.last().unwrap())),
              (pos.1.saturating_sub(1), pos.1 + 1),
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
                  (tokenizer.pos.0, &tokenizer.source.lines.get(pos.0.saturating_sub(1)).unwrap_or(tokenizer.source.lines.last().unwrap())),
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

          // check for valid closing delimeter and alternative
          c => if c == delimeter {
            if string.len() > 0 && string != " " {
              break
            } else {
              string.push(tokenizer.next().unwrap())
            }
          } else {
            string.push(tokenizer.next().unwrap())
          },
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
            Wrong("character literal may not contain more than one codepoint"),
            tokenizer.source.file,
            TokenElement::Pos(
              (pos.0, &tokenizer.source.lines.get(pos.0.saturating_sub(1)).unwrap_or(tokenizer.source.lines.last().unwrap())),
              (pos.1 + 2, pos.1 + string.len() + 1),
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


pub struct NumberLiteralMatcher;

impl<'t> Matcher<'t> for NumberLiteralMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    let mut accum = String::new();

    let curr = tokenizer.next().unwrap();
    if curr.is_digit(10) {
      accum.push(curr)
    } else if curr == '.' {
      accum.push_str("0.")
    } else {
      return Ok(None)
    }

    while !tokenizer.end() {
      let current = tokenizer.peek().unwrap();
      if !current.is_whitespace() && current.is_digit(10) || current == '.' {
        if current == '.' && accum.contains('.') {
          let pos = tokenizer.pos;

          return Err(
            response!(
              Wrong("unexpected extra decimal point"),
              tokenizer.source.file,
              TokenElement::Pos(
                (pos.0, &tokenizer.source.lines.get(pos.0.saturating_sub(1)).unwrap_or(tokenizer.source.lines.last().unwrap())),
                (pos.1 + 1, pos.1 + 1),
              )
            )
          )
        }
        accum.push(tokenizer.next().unwrap())
      } else {
        break
      }
    }

    if &accum == "0." {
      Ok(None)
    } else {
      if accum.contains(".") {
        let literal: String = match accum.parse::<f64>() {
          Ok(result) => result.to_string(),
          Err(error) => panic!("unable to parse float: {}", error)
        };

        Ok(Some(token!(tokenizer, Float, accum)))
      } else {
        let literal: String = match accum.parse::<i64>() {
          Ok(result) => result.to_string(),
          Err(error) => panic!("unable to parse int: {}", error)
        };

        Ok(Some(token!(tokenizer, Int, accum)))
      }
    }
  }
}

pub struct KeyMatcher {
  token_type: TokenType,
  constants: &'static [&'static str],
}

impl KeyMatcher {
  pub fn new(token_type: TokenType, constants:  &'static [&'static str]) -> Self {
    KeyMatcher {
      token_type,
      constants,
    }
  }
}

impl<'t> Matcher<'t> for KeyMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    for constant in self.constants {
      if let Some(s) = tokenizer.peek_range(constant.len()) {
        if s == *constant {
          if let Some(c) = tokenizer.peek_n(constant.len()) {
            if "_!?".contains(c) || c.is_alphanumeric() {
                return Ok(None)
            }
          }

          tokenizer.advance_n(constant.len());
          return Ok(Some(token!(tokenizer, self.token_type.clone(), constant.to_string())))
        }
      }
    }

    Ok(None)
  }
}



pub struct EOLMatcher;

impl<'t> Matcher<'t> for EOLMatcher {
  fn try_match(&self, tokenizer: &mut Tokenizer<'t>) -> Result<Option<Token<'t>>, ()> {
    if tokenizer.peek() == Some('\n') {
      tokenizer.pos.0 += 1;
      tokenizer.pos.1 = 0;
      tokenizer.index += 1;

      Ok(Some(token!(tokenizer, TokenType::EOL, String::from("\n"))))
    } else {
      Ok(None)
    }
  }
}
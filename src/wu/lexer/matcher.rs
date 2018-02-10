use super::*;

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
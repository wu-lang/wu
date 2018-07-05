use colored::Colorize;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
  Int,
  Float,
  Str,
  Char,
  Bool,
  Identifier,
  Keyword,
  Symbol,
  Operator,
  Whitespace,
  EOL,
  EOF,
}

impl fmt::Display for TokenType {
  fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::TokenType::*;

    match *self {
      Int        => write!(f, "Int"),
      Float     => write!(f, "Float"),
      Str        => write!(f, "Str"),
      Char       => write!(f, "Char"),
      Bool       => write!(f, "Bool"),
      Identifier => write!(f, "Identifier"),
      Symbol     => write!(f, "Symbol"),
      Keyword    => write!(f, "Keyword"),
      Operator   => write!(f, "Operator"),
      Whitespace => write!(f, "Whitespace"),
      EOL        => write!(f, "EOL"),
      EOF        => write!(f, "EOF"),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenElement {
  Type(TokenType),
  Line((usize, String)),
  Pos((usize, String), (usize, usize)),
}

use self::TokenElement::*;

impl<'t> PartialEq<Token> for TokenElement {
  fn eq (&self, rhs: &Token) -> bool {
    rhs == self
  }
}

impl fmt::Display for TokenElement {
  fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Line(ref line) => {
        let linepad = format!("{:5} │", " ").blue().bold();
        let lineno = format!("{:5} │ ", line.0);
        let srcline = format!("{}{}", lineno.blue().bold(), line.1);

        write!(f, "\n{}\n{}\n{}",
          linepad,
          srcline,
          linepad
        )
      },

      Pos(ref line, ref slice) => {
        let linepad = format!("{:5} │", " ").blue().bold();
        let lineno = format!("{:5} │ ", line.0).blue().bold();
        let mut mark = line.1[slice.0.saturating_sub(1) .. slice.1].to_string();

        if mark.split_whitespace().count() == 0 {
          mark = format!("{:─>count$}", ">".bold().magenta(), count=mark.len());
        } else {
          mark = format!("{}", mark.bold().magenta());
        }

        write!(f, "\n{}\n{}{}{}{}\n{}",
          linepad,
          lineno, &line.1[..slice.0.saturating_sub(1)], mark, &line.1[slice.1..],
          linepad
        )
      },

      _ => write!(f, ""),
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Token {
  pub token_type: TokenType,
  pub line:       (usize, String),
  pub slice:      (usize, usize),
  pub lexeme:     String,
}

impl Token {
  pub fn new(token_type: TokenType, line: (usize, String), slice: (usize, usize), lexeme: &str) -> Self {
    Token {
      token_type,
      line,
      slice,
      lexeme: lexeme.to_string()
    }
  }
}

impl PartialEq<TokenElement> for Token {
  fn eq (&self, rhs: &TokenElement) -> bool {
    match *rhs {
      Type(ref t)        => self.token_type == *t,
      _                  => false
    }
  }
}
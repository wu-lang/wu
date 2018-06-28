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
pub enum TokenElement<'e> {
  Type(TokenType),
  Lexeme(&'e str),
  Pair(TokenType, &'e str),
  Ref(&'e Token<'e>),
  Line((usize, &'e str)),
  Pos((usize, &'e str), (usize, usize)),
  Row(&'e [&'e Token<'e>]),
}

use self::TokenElement::{ Row, Line, Pair, Type, Pos, Ref, Lexeme, };

impl<'t> PartialEq<Token<'t>> for TokenElement<'t> {
  fn eq (&self, rhs: &Token<'t>) -> bool {
    rhs == self
  }
}

impl<'s> fmt::Display for TokenElement<'s> {
  fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Ref (r) => {
        if r.slice.1 > r.line.1.len() {
          write!(f, "{}", Pos(r.line, (r.slice.0, r.line.1.len())))
        } else {
          write!(f, "{}", Pos(r.line, r.slice))
        }
      }

      Line(line) => {
        let linepad = format!("{:5} │", " ").blue().bold();
        let lineno = format!("{:5} │ ", line.0);
        let srcline = format!("{}{}", lineno.blue().bold(), line.1);

        write!(f, "\n{}\n{}\n{}",
          linepad,
          srcline,
          linepad
        )
      },

      Pos(line, slice) => {
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

      Row(row) => {
        let mut mark = format!("{:offset$}", "", offset = row[0].slice.0);
        let mut len = row[0].slice.0;

        for token in row {
          mark = format!(
            "{}{:offset$}{:▔<count$}", mark, "", "".magenta().bold(),
            offset = token.slice.0 - len,
            count = token.slice.1 - token.slice.0 + 1
          );
          len = token.slice.1 + 1;
        }

        write!(f, "{}{}", Line(row[0].line), mark)
      },

      _ => write!(f, ""),
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Token<'t> {
  pub token_type: TokenType,
  pub line:       (usize, &'t str),
  pub slice:      (usize, usize),
  pub lexeme:     String,
}

impl<'t> Token<'t> {
  pub fn new(token_type: TokenType, line: (usize, &'t str), slice: (usize, usize), lexeme: &str) -> Self {
    Token {
      token_type,
      line,
      slice,
      lexeme: lexeme.to_string()
    }
  }
}

impl<'t> PartialEq<TokenElement<'t>> for Token<'t> {
  fn eq (&self, rhs: &TokenElement<'t>) -> bool {
    match *rhs {
      Type (ref t)        => self.token_type == *t,
      Lexeme (ref l)      => self.lexeme     == *l,
      Pair (ref t, ref l) => self.lexeme     == *l && self.token_type == *t,
      Ref (ref t)         => self            == *t,
      _                   => false
    }
  }
}
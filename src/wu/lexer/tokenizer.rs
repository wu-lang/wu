use super::token::*;
use super::{ Source, Matcher, };

pub struct Snapshot {
  pub index: usize,
  pub pos:   (usize, usize),
}

impl Snapshot {
  fn new(index: usize, pos: (usize, usize)) -> Self {
    Snapshot {
      index,
      pos,
    }
  }
}



pub struct Tokenizer<'t> {
  pub pos: (usize, usize),

  pub index:     usize,
  pub items:     Vec<char>,
  pub source:    &'t Source,
  pub snapshots: Vec<Snapshot>
}

impl<'t> Tokenizer<'t> {
  pub fn new(items: Vec<char>, source: &'t Source) -> Self {
    Tokenizer {
      pos: (1, 0),

      items,
      source,
      index:     0,
      snapshots: Vec::new(),
    }
  }

  pub fn end(&self) -> bool {
    self.index >= self.items.len()
  }

  pub fn advance(&mut self) {
    if let Some(item) = self.items.get(self.index + 1) {
      self.pos.1 += 1
    }

    self.index += 1
  }

  pub fn advance_n(&mut self, n: usize) {
    for _ in 0 .. n {
      self.advance()
    }
  }

  pub fn peek_range(&self, n: usize) -> Option<String> {
    self.items.get(self.index..self.index + n).map(|chars| chars.iter().collect::<String>())
  }

  pub fn peek_n(&self, n: usize) -> Option<char> {
    self.items.get(self.index + n).cloned()
  }

  pub fn peek(&self) -> Option<char> {
    self.peek_n(0)
  }

  pub fn take_snapshot(&mut self) {
    self.snapshots.push(Snapshot::new(self.index, self.pos));
  }

  pub fn peek_snapshot(&self) -> Option<&Snapshot> {
    self.snapshots.last()
  }

  pub fn rollback_snapshot(&mut self) {
    let snapshot = self.snapshots.pop().unwrap();
    self.index = snapshot.index;
    self.pos = snapshot.pos;
  }

  pub fn commit_snapshot(&mut self) {
    self.snapshots.pop();
  }

  pub fn last_position(&self) -> (usize, usize) {
    self.peek_snapshot().unwrap_or(&Snapshot::new(0, (0, 0))).pos
  }

  pub fn try_match_token(&mut self, matcher: &Matcher<'t>) -> Result<Option<Token>, ()> {
    if self.end() {
      return Ok(
        Some(
          Token::new(
            TokenType::EOF,
            (self.pos.0, if self.source.lines.len() > 0 {
                self.source.lines.get(self.pos.0).unwrap_or(self.source.lines.first().unwrap()).to_string()
              } else {
                String::new()
              }
            ),
            (self.pos.1, 0),
            ""
          )
        )
      )
    }

    self.take_snapshot();

    match matcher.try_match(self)? {
      Some(t) => {
        self.commit_snapshot();
        Ok(Some(t))
      }

      None => {
        self.rollback_snapshot();
        Ok(None)
      }
    }
  }

  pub fn collect_while(&mut self, func: fn(char) -> bool) -> String {
    let mut accum = String::new();
    while let Some(c) = self.peek() {
      if func(c) {
        accum.push(c);
      } else {
        break
      }

      self.advance();
    }

    accum
  }
}



impl<'t> Iterator for Tokenizer<'t> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.items.get(self.index).cloned();
        self.advance();
        c
    }
}

use super::*;

#[derive(Clone)]
pub struct Snapshot {
    pub index: usize,
    pub pos: TokenPosition,
}

impl Snapshot {
    fn new(index: usize, pos: TokenPosition) -> Snapshot {
        Snapshot {
            index,
            pos,
        }
    }
}

#[derive(Clone)]
pub struct Tokenizer {
    pub pos: TokenPosition,
    index: usize,
    items: Vec<char>,
    snapshots: Vec<Snapshot>,
}

impl Tokenizer {
    pub fn new(items: Vec<char>) -> Tokenizer {
        Tokenizer {
            index: 0,
            pos: TokenPosition::default(),
            items,
            snapshots: Vec::new(),
        }
    }

    pub fn end(&self) -> bool {
        self.index >= self.items.len()
    }

    pub fn advance(&mut self) {
        if let Some(a) = self.items.get(self.index + 1) {
            match *a {
                '\n' => {
                    self.pos.line += 1;
                    self.pos.col = 0;
                }
                _ => self.pos.col += 1
            }
        }
        self.index += 1;
    }

    pub fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    pub fn peek_n(&self, n: usize) -> Option<&char> {
        self.items.get(self.index + n)
    }

    pub fn peek(&self) -> Option<&char> {
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

    pub fn last_position(&self) -> TokenPosition {
        self.peek_snapshot().unwrap().pos
    }

    pub fn try_match_token(&mut self, matcher: &Matcher) -> Response<Option<Token>> {
        if self.end() {
            return Ok(Some(Token::new(TokenType::EOF,
                                   TokenPosition::new(self.index, self.index),
                                   String::new())));
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

    pub fn collect_if(&mut self, func: fn(&char) -> bool) -> String {
        let mut accum = String::new();
        loop {
            if let Some(c) = self.peek() {
                if func(c) {
                    accum.push(*c);
                } else {
                    break
                }
            } else {
                break
            }
            self.advance();
        }
        accum
    }
}

impl Iterator for Tokenizer {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.items.get(self.index).cloned();
        self.advance();
        c
    }
}

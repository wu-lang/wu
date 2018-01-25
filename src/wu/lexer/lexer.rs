use super::*;

use std::rc::Rc;

pub fn make_lexer<'l>(data: Vec<char>, lines: &'l Vec<String>, path: &'l str) -> Lexer<'l> {
    let tokenizer = Tokenizer::new(data);
    let mut lexer = Lexer::new(tokenizer, lines, path);

    lexer.matchers_mut().push(Rc::new(FloatLiteralMatcher));
    lexer.matchers_mut().push(Rc::new(StringLiteralMatcher));

    let bool_matcher = KeyMatcher::new(TokenType::Bool, &["true", "false"]);
    lexer.matchers_mut().push(Rc::new(bool_matcher));

    lexer.matchers_mut().push(Rc::new(CommentMatcher));

    let key_matcher = KeyMatcher::new(TokenType::Keyword, &[
        "return", "->", "if", "elif", "else", "match", "struct", "while", "module", "import", "expose",
    ]);
    lexer.matchers_mut().push(Rc::new(key_matcher));

    lexer.matchers_mut().push(Rc::new(IdentifierMatcher));

    let eol_matcher = ConstantCharMatcher::new(TokenType::EOL, &['\n']);
    lexer.matchers_mut().push(Rc::new(eol_matcher));

    let operator_matcher = ConstantStringMatcher::new(TokenType::Operator, &[
        "+=", "-=", "*=", "/=", "%=", "^=", "++=",
        "++", "+", "-", "*", "/", "^", ">=", "<=", "==", "!=", "<|", "|>", "<", ">", "%", "!",
    ]);

    lexer.matchers_mut().push(Rc::new(operator_matcher));

    let symbol_matcher = ConstantCharMatcher::new(TokenType::Symbol, &[
        '(', ')', '[', ']', '{', '}', ',', ':', ';', '|', '=', '.'
    ]);

    lexer.matchers_mut().push(Rc::new(symbol_matcher));

    lexer.matchers_mut().push(Rc::new(WhitespaceMatcher));

    lexer
}

pub struct Lexer<'l> {
    tokenizer: Tokenizer,
    matchers:  Vec<Rc<Matcher>>,
    lines:     &'l Vec<String>,
    path:      &'l str,
}

impl<'t> Lexer<'t> {
    pub fn new(tokenizer: Tokenizer, lines: &'t Vec<String>, path: &'t str) -> Self {
        Self {
            tokenizer,
            matchers: Vec::new(),
            lines,
            path,
        }
    }

    pub fn match_token(&mut self) -> Response<Option<Token>> {
        for matcher in &mut self.matchers {
            match self.tokenizer.try_match_token(matcher.as_ref())? {
                Some(t) => return Ok(Some(t)),
                None    => continue,
            }
        }
        Ok(None)
    }

    pub fn matchers(&self) -> &Vec<Rc<Matcher>> {
        &self.matchers
    }

    pub fn matchers_mut(&mut self) -> &mut Vec<Rc<Matcher>> {
        &mut self.matchers
    }
}

impl<'t> Iterator for Lexer<'t> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let token = match self.match_token() {
            Ok(hmm) => match hmm {
                Some(n) => n,
                None    => return None,
            },

            Err(res) => {
                res.display(self.lines, self.path);
                return None
            },
        };

        match token.token_type {
            TokenType::EOF => None,
            _ => Some(token),
        }
    }
}

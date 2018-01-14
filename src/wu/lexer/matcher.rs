use super::*;

macro_rules! token {
    ($tokenizer:expr, $token_type:ident, $accum:expr) => {{
        token!($tokenizer , TokenType::$token_type, $accum)
    }};
    ($tokenizer:expr, $token_type:expr, $accum:expr) => {{
        let tokenizer  = $tokenizer  as &$crate::wu::lexer::tokenizer::Tokenizer;
        let token_type = $token_type as $crate::wu::lexer::token::TokenType;
        Token::new(token_type, tokenizer.last_position(), $accum)
    }};
}

pub trait Matcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>>;
}

pub struct IntLiteralMatcher;

impl Matcher for IntLiteralMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        let mut accum = String::new();
        while let Some(c) = tokenizer.next() {
            if c.is_digit(10) {
                accum.push(c.clone());
            } else {
                break
            }
        }

        if accum.is_empty() {
            Ok(None)
        } else {
            Ok(Some(token!(tokenizer, Int, accum)))
        }
    }
}

pub struct FloatLiteralMatcher;

impl Matcher for FloatLiteralMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
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
            let current = *tokenizer.peek().unwrap();
            if !current.is_whitespace() && current.is_digit(10) || current == '.' {
                if current == '.' && accum.contains('.') {                    
                    return Err(make_error(Some(TokenPosition::new(tokenizer.pos.line, tokenizer.pos.col - 1)), "extra decimal point".to_owned()))
                }
                accum.push(tokenizer.next().unwrap())
            } else {
                break
            }
        }

        if accum == "0.".to_owned() {
            Ok(None)
        } else if accum.contains('.') {

            let literal: String = match accum.parse::<f64>() {
                Ok(result) => result.to_string(),
                Err(error) => panic!("unable to parse float: {}", error)
            };

            Ok(Some(token!(tokenizer, Float, literal)))
        } else {
            let literal: String = match u64::from_str_radix(accum.as_str(), 10) {
                Ok(result) => result.to_string(),
                Err(error) => panic!("unable to parse int: {}", error)
            };

            Ok(Some(token!(tokenizer, Float, literal)))
        }
    }
}

pub struct StringLiteralMatcher;

impl Matcher for StringLiteralMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        let mut raw_marker = false;
        let delimeter  = match *tokenizer.peek().unwrap() {
            '"'  => Some('"'),
            '\'' => Some('\''),
            'r' => {
                if tokenizer.peek_n(1) == Some(&'"') {
                    raw_marker = true;
                    tokenizer.advance();

                    Some('"')
                } else {
                    None
                }
            },
            _ => return Ok(None),
        };

        tokenizer.advance();

        let mut string       = String::new();
        let mut found_escape = false;

        while !tokenizer.end() {
            if raw_marker {
                if tokenizer.peek().unwrap() == &'"' {
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
                        s => panic!("invalid character escape: {}", s),
                    }
                );
                found_escape = false
            } else {
                match *tokenizer.peek().unwrap() {
                    '\\' => {
                        tokenizer.next();
                        found_escape = true
                    },
                    c if delimeter.is_some() && c == delimeter.unwrap() => break,
                    _ => string.push(tokenizer.next().unwrap()),
                }
            }
        }
        tokenizer.advance();
        if delimeter.is_some() {
            Ok(Some(token!(tokenizer, Str, string)))
        } else {
            Ok(None)
        }
    }
}

pub struct IdentifierMatcher;

impl Matcher for IdentifierMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        if !tokenizer.peek().unwrap().is_alphabetic() && !"_".contains(*tokenizer.peek().unwrap()) {
            return Ok(None)
        }

        let string = tokenizer.collect_if(|c| c.is_alphanumeric() || "_!?".contains(*c));

        if string.is_empty() {
            Ok(None)
        } else {
            Ok(Some(token!(tokenizer, Identifier, string)))
        }
    }
}

pub struct WhitespaceMatcher;

impl Matcher for WhitespaceMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        let string = tokenizer.collect_if(|c| c.is_whitespace() && c != &'\n');

        if string.len() > 0 {
            Ok(Some(token!(tokenizer, Whitespace, string)))
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

impl Matcher for ConstantCharMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        let c = tokenizer.peek().unwrap().clone();
        for constant in self.constants {
            if c == *constant {
                tokenizer.advance();
                return Ok(Some(token!(tokenizer, self.token_type.clone(), constant.to_string())))
            }
        }
        Ok(None)
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

impl Matcher for ConstantStringMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        for constant in self.constants {
            let dat = tokenizer.clone().take(constant.len());
            if dat.size_hint().1.unwrap() != constant.len() {
                return Ok(None)
            }
            if dat.collect::<String>() == *constant {
                tokenizer.advance_n(constant.len());
                return Ok(Some(token!(tokenizer, self.token_type.clone(), constant.to_string())))
            }
        }
        Ok(None)
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

impl Matcher for KeyMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        for constant in self.constants.clone() {
            let dat = tokenizer.clone().take(constant.len());
            if dat.size_hint().1.unwrap() != constant.len() {
                return Ok(None)
            } else if &&dat.collect::<String>() == constant {
                if let Some(c) = tokenizer.peek_n(constant.len()) {
                    if "_!?".contains(*c) || c.is_alphanumeric() {
                        return Ok(None)
                    }
                }

                tokenizer.advance_n(constant.len());
                return Ok(Some(token!(tokenizer, self.token_type.clone(), constant.to_string())))
            }
        }
        Ok(None)
    }
}

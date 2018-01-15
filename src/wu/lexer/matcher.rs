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
                    return Err(make_error(Some(tokenizer.last_position()), "extra decimal point".to_owned()))
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

            Ok(Some(token!(tokenizer, Int, literal)))
        }
    }
}

pub struct StringLiteralMatcher;

impl Matcher for StringLiteralMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        let mut raw_marker = false;

        //Check which delimeter this string is using
        //If it's not using a delimeter, then it's not a string
        let delimeter  = match *tokenizer.peek().unwrap() {
            '"'  => '"',
            '\'' => '\'',
            //Used for raw strings
            'r' => {
                if tokenizer.peek_n(1) == Some(&'"') {
                    raw_marker = true;
                    tokenizer.advance();

                    '"'
                } else {
                    //It's not actually a raw string, it's just something that starts with an r
                    return Ok(None)
                }
            },
            //Not using a delimeter so it's not a string
            _ => return Ok(None),
        };

        tokenizer.advance();

        let mut string       = String::new();
        let mut found_escape = false;

        loop {
            // check if file ends before the string
            // basically this means there's no end delimiter
            if tokenizer.end() {
                return Err(make_error(Some(TokenPosition::new(tokenizer.last_position().line - 1, tokenizer.last_position().col)), format!("expected closing delimeter '{}' found end", delimeter)))
            }

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
                    c if c == delimeter => break,
                    _ => string.push(tokenizer.next().unwrap()),
                }
            }
        }
        tokenizer.advance();
        Ok(Some(token!(tokenizer, Str, string)))
    }
}

pub struct IdentifierMatcher;

impl Matcher for IdentifierMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        //Make sure the first character is alphabetic or '_' if not it's not an identifier
        if !tokenizer.peek().unwrap().is_alphabetic() && !(tokenizer.peek().unwrap() == &'_') {
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

pub struct CommentMatcher;

impl Matcher for CommentMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Response<Option<Token>> {
        if tokenizer.peek_range(2).unwrap() == "--" {
            while !tokenizer.end() && tokenizer.peek().unwrap() != &'\n' {
                tokenizer.advance();
            }
            Ok(Some(token!(tokenizer, EOL, "\n".into())))
        } else {
            Ok(None)
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
            if let Some(s) = tokenizer.peek_range(constant.len()) {
                if s == *constant {
                    tokenizer.advance_n(constant.len());
                    return Ok(Some(token!(tokenizer, self.token_type.clone(), constant.to_string())))
                }
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
        for constant in self.constants {
            if let Some(s) = tokenizer.peek_range(constant.len()) {
                if s == *constant {
                    if let Some(c) = tokenizer.peek_n(constant.len()) {
                        println!("{:?}", s);
                        if "_!?".contains(*c) || c.is_alphanumeric() {
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

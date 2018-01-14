#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Number,
    Str,
    Bool,
    Symbol,
    Operator,
    Identifier,
    Keyword,
    Whitespace,
    EOL,
    EOF,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TokenPosition {
    pub line: usize,
    pub col: usize,
}

impl TokenPosition {
    pub fn new(line: usize, col: usize) -> TokenPosition {
        TokenPosition {
            line,
            col,
        }
    }
}

impl Default for TokenPosition {
    fn default() -> Self {
        TokenPosition {
            line: 1,
            col: 1,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub position: TokenPosition,
    pub content: String,
}

impl Token {
    pub fn new(token_type: TokenType, position: TokenPosition, content: String) -> Token {
        Token {
            token_type,
            position,
            content,
        }
    }
}

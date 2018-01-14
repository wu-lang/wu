use super::lexer::*;
use super::visitor::TypeNode;
use super::*;

use std::rc::Rc;

pub struct Parser<'p> {
    pub tokens: Vec<Token>,
    pub top:    usize,

    // for displaying pretty warnings without returning Err
    pub lines:  &'p Vec<String>,
    pub path:   &'p str,
    
    pub inside: String,
}

impl<'p> Parser<'p> {
    pub fn new(tokens: Vec<Token>, lines: &'p Vec<String>, path: &'p str) -> Self {
        Parser {
            tokens,
            top: 0,
            lines,
            path,
            inside: String::new(),
        }
    }

    pub fn parse(&mut self) -> Response<Vec<Statement>> {
        let mut statements: Vec<Statement> = Vec::new();

        while self.remaining() > 1 {
            statements.push(self.statement()?)
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Response<Statement> {
        use TokenType::*;

        self.skip_types(vec![TokenType::Whitespace, TokenType::EOL])?;

        let node = match self.current_type() {
            Identifier => {
                let identifier_node = self.atom()?;

                let backup = self.top;

                self.skip_types(vec![TokenType::Whitespace])?;

                match self.current_content().as_str() {
                    ":" => {
                        self.next()?;

                        if self.current_content() == ":" {
                            self.next()?;

                            let right = self.expression()?;

                            StatementNode::ConstDefinition {
                                left:  identifier_node,
                                right: right,
                            }
                        } else {
                            if self.current_content() == "=" {
                                self.consume_content("=")?;

                                let right = self.expression()?;

                                StatementNode::Definition {
                                    kind:  TypeNode::Nil,
                                    left:  identifier_node,
                                    right: Some(right),
                                }
                            } else {
                                self.skip_types(vec![TokenType::Whitespace])?;

                                let kind = self.type_node()?;

                                self.skip_types(vec![TokenType::Whitespace])?;
                                
                                self.consume_content("=")?;

                                let right = self.expression()?;

                                StatementNode::Definition {
                                    kind,
                                    left:  identifier_node,
                                    right: Some(right),
                                }
                            }
                        }
                    },

                    "=" => {
                        self.next()?;

                        let right = self.expression()?;

                        StatementNode::Assignment {
                            left: identifier_node,
                            right,
                        }
                    },

                    _ => {
                        self.top = backup;
                        StatementNode::Expression(identifier_node)
                    }
                }
            },
            _ => StatementNode::Expression(self.expression()?)
        };

        Ok(Statement::new(node, self.position()))
    }

    fn type_node(&mut self) -> Response<TypeNode> {
        use TypeNode::*;
        
        let t = match self.consume_type(TokenType::Identifier)?.as_str() {
            "int"     => Int,
            "float"   => Float,
            "string"  => Str,
            "boolean" => Bool,
            other     => Id(other.to_owned()), 
        };

        Ok(t)
    }

    // grouping atoms into e.g. operations
    fn expression(&mut self) -> Response<Expression> {
        let expression = self.atom()?;

        if expression.0 == ExpressionNode::EOF {
            Ok(expression)
        } else {
            let backup_top = self.top;

            self.skip_types(vec![TokenType::Whitespace])?;

            if self.current_type() == TokenType::Operator {
                self.binary(expression)
            } else {
                self.top = backup_top;

                Ok(expression)
            }
        }
    }

    fn atom(&mut self) -> Response<Expression> {
        use self::ExpressionNode::*;

        self.skip_types(vec![TokenType::Whitespace])?;

        if self.remaining() == 0 {
            return Ok(Expression::new(EOF, self.position()))
        }

        let node = match self.current_type() {
            TokenType::Int        => Int(self.consume_type(TokenType::Int)?.parse().unwrap()),
            TokenType::Float      => Float(self.consume_type(TokenType::Float)?.parse().unwrap()),
            TokenType::Str        => Str(self.consume_type(TokenType::Str)?),
            TokenType::Bool       => Bool(self.consume_type(TokenType::Bool)? == "true"),
            TokenType::Identifier => Identifier(self.consume_type(TokenType::Identifier)?),

            TokenType::Symbol => match self.current_content().as_str() {
                "(" => {
                    let content = self.block_of(&Self::expression_, ("(", ")"))?;
                    self.skip_types(vec![TokenType::Whitespace])?;
                    
                    if content.len() > 1 {
                        return Err(make_error(Some(content.get(0).unwrap().1), format!("claused expression can only contain one item")))
                    }

                    return Ok(content.get(0).unwrap().clone())
                },

                ref t => return Err(make_error(Some(self.position()), format!("unexpected symbol: {}", t)))
            },

            TokenType::Whitespace => {
                self.next()?;
                return Ok(self.atom()?)
            },

            t => return Err(make_error(Some(self.position()), format!("token type '{:?}' currently unimplemented", t)))
        };

        Ok(Expression::new(node, self.position()))
    }

    fn expression_(self: &mut Self) -> Response<Option<Expression>> {
        let expression = self.expression()?;

        match expression.0 {
            ExpressionNode::EOF => Ok(None),
            _                   => Ok(Some(expression)),
        }
    }

    fn block_of<B>(&mut self, match_with: &Fn(&mut Self) -> Response<Option<B>>, delimeters: (&str, &str)) -> Response<Vec<B>> {
        let backup_inside = self.inside.clone();
        self.inside       = delimeters.0.to_owned();

        if self.current_content() == delimeters.0 {
            self.next()?
        }

        let mut stack  = Vec::new();
        let mut nested = 1;

        while nested != 0 {
            if self.current_content() == delimeters.1 {
                nested -= 1
            } else if self.current_content() == delimeters.0 {
                nested += 1
            }

            if nested == 0 {
                break
            }

            stack.push(self.current().clone());
            
            self.next()?
        }

        self.next()?;

        let mut parser  = Parser::new(stack, self.lines, self.path);
        parser.inside   = self.inside.clone();

        let mut stack_b = Vec::new();
        
        while let Some(n) = match_with(&mut parser)? {
            stack_b.push(n)
        }

        self.inside = backup_inside;

        Ok(stack_b)
    }

    // parsing operations using the Dijkstra shunting yard algorithm
    fn binary(&mut self, expression: Expression) -> Response<Expression> {
        let mut ex_stack = vec![expression];
        let mut op_stack: Vec<(Operator, u8)> = Vec::new();

        let position = self.position();
        op_stack.push(Operator::from(&self.current_content()).unwrap());
        self.next()?;

        let atom = self.atom()?;

        if atom.0 != ExpressionNode::EOF {
            ex_stack.push(atom)
        } else {
            return Err(make_error(Some(atom.1), format!("EOF is not good")))
        }

        let mut done = false;

        while ex_stack.len() > 1 {
            if !done {
                self.skip_types(vec![TokenType::Whitespace])?;

                if self.current_type() != TokenType::Operator {
                    done = true;
                    continue
                }

                if self.remaining() == 0 {
                    return Err(make_error(Some(self.position()), "missing right hand expression".to_owned()))
                }

                let position         = self.position();
                let (op, precedence) = Operator::from(&self.consume_type(TokenType::Operator)?).unwrap();

                if precedence >= op_stack.last().unwrap().1 {
                    let left  = ex_stack.pop().unwrap();
                    let right = ex_stack.pop().unwrap();

                    ex_stack.push(
                        Expression::new(
                            ExpressionNode::Binary {
                                right: Rc::new(left),
                                op:    op_stack.pop().unwrap().0,
                                left:  Rc::new(right),
                            },
                            position,
                        )
                    );

                    let atom = self.atom()?;

                    let term = if atom.0 != ExpressionNode::EOF {
                        atom
                    } else {
                        return Err(make_error(Some(atom.1), format!("EOF is not good")))
                    };

                    ex_stack.push(term);
                    op_stack.push((op, precedence));

                    continue
                }
                
                let term = self.atom()?;

                ex_stack.push(term);
                op_stack.push((op, precedence));
            }

            let left  = ex_stack.pop().unwrap();
            let right = ex_stack.pop().unwrap();

            ex_stack.push(
                Expression::new(
                    ExpressionNode::Binary {
                        right: Rc::new(left),
                        op:    op_stack.pop().unwrap().0,
                        left:  Rc::new(right),
                    },
                    position,
                )
            );
        }

        Ok(ex_stack.pop().unwrap())
    }
    
    // skipping tokens
    fn next(&mut self) -> Response<()> {
        if self.top <= self.tokens.len() {
            self.top += 1;
            Ok(())
        } else {
            Err(make_error(None, "nexting outside token stack".to_owned()))
        }
    }

    // going backwards
    fn back(&mut self) -> Response<()> {
        if self.top > 0 {
            self.top -= 1;
            Ok(())
        } else {
            Err(make_error(None, "backing outside token stack".to_owned()))
        }
    }

    // primarily for skipping whitespace
    fn skip_types(&mut self, tokens: Vec<TokenType>) -> Response<()> {
        loop {
            if self.remaining() > 1 {
                if tokens.contains(&self.current_type()) {
                    self.next()?
                } else {
                    break
                }
            } else {
                break
            }
        }

        Ok(())
    }

    fn remaining(&self) -> usize {
        if self.top >= self.tokens.len() {
            0
        } else {
            self.tokens.len() - self.top
        }
    }

    // getting the top of the token stack
    pub fn current(&self) -> &Token {
        if self.top > self.tokens.len() - 1 {
            return &self.tokens[self.tokens.len() - 1];
        }
        &self.tokens[self.top]
    }

    // easy access
    pub fn current_content(&self) -> String {
        self.current().content.clone()
    }

    pub fn current_type(&self) -> TokenType {
        self.current().token_type.clone()
    }

    pub fn position(&self) -> TokenPosition {
        self.current().position
    }

    pub fn expect_type(&self, token: TokenType) -> Response<()> {
        if self.current().token_type == token {
            Ok(())
        } else {
            Err(make_error(
                Some(self.current().position),
                format!("expecting type '{:?}', found '{:?}'", token, self.current_content())
            ))
        }
    }

    pub fn consume_type(&mut self, token: TokenType) -> Response<String> {
        if self.current().token_type == token {
            let content = self.current_content();
            self.next()?;
            Ok(content)
        } else {
            Err(make_error(
                Some(self.current().position),
                format!("expecting type '{:?}', found '{:?}'", token, self.current_content())
            ))
        }
    }

    pub fn expect_content(&self, content: &str) -> Response<()> {
        if self.current_content() == content {
            Ok(())
        } else {
            Err(make_error(
                Some(self.current().position),
                format!("expecting '{}', found '{}'", content, self.current_content())
            ))
        }
    }

    pub fn consume_content(&mut self, content: &str) -> Response<String> {
        if self.current().content == content {
            let content = self.current_content();
            self.next()?;
            Ok(content)
        } else {
            Err(make_error(
                Some(self.current().position),
                format!("expecting '{}', found '{}'", content, self.current_content())
            ))
        }
    }
}

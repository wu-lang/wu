use super::lexer::*;
use super::visitor::{TypeNode, Type, TypeMode};
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
        
        let position = self.position();

        let node = match self.current_type() {
            Keyword => match self.current_content().as_str() {
                "return" => {
                    self.next()?;
                    self.skip_types(vec![Whitespace])?;

                    if self.current_content() == "\n" {
                        self.next()?;
                        
                        StatementNode::Return(None)
                    } else {
                        let ret = StatementNode::Return(Some(self.expression()?));

                        if self.current_content() == "\n" {
                            self.next()?;
                        }

                        ret
                    }
                },

                "if" => StatementNode::If(self.if_node()?),

                _ => StatementNode::Expression(self.expression()?),
            },

            Identifier => {
                let backup_top = self.top;

                let position        = self.position();
                let identifier_node = Expression::new(ExpressionNode::Identifier(self.consume_type(TokenType::Identifier)?), position);

                self.skip_types(vec![TokenType::Whitespace])?;

                match self.current_content().as_str() {
                    ":" => {
                        self.next()?;

                        if self.current_content() == ":" {
                            self.next()?;

                            let right = self.expression()?;
                            
                            if self.current_content() == "\n" {
                                self.next()?
                            }

                            StatementNode::ConstDefinition {
                                kind:  TypeNode::Nil,
                                left:  identifier_node,
                                right: right,
                            }

                        } else {
                            if self.current_content() == "=" {
                                self.consume_content("=")?;

                                let right = self.expression()?;
                                
                                if self.current_content() == "\n" {
                                    self.next()?
                                }

                                StatementNode::Definition {
                                    kind:  TypeNode::Nil,
                                    left:  identifier_node,
                                    right: Some(right),
                                }

                            } else {
                                self.skip_types(vec![TokenType::Whitespace])?;

                                let kind = self.type_node()?;

                                self.skip_types(vec![TokenType::Whitespace])?;

                                if self.current_content() == "=" {
                                    self.next()?;

                                    let right = self.expression()?;
                                    
                                    if self.current_content() == "\n" {
                                        self.next()?
                                    }

                                    StatementNode::Definition {
                                        kind,
                                        left:  identifier_node,
                                        right: Some(right),
                                    }
                                } else if self.current_content() == ":" {
                                    self.next()?;

                                    let right = self.expression()?;

                                    if self.current_content() == "\n" {
                                        self.next()?
                                    }

                                    StatementNode::ConstDefinition {
                                        kind,
                                        left: identifier_node,
                                        right,
                                    }

                                } else {
                                    self.consume_content("\n")?;

                                    StatementNode::Definition {
                                        kind,
                                        left:  identifier_node,
                                        right: None,
                                    }
                                }
                            }
                        }
                    },

                    "=" => {
                        self.next()?;

                        let right = self.expression()?;

                        if self.current_content() == "\n" {
                            self.next()?
                        }

                        StatementNode::Assignment {
                            left: identifier_node,
                            right,
                        }
                    },

                    _ => {
                        self.top = backup_top;
                        StatementNode::Expression(self.expression()?)
                    }
                }
            },
            _ => StatementNode::Expression(self.expression()?)
        };

        Ok(Statement::new(node, position))
    }

    fn if_node(&mut self) -> Response<IfNode> {
        self.consume_content("if")?;
        self.skip_types(vec![TokenType::Whitespace])?;
        
        let condition = self.expression()?;
        
        self.skip_types(vec![TokenType::Whitespace])?;

        self.expect_content("{")?;

        let position = self.position();
        let body = Expression::new(ExpressionNode::Block(self.block_of(&mut Self::statement_, ("{", "}"))?), position);

        Ok(IfNode { condition, body, elses: None })
    }

    fn type_node(&mut self) -> Response<TypeNode> {
        use TypeNode::*;
        
        let t = match self.current_content().as_str() {
            "int"     => Int,
            "float"   => Float,
            "string"  => Str,
            "bool"    => Bool,
            "("       => {
                self.next()?;

                let mut params = Vec::new();
                
                let mut nested = 1;

                while nested != 0 {
                    if self.current_content() == ")" {
                        self.next()?;
                        nested -= 1;
                    } else if self.current_content() == "(" {
                        nested += 1
                    }

                    if nested == 0 {
                        break
                    }

                    self.skip_types(vec![TokenType::Whitespace])?;

                    params.push(Rc::new(Type::new(self.type_node()?, TypeMode::Just)));
                    
                    self.skip_types(vec![TokenType::Whitespace])?;
                    
                    if self.current_content() == "," {
                        self.next()?
                    }
                }

                self.skip_types(vec![TokenType::Whitespace])?;

                let retty = Type::new(self.type_node()?, TypeMode::Just);

                return Ok(Fun(params, Rc::new(retty)))
            },

            _ => return Ok(Id(self.consume_type(TokenType::Identifier)?)), 
        };

        self.next()?;

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

        let position = self.position();

        let node = match self.current_type() {
            TokenType::Int        => Int(self.consume_type(TokenType::Int)?.parse().unwrap()),
            TokenType::Float      => Float(self.consume_type(TokenType::Float)?.parse().unwrap()),
            TokenType::Str        => Str(self.consume_type(TokenType::Str)?),
            TokenType::Bool       => Bool(self.consume_type(TokenType::Bool)? == "true"),
            TokenType::Identifier => Identifier(self.consume_type(TokenType::Identifier)?),

            TokenType::Symbol => match self.current_content().as_str() {
                "{" => {
                    let content = self.block_of(&mut Self::statement_, ("{", "}"))?;

                    Block(content)
                },

                "(" => {
                    let backup_top = self.top;
                    self.next()?;
                    
                    let mut nested = 1;

                    while nested != 0 {
                        if self.current_content() == ")" {
                            nested -= 1
                        } else if self.current_content() == "(" {
                            nested += 1
                        }
                        
                        if nested == 0 {
                            break
                        }

                        self.next()?
                    }

                    self.next()?;
                    
                    self.skip_types(vec![TokenType::Whitespace])?;

                    if self.current_content() != "->" {
                        match self.type_node() {
                            _ => ()
                        }

                        self.skip_types(vec![TokenType::Whitespace])?;
                    }

                    if self.current_content() != "->" {
                        self.top = backup_top;

                        let content = self.block_of(&Self::expression_, ("(", ")"))?;

                        if content.len() > 1 {
                            return Err(make_error(Some(content.get(0).unwrap().1), format!("claused expression can only contain one item")))
                        }

                        content.get(0).unwrap().clone().0
                    } else {
                        self.top = backup_top;
                        self.function()?
                    }
                },

                ref t => return Err(make_error(Some(self.position()), format!("unexpected symbol: {}", t)))
            },

            TokenType::Whitespace | TokenType::EOL => {
                self.next()?;
                return Ok(self.atom()?)
            },

            t => return Err(make_error(Some(position), format!("token type '{:?}' currently unimplemented", t)))
        };

        self.maybe_call(Expression::new(node, position))
    }
    
    fn statement_(self: &mut Self) -> Response<Option<Statement>> {
        let statement = self.statement()?;

        let statement = match statement.0 {
            StatementNode::Expression(ref e) => if e.0 == ExpressionNode::EOF {
                None
            } else {
                Some(statement.clone())
            },

            _ => Some(statement),
        };

        Ok(statement)
    }

    fn function(&mut self) -> Response<ExpressionNode> {
        let params = self.block_of(&Self::param_, ("(", ")"))?;

        self.skip_types(vec![TokenType::Whitespace])?;

        let mut return_type = TypeNode::Nil;

        if self.current_content() != "->" {
            return_type = self.type_node()?;
            
            self.skip_types(vec![TokenType::Whitespace])?;
        }

        self.consume_content("->")?;

        let body = Rc::new(self.expression()?);

        Ok(ExpressionNode::Function {params, return_type, body})
    }

    fn param_(self: &mut Self) -> Response<Option<(String, TypeNode, Option<Rc<Expression>>)>> {
        if self.remaining() == 0 {
            return Ok(None)
        }

        let name = self.consume_type(TokenType::Identifier)?;
        self.skip_types(vec![TokenType::Whitespace, TokenType::EOL])?;
        
        self.consume_content(":")?;
        self.skip_types(vec![TokenType::Whitespace, TokenType::EOL])?;

        let kind = self.type_node()?;
        self.skip_types(vec![TokenType::Whitespace, TokenType::EOL])?;

        let value = if self.current_content() == "=" {
            self.next()?;
            self.skip_types(vec![TokenType::Whitespace, TokenType::EOL])?;

            Some(Rc::new(self.expression()?))
        } else {
            None
        };

        if self.remaining() > 1 {
            self.consume_content(",")?;
            self.skip_types(vec![TokenType::Whitespace, TokenType::EOL])?;
        }

        if self.remaining() == 0 {
            Ok(Some((name, kind, value)))
        } else {
            Ok(Some((name, kind, value)))
        }
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

        self.consume_content(delimeters.0)?;

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

        if stack.len() > 0 {
            let mut parser  = Parser::new(stack, self.lines, self.path);
            parser.inside   = self.inside.clone();

            let mut stack_b = Vec::new();
            
            while let Some(n) = match_with(&mut parser)? {
                stack_b.push(n)
            }

            self.inside = backup_inside;

            Ok(stack_b)
        } else {
            Ok(Vec::new())
        }

    }

    fn maybe_call(&mut self, atom: Expression) -> Response<Expression> {
        use ExpressionNode::*;

        let backup_top = self.top;
        
        self.skip_types(vec![TokenType::Whitespace])?;

        let node = match atom.0 {
            Identifier(_) => match self.current_content().as_str() {
                "(" => {
                    let args = self.block_of(&mut Self::arg_, ("(", ")"))?;
                    let pos  = atom.1.clone();
                    let call = Expression(Call(Rc::new(atom), args.iter().map(|x| Rc::new(x.clone())).collect()), pos);

                    return Ok(call)
                },

                _ => atom
            },
            
            _ => atom
        };

        self.top = backup_top;
        
        Ok(node)
    }

    fn arg_(self: &mut Self) -> Response<Option<Expression>> {
        let expression = Self::expression_(self);
        
        self.skip_types(vec![TokenType::Whitespace])?;
        
        if self.remaining() > 1 {
            self.consume_content(",")?;
        }

        expression
    }

    // parsing operations using the Dijkstra shunting yard algorithm
    fn binary(&mut self, expression: Expression) -> Response<Expression> {
        let mut ex_stack = vec![expression];                // initial expression on the stack
        let mut op_stack: Vec<(Operator, u8)> = Vec::new(); // the operator stack

        let position = self.position();
        op_stack.push(Operator::from(&self.consume_type(TokenType::Operator)?).unwrap()); // find operator

        // covering bad case
        if self.current_content() == "\n" {
            return Err(make_error(Some(position), format!("EOL is not good")))
        }

        // the right hand of operation
        let atom = self.atom()?;

        if atom.0 != ExpressionNode::EOF {
            // pushing right hand of operation onto the stack
            ex_stack.push(atom)
        } else {
            return Err(make_error(Some(atom.1), format!("EOF is not good")))
        }

        let mut done = false;

        // loop for getting nested operations
        while ex_stack.len() > 1 {
            if !done {
                self.skip_types(vec![TokenType::Whitespace])?;

                if self.current_type() != TokenType::Operator { // stop looking when running into non-op
                    done = true;
                    continue
                }

                if self.remaining() == 0 {
                    return Err(make_error(Some(self.position()), "missing right hand expression".to_owned()))
                }

                let position         = self.position();
                let (op, precedence) = Operator::from(&self.consume_type(TokenType::Operator)?).unwrap(); // the next operator has been found

                // we're now comparing precedence, sorting the operators
                if precedence >= op_stack.last().unwrap().1 {
                    // in this case, found operator is assembled and pushed onto the stack later
                    let left  = ex_stack.pop().unwrap();
                    let right = ex_stack.pop().unwrap();

                    // the first operation, with lower precedence is pushed onto the stack
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
                    
                    // right hand of the higher precedence operation is found
                    let atom = self.atom()?;

                    if atom.0 == ExpressionNode::EOF {
                        return Err(make_error(Some(atom.1), format!("EOF is not good")))
                    }

                    ex_stack.push(atom); // and is pushed onto the stack
                    op_stack.push((op, precedence)); // along with the operator from before

                } else { // otherwise, we just push the lower precedence operation onto the stack
                    let term = self.atom()?;

                    ex_stack.push(term);
                    op_stack.push((op, precedence));
                }
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
                format!("expected type '{:?}', found '{:?}'", token, self.current_content())
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
                format!("expected type '{:?}', found '{:?}'", token, self.current_content())
            ))
        }
    }

    pub fn expect_content(&self, content: &str) -> Response<()> {
        if self.current_content() == content {
            Ok(())
        } else {
            Err(make_error(
                Some(self.current().position),
                format!("expected '{}', found '{}'", content, self.current_content())
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
                format!("expected '{}', found '{}'", content, self.current_content())
            ))
        }
    }
}

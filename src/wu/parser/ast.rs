use super::lexer::*;
use super::visitor::*;

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode {
    Expression(Expression),

    Return(Option<Expression>),

    Definition {
        kind:  TypeNode,
        left:  Expression,
        right: Option<Expression>,
    },

    ConstDefinition {
        kind:  TypeNode,
        left:  Expression,
        right: Expression,
    },

    Assignment {
        left:  Expression,
        right: Expression,
    },

    Struct {
        name:    String,
        members: Vec<(String, TypeNode, Option<Rc<Expression>>)>
    },

    If(IfNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement(pub StatementNode, pub TokenPosition);

impl Statement {
    pub fn new(node: StatementNode, position: TokenPosition) -> Statement {
        Statement(node, position)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode {
    Float(f64),
    Int(i64),
    Bool(bool),
    Str(String),
    Array(Vec<Expression>),
    Identifier(String),
    Binary {left: Rc<Expression>, op: Operator, right: Rc<Expression>,},
    Function {params: Vec<(String, TypeNode, Option<Rc<Expression>>)>, return_type: TypeNode, body: Rc<Expression>},
    Call(Rc<Expression>, Vec<Expression>),
    Block(Vec<Statement>),
    Index(Rc<Expression>, Rc<Expression>),
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression(pub ExpressionNode, pub TokenPosition);

impl Expression {
    pub fn new(node: ExpressionNode, position: TokenPosition) -> Expression {
        Expression(node, position)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfNode {
    pub condition: Expression,
    pub body:      Expression,
    pub elses:     Option<Vec<(Option<Expression>, Expression, TokenPosition)>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Pow,
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Equal,
    NEqual,
    Lt,
    Gt,
    LtEqual,
    GtEqual,
    Concat,
    PipeLeft,
    PipeRight,
}

impl Operator {
    pub fn from(v: &str) -> Option<(Operator, u8)> {
        use self::Operator::*;

        match v {
            "^"   => Some((Pow, 0)),
            "*"   => Some((Mul, 1)),
            "/"   => Some((Div, 1)),
            "%"   => Some((Mod, 1)),
            "+"   => Some((Add, 2)),
            "-"   => Some((Sub, 2)),
            "++"  => Some((Concat, 2)),
            "=="  => Some((Equal, 3)),
            "!="  => Some((NEqual, 3)),
            "<"   => Some((Lt, 4)),
            ">"   => Some((Gt, 4)),
            "<="  => Some((LtEqual, 4)),
            ">="  => Some((GtEqual, 4)),
            "<|"  => Some((PipeLeft, 5)),
            "|>"  => Some((PipeRight, 5)),
            _     => None,
        }
    }
}

use crate::wu::{source::Source, parser::ast::Statement, lexer::token::Pos};

use std::collections::HashMap;

pub struct Evaluator<'g> {
    source: &'g Source,
    method_calls: &'g HashMap<Pos, bool>,
}

impl<'g> Evaluator<'g> {
    pub fn new(source: &'g Source, method_calls: &'g HashMap<Pos, bool>) -> Self {
        Self {
            source,

            method_calls,
        }
    }

    pub fn eval(&mut self, ast: &'g Vec<Statement>) {
        panic!("me no know how eval");
    }
}

mod context;
use context::Context;

use crate::wu::{
    lexer::{
        token::{Pos, Token},
        Lexer,
    },
    parser::{
        ast::{Expression, ExpressionNode, Statement, StatementNode},
        Parser,
    },
    source::Source,
    visitor::Visitor,
};

use std::collections::HashMap;

type RustFn = Box<dyn Fn(Vec<&Expression>) -> Option<ExpressionNode>>;

/// Runs wu code.
pub struct Evaluator<'g> {
    source: &'g Source,
    method_calls: &'g HashMap<Pos, bool>,

    /// The scope of available variables.
    contexts: Vec<Context>,

    /// Builtin Rust functions
    builtins: HashMap<String, RustFn>,
}

impl<'g> Evaluator<'g> {
    pub fn new(source: &'g Source, method_calls: &'g HashMap<Pos, bool>) -> Self {
        let ctx = Context {
            map: HashMap::new(),
            parent: None,
        };

        let mut builtins: HashMap<String, RustFn> = HashMap::new();
        builtins.insert(
            "print".to_string(),
            Box::new(|args: Vec<_>| {
                println!("{}", args[0].string_ref().expect("can only print strings"));
                None
            }),
        );

        Self {
            source,
            method_calls,
            contexts: vec![ctx],
            builtins,
        }
    }

    /// Start evaluating some AST the top most level, i.e. Context 0.
    /// This implies that the code being evaluated is at the top most scope,
    /// and all other scopes spiral down from that one.
    pub fn eval(&mut self, ast: &'g Vec<Statement>) {
        self.eval_recurse(ast, 0);
    }

    /// Evaluate some AST, referring to the provided index as the context to use.
    /// Because this function can use any level of context, it's more useful for recursion;
    /// to evaluate a block or module in a new scope, you can simply call this function on the AST
    /// and spawn a new child context off of the current context for it to be evaluated in.
    pub fn eval_recurse(&mut self, ast: &'g Vec<Statement>, ctx: usize) {
        for Statement { node, .. } in ast {
            println!("node: {:?}", node);
            match node {
                StatementNode::Expression(Expression { node, .. }) => {
                    self.eval_expression(node, ctx);
                }
                StatementNode::Assignment(id_expr, value) => {
                    let id = id_expr.identifier_ref().expect("can only assign to identifiers");
                    self.assign(ctx, id.clone(), value.clone());
                }
                StatementNode::Variable(_, id, value) => {
                    if let Some(value) = value {
                        self.assign(ctx, id.clone(), value.clone());
                    }
                }
                _ => {}
            }
        }
    }

    fn eval_expression(&mut self, expr: &ExpressionNode, ctx: usize) {
        use ExpressionNode::*;

        match expr {
            Call(called, args) => {
                match &(*called).node {
                    Identifier(id) => {
                        if id == "__rust_call" {
                            // __rust_call takes one string parameter, which is composed of words
                            // separated by spaces. The first word is the name of the function to
                            // call, the second word is the name of variables to pass it.
                            let arg = args[0]
                                .string_ref()
                                .expect("__rust_call must take string arg");

                            // the string parameter that's passed to rust_call, split into words.
                            let mut call = arg.split(" ");

                            // the first thing in that giant string parameter, which should be the
                            // Rust function to call
                            let fn_name = call.next()
                                .expect("the first argument to __rust_call must be the name of the fn to call");

                            // if we can find a function with that name, call it. Look up the
                            // values the variable names refer to, and pass the function those
                            // values as parameters.
                            if let Some(builtin) = self.builtins.get(fn_name) {
                                builtin(call.map(|arg| {
                                    self.fetch(ctx, arg).unwrap_or_else(|why| {
                                        panic!("no value for rust_call var {}: {}", called.pos, why)
                                    })
                                }).collect::<Vec<_>>());
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Recursively searches through a given context and then all of its ancestors for a certain
    /// value, and return a reference to that value.
    fn fetch(&self, ctx: usize, id: &str) -> Result<&Expression, String> {
        let Context { map, parent } = &self.contexts[ctx];
        map.get(id)
            .or_else(move || parent.and_then(move |parent| self.fetch(parent, id).ok()))
            .ok_or(format!("couldn't find variable with identifier \"{}\"", id))
    }
    
    /// Stores a new value in the most local context.
    fn assign(&mut self, ctx: usize, id: String, to: Expression) -> Option<Expression> {
        self.contexts[ctx].map.insert(id, to)
    }
}

#[test]
fn test_eval() {
    fn eval(content: &str) -> String {
        use std::sync::{Arc, Mutex};

        let content = format!(
            //"__rust_call: fun() {{}} \n\
            "{}",
            content
        );
        let source = Source::from(
            "testing.wu",
            content.lines().map(|x| x.into()).collect::<Vec<String>>(),
        );
        let tokens = Lexer::default(content.chars().collect(), &source)
            .filter_map(|x| x.ok())
            .collect::<Vec<Token>>();

        let mut parser = Parser::new(tokens, &source);

        let ast = parser.parse().expect("couldn't parse");

        /*
        let visitor = {
            let mut v = Visitor::new(&ast, &source);
            v.visit().expect("visit error");
            v
        };*/

        let output = Arc::new(Mutex::new(String::new()));

        let method_calls = HashMap::new();
        let mut evalr = Evaluator::new(&source, &method_calls); //&visitor.method_calls)
        evalr.builtins.insert(
            "print".to_string(),
            {
                let output = output.clone();

                Box::new(move |args: Vec<_>| {
                    let mut output = output.lock().unwrap();
                    output.push_str(args[0].string_ref().expect("can only print strings"));
                    output.push('\n');

                    None
                })
            },
        );
        evalr.eval(&ast);

        let output = output.lock().unwrap().to_string();
        output
    }

    assert_eq!(
        eval(
            "hi := \"hello wurld!\" \n\
            __rust_call(\"print hi\")"
        ),
        "hello wurld!\n".to_string(),
    );
}

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
use std::rc::Rc;

type RustFn = Box<dyn Fn(Vec<Rc<Expression>>) -> Option<ExpressionNode>>;

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
                println!("normal builtin call!");
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
    pub fn eval(&mut self, ast: &Vec<Statement>) {
        self.eval_recurse(0, ast.clone());
    }

    /// Evaluate some AST, referring to the provided index as the context to use.
    /// Because this function can use any level of context, it's more useful for recursion;
    /// to evaluate a block or module in a new scope, you can simply call this function on the AST
    /// and spawn a new child context off of the current context for it to be evaluated in.
    pub fn eval_recurse(&mut self, ctx: usize, ast: Vec<Statement>) {
        for Statement { node, .. } in ast {
            println!("node: {:?}", node);
            match node {
                StatementNode::Expression(expr) => {
                    self.eval_expression(ctx, Rc::new(expr));
                }
                StatementNode::Assignment(id_expr, value) => {
                    let id = id_expr
                        .identifier_ref()
                        .expect("can only assign to identifiers");
                    self.assign(ctx, id.clone(), Rc::new(value));
                }
                StatementNode::Variable(_, id, value) => {
                    if let Some(value) = value {
                        self.assign(ctx, id.clone(), Rc::new(value));
                    }
                }
                _ => {}
            }
        }
    }

    fn eval_expression(&mut self, ctx: usize, expr: Rc<Expression>) -> Option<Rc<Expression>> {
        use ExpressionNode::*;

        match &expr.node {
            Call(called, args) => {
                match &(*called).node {
                    Identifier(id) => {
                        println!("id: {}", id);
                        if self.builtins.contains_key(id) {
                            println!("found builtin call!");
                            println!("args: {:?}", args);

                            let args = args
                                .into_iter()
                                .map(|arg| {
                                    self.eval_expression(ctx, Rc::new(arg.clone()))
                                        .unwrap_or_else(|| {
                                            panic!("can't pass as function arg: {}", arg.pos)
                                        })
                                })
                                .collect::<Vec<_>>();

                            let builtin = self
                                .builtins
                                .get(id)
                                .expect("found builtin but couldn't pull it from HashMap");

                            // if we can find a function with that name, call it. Look up the
                            // values the variable names refer to, and pass the function those
                            // values as parameters.
                            return builtin(args)
                                .map(|node| Rc::new(Expression::new(node, expr.pos.clone())));
                        }
                    }
                    _ => {}
                }
            }
            Identifier(id) => {
                return self.fetch(ctx, &id);
            }
            Int(_) => return Some(expr),
            Float(_) => return Some(expr),
            Str(_) => return Some(expr),
            Char(_) => return Some(expr),
            Bool(_) => return Some(expr),
            _ => {}
        }

        None
    }

    /// Recursively searches through a given context and then all of its ancestors for a certain
    /// value, and return a reference to that value.
    //fn fetch(&self, ctx: usize, id: &str) -> Result<&Expression, String> {
    fn fetch(&self, ctx: usize, id: &str) -> Option<Rc<Expression>> {
        let Context { map, parent } = &self.contexts[ctx];
        map.get(id)
            .map(|x| Rc::clone(x))
            .or_else(move || parent.and_then(move |parent| self.fetch(parent, id)))
        //.ok()))
        //.ok_or(format!("couldn't find variable with identifier \"{}\"", id))
    }

    /// Stores a new value in the most local context.
    fn assign(&mut self, ctx: usize, id: String, to: Rc<Expression>) -> Option<Rc<Expression>> {
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
        evalr.builtins.insert("print".to_string(), {
            let output = output.clone();

            Box::new(move |args: Vec<_>| {
                let mut output = output.lock().unwrap();
                output.push_str(args[0].string_ref().expect("can only print strings"));
                output.push('\n');

                println!("overrriden builtin call!");

                None
            })
        });
        evalr.eval(&ast);

        let output = output.lock().unwrap().to_string();
        output
    }

    assert_eq!(
        eval("print(\"hello wurld!\")"),
        "hello wurld!\n".to_string(),
    );
}

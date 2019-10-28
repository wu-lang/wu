mod context;
use context::Context;

use crate::wu::{
    lexer::{
        token::{Pos, Token},
        Lexer,
    },
    parser::{
        ast::{Expression, ExpressionNode, Operator, Statement, StatementNode},
        Parser,
    },
    source::Source,
    visitor::{TypeNode, Visitor},
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
    /// Functions get special contexts that can be accessed whenever they're referred to.
    /// When someone mentions a function name, we need to be able to figure out which context it has.
    function_contexts: HashMap<String, usize>, //TODO: find a better way to store this

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
                println!(
                    "{}\n",
                    args.iter()
                        .filter_map(|arg| arg.try_str())
                        .collect::<Vec<_>>()
                        .join("\t")
                );
                None
            }),
        );

        Self {
            source,
            method_calls,
            contexts: vec![ctx],
            function_contexts: HashMap::new(),
            builtins,
        }
    }

    /// Start evaluating some AST the top most level, i.e. Context 0.
    /// This implies that the code being evaluated is at the top most context,
    /// and all other contexts spiral down from that one.
    pub fn eval(&mut self, ast: &Vec<Statement>) {
        self.eval_recurse(0, ast.clone());
    }

    /// Evaluate some AST, referring to the provided index as the context to use.
    /// Because this function can use any level of context, it's more useful for recursion;
    /// to evaluate a block or module in a new context, you can simply call this function on the AST
    /// and spawn a new child context off of the current context for it to be evaluated in.
    pub fn eval_recurse(&mut self, ctx: usize, ast: Vec<Statement>) -> Option<Rc<Expression>> {
        ast.into_iter()
            .filter_map(|Statement { node, .. }| {
                println!("\n node: {:?}", node);
                match node {
                    StatementNode::Expression(expr) => self.eval_expression(ctx, Rc::new(expr)),
                    StatementNode::Assignment(id_expr, value) => {
                        let id = id_expr
                            .identifier_ref()
                            .expect("can only assign to identifiers");
                        self.assign(ctx, id.clone(), Rc::new(value));
                        None
                    }
                    StatementNode::Variable(_, id, value) => {
                        if let Some(value) = value {
                            // if this variable is a function, allocate a context for it,
                            // so it has somewhere to store its own variables in n' stuff.
                            if let ExpressionNode::Function(_, ..) = value.node {
                                let fn_context = self.new_ctx(ctx);
                                self.function_contexts.insert(id.clone(), fn_context);
                            }

                            self.assign(ctx, id.clone(), Rc::new(value));
                        }

                        None
                    }
                    _ => None,
                }
            })
            .last()
    }

    fn eval_expression(&mut self, ctx: usize, expr: Rc<Expression>) -> Option<Rc<Expression>> {
        use ExpressionNode::*;

        match &expr.node {
            Call(called, args) => {
                match &(*called).node {
                    Identifier(id) => {
                        // find out what the arguments are referencing.
                        // if they're just values, those values will be directly
                        // returned, but if they're variable references or function
                        // calls, those will be executed/fetched.
                        let arg_vals = args
                            .into_iter()
                            .map(|arg| {
                                self.eval_expression(ctx, Rc::new(arg.clone()))
                                    .unwrap_or_else(|| {
                                        panic!("can't pass as function arg: {}", arg.pos)
                                    })
                            })
                            .collect::<Vec<_>>();

                        // builtins don't rely on scope and take precedence over user defined
                        // functions. this means that they cannot be shadowed.
                        if self.builtins.contains_key(id) {
                            let builtin = self
                                .builtins
                                .get(id)
                                .expect("found builtin but couldn't pull it from HashMap");

                            // if we can find a function with that name, call it. Look up the
                            // values the variable names refer to, and pass the function those
                            // values as parameters.
                            return builtin(arg_vals)
                                .map(|node| Rc::new(Expression::new(node, expr.pos.clone())));
                        } else {
                            // retrieve the function from the context
                            let fun = self.fetch(ctx, &id).unwrap_or_else(|| {
                                panic!("No function with the name \"{}\" found", id)
                            });

                            // get the arguments and block out of the function
                            let (args, _, block, _) =
                                fun.function_ref().expect("Can only call functions");

                            // we only need the names of the arguments, not their types.
                            let arg_names = args.into_iter().map(|(id, _type)| id);

                            // locate the context for the provided function
                            let fun_context_index = *self
                                .function_contexts
                                .get(id)
                                .expect("This function wasn't properly defined!");

                            // fetch the context so we can add the variables to it
                            let mut fun_context = self
                                .contexts
                                .get_mut(fun_context_index)
                                .expect("invalid function context index!");

                            // load all of the arguments and their values into that function
                            for (name, val) in arg_names.zip(arg_vals.into_iter()) {
                                fun_context.map.insert(name.to_string(), val);
                            }

                            // extract the code out of the block it's in.
                            // if we don't it will get another extra scope,
                            // which is inefficient.
                            let block = (**block)
                                .clone()
                                .block()
                                .expect("this function wasn't in a block for some reason");

                            println!("CALLING USER DEFINED FUNCTION! \n");
                            return dbg!(self.eval_recurse(fun_context_index, block));
                        }
                    }
                    _ => {}
                }
            }
            Identifier(id) => {
                return self.fetch(ctx, &id);
            }
            Cast(to_cast, new_type) => {
                use TypeNode::*;

                let to_cast = self
                    .eval_expression(ctx, Rc::clone(to_cast))
                    .expect("Can't perfom cast on value:");

                if let Ok(f) = to_cast.float() {
                    return match new_type.node {
                        Str => Some(Rc::new(Expression::new(
                            ExpressionNode::Str(format!("{:?}", f)),
                            expr.pos.clone(),
                        ))),
                        _ => None,
                    };
                }

                return None;
            }
            Binary(left, op, right) => {
                pub use Operator::*;

                let l = self
                    .eval_expression(ctx, Rc::clone(left))
                    .expect("Can't perfom binary operation on left value:");
                let r = self
                    .eval_expression(ctx, Rc::clone(right))
                    .expect("Can't perfom binary operation on right value:");

                return Some(Rc::new(Expression::new(
                    {
                        if let (Ok(l), Ok(r)) = (l.float(), r.float()) {
                            match op {
                                Add => Float(l + r),
                                Sub => Float(l - r),
                                Mul => Float(l * r),
                                Div => Float(l / r),
                                Mod => Float(l % r),
                                Pow => Float(l.powf(r)),
                                Eq => Bool(l == r),
                                Lt => Bool(l < r),
                                Gt => Bool(l > r),
                                NEq => Bool(l != r),
                                LtEq => Bool(l <= r),
                                GtEq => Bool(l >= r),
                                _ => return None,
                            }
                        } else if let (Ok(l), Ok(r)) = (l.float(), r.int()) {
                            use std::convert::TryInto;
                            match op {
                                Pow => Float(l.powi(r.try_into().unwrap())),
                                _ => return None,
                            }
                        } else {
                            return None;
                        }
                    },
                    expr.pos.clone(),
                )));
            }
            Int(_) | Float(_) | Str(_) | Char(_) | Bool(_) | Function(_, ..) => return Some(expr),
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

    /// This allocates a new empty context on the stack of contexts and returns the index of the
    /// new context that is created. All values in all ancestors of a context are accessible from
    /// the child context.
    fn new_ctx(&mut self, parent: usize) -> usize {
        self.contexts.push(Context::empty_child(parent));
        self.contexts.len() - 1
    }
}

#[test]
fn test_eval() {
    fn eval<S: Into<String>>(content_raw: S, inputs: &[&'static str]) -> String {
        use std::sync::{Arc, Mutex};

        // making the source usable
        let content = content_raw.into();
        let source = Source::from(
            "testing.wu",
            content.lines().map(|x| x.into()).collect::<Vec<String>>(),
        );

        // lexing
        let tokens = Lexer::default(content.chars().collect(), &source)
            .filter_map(|x| x.ok())
            .collect::<Vec<Token>>();

        // parsing
        let mut parser = Parser::new(tokens, &source);
        let ast = parser.parse().expect("couldn't parse");

        // type checking
        let visitor = {
            let mut v = Visitor::new(&ast, &source);
            v.visit().expect("visit error");
            v
        };

        // evaluating
        let mut evalr = Evaluator::new(&source, &visitor.method_calls);

        // overriding some of the STD to make things easier to test
        let output = Arc::new(Mutex::new(String::new()));
        let input: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(inputs.to_vec()));

        evalr.builtins.insert("print".to_string(), {
            let output = output.clone();

            Box::new(move |args: Vec<_>| {
                let mut output = output.lock().unwrap();
                output.push_str(
                    &args
                        .iter()
                        .filter_map(|arg| arg.try_str())
                        .collect::<Vec<_>>()
                        .join("\t"),
                );
                output.push('\n');

                None
            })
        });
        evalr.builtins.insert("input".to_string(), {
            let input = input.clone();

            Box::new(move |args: Vec<_>| {
                let mut input = input.lock().unwrap();

                Some(ExpressionNode::Str(
                    input
                        .pop()
                        .expect("Not enough inputs were supplied to test!")
                        .to_owned(),
                ))
            })
        });

        // finally evaluating
        evalr.eval(&ast);

        let output = output.lock().unwrap().to_string();
        output
    }

    assert_eq!(
        eval(
            "\n\
             print: extern fun(...) \n\
             print(\"hello wurld!\") ",
            &[]
        ),
        "hello wurld!\n".to_string(),
    );

    assert_eq!(
        eval(
            [
                "print: extern fun(...)",
                "",
                "square: fun(x: float) -> float {",
                "   # alternatively x^2",
                "   x * x",
                "}",
                "",
                "print(square(5.0) as str)",
            ]
            .join("\n"),
            &[]
        ),
        "25.0\n".to_string(),
    )
    /*
    assert_eq!(
        eval([
            "print: extern fun(...)",
            "square: fun(x: float) -> float {",
            "   # alternatively x^2",
            "   x * x",
            "}",
            "",
            "many := input(\"how many? \") as float",
            "while many > 0.0 {",
            "   many -= 1.0",
            "   ",
            "   print(square(many))",
            "}",
        ].join("\n"), &["10"]),
        "hello wurld!\n".to_string(),
    );*/
}

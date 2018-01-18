use super::*;
use std::fmt::*;

#[derive(Debug, Clone)]
pub struct Codegen<'c> {
    pub ast: &'c Vec<Statement>,
}

impl<'c> Codegen<'c> {
    pub fn new(ast: &'c Vec<Statement>)-> Self {
        Codegen {
            ast,
        }
    }

    pub fn generate(&self)-> String {
        let mut code = String::new();

        for statement in self.ast.iter() {
            code.push_str(&format!("{}\n", self.gen_statement(&statement.0)))
        }

        code
    }

    fn gen_statement(&self, statement: &StatementNode) -> String {
        use StatementNode::*;

        match *statement {
            Expression(ref expression) => self.gen_expression(&expression.0),
            Return(ref value)          => format!("return{}", match *value {
                Some(ref v) => format!(" {}", self.gen_expression(&v.0)),
                None        => String::from("\n"),
            }),
            _                          => String::new()
        }
    }
    
    fn gen_expression(&self, expression: &ExpressionNode) -> String {
        use ExpressionNode::*;

        match *expression {
            Float(ref n)      => format!("{}", n),
            Int(ref n)        => format!("{}", n),
            Str(ref n)        => format!(r#""{}"""#, n),
            Bool(ref n)       => format!("{}", n),
            Identifier(ref n) => format!("{}", n),

            Call(ref callee, ref args) => {
                let mut code = self.gen_expression(&callee.0);

                code.push('(');

                let mut acc = 1;

                for arg in args.iter() {
                    code.push_str(&self.gen_expression(&arg.0));

                    if acc != args.len() {
                        code.push(',');
                    }

                    acc += 1
                }

                code.push(')');

                code
            }

            Binary { ref left, ref op, ref right } => self.gen_operation(&left.0, op, &right.0),

            Block(ref statements) => {
                let mut code = "do\n".to_string();

                for statement in statements {
                    code.push_str(&self.gen_statement(&statement.0))
                }

                code.push_str("\nend");

                code
            }

            Function { ref params, ref body, .. } => {
                let mut code = "function(".to_string();
                
                let mut acc = 1;
                
                for param in params {
                    code.push_str(&param.0);

                    if acc != params.len() {
                        code.push(',');
                    }

                    acc += 1
                }

                code.push_str(")\n");

                code.push_str(&self.gen_expression(&body.0));

                code.push_str("\nend");

                code
            }

            _ => String::new(),
        }
    }

    fn gen_operation(&self, left: &ExpressionNode, op: &Operator, right: &ExpressionNode) -> String {
        use Operator::*;
        use ExpressionNode::*;
        
        let c = match *op {
            PipeRight => {
                let compiled_left  = self.gen_expression(left);
                let compiled_right = self.gen_expression(right);

                format!("{}({})", compiled_right, compiled_left)
            },
            
            PipeLeft => {
                let compiled_left  = self.gen_expression(left);
                let compiled_right = self.gen_expression(right);

                format!("{}({})", compiled_left, compiled_right)
            },
            
            _ => {
                let compiled_left  = self.gen_expression(left);
                let compiled_op    = self.gen_operator(op);
                let compiled_right = self.gen_expression(right);

                match *right {
                    Int(_)     |
                    Float(_)     |
                    Str(_)        |
                    Bool(_)       |
                    Identifier(_) => format!("{}{}{}", compiled_left, compiled_op, compiled_right),
                    _             => format!("{}{}({})", compiled_left, compiled_op, compiled_right),
                }
            }
        };

        c
    }
    
    fn gen_operator(&self, op: &Operator) -> String {
        use Operator::*;
        
        let c = match *op {
            Add     => "+",
            Sub     => "-",
            Mul     => "*",
            Div     => "/",
            Mod     => "%",
            Pow     => "^",
            Equal   => "==",
            NEqual  => "~=",
            Lt      => "<",
            LtEqual => "<=",
            Gt      => ">",
            GtEqual => ">=",
            Concat  => "..",
            _       => "",
        };

        c.to_owned()
    }
}

impl<'c> Display for Codegen<'c> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.generate())
    }
}

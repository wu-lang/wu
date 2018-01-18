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
            Definition { ref left, ref right, .. } => match *right {
                Some(ref right) => match right.0 {
                    ref block @ ExpressionNode::Block(_) => {
                        format!("local {}\n{}", self.gen_expression(&left.0), self.gen_block_assignment(&block, &left.0))
                        
                    },
                    _ => format!("local {} = {}", self.gen_expression(&left.0), self.gen_expression(&right.0)) 
                }
                None            => format!("local {}", self.gen_expression(&left.0)),
            },
            
            ConstDefinition { ref left, ref right, .. } => format!("local {} = {}", self.gen_expression(&left.0), self.gen_expression(&right.0)),
            Assignment { ref left, ref right, .. }      => format!("{} = {}", self.gen_expression(&left.0), self.gen_expression(&right.0)),
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

                match body.0 {
                    Block(_) => (),
                    _ => code.push_str("return "),
                }

                code.push_str(&self.gen_expression(&body.0));

                code.push_str("\nend");

                code
            }

            EOF => String::new(),
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

    fn gen_block_assignment(&self, block: &ExpressionNode, left: &ExpressionNode) -> String {
        use ExpressionNode::*;

        if let Block(ref statements) = *block {
            let mut code = "do\n".to_string();
            
            let mut acc = 1;

            for statement in statements {
                if acc == statements.len() {
                    code.push_str(&format!("{} = {}", self.gen_expression(left), self.gen_statement(&statement.0)))
                } else {
                    code.push_str(&self.gen_statement(&statement.0))
                }

                acc += 1
            }

            code.push_str("\nend");

            code
        } else {
            String::new()
        }
    }
}

impl<'c> Display for Codegen<'c> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.generate())
    }
}

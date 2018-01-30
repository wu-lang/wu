use std::rc::Rc;
use super::*;

use std::path::Path;

pub struct Codegen<'c> {
    pub ast:     &'c [Statement],
    pub visitor: &'c mut Visitor<'c>,
}

impl<'c> Codegen<'c> {
    pub fn new(ast: &'c [Statement], visitor: &'c mut Visitor<'c>)-> Self {
        Codegen {
            ast,
            visitor,
        }
    }

    pub fn generate(&mut self) -> String {
        let mut code = String::new();
        
        let ast_module = StatementNode::Module {
            name:    "__mod__".to_string(),
            content: Some(
                    Expression::new(
                    ExpressionNode::Block(self.ast.clone().to_vec()),
                    TokenPosition::default(),
                )
            )
        };

        code.push_str(&format!("{}\n", self.gen_statement_local(&ast_module)));
        code.push_str("return __mod__");

        code
    }

    fn gen_statement_local(&mut self, statement: &StatementNode) -> String {
        use self::StatementNode::*;

        match *statement {
            Definition { ref left, ref right, .. } => {
                match *right {
                    Some(ref right) => match right.0 {
                        ref block @ ExpressionNode::Block(_) => {
                            if let ExpressionNode::Identifier(_) = left.0 {
                                format!("local {}\n{}\n", self.gen_expression(&left.0), self.gen_block_assignment(block, &left.0))
                            } else {
                                format!("{}\n", self.gen_block_assignment(block, &left.0))
                            }
                        },
                        _ => format!("local {} = {}\n", self.gen_expression(&left.0), self.gen_expression(&right.0))
                    }

                    None => if let ExpressionNode::Identifier(_) = left.0 {
                        format!("local {}\n", self.gen_expression(&left.0))
                    } else {
                        String::new()
                    }
                }
            },

            ConstDefinition { ref left, ref right, .. } => match right.0 {
                ref block @ ExpressionNode::Block(_) => {
                    if let ExpressionNode::Identifier(_) = left.0 {
                        format!("local {}\n{}\n", self.gen_expression(&left.0), self.gen_block_assignment(block, &left.0))
                    } else {
                        format!("{}\n", self.gen_block_assignment(&block, &left.0))
                    }
                },
                _ => format!("local {} = {}\n", self.gen_expression(&left.0), self.gen_expression(&right.0))
            },

            Module { ref name, ref content } => {
                if let Some(ref content) = *content {
                    let mut code = format!("local {} = (function()\n", name);

                    let mut returns = Vec::new();

                    if let super::Expression(ExpressionNode::Block(ref content), _) = *content {
                        for statement in content {
                            match statement.0 {
                                Definition { ref left, .. } | ConstDefinition { ref left, .. } => if let ExpressionNode::Identifier(ref name) = left.0 {
                                    returns.push(name)
                                },

                                Struct { ref name, .. } | Module { ref name, .. } => returns.push(name),

                                _ => (),
                            }

                            code.push_str(&format!("{}\n", self.gen_statement_local(&statement.0)))
                        }
                    }

                    code.push_str("return {\n");

                    for ret in returns {
                        code.push_str(&format!("{0} = {0},\n", ret))
                    }

                    code.push_str("}\nend)()");
                    code
                } else {
                    format!("local {} = require('{}')", name, format!("{}/{}", Path::new(self.visitor.path).parent().unwrap().display(), name))
                }
            },

            Expose { ref origin, ref expose } => {
                let mut code = String::new();

                if let Some(ref expose) = *expose {
                    if expose.contains(&"*".to_string()) {

                        if let TypeNode::Module(ref members) = self.visitor.type_expression(origin).unwrap().0 {
                            for name in members.keys() {
                                code.push_str(&format!("local {0} = {1}.{0}\n", name, self.gen_expression(&origin.0)))
                            }
                        }
                    } else {
                        for exposed in expose {
                            code.push_str(&format!("local {0} = {1}.{0}\n", exposed, self.gen_expression(&origin.0)))
                        }
                    }
                } else {
                    code.push_str(&format!("local {0} = {0}\n", self.gen_expression(&origin.0)))
                }

                code
            },

            Struct {ref name, ref members} => {
                let mut code = format!("local {} = {{\n", name);

                code.push_str("__construct__ = function(__constructor)\n");

                code.push_str("return {\n");

                for &(ref name, _) in members {
                    code.push_str(&format!("{0} = __constructor.{0},\n", name))
                }

                code.push('}');

                code.push_str("\nend\n}");
                code
            },

            Extern(_) => String::new(),

            ref other => self.gen_statement(other),
        }
    }

    fn gen_statement(&mut self, statement: &StatementNode) -> String {
        use self::StatementNode::*;

        match *statement {
            Expression(ref expression) => format!("{}\n", self.gen_expression(&expression.0)),
            Return(ref value)          => format!("return{}\n", match *value {
                Some(ref v) => format!(" {}", self.gen_expression(&v.0)),
                None        => String::from("\n"),
            }),
            Definition { ref left, ref right, .. } => {
                match *right {
                    Some(ref right) => match right.0 {
                        ref block @ ExpressionNode::Block(_) => {
                            if let ExpressionNode::Identifier(_) = left.0 {
                                format!("{}\n{}\n", self.gen_expression(&left.0), self.gen_block_assignment(block, &left.0))
                            } else {
                                format!("{}\n", self.gen_block_assignment(block, &left.0))
                            }
                        },
                        _ => format!("{} = {}\n", self.gen_expression(&left.0), self.gen_expression(&right.0))
                    }

                    None => if let ExpressionNode::Identifier(_) = left.0 {
                        format!("{}\n", self.gen_expression(&left.0))
                    } else {
                        String::new()
                    }
                }
            },

            While { ref condition, ref body } => format!("while {} do\n{}\nend", self.gen_expression(&condition.0), self.gen_expression(&body.0)),

            Expose { ref origin, ref expose } => {
                let mut code = String::new();

                if let Some(ref expose) = *expose {
                    for exposed in expose {
                        code.push_str(&format!("{0} = {1}.{0}", exposed, self.gen_expression(&origin.0)))
                    }
                } else {
                    code.push_str(&format!("{0} = {0}", self.gen_expression(&origin.0)))
                }

                code
            },

            ConstDefinition { ref left, ref right, .. } => match right.0 {
                ref block @ ExpressionNode::Block(_) => {
                    if let ExpressionNode::Identifier(_) = left.0 {
                        format!("{}\n{}\n", self.gen_expression(&left.0), self.gen_block_assignment(block, &left.0))
                    } else {
                        format!("{}\n", self.gen_block_assignment(block, &left.0))
                    }
                },
                _ => format!("{} = {}\n", self.gen_expression(&left.0), self.gen_expression(&right.0))
            },

            Assignment { ref left, ref right, .. } => format!("{} = {}", self.gen_expression(&left.0), self.gen_expression(&right.0)),
            If(ref if_node) => self.gen_if_node(if_node),

            Struct {ref name, ref members} => {
                let mut code = format!("{} = {{\n", name);

                code.push_str("__construct__ = function(__constructor)\n");

                code.push_str("return {\n");

                for &(ref name, _) in members {
                    code.push_str(&format!("{0} = __constructor.{0},\n", name))
                }

                code.push('}');

                code.push_str("\nend\n}");
                code
            },

            _ => self.gen_statement_local(statement),
        }
    }

    fn gen_if_node(&mut self, if_node: &IfNode) -> String {
        let mut code = format!("if {} then\n{}", self.gen_expression(&if_node.condition.0), self.gen_expression(&if_node.body.0));

        if let Some(ref cases) = if_node.elses {
            for case in cases {
                let case_code = match *case {
                    (Some(ref condition), ref body, _) => format!("elseif {} then\n{}", self.gen_expression(&condition.0), self.gen_expression(&body.0)),
                    (None,                ref body, _) => format!("else\n{}", self.gen_expression(&body.0)),
                };

                code.push_str(&case_code)
            }

            code.push_str("end\n")
        } else {
            code.push_str("end\n");
        }

        code
    }

    fn gen_if_node_return(&mut self, if_node: &IfNode) -> String {
        let mut code = format!("if {} then\n{}", self.gen_expression(&if_node.condition.0), self.gen_block_return(&if_node.body.0));

        if let Some(ref cases) = if_node.elses {
            for case in cases {
                let case_code = match *case {
                    (Some(ref condition), ref body, _) => format!("elseif {} then\n{}\n", self.gen_expression(&condition.0), self.gen_block_return(&body.0)),
                    (None,                ref body, _) => format!("else\n{}\n", self.gen_block_return(&body.0)),
                };

                code.push_str(&case_code)
            }

            code.push_str("end\n")
        } else {
            code.push_str("end\n");
        }

        code
    }

    fn gen_if_node_assignment(&mut self, if_node: &IfNode, left: &ExpressionNode) -> String {
        let mut code = format!("if {} then\n{}", self.gen_expression(&if_node.condition.0), self.gen_block_assignment(&if_node.body.0, left));

        if let Some(ref cases) = if_node.elses {
            for case in cases {
                let case_code = match *case {
                    (Some(ref condition), ref body, _) => format!("elseif {} then\n{}\n", self.gen_expression(&condition.0), self.gen_block_assignment(&body.0, left)),
                    (None,                ref body, _) => format!("else\n{}\n", self.gen_block_assignment(&body.0, left)),
                };

                code.push_str(&case_code)
            }

            code.push_str("end\n")
        } else {
            code.push_str("end\n");
        }

        code
    }

    fn gen_statement_return(&mut self, statement: &StatementNode) -> String {
        use self::StatementNode::*;

        match *statement {
            If(ref if_node) => self.gen_if_node_return(if_node),
            Expression(ref expression) => match expression.0 {
                ExpressionNode::Binary {ref op, ..} => match *op {
                    Operator::Compound(_) => format!("{}\n", self.gen_statement_local(statement)),
                    _                     => format!("return {}", self.gen_statement_local(statement))
                }

                _ => format!("return {}", self.gen_statement_local(statement))
            }
            _ => format!("{}\n", self.gen_statement_local(statement)),
        }
    }

    fn gen_statement_assignment(&mut self, statement: &StatementNode, left: &ExpressionNode) -> String {
        use self::StatementNode::*;

        match *statement {
            If(ref if_node) => self.gen_if_node_assignment(if_node, left),
            Return(_)       => self.gen_statement_local(statement),
            Expression(ref expression) => match expression.0 {
                ExpressionNode::Binary {ref op, ..} => match *op {
                    Operator::Compound(_) => format!("{}\n", self.gen_statement_local(statement)),
                    _                     => format!("{} = {}\n", self.gen_expression(left), self.gen_statement_local(statement)),
                }

                _ => format!("{} = {}\n", self.gen_expression(left), self.gen_statement_local(statement)),
            }

            _ => format!("{} = {}\n", self.gen_expression(left), self.gen_statement_local(statement)),
        }
    }

    fn gen_expression(&mut self, expression: &ExpressionNode) -> String {
        use self::ExpressionNode::*;

        match *expression {
            Float(ref n)      => format!("{}", n),
            Int(ref n)        => format!("{}", n),
            Str(ref n)        => format!("{:?}", n),
            Bool(ref n)       => format!("{}", n),
            Identifier(ref n) => format!("{}", n),

            Unary(ref op, ref expression) => format!("{}({})", self.gen_operator(op), self.gen_expression(&expression.0)),

            Index(ref indexed, ref index) => {
                let right = match index.0 {
                    Block(_) => format!("(function()\n{}end)()", self.gen_block_return(&index.0)),
                    _        => self.gen_expression(&index.0),
                };

                format!("{}[{}]", self.gen_expression(&indexed.0), right)
            },

            Constructor(ref name, ref members) => {
                let mut code = format!("{}.__construct__({{\n", self.gen_expression(&name.0));

                for member in members {
                    code.push_str(&format!("{} = {},\n", member.0, self.gen_expression(&(member.1).0)))
                }

                code.push_str("})\n");
                code
            },

            Array(ref content) => {
                let mut code = String::new();

                code.push_str("{\n");

                for (acc, element) in content.iter().enumerate() {
                    let right = match element.0 {
                        Block(_) => format!("(function()\n{}end)()", self.gen_block_return(&element.0)),
                        _        => self.gen_expression(&element.0),
                    };

                    code.push_str(&format!("[{}] = {}", acc, right));
                    code.push_str(",\n");
                }

                code.push('}');

                code
            }

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
                if statements.len() == 1 {
                    format!("{}", self.gen_statement_local(&statements.last().unwrap().0))
                } else {
                    let mut code = "do\n".to_string();

                    for statement in statements {
                        code.push_str(&self.gen_statement_local(&statement.0))
                    }

                    code.push_str("end\n");

                    code
                }
            }

            Function { ref params, ref body, .. } => {
                let mut code   = "(function(".to_string();
                let mut acc    = 1;
                let mut guards = Vec::new();

                for param in params {
                    if let Some(ref value) = param.2 {
                        guards.push((param.0.clone(), Rc::clone(value)))
                    }

                    code.push_str(&param.0);

                    if acc != params.len() {
                        code.push(',');
                    }

                    acc += 1
                }

                code.push_str(")\n");

                for guard in guards {
                    let definition = StatementNode::ConstDefinition {
                        left:  super::Expression(Identifier(format!("{}", guard.0)), (guard.1).1),
                        right: (*guard.1).clone(),
                        kind:  TypeNode::Nil,
                    };

                    code.push_str(&self.gen_statement_local(&definition));
                    code.push_str(&format!("{0} = {0} and {0} or optional_{0}\n", guard.0));
                }

                match body.0 {
                    Block(_) => code.push_str(&self.gen_block_return(&body.0)),
                    _        => code.push_str(&format!("return {}\n", self.gen_expression(&body.0))),
                }

                code.push_str("end)");

                code
            }

            _ => String::new(),
        }
    }

    fn gen_operation(&mut self, left: &ExpressionNode, op: &Operator, right: &ExpressionNode) -> String {
        use self::Operator::*;
        use self::ExpressionNode::*;

        match *op {
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

            Compound(ref op) => {
                let compiled_left  = self.gen_expression(left);
                let compiled_right = self.gen_expression(right);

                format!("{0} = {0}{1}{2}", compiled_left, self.gen_operator(&*op), compiled_right)
            },

            _ => {
                let compiled_left  = self.gen_expression(left);
                let compiled_op    = self.gen_operator(op);
                let compiled_right = self.gen_expression(right);

                match *right {
                    Int(_)        |
                    Float(_)      |
                    Str(_)        |
                    Bool(_)       |
                    Identifier(_) |
                    Call(..)      => format!("{}{}{}", compiled_left, compiled_op, compiled_right),
                    _             => format!("{}{}({})", compiled_left, compiled_op, compiled_right),
                }
            }
        }
    }

    fn gen_operator(&self, op: &Operator) -> String {
        use self::Operator::*;

        match *op {
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
        }.to_owned()
    }

    fn gen_block_assignment(&mut self, block: &ExpressionNode, left: &ExpressionNode) -> String {
        use self::ExpressionNode::*;

        if let Block(ref statements) = *block {
            if statements.len() == 1 {
                self.gen_statement_assignment(&statements.last().unwrap().0, left)
            } else {
                let mut code = "do\n".to_string();

                let mut acc = 1;

                for statement in statements {
                    if acc == statements.len() {
                        code.push_str(&self.gen_statement_assignment(&statement.0, left))
                    } else {
                        code.push_str(&format!("{}\n", self.gen_statement_local(&statement.0)));
                    }

                    acc += 1
                }

                code.push_str("end\n");

                code
            }
        } else {
            String::new()
        }
    }

    fn gen_block_return(&mut self, block: &ExpressionNode) -> String {
        use self::ExpressionNode::*;

        if let Block(ref statements) = *block {
            if statements.len() == 1 {
                self.gen_statement_return(&statements.last().unwrap().0)
            } else {
                let mut code = "do\n".to_string();

                let mut acc = 1;

                for statement in statements {
                    if acc == statements.len() {
                        code.push_str(&self.gen_statement_return(&statement.0))
                    } else {
                        code.push_str(&format!("{}\n", self.gen_statement_local(&statement.0)));
                    }

                    acc += 1
                }

                code.push_str("end\n");

                code
            }
        } else {
            String::new()
        }
    }
}

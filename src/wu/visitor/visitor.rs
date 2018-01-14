use super::*;

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Nil,
    Id(String),
}

impl PartialEq for Type {
    fn eq(&self, other: &Type) -> bool {
        use Type::*;

        match (self, other) {
            (&Float, &Int)           => true,
            (&Float, &Float)         => true,
            (&Int, &Int)             => true,
            (&Bool, &Bool)           => true,
            (&Str, &Str)             => true,
            (&Nil, &Nil)             => true,
            (&Id(ref a), &Id(ref b)) => a == b,
            _                        => false,
        }
    }
}

pub struct Visitor<'v> {
    pub ast:    &'v Vec<Statement>,
    pub symtab: SymTab,

    pub lines: &'v Vec<String>,
    pub path:  &'v str,
}

impl<'v> Visitor<'v> {
    pub fn new(ast: &'v Vec<Statement>, lines: &'v Vec<String>, path: &'v str) -> Self {
        Visitor {
            ast,
            symtab: SymTab::new_global(),
            lines,
            path
        }
    }

    pub fn validate(&self) -> Response<()> {
        for statement in self.ast.iter() {
            self.visit_statement(statement)?
        }
        
        Ok(())
    }

    fn visit_statement(&self, statement: &Statement) -> Response<()> {
        use StatementNode::*;

        match (&statement.0, statement.1) {
            (&Expression(ref expr), _)                       => self.visit_expression(expr),
            (&Definition {ref kind, ref left, ref right}, _) => self.visit_definition(kind, left, right),
            (&ConstDefinition {ref left, ref right}, _)      => self.visit_definition(&None, left, &Some(right.clone())),
            _                                                => Ok(()),
        }
    }

    fn visit_expression(&self, expression: &Expression) -> Response<()> {
        use ExpressionNode::*;

        match (&expression.0, expression.1) {
            (&Identifier(ref name), position) => match self.symtab.get_name(&*name) {
                Some(_) => Ok(()),
                None    => Err(make_error(Some(position), format!("undefined '{}'", name)))
            },

            (&Binary { .. }, _) => match self.type_expression(&expression) {
                Ok(_)    => Ok(()),
                Err(err) => Err(err),
            }
            _ => Ok(())
        }
    }

    fn type_expression(&self, expression: &Expression) -> Response<Type> {
        use ExpressionNode::*;

        let t = match (&expression.0, expression.1) {
            (&Int(_), _)    => Type::Int,
            (&Float(_), _)  => Type::Float,
            (&Bool(_), _)   => Type::Bool,
            (&Str(_), _)    => Type::Str,
            (&Binary { ref left, ref op, ref right }, position) => {
                use Operator::*;

                match (self.type_expression(&*left)?, op, self.type_expression(&*right)?) {
                    (a, &Add, b) => match (a, b) {
                        (Type::Int, Type::Int)     => Type::Int,
                        (Type::Float, Type::Float) => Type::Float,
                        (Type::Float, Type::Int)   => Type::Float,
                        (a, b)                     => return Err(make_error(Some(position), format!("can't add {:?} and {:?}", a, b)))
                    },

                    (a, &Sub, b) => match (a, b) {
                        (Type::Int, Type::Int)     => Type::Int,
                        (Type::Float, Type::Float) => Type::Float,
                        (Type::Float, Type::Int)   => Type::Float,
                        (a, b)                       => return Err(make_error(Some(position), format!("can't subtract {:?} and {:?}", a, b)))
                    },

                    (a, &Mul, b) => match (a, b) {
                        (Type::Int, Type::Int)     => Type::Int,
                        (Type::Float, Type::Float) => Type::Float,
                        (Type::Float, Type::Int)   => Type::Float,
                        (a, b)                       => return Err(make_error(Some(position), format!("can't multiply {:?} and {:?}", a, b)))
                    },

                    _ => Type::Nil,
                }
            },
            _ => Type::Nil,
        };

        Ok(t)
    }

    fn visit_definition(&self, kind: &Option<Type>, left: &Expression, right: &Option<Expression>) -> Response<()> {
        use ExpressionNode::*;

        match left.0 {
            Identifier(ref name) => { self.symtab.add_name(&name); },
            _                    => return Err(make_error(Some(left.1), format!("can't assign anything but identifiers"))),
        }
    
        if let Some(ref right) = *right {
            self.visit_expression(&right)?;

            if let Some(ref kind) = *kind {
                let right_kind = self.type_expression(&right)?;
                if *kind != right_kind {
                    return Err(make_error(Some(left.1), format!("mismatched types: expected '{:?}', found '{:?}'", kind, right_kind)))
                }
            }
        }
        Ok(())
    }
}

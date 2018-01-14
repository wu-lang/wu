use super::*;

use std::fmt::*;

#[derive(Debug, Clone)]
pub enum TypeNode {
    Int,
    Float,
    Bool,
    Str,
    Nil,
    Id(String),
}

impl PartialEq for TypeNode {
    fn eq(&self, other: &TypeNode) -> bool {
        use TypeNode::*;

        match (self, other) {
            (&Float,     &Int)       => true,
            (&Float,     &Float)     => true,
            (&Int,       &Int)       => true,
            (&Bool,      &Bool)      => true,
            (&Str,       &Str)       => true,
            (&Nil,       &Nil)       => true,
            (&Id(ref a), &Id(ref b)) => a == b,
            _                        => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeMode {
    Undeclared,
    Constant,
    Just,
}

impl PartialEq for TypeMode {
    fn eq(&self, other: &TypeMode) -> bool {
        use TypeMode::*;
        
        match (self, other) {
            (&Just,       &Just)       => true,
            (&Just,       &Constant)   => true,
            (&Constant,   &Constant)   => true,
            (&Constant,   &Just)       => true,
            (&Undeclared, _)           => false,
            (_,           &Undeclared) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type(TypeNode, TypeMode);

impl Type {
    pub fn new(node: TypeNode, mode: TypeMode) -> Type {
        Type(node, mode)
    }
    
    pub fn int() -> Type {
        Type(TypeNode::Int, TypeMode::Just)
    }
    
    pub fn string() -> Type {
        Type(TypeNode::Str, TypeMode::Just)
    }
    
    pub fn float() -> Type {
        Type(TypeNode::Float, TypeMode::Just)
    }
    
    pub fn boolean() -> Type {
        Type(TypeNode::Bool, TypeMode::Just)
    }

    pub fn nil() -> Type {
        Type(TypeNode::Nil, TypeMode::Just)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use TypeNode::*;
        
        match self.0 {
            Int   => write!(f, "int"),
            Float => write!(f, "float"),
            Bool  => write!(f, "boolean"),
            Str   => write!(f, "string"),
            Nil   => write!(f, "nil"),
            Id(ref a) => write!(f, "{}", a),
        }
    }
}

pub struct Visitor<'v> {
    pub ast:     &'v Vec<Statement>,
    pub symtab:  SymTab,
    pub typetab: TypeTab,

    pub lines: &'v Vec<String>,
    pub path:  &'v str,
}

impl<'v> Visitor<'v> {
    pub fn new(ast: &'v Vec<Statement>, lines: &'v Vec<String>, path: &'v str) -> Self {
        Visitor {
            ast,
            symtab:  SymTab::global(),
            typetab: TypeTab::global(),
            lines,
            path
        }
    }

    pub fn validate(&mut self) -> Response<()> {
        for statement in self.ast.iter() {
            self.visit_statement(statement)?
        }
        
        Ok(())
    }

    fn visit_statement(&mut self, statement: &Statement) -> Response<()> {
        use StatementNode::*;

        match (&statement.0, statement.1) {
            (&Expression(ref expr), _)                       => self.visit_expression(expr),
            (&Definition {ref kind, ref left, ref right}, _) => self.visit_definition(kind, left, right),
            (&ConstDefinition {ref left, ref right}, _)      => self.visit_constant(left, right),
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
            (&Int(_), _)   => Type::int(),
            (&Float(_), _) => Type::float(),
            (&Bool(_), _)  => Type::boolean(),
            (&Str(_), _)   => Type::string(),
            (&Identifier(ref name), position) => match self.symtab.get_name(&*name) {
                Some((i, env_index)) => self.typetab.get_type(i, env_index)?,
                None                 => return Err(make_error(Some(position), format!("undefined: {}", name)))
            },

            (&Binary { ref left, ref op, ref right }, position) => {
                use Operator::*;

                match (self.type_expression(&*left)?, op, self.type_expression(&*right)?) {
                    (a, &Add, b) => if a == b {
                        a
                    } else {
                        return Err(make_error(Some(position), format!("can't add {} and {}", a, b)))
                    },

                    (a, &Sub, b) => if a == b {
                        a
                    } else {
                        return Err(make_error(Some(position), format!("can't subtract {} and {}", a, b)))
                    },

                    (a, &Mul, b) => if a == b {
                        a
                    } else {
                        return Err(make_error(Some(position), format!("can't multiply {} and {}", a, b)))
                    }

                    _ => Type::nil(),
                }
            },

            _ => Type::nil(),
        };

        Ok(t)
    }

    fn visit_definition(&self, kind: &TypeNode, left: &Expression, right: &Option<Expression>) -> Response<()> {
        use ExpressionNode::*;

        match left.0 {
            Identifier(ref name) => { self.symtab.add_name(&name); },
            _                    => return Err(make_error(Some(left.1), format!("can't define anything but identifiers"))),
        }

        if let Some(ref right) = *right {
            self.visit_expression(&right)?;

            if *kind != TypeNode::Nil {
                let right_kind = self.type_expression(&right)?;
                if *kind != right_kind.0 {
                    return Err(make_error(Some(left.1), format!("mismatched types: expected '{:?}', found '{:?}'", kind, right_kind)))
                }
            }
        }
        Ok(())
    }

    fn visit_constant(&mut self, left: &Expression, right: &Expression) -> Response<()> {
        use ExpressionNode::*;
        
        let index = match left.0 {
            Identifier(ref name) => {
                self.typetab.grow();
                self.symtab.add_name(&name)
            },
            _                    => return Err(make_error(Some(left.1), format!("can't define anything but identifiers"))),
        };

        self.visit_expression(right)?;

        self.typetab.set_type(index, 0, Type::new(self.type_expression(right)?.0, TypeMode::Constant))
    }
}

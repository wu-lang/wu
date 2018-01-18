use super::lexer::*;
use super::*;

use std::fmt::*;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum TypeNode {
    Int,
    Float,
    Bool,
    Str,
    Nil,
    Id(String),
    Fun(Vec<Rc<Type>>, Rc<Type>),
}

// this is for typechecking
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
            (&Fun(ref a_params, ref a_retty), &Fun(ref b_params, ref b_retty)) => a_params == b_params && a_retty == b_retty,
            (&Id(ref a), &Id(ref b)) => a == b,
            _                        => false,
        }
    }
}

impl Display for TypeNode {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use TypeNode::*;

        match *self {
            Int   => write!(f, "int"),
            Float => write!(f, "float"),
            Bool  => write!(f, "boolean"),
            Str   => write!(f, "string"),
            Nil   => write!(f, "nil"),
            Fun(ref params, ref return_type) => {
                write!(f, "(")?;
                
                let mut acc = 1;

                for param in params {
                    if acc == params.len() {
                        write!(f, "{}", param)?;
                    } else {
                        write!(f, "{}, ", param)?;
                    }

                    acc += 1
                }

                write!(f, ") {}", return_type)
            },
            Id(ref a) => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeMode {
    Undeclared,
    Constant,
    Just,
}

impl TypeMode {
    // this is for actual, reliable checking
    pub fn check(&self, other: &TypeMode) -> bool {
        use TypeMode::*;

        match (self, other) {
            (&Just,       &Just)       => true,
            (&Constant,   &Constant)   => true,
            (&Undeclared, &Undeclared) => true,
            _ => false,
        }
    }
}

impl Display for TypeMode {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use TypeMode::*;

        match *self {
            Just       => Ok(()),
            Constant   => write!(f, "constant "),
            Undeclared => write!(f, "undeclared "),
        }
    }
}

// this is for typechecking
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
        write!(f, "{}{}", self.1, self.0)
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

    pub fn from(ast: &'v Vec<Statement>, symtab: SymTab, typetab: TypeTab, lines: &'v Vec<String>, path: &'v str) -> Self {
        Visitor {
            ast,
            symtab,
            typetab,
            lines,
            path,
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
            (&Assignment {ref left, ref right}, _)           => self.visit_assignment(left, right),
            (&Return(ref value), _)                          => if let Some(ref value) = *value {
                self.visit_expression(value)
            } else {
                Ok(())
            },
        }
    }

    fn visit_expression(&mut self, expression: &Expression) -> Response<()> {
        use ExpressionNode::*;

        match (&expression.0, expression.1) {
            (&Identifier(ref name), position) => match self.symtab.get_name(&*name) {
                Some(_) => Ok(()),
                None    => Err(make_error(Some(position), format!("undefined '{}'", name)))
            },

            (&Binary { .. }, _) => match self.type_expression(&expression) {
                Ok(_)    => Ok(()),
                Err(err) => Err(err),
            },

            (&Function {ref params, ref return_type, ref body}, position) => self.visit_function(&position, params, return_type, body),

            (&Call(ref callee, ref args), _) => self.visit_call(&callee, &args),

            (&Block(ref statements), _) => {
                for statement in statements {
                    self.visit_statement(&statement)?
                }

                Ok(())
            }

            _ => Ok(())
        }
    }

    fn visit_function(&mut self, position: &TokenPosition, params: &Vec<(String, TypeNode)>, return_type: &TypeNode, body: &Expression) -> Response<()> {
        let mut param_names = Vec::new();
        let mut param_types = Vec::new();

        for &(ref name, ref t) in params.iter() {
            param_names.push(name.clone());
            param_types.push(Type::new(t.clone(), TypeMode::Just))
        }

        let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &param_names.as_slice());
        let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &param_types, &HashMap::new());

        let mut visitor = Visitor::from(self.ast, local_symtab, local_typetab, self.lines, self.path);

        visitor.visit_expression(body)?;

        if Type::new(return_type.clone(), TypeMode::Just) == visitor.type_expression(body)? {
            Ok(())
        } else {
            Err(make_error(Some(body.1), format!("mismatched return type, expected '{}'", return_type)))
        }
    }
    
    fn visit_call(&mut self, callee: &Rc<Expression>, args: &Vec<Rc<Expression>>) -> Response<()> {
        let callee_t = self.type_expression(&**callee)?;

        if callee_t.1 == TypeMode::Undeclared {
            Err(make_error(Some(callee.1), format!("don't call an undeclared: '{}'", callee_t)))
        } else {
            match callee_t.0 {
                TypeNode::Fun(ref params, _) => {
                    let mut acc = 0;

                    for param in params {
                        if self.type_expression(&args[acc])? != **param {
                            return Err(make_error(Some(args[acc].1), format!("mismatching argument type: '{}', expected: '{}'", self.type_expression(&args[acc])?, param)))
                        }
                        acc += 1
                    }
                    Ok(())
                },

                ref t => Err(make_error(Some(callee.1), format!("can't call: '{}'", t))),
            }
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
            
            (&Function {ref params, ref return_type, ..}, _) => {
                Type::new(TypeNode::Fun(
                    params.iter().map(|x| Rc::new(Type::new(x.1.clone(), TypeMode::Just))).collect::<Vec<Rc<Type>>>(),
                    Rc::new(Type::new(return_type.clone(), TypeMode::Just)),
                ), TypeMode::Just)
            },

            (&Call(ref callee, _), _) => match self.type_expression(&**callee)?.0 {
                TypeNode::Fun(_, ref retty) => (**retty).clone(),
                _ => unreachable!(),
            },

            (&Binary { ref left, ref op, ref right }, position) => {
                use Operator::*;
                
                match (self.type_expression(&*left)?, op, self.type_expression(&*right)?) {
                    (a, &Add, b) => if a == b {
                        Type::new(a.0, TypeMode::Just)
                    } else {
                        return Err(make_error(Some(position), format!("can't add '{}' and '{}'", a, b)))
                    },

                    (a, &Sub, b) => if a == b {
                        Type::new(a.0, TypeMode::Just)
                    } else {
                        return Err(make_error(Some(position), format!("can't subtract '{}' and '{}'", a, b)))
                    },

                    (a, &Mul, b) => if a == b {
                        Type::new(a.0, TypeMode::Just)
                    } else {
                        return Err(make_error(Some(position), format!("can't multiply '{}' and '{}'", a, b)))
                    }

                    _ => Type::nil(),
                }
            },

            (&Block(ref statements), _) => {
                use StatementNode::*;

                match statements.last() {
                    Some(last) => match last.0 {
                        Expression(ref expression) => self.type_expression(expression)?,
                        Return(ref value)          => if let Some(ref value) = *value {
                            self.type_expression(value)?
                        } else {
                            Type::nil()
                        },

                        _                          => Type::nil(),
                    },

                    None => Type::nil(),
                }
            },

            _ => Type::nil(),
        };

        Ok(t)
    }
    
    fn visit_definition(&mut self, kind: &TypeNode, left: &Expression, right: &Option<Expression>) -> Response<()> {
        use ExpressionNode::*;

        let index = match left.0 {
            Identifier(ref name) => {
                self.typetab.grow();
                self.symtab.add_name(&name)
            },
            _ => return Err(make_error(Some(left.1), format!("can't define anything but identifiers"))),
        };

        if let Some(ref right) = *right {
            self.visit_expression(&right)?;

            if *kind != TypeNode::Nil {
                let right_kind = self.type_expression(&right)?;
                if *kind != right_kind.0 {
                    return Err(make_error(Some(right.1), format!("mismatched types: expected '{}', found '{}'", kind, right_kind)))
                } else {
                    self.typetab.set_type(index, 0, self.type_expression(right)?)?;
                }
            } else {
                self.typetab.set_type(index, 0, self.type_expression(right)?)?;
            }            
        } else {
            self.typetab.set_type(index, 0, Type::new(kind.clone(), TypeMode::Undeclared))?;
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
            _ => return Err(make_error(Some(left.1), format!("can't define anything but identifiers"))),
        };

        self.typetab.set_type(index, 0, Type::new(self.type_expression(right)?.0, TypeMode::Constant))?;

        self.visit_expression(right)
    }

    fn visit_assignment(&mut self, left: &Expression, right: &Expression) -> Response<()> {
        self.visit_expression(left)?;
        self.visit_expression(right)?;

        let left_type  = self.type_expression(left)?;
        let right_type = self.type_expression(right)?;

        if left_type.1.check(&TypeMode::Constant) {
            Err(make_error(Some(left.1), format!("can't reassign constant")))
        } else {
            if left_type != right_type {
                Err(make_error(Some(right.1), format!("mismatched types: expected '{}', found '{}'", left_type, right_type)))
            } else {
                Ok(())
            }
        }
    }
}
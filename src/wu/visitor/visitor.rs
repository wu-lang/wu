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
            Bool  => write!(f, "bool"),
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
    Optional,
}

impl TypeMode {
    // this is for actual, reliable checking
    pub fn check(&self, other: &TypeMode) -> bool {
        use TypeMode::*;

        match (self, other) {
            (&Just,       &Just)       => true,
            (&Constant,   &Constant)   => true,
            (&Undeclared, &Undeclared) => true,
            (&Optional,   &Optional)   => true,
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
            Optional   => write!(f, "optional "),
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
            (_,           &Optional)   => true,
            (&Optional,   _)           => true,
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
            (&Expression(ref expr), _)                            => self.visit_expression(expr),
            (&Definition {ref kind, ref left, ref right}, _)      => self.visit_definition(kind, left, right),
            (&ConstDefinition {ref kind, ref left, ref right}, _) => self.visit_constant(kind, left, right),
            (&Assignment {ref left, ref right}, _)                => self.visit_assignment(left, right),
            (&Return(ref value), _)                               => if let Some(ref value) = *value {
                self.visit_expression(value)
            } else {
                Ok(())
            },
            (&If(ref if_node), ref position) => self.visit_if(if_node, position),
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

            (&Function {ref params, ref return_type, ref body}, _) => self.visit_function(params, return_type, body),

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

    fn visit_function(&mut self, params: &Vec<(String, TypeNode, Option<Rc<Expression>>)>, return_type: &TypeNode, body: &Expression) -> Response<()> {
        let mut param_names = Vec::new();
        let mut param_types = Vec::new();

        for &(ref name, ref t, ref value) in params.iter() {
            param_names.push(name.clone());
            param_types.push(Type::new(t.clone(), TypeMode::Just));

            if let Some(ref value) = *value {
                let value_t = self.type_expression(value)?.0;
                if *t != value_t {
                    return Err(make_error(Some(value.1), format!("mismatched parameter type, expected '{}' .. found '{}'", t, value_t)))
                }
            }
        }

        let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &param_names.as_slice());
        let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &param_types, &HashMap::new());

        let mut visitor = Visitor::from(self.ast, local_symtab, local_typetab, self.lines, self.path);

        visitor.visit_expression(body)?;

        let body_t = visitor.type_expression(body)?;

        if Type::new(return_type.clone(), TypeMode::Just) != body_t {
            Err(make_error(Some(body.1), format!("mismatched return type, expected '{}' .. found '{}'", return_type, body_t)))
        } else {
            Ok(())
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

                    if params.len() != args.len() {
                        if params.len() < args.len() {
                            Err(make_error(Some(args[acc].1), format!("function expected {} arg{}, got {}", params.len(), if params.len() > 1 { "s" } else { "" }, args.len())))
                        } else {
                            for param in &params[args.len() .. params.len()] {
                                if !param.1.check(&TypeMode::Optional) {
                                    return Err(make_error(Some(callee.1), format!("can't ommit non-optional argument")))
                                }
                            }

                            Ok(())
                        }
                    } else {
                        for param in params {
                            if **param != self.type_expression(&args[acc])? {
                                return Err(make_error(Some(args[acc].1), format!("mismatched argument type: '{}', expected: '{}'", self.type_expression(&args[acc])?, param)))
                            }
                            acc += 1
                        }
                        Ok(())
                    }
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
                    params.iter().map(|x| Rc::new(Type::new(x.1.clone(), if let Some(_) = x.2 { TypeMode::Optional } else { TypeMode::Just }))).collect::<Vec<Rc<Type>>>(),
                    Rc::new(Type::new(return_type.clone(), TypeMode::Just)),
                ), TypeMode::Just)
            },

            (&Call(ref callee, _), _) => match self.type_expression(&**callee)?.0 {
                TypeNode::Fun(_, ref retty) => (**retty).clone(),
                _ => unreachable!(),
            },

            (&Binary { ref left, ref op, ref right }, position) => {
                use Operator::*;
                use TypeNode::*;

                match (self.type_expression(&*left)?, op, self.type_expression(&*right)?) {
                    (a, &Pow, b) => if vec![Float, Int].contains(&a.0) {
                        if a == b {
                            Type::new(a.0, TypeMode::Just)
                        } else {
                            return Err(make_error(Some(position), format!("can't pow '{}' and '{}'", a, b)))
                        }
                    } else {
                        return Err(make_error(Some(position), format!("can't pow '{}' and '{}'", a, b)))
                    },
                    
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

                    (_, &Equal, _)   => Type::new(Bool, TypeMode::Just),
                    (_, &NEqual, _)  => Type::new(Bool, TypeMode::Just),
                    (_, &Lt, _)      => Type::new(Bool, TypeMode::Just),
                    (_, &Gt, _)      => Type::new(Bool, TypeMode::Just),
                    (_, &LtEqual, _) => Type::new(Bool, TypeMode::Just),
                    (_, &GtEqual, _) => Type::new(Bool, TypeMode::Just),

                    _ => Type::nil(),
                }
            },

            (&Block(ref statements), _) => {
                let mut return_type = None;

                let mut acc = 1;

                for statement in statements {
                    if return_type == None {
                        if let Some(t) = self.find_return_type(&statement.0, acc == statements.len())? {
                            return_type = Some(t)
                        }
                    }

                    acc += 1
                }

                return_type.unwrap_or(Type::nil())
            },

            _ => Type::nil(),
        };

        Ok(t)
    }

    fn find_return_type(&self, statement: &StatementNode, is_last: bool) -> Response<Option<Type>> {
        use StatementNode::*;

        let return_type = match *statement {
            Expression(ref expression) => if is_last {
                Some(self.type_expression(expression)?)
            } else {
                None
            },

            Return(ref value)          => if let Some(ref value) = *value {
                Some(self.type_expression(value)?)
            } else {
                Some(Type::nil())
            },

            If(ref if_node) => Some(self.type_expression(&if_node.body)?),

            _ => Some(Type::nil()),
        };

        Ok(return_type)
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

    fn visit_constant(&mut self, kind: &TypeNode, left: &Expression, right: &Expression) -> Response<()> {
        use ExpressionNode::*;

        let index = match left.0 {
            Identifier(ref name) => {
                self.typetab.grow();
                self.symtab.add_name(&name)
            },
            _ => return Err(make_error(Some(left.1), format!("can't define anything but identifiers"))),
        };
        
        if *kind != TypeNode::Nil {
            let right_kind = self.type_expression(&right)?;

            if *kind != right_kind.0 {
                return Err(make_error(Some(right.1), format!("mismatched types: expected '{}', found '{}'", kind, right_kind)))
            } else {
                self.typetab.set_type(index, 0, Type::new(self.type_expression(right)?.0, TypeMode::Constant))?;
            }
        } else {
            self.typetab.set_type(index, 0, Type::new(self.type_expression(right)?.0, TypeMode::Constant))?;
        }

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

    fn visit_if(&mut self, if_node: &IfNode, position: &TokenPosition) -> Response<()> {
        self.visit_expression(&if_node.condition)?;

        if TypeNode::Bool != self.type_expression(&if_node.condition)?.0 {
            Err(make_error(Some(position.clone()), "non-boolean condition".to_owned()))
        } else {
            self.visit_expression(&if_node.body)?;
            Ok(())
        }
    }
}

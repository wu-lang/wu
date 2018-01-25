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
    Fun(Vec<Type>, Rc<Type>),
    Array(Rc<Type>),
    Struct(HashMap<String, TypeNode>),
    Module(HashMap<String, Type>),
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
            (&Array(ref a), &Array(ref b)) => a == b,
            (&Fun(ref a_params, ref a_retty), &Fun(ref b_params, ref b_retty)) => a_params == b_params && a_retty == b_retty,
            (&Id(ref a), &Id(ref b)) => a == b,
            (&Struct(ref a), &Struct(ref b)) => a == b,
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
            Struct(_) => write!(f, "struct"),
            Module(_) => write!(f, "module"),
            Array(ref content) => write!(f, "[{}]", content),
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
    Unconstructed,
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
            (&Unconstructed, &Unconstructed) => true,
            (&Optional,   &Optional)   => true,
            _ => false,
        }
    }
}

impl Display for TypeMode {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use TypeMode::*;

        match *self {
            Just          => Ok(()),
            Constant      => write!(f, "constant "),
            Undeclared    => write!(f, "undeclared "),
            Unconstructed => write!(f, "unconstructed "),
            Optional      => write!(f, "optional "),
        }
    }
}

// this is for typechecking
impl PartialEq for TypeMode {
    fn eq(&self, other: &TypeMode) -> bool {
        use TypeMode::*;

        match (self, other) {
            (&Just,       &Just)          => true,
            (&Just,       &Constant)      => true,
            (&Constant,   &Constant)      => true,
            (&Constant,   &Just)          => true,
            (_,           &Optional)      => true,
            (&Optional,   _)              => true,
            (&Undeclared, _)              => false,
            (&Unconstructed, _)           => false,
            (_,           &Undeclared)    => false,
            (_,           &Unconstructed) => false,
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

    pub fn dealias(&mut self, alias: &Type) -> Response<Type> {
        use TypeNode::*;

        let a = match alias.0 {
            Id(ref id)   => self.type_expression(&Expression::new(ExpressionNode::Identifier(id.clone()), TokenPosition::default())),
            Array(ref t) => Ok(Type::new(Array(Rc::new(self.dealias(&*t)?)), alias.1.clone())),
            _            => Ok(alias.clone()),
        };

        a
    }

    pub fn validate(&mut self) -> Response<()> {
        let mut responses = Vec::new();

        for statement in self.ast.iter() {
            match self.visit_statement(statement) {
                Err(response) => responses.push(response),
                Ok(_)         => (),
            }
        }

        if responses.len() > 0 {
            Err(ResponseNode { kind: ResponseType::Group(responses), position: None, message: "fix the errors above, or consequences".to_owned() } )
        } else {
            Ok(())
        }
    }

    fn visit_statement(&mut self, statement: &Statement) -> Response<()> {
        use StatementNode::*;

        match (&statement.0, statement.1) {
            (&Expression(ref expr), _)                            => self.visit_expression(expr),
            (&Definition {ref kind, ref left, ref right}, _)      => self.visit_definition(kind, left, right),
            (&ConstDefinition {ref kind, ref left, ref right}, _) => self.visit_constant(kind, left, right),
            (&Assignment {ref left, ref right}, _)                => self.visit_assignment(left, right),
            (&Struct {ref name, ref members}, position) => {
                if self.symtab.get_name(name).is_some() {
                    Err(make_error(Some(position), format!("struct '{}' defined multiple times", name)))
                } else {
                    self.typetab.grow();
                    let index = self.symtab.add_name(&name);

                    let mut hash_members = HashMap::new();

                    for &(ref name, ref member_type) in members.iter() {
                        hash_members.insert(name.clone(), member_type.clone());
                    }

                    self.typetab.set_type(index, 0, Type::new(TypeNode::Struct(hash_members), TypeMode::Unconstructed))
                }
            },

            (&Return(ref value), _) => if let Some(ref value) = *value {
                self.visit_expression(value)
            } else {
                Ok(())
            },

            (&If(ref if_node), ref position) => self.visit_if(if_node, position),
            (&While { ref condition, ref body }, position) => {
                let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                let mut visitor = Visitor::from(self.ast, local_symtab, local_typetab, self.lines, self.path);

                self.visit_expression(&condition)?;

                if self.type_expression(&condition)? != Type::boolean() {
                    Err(make_error(Some(position.clone()), "non-boolean condition".to_owned()))
                } else {
                    visitor.visit_expression(body)
                }
            },

            (&Module { ref name, ref content }, position) => {
                if self.symtab.get_name(name).is_some() {
                    Err(make_error(Some(position), format!("module '{}' defined multiple times", name)))
                } else {
                    self.typetab.grow();
                    let index = self.symtab.add_name(&name);

                    let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                    let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                    let mut visitor = Visitor::from(self.ast, local_symtab, local_typetab, self.lines, self.path);

                    visitor.visit_expression(&content)?;
                    
                    let mut hash_types = HashMap::new();

                    for (ref name, ref index) in visitor.symtab.names.borrow().clone() {
                        hash_types.insert(name.clone(), visitor.typetab.get_type(*index, 0)?);
                    }
                    
                    self.typetab.set_type(index, 0, Type::new(TypeNode::Module(hash_types), TypeMode::Just))
                }
            },

            (&Import { ref origin, ref expose }, position) => {
                let origin_type = self.type_expression(origin)?;

                match origin_type.0 {
                    TypeNode::Module(ref members) => {
                        if let Some(ref expose) = *expose {
                            for exposed in expose {
                                match members.get(exposed) {
                                    Some(ref member) => {
                                        self.typetab.grow();
                                        let index = self.symtab.add_name(&exposed);
                                        self.typetab.set_type(index, 0, (**member).clone())?;
                                    },
                                    None    => return Err(make_error(Some(position), format!("can't expose non-existing member '{}'", exposed)))
                                }
                            }

                            Ok(())
                        } else {
                            Ok(())
                        }
                    },

                    _ => Err(make_error(Some(position), format!("can't import from '{}'", origin_type)))
                }
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

            (&Function {ref params, ref return_type, ref body}, _) => self.visit_function(params, return_type, body),

            (&Array(ref content), position) => {
                let array_type = self.type_expression(&content[0])?;

                for element in content {
                    let element_type = self.type_expression(element)?;

                    if array_type != element_type {
                        return Err(make_error(Some(position), format!("mismatched element type '{}'", element_type)))
                    }
                }

                Ok(())
            },

            (&Constructor(ref name, ref members), position) => {
                use TypeNode::*;
                
                let name_type = self.type_expression(&name)?;
                
                match name_type.0 {
                    Struct(ref types) => {
                        if !name_type.1.check(&TypeMode::Unconstructed) {
                            return Err(make_error(Some(position), format!("expected unconstructed struct, found '{}'", name_type)))
                        }

                        let mut acc = 0;

                        for (member_name, member_type) in types {
                            let con_member = match members.get(acc) {
                                Some(member) => member,
                                None         => return Err(make_error(Some(position), format!("missing initialization '{}'", member_name)))
                            };

                            self.visit_expression(&con_member.1)?;
                            let con_type = self.type_expression(&con_member.1)?;

                            if *member_type != con_type.0 {
                                return Err(make_error(Some(position), format!("mismatching member '{}': expected '{}', found '{}'", member_name, member_type, con_type)))
                            }

                            acc += 1
                        }

                        Ok(())
                    },
                    _ => Err(make_error(Some(position), format!("can't initialize '{}'", name_type)))
                }
            },

            (&Index(ref indexed, ref index), position) => {
                let indexed_type = self.type_expression(&indexed)?;
                
                if indexed_type.1.check(&TypeMode::Undeclared) || indexed_type.1.check(&TypeMode::Unconstructed) {
                    return Err(make_error(Some(position), format!("can't index '{}'", indexed_type)))
                }

                match indexed_type.0 {
                    TypeNode::Array(_) => {
                        let index_type = self.type_expression(index)?;

                        if Type::int() != index_type {
                            Err(make_error(Some(position), format!("can't index '{}' with '{}'", indexed_type, index_type)))
                        } else {
                            Ok(())
                        }
                    },

                    TypeNode::Struct(ref members) => match index.0 {
                        Identifier(ref name) |
                        Str(ref name)        => {
                            if members.get(name).is_some() {
                                Ok(())
                            } else {
                                Err(make_error(Some(position), format!("no field '{}'", name)))
                            }
                        },
                        _ => Err(make_error(Some(position), format!("indexing struct with non-key '{:?}'", index.0)))
                    },

                    TypeNode::Module(ref members) => match index.0 {
                        Identifier(ref name) |
                        Str(ref name)        => {
                            if members.get(name).is_some() {
                                Ok(())
                            } else {
                                Err(make_error(Some(position), format!("no field '{}'", name)))
                            }
                        },
                        _ => Err(make_error(Some(position), format!("indexing module with non-key '{:?}'", index.0)))
                    },

                    _ => Err(make_error(Some(position), format!("can't index type '{}'", indexed_type)))
                }
            }

            (&Call(ref callee, ref args), _) => self.visit_call(callee, args),

            (&Block(ref statements), _) => {
                let mut acc = 1;
                for statement in statements {
                    if acc < statements.len() {
                        match statement.0 {
                            StatementNode::Expression(ref expr) => match expr.0 {
                                ExpressionNode::Block(..) |
                                ExpressionNode::Call(..)  => (),
                                _                         => return Err(make_error(Some(statement.1), "a wild expression appeared".to_owned()))
                            },

                            _ => (),
                        }
                    }

                    self.visit_statement(&statement)?;

                    acc += 1
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
        
        let return_type = self.dealias(&Type::new(return_type.clone(), TypeMode::Just))?;

        if return_type != body_t {
            Err(make_error(Some(body.1), format!("mismatched return type, expected '{}' .. found '{}'", return_type, body_t)))
        } else {
            Ok(())
        }
    }

    fn visit_call(&mut self, callee: &Rc<Expression>, args: &Vec<Expression>) -> Response<()> {
        let callee_t = self.type_expression(&**callee)?;

        if callee_t.1 == TypeMode::Undeclared {
            Err(make_error(Some(callee.1), format!("don't call an undeclared: '{}'", callee_t)))
        } else {
            match callee_t.0 {
                TypeNode::Fun(ref params, _) => {
                    let mut acc = 0;

                    if params.len() != args.len() {
                        if params.len() < args.len() {
                            Err(make_error(Some(args[acc].1), format!("function expected {} arg{}, got {}", params.len(), if params.len() != 1 { "s" } else { "" }, args.len())))
                        } else {
                            for param in &params[args.len() .. params.len()] {
                                if !param.1.check(&TypeMode::Optional) {
                                    return Err(make_error(Some(callee.1), format!("can't ommit non-optional argument")))
                                }
                            }

                            self.visit_expression(&**callee)
                        }
                    } else {
                        for param in params {
                            let param = match param.0 {
                                TypeNode::Id(ref id) => self.type_expression(&Expression::new(ExpressionNode::Identifier(id.clone()), args[acc].1))?,
                                _                    => param.clone(),
                            };
                            
                            if param != self.type_expression(&args[acc])? {
                                return Err(make_error(Some(args[acc].1), format!("mismatched argument type: '{}', expected: '{}'", self.type_expression(&args[acc])?, param)))
                            }
                            acc += 1
                        }

                        self.visit_expression(&**callee)
                    }
                },

                ref t => Err(make_error(Some(callee.1), format!("can't call: '{}'", t))),
            }
        }
    }

    fn type_expression(&mut self, expression: &Expression) -> Response<Type> {
        use ExpressionNode::*;

        let t = match (&expression.0, expression.1) {
            (&Int(_), _)   => Type::int(),
            (&Float(_), _) => Type::float(),
            (&Bool(_), _)  => Type::boolean(),
            (&Str(_), _)   => Type::string(),
            (&Identifier(ref name), position) => {
                match self.symtab.get_name(&*name) {
                    Some((i, env_index)) => self.typetab.get_type(i, env_index)?,
                    None                 => return Err(make_error(Some(position), format!("undefined type of: {}", name)))
                }
            }

            (&Array(ref content), _) => Type::new(TypeNode::Array(Rc::new(self.type_expression(&content[0])?)), TypeMode::Just),

            (&Index(ref indexed, ref index), position) => {
                match self.type_expression(indexed)?.0 {
                    TypeNode::Array(ref content) => (**content).clone(),
                    TypeNode::Struct(ref members) => match index.0 {
                        Identifier(ref name) |
                        Str(ref name)        => Type::new(members.get(name).unwrap().clone(), TypeMode::Just),
                        _ => unreachable!(),
                    },
                    
                    TypeNode::Module(ref members) => match index.0 {
                        Identifier(ref name) |
                        Str(ref name)        => match members.get(name) {
                            Some(a) => a.clone(),
                            None    => return Err(make_error(Some(position), format!("undefined member '{}'", name))),
                        },
                        _ => unreachable!(),
                    },

                    ref t => return Err(make_error(Some(position), format!("can't index '{}'", t))),
                }
            }

            (&Constructor(ref name, _), _) => Type::new(self.type_expression(name)?.0, TypeMode::Just),

            (&Function {ref params, ref return_type, ..}, _) => {
                Type::new(TypeNode::Fun(
                    params.iter().map(|x| Type::new(x.1.clone(), if let Some(_) = x.2 { TypeMode::Optional } else { TypeMode::Just })).collect::<Vec<Type>>(),
                    Rc::new(self.dealias(&Type::new(return_type.clone(), TypeMode::Just))?),
                ), TypeMode::Just)
            },

            (&Call(ref callee, _), position) => match self.type_expression(&**callee)?.0 {
                TypeNode::Fun(_, ref retty) => (**retty).clone(),
                ref t                       => return Err(make_error(Some(position), format!("can't call: {}", t))),
            },
            
            (&Unary(ref op, ref expression), position) => {
                use Operator::*;
                use TypeNode::*;

                match (op, &self.type_expression(&*expression)?) {
                    (&Sub, a) => if vec![Float, Int].contains(&a.0) {
                        a.clone()
                    } else {
                        return Err(make_error(Some(position), format!("can't negate '{}'", a)))
                    }

                    (&Not, a) => if a.0 == Bool {
                        a.clone()
                    } else {
                        return Err(make_error(Some(position), format!("can't negate non-bool '{}'", a)))
                    }

                    _ => Type::nil(),
                }
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

                    (a, c @ &Mul, b) |
                    (a, c @ &Div, b) |
                    (a, c @ &Sub, b) |
                    (a, c @ &Add, b) => if a == b {
                        Type::new(a.0, TypeMode::Just)
                    } else {
                        return Err(make_error(Some(position), format!("failed to {} '{}' and '{}'", format!("{:?}", c).to_lowercase(), a, b)))
                    }

                    (_, &Equal, _)   |
                    (_, &NEqual, _)  |
                    (_, &Lt, _)      |
                    (_, &Gt, _)      |
                    (_, &LtEqual, _) |
                    (_, &GtEqual, _) => Type::new(Bool, TypeMode::Just),

                    (_, &Not, _) => return Err(make_error(Some(position), format!("can't use '~' as a binary operation"))),

                    (ref left_type, &Compound(ref op), _) => {
                        if left_type.1.check(&TypeMode::Constant) {
                            return Err(make_error(Some(position), "can't reassign immutable".to_owned()))
                        } else {
                            match self.type_expression(&Expression::new(Binary { left: left.clone(), op: (**op).clone(), right: right.clone() }, position)) {
                                Ok(_)      => Type::nil(),
                                e @ Err(_) => return e,
                            }
                        }
                    },

                    _ => Type::nil(),
                }
            },

            (&Block(ref statements), _) => {
                let mut return_type = None;

                let mut acc = 1;

                let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                let mut visitor = Visitor::from(self.ast, local_symtab, local_typetab, self.lines, self.path);

                for statement in statements {
                    if acc != statements.len() {
                        visitor.visit_statement(&statement)?
                    }

                    if return_type == None {
                        if let Some(t) = visitor.find_return_type(&statement.0, acc == statements.len())? {
                            return_type = Some(t)
                        }
                    }

                    acc += 1
                }

                return_type.unwrap_or(Type::nil())
            },

            _ => Type::nil(),
        };

        self.dealias(&t)
    }

    fn find_return_type(&mut self, statement: &StatementNode, is_last: bool) -> Response<Option<Type>> {
        use StatementNode::*;

        let return_type = match *statement {
            Expression(ref expression) => if is_last {
                Some(self.type_expression(expression)?)
            } else {
                None
            },

            Return(ref value) => if let Some(ref value) = *value {
                Some(self.type_expression(value)?)
            } else {
                Some(Type::nil())
            },

            If(ref if_node) => Some(self.type_expression(&if_node.body)?),

            _ => None,
        };

        Ok(return_type)
    }

    fn visit_definition(&mut self, kind: &TypeNode, left: &Expression, right: &Option<Expression>) -> Response<()> {
        use ExpressionNode::*;
        
        let kind = self.dealias(&Type::new(kind.clone(), TypeMode::Constant))?.0;

        let var_type = Type::new(kind.clone(), TypeMode::Just);

        if let Some(ref right) = *right {
            match right.0 {
                Function { .. } | Block(_) => (),
                _                          => self.visit_expression(right)?,
            }
            
            let right_kind = self.type_expression(&right)?;
            
            let index = match left.0 {
                Identifier(ref name) => {
                    self.typetab.grow();
                    self.symtab.add_name(&name)
                },

                Index(..) => return Ok(()),
                _         => return Err(make_error(Some(left.1), format!("can't define anything but identifiers"))),
            };

            if kind != TypeNode::Nil {
                if kind != right_kind.0 {
                    return Err(make_error(Some(right.1), format!("mismatched types: expected '{}', found '{}'", kind, right_kind)))
                } else {
                    self.typetab.set_type(index, 0, var_type)?;
                }
            } else {                
                self.typetab.set_type(index, 0, right_kind)?;
            }

            match right.0 {
                Function { .. } | Block(_) => self.visit_expression(right),
                _                          => Ok(()),
            }
        } else {
            let index = match left.0 {
                Identifier(ref name) => {
                    self.typetab.grow();
                    self.symtab.add_name(&name)
                },

                Index(..) => return Ok(()),
                _         => return Err(make_error(Some(left.1), format!("can't define anything but identifiers"))),
            };

            self.typetab.set_type(index, 0, Type::new(kind.clone(), TypeMode::Undeclared))
        }
    }

    fn visit_constant(&mut self, kind: &TypeNode, left: &Expression, right: &Expression) -> Response<()> {
        use ExpressionNode::*;

        let kind = self.dealias(&Type::new(kind.clone(), TypeMode::Constant))?.0;

        let index = match left.0 {
            Identifier(ref name) => {
                self.typetab.grow();
                self.symtab.add_name(&name)
            },
            Index(..) => return Ok(()),
            _         => return Err(make_error(Some(left.1), format!("can't define anything but identifiers"))),
        };

        let const_type = Type::new(kind.clone(), TypeMode::Constant);

        match right.0 {
            Function { .. } | Block(_) => (),
            _                          => self.visit_expression(right)?,
        }

        let right_kind = Type::new(self.type_expression(&right)?.0, TypeMode::Constant);

        if right_kind.0 == TypeNode::Nil {
            return Err(make_error(Some(right.1), format!("expected non-nil")))
        }

        if kind != TypeNode::Nil {
            if kind != right_kind.0 {
                return Err(make_error(Some(right.1), format!("mismatched types: expected '{}', found '{}'", kind, right_kind)))
            } else {
                self.typetab.set_type(index, 0, const_type)?;
            }
        } else {
            self.typetab.set_type(index, 0, right_kind)?;
        }

        match right.0 {
            Function { .. } => self.visit_expression(right),
            _               => Ok(()),
        }
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

        let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
        let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

        let mut visitor = Visitor::from(self.ast, local_symtab, local_typetab, self.lines, self.path);

        if self.type_expression(&if_node.condition)? != Type::boolean() {
            Err(make_error(Some(position.clone()), "non-boolean condition".to_owned()))
        } else {
            visitor.visit_expression(&if_node.body)?;

            if let Some(ref cases) = if_node.elses {
                let return_type = visitor.type_expression(&if_node.body)?;

                for case in cases {
                    if let Some(ref condition) = case.0 {
                        self.visit_expression(condition)?;

                        if self.type_expression(&condition)? != Type::boolean() {
                            return Err(make_error(Some(position.clone()), "non-boolean condition".to_owned()))
                        }
                    }

                    let local_symtab  = SymTab::new(Rc::new(self.symtab.clone()), &[]);
                    let local_typetab = TypeTab::new(Rc::new(self.typetab.clone()), &Vec::new(), &HashMap::new());

                    let mut visitor = Visitor::from(self.ast, local_symtab, local_typetab, self.lines, self.path);

                    visitor.visit_expression(&case.1)?;

                    let case_t = visitor.type_expression(&case.1)?;

                    if return_type != case_t {
                        return Err(make_error(Some(case.2.clone()), format!("mismatched types: expected '{}', found '{}'", return_type, case_t)))
                    }
                }

                Ok(())
            } else {
                Ok(())
            }
        }
    }
}

use std::fmt::{ self, Display, Write, Formatter };
use std::rc::Rc;
use std::collections::HashMap;

use super::super::error::Response::Wrong;

use super::*;
use super::TokenElement;



#[derive(Debug, Clone)]
pub enum TypeNode<'t> {
  Int,
  Float,
  Bool,
  Str,
  Char,
  Nil,
  Id(String),
  Array(Rc<Type<'t>>, usize),
  Func(Vec<Type<'t>>, Rc<Type<'t>>, Vec<String>, Option<&'t ExpressionNode<'t>>),
  Module(HashMap<String, Type<'t>>),
}

impl<'t> TypeNode<'t> {
  pub fn check_expression(&self, other: &'t ExpressionNode<'t>) -> bool {
    use self::TypeNode::*;

    match *other {
      ExpressionNode::Int(_) => match *self {
        Int | Float => true,
        _           => false,
      },

      ExpressionNode::Array(ref content) => {
        let array_content = if let &Array(ref array_content, ref len) = self {
          if *len != content.len() {
            return false
          }

          array_content
        } else {
          return false
        };

        for element in content {
          if !array_content.node.check_expression(&element.node) {
            return false
          }
        }

        true
      },

      _ => false
    }
  }
}



impl<'t> PartialEq for TypeNode<'t> {
  fn eq(&self, other: &Self) -> bool {
    use self::TypeNode::*;

    match (self, other) {
      (&Int,                                 &Int)                                 => true,
      (&Str,                                 &Str)                                 => true,
      (&Float,                               &Float)                               => true,
      (&Char,                                &Char)                                => true,
      (&Bool,                                &Bool)                                => true,
      (&Nil,                                 &Nil)                                 => true,
      (&Array(ref a, ref la),                &Array(ref b, ref lb))                => a == b && la == lb,
      (&Id(ref a),                           &Id(ref b))                           => a == b,
      (&Func(ref a_params, ref a_retty, ..), &Func(ref b_params, ref b_retty, ..)) => a_params == b_params && a_retty == b_retty,

      _ => false,
    }
  }
}



#[derive(Debug, Clone)]
pub enum TypeMode {
  Undeclared,
  Immutable,
  Optional,
  Regular,
  Splat(Option<usize>),
  Unwrap(usize),
}

impl<'t> Display for TypeNode<'t> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::TypeNode::*;

    match *self {
      Int              => write!(f, "int"),
      Float            => write!(f, "float"),
      Bool             => write!(f, "bool"),
      Str              => write!(f, "str"),
      Char             => write!(f, "char"),
      Nil              => write!(f, "nil"),
      Array(ref n, l)  => write!(f, "[{}; {}]", n, l),
      Id(ref n)        => write!(f, "{}", n),
      Module(_)        => write!(f, "module"),

      Func(ref params, ref return_type, ..) => {
        write!(f, "(");

        for (index, element) in params.iter().enumerate() {
          if index < params.len() - 1 {
            write!(f, "{}, ", element)?
          } else {
            write!(f, "{}", element)?
          }
        }

        write!(f, ") -> {}", return_type)
      },
    }
  }
}



impl PartialEq for TypeMode {
  fn eq(&self, other: &TypeMode) -> bool {
    use self::TypeMode::*;

    match (self, other) {
      (&Regular,    &Regular)     => true,
      (&Regular,    &Immutable)   => true,
      (&Immutable,  &Immutable)   => true,
      (&Immutable,  &Regular)     => true,
      (_,           &Optional)    => true,
      (&Optional,   _)            => true,
      (&Undeclared, _)            => false,
      (_,           &Undeclared)  => false,
      (&Splat(a),      &Splat(b)) => &a == &b,
      (&Unwrap(_),  _)            => true,
      (_,           &Unwrap(_))   => true,
      _                           => false,
    }
  }
}

impl Display for TypeMode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::TypeMode::*;

    match *self {
      Regular    => Ok(()),
      Immutable  => write!(f, "constant "),
      Undeclared => write!(f, "undeclared "),
      Optional   => write!(f, "optional "),
      Splat(_)   => write!(f, ".."),
      Unwrap(_)  => write!(f, "*"),
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Type<'t> {
  pub node: TypeNode<'t>,
  pub mode: TypeMode,
}

impl<'t> Type<'t> {
  pub fn new(node: TypeNode<'t>, mode: TypeMode) -> Self {
    Self {
      node, mode,
    }
  }

  pub fn id(id: &str) -> Self {
    Type::new(TypeNode::Id(id.to_owned()), TypeMode::Regular)
  }

  pub fn from(node: TypeNode<'t>) -> Type<'t> {
    Type::new(node, TypeMode::Regular)
  }

  pub fn array(t: Type<'t>, len: usize) -> Type<'t> {
    Type::new(TypeNode::Array(Rc::new(t), len), TypeMode::Regular)
  }

  pub fn function(params: Vec<Type<'t>>, return_type: Type<'t>) -> Self {
    Type::new(TypeNode::Func(params, Rc::new(return_type), Vec::new(), None), TypeMode::Regular)
  }
}

impl<'t> Display for Type<'t> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}{}", self.mode, self.node)
  }
}



#[derive(Debug, Clone)]
pub enum FlagContext<'f> {
  Block(Option<Type<'f>>),
  Nothing,
}



pub struct Visitor<'v> {
  pub tabs:       Vec<(SymTab, TypeTab<'v>)>,
  pub tab_frames: Vec<(SymTab, TypeTab<'v>)>,

  pub source:  &'v Source,
  pub ast:     &'v Vec<Statement<'v>>,

  pub flag: Option<FlagContext<'v>>,
}

impl<'v> Visitor<'v> {
  pub fn new(source: &'v Source, ast: &'v Vec<Statement<'v>>) -> Self {
    Visitor {
      tabs:       vec!((SymTab::global(), TypeTab::global())),
      tab_frames: Vec::new(), // very intelligent hack

      source,
      ast,

      flag: None,
    }
  }

  pub fn visit(&mut self) -> Result<(), ()> {
    for statement in self.ast {
      self.visit_statement(&statement)?
    }

    self.tab_frames.push(self.tabs.last().unwrap().clone());

    Ok(())
  }

  pub fn visit_statement(&mut self, statement: &'v Statement<'v>) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.visit_expression(expression),
      Variable(..)               => self.visit_variable(&statement.node),

      Assignment(ref left, ref right) => {
        let left_type  = self.type_expression(left)?;
        let right_type = self.type_expression(right)?;

        if !left_type.node.check_expression(&Parser::fold_expression(right)?.node) && left_type.node != right_type.node {
          return Err(
            response!(
              Wrong(format!("mismatched types, expected type `{}` got `{}`", left_type.node, right_type)),
              self.source.file,
              right.pos
            )
          )
        }

        Ok(())
      },

      _ => Ok(())
    }
  }

  fn ensure_no_implicit(&self, expression: &'v Expression<'v>) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Block(ref statements) => if let Some(statement) = statements.last() {
        if let StatementNode::Expression(ref expression) = statement.node {
          match expression.node {

            Call(..)   => (),
            Block(..)  => { self.ensure_no_implicit(expression)?; }

            If(_, ref expr, _) => self.ensure_no_implicit(&*expr)?,

            _ => return Err(
              response!(
                Wrong("unexpected expression without context"),
                self.source.file,
                expression.pos
              )
            )
          }
        }

        ()
      } else {
        ()
      },

      Call(..)   => (),

      If(_, ref expr, _) => self.ensure_no_implicit(&*expr)?,

      _ => return Err(
        response!(
          Wrong("unexpected expression without context"),
          self.source.file,
          expression.pos
        )
      )
    }

    Ok(())
  }

  fn visit_expression(&mut self, expression: &'v Expression<'v>) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Identifier(ref name) => if self.current_tab().0.get_name(name).is_none() {
        Err(
          response!(
            Wrong(format!("no such value `{}` in this scope", name)),
            self.source.file,
            expression.pos
          )
        )
      } else {
        Ok(())
      },

      Module(ref content) => self.visit_expression(content),

      Unwrap(ref expression) => {
        self.visit_expression(&**expression)?;

        if let TypeMode::Splat(_) = self.type_expression(&**expression)?.mode {
          Ok(())
        } else {
          Err(
            response!(
              Wrong("can't unwrap a non-splat value"),
              self.source.file,
              expression.pos
            )
          )
        }
      }

      Block(ref statements) => {
        self.push_scope();

        for (i, statement) in statements.iter().enumerate() {
          if i < statements.len() - 1 {
            if let StatementNode::Expression(ref expression) = statement.node {
              self.ensure_no_implicit(expression)?
            }
          }

          self.visit_statement(statement)?
        }

        self.pop_scope();

        Ok(())
      },

      If(ref condition, ref body, ref elses) => {
        self.visit_expression(&*condition)?;

        let condition_type = self.type_expression(&*condition)?.node;

        if condition_type == TypeNode::Bool {

          self.push_scope();

          self.visit_expression(body)?;
          let body_type = self.type_expression(body)?;

          self.pop_scope();

          if let &Some(ref elses) = elses {
            for &(ref maybe_condition, ref body, _) in elses {
              if let Some(ref condition) = *maybe_condition {
                let condition_type = self.type_expression(condition)?.node;

                if condition_type != TypeNode::Bool {
                  return Err(
                    response!(
                      Wrong(format!("mismatched condition, must be `bool` got `{}`", condition_type)),
                      self.source.file,
                      condition.pos
                    )
                  )
                }
              }

              self.push_scope();

              self.visit_expression(body)?;
              let else_body_type = self.type_expression(body)?;

              self.pop_scope();

              if body_type != else_body_type {
                return Err(
                  response!(
                    Wrong(format!("mismatched types, expected `{}` got `{}`", body_type, else_body_type)),
                    self.source.file,
                    body.pos
                  )
                )
              }
            }
          }

          Ok(())

        } else {
          return Err(
            response!(
              Wrong(format!("mismatched condition, must be `bool` got `{}`", condition_type)),
              self.source.file,
              expression.pos
            )
          )
        }
      },

      Call(ref expression, ref args) => {
        self.visit_expression(expression)?;

        let expression_type = self.type_expression(expression)?.node;

        let mut covers = HashMap::new();

        let mut corrected_params = Vec::new(); // because functions don't know what's best for them >:()

        if let TypeNode::Func(ref params, _, ref generics, ref func) = expression_type {
          let mut actual_arg_len = args.len();

          let mut type_buffer: Option<Type<'v>> = None; // for unwraps


          let mut has_unwrap = false;

          for (index, param) in params.iter().enumerate() {
            let arg_type = if index < args.len() {
              self.type_expression(&args[index])?
            } else {
              type_buffer.as_ref().unwrap().clone()
            };

            corrected_params.push(arg_type.clone());

            let mode = arg_type.mode.clone();

            if let TypeMode::Unwrap(ref len) = mode {
              has_unwrap = true;

              type_buffer = Some(arg_type.clone());

              actual_arg_len += len
            }

            if let TypeNode::Id(ref name) = param.node {
              if generics.contains(name) {
                if let Some(kind) = covers.get(name) {
                  if &arg_type == kind {
                    continue
                  } else {
                    return Err(
                      response!(
                        Wrong(format!("mismatched argument, expected `{}` got `{}`", expression_type, arg_type)),
                        self.source.file,
                        expression.pos
                      )
                    )
                  }
                }

                covers.insert(name.clone(), arg_type.clone());

                continue
              }
            }

            if (index < args.len() && !param.node.check_expression(&args[index].node)) && param != &arg_type {
              return Err(
                response!(
                  Wrong(format!("mismatched argument, expected `{}` got `{}`", param, arg_type)),
                  self.source.file,
                  args[index].pos
                )
              )
            }
          }

          if has_unwrap {
            actual_arg_len -= 0
          }

          if actual_arg_len > params.len() {
            let last = params.last().unwrap();

            if let TypeMode::Splat(_) = last.mode {
              for splat in &args[params.len()..] {
                let splat_type = self.type_expression(&splat)?;

                if let TypeNode::Id(ref name) = last.node {
                  if generics.contains(name) {
                    if let Some(kind) = covers.get(name) {
                      if &splat_type == kind {
                        continue
                      } else {
                        return Err(
                          response!(
                            Wrong(format!("mismatched splat argument, expected `{}` got `{}`", kind, splat_type)),
                            self.source.file,
                            splat.pos
                          )
                        )
                      }
                    }
                  }
                }

                if !last.node.check_expression(&splat.node) && last != &splat_type {
                  return Err(
                    response!(
                      Wrong(format!("mismatched splat argument, expected `{}` got `{}`", last, splat_type)),
                      self.source.file,
                      splat.pos
                    )
                  )
                }
              }
            }
          }
          

          if actual_arg_len > params.len() {
            match params.last().unwrap().mode {
              TypeMode::Splat(_) => (),
              _                  => return Err(
                response!(
                  Wrong(format!("too many arguments, expected {} got {}", params.len(), actual_arg_len)),
                  self.source.file,
                  args.last().unwrap().pos
                )
              )
            }
          }


          if covers.len() > 0 || actual_arg_len > params.len() {
            if let Function(ref params, ref return_type, ref body, ref generics) = *func.unwrap() {

              let mut real_params = Vec::new();

              for (i, param) in params.iter().enumerate() {
                real_params.push(
                  (
                    param.0.clone(),
                    if let TypeNode::Func(..) = param.1.node {
                      corrected_params[i].clone()
                    } else {
                      param.1.clone()
                    }
                  )
                )
              }

              self.visit_function(expression.pos.clone(), &real_params, return_type, body, generics, Some(covers), actual_arg_len - params.len())?;
            } else {
              unreachable!()
            }
          }

        } else {
          return Err(
            response!(
              Wrong(format!("expected function, found `{}`", expression_type)),
              self.source.file,
              expression.pos
            )
          )
        }

        Ok(())
      },

      Function(ref params, ref return_type, ref body, ref generics) => self.visit_function(expression.pos.clone(), params, return_type, body, generics, None, 0),

      Array(ref content) => {
        let t = self.type_expression(content.first().unwrap())?;

        for element in content {
          let element_type = self.type_expression(element)?;

          if !t.node.check_expression(&Parser::fold_expression(element)?.node) && t.node != element_type.node {
            return Err(
              response!(
                Wrong(format!("mismatched types in array, expected `{}` got `{}`", t, element_type)),
                self.source.file,
                element.pos
              )
            )
          }
        }

        Ok(())
      },

      Index(ref left, ref index) => {
        let left_type = self.type_expression(left)?;

        if let TypeNode::Array(_, ref len) = left_type.node {
          let index_type = self.type_expression(index)?;

          match index_type.node {
            TypeNode::Int => {
              if let Int(ref a) = Parser::fold_expression(index)?.node {
                if *a as usize > *len {
                  return Err(
                    response!(
                      Wrong(format!("index out of bounds, len is {} got {}", len, a)),
                      self.source.file,
                      left.pos
                    )
                  )
                }
              }
            },

            _ => return Err(
              response!(
                Wrong(format!("can't index with `{}`, must be positive integer", index_type)),
                self.source.file,
                left.pos
              )
            )
          }

        } else {
          return Err(
            response!(
              Wrong(format!("can't index `{}`", left_type)),
              self.source.file,
              left.pos
            )
          )
        }

        Ok(())
      },

      _ => Ok(())
    }
  }



  fn visit_function(
      &mut self,
      pos: TokenElement<'v>,
      params: &Vec<(String, Type<'v>)>, return_type: &'v Type<'v>,
      body: &'v Rc<Expression<'v>>, generics: &Option<Vec<String>>, generic_covers: Option<HashMap<String, Type<'v>>>,
      splat_len: usize
  ) -> Result<(), ()> {
    let mut param_names = Vec::new();
    let mut param_types = Vec::new();

    let mut return_type = return_type;

    for param in params {
      param_names.push(param.0.clone());

      let kind = if let Some(ref generics) = *generics {
        if let TypeNode::Id(ref name) = return_type.node {
          if generics.contains(name) {
            if let Some(ref covers) = generic_covers {
              return_type = covers.get(name).unwrap()
            }
          }
        }

        if let TypeNode::Id(ref name) = param.1.node {
          if generics.contains(name) {
            if let Some(ref covers) = generic_covers {
              Type::new(covers.get(name).unwrap().clone().node, param.1.mode.clone())
            } else {
              param.1.clone()
            }
          } else {
            param.1.clone()
          }
        } else {
          param.1.clone()
        }
      } else {
        param.1.clone()
      };

      param_types.push(kind);
    }

    let last_type = param_types.last().unwrap().clone();

    if let TypeMode::Splat(_) = last_type.mode {
      let len = param_types.len();

      param_types[len - 1] = Type::new(last_type.node, TypeMode::Splat(Some(splat_len)))
    }

    if generics.is_none() != generic_covers.is_none() && splat_len == 0 {
      return Ok(())
    }

    let parent = self.current_tab().clone();

    self.tabs.push(
      (
        SymTab::new(Rc::new(parent.0), &param_names),
        TypeTab::new(Rc::new(parent.1), &param_types, HashMap::new())
      )
    );

    self.visit_expression(body)?;

    let body_type = self.type_expression(body)?;

    self.pop_scope();

    if return_type != &body_type {
      Err(
        response!(
          Wrong(format!("mismatched return type, expected `{}` got `{}`", return_type, body_type)),
          self.source.file,
          pos
        )
      )
    } else {
      Ok(())
    }
  }



  fn visit_variable(&mut self, variable: &'v StatementNode) -> Result<(), ()> {
    use self::ExpressionNode::*;

    if let &StatementNode::Variable(ref variable_type, ref name, ref right) = variable {
      let index = if let Some((index, _)) = self.current_tab().0.get_name(name) {
        
        index
      } else {
        self.current_tab().1.grow();
        self.current_tab().0.add_name(name)
      };

      if let &Some(ref right) = right {
        let right_type = self.type_expression(&right)?;

        match right.node {
          Function(..) | Block(_) | If(..) => (),
          _ => self.visit_expression(right)?,
        }

        if variable_type.node != TypeNode::Nil {
          if !variable_type.node.check_expression(&Parser::fold_expression(right)?.node) && variable_type.node != right_type.node {
            return Err(
              response!(
                Wrong(format!("mismatched types, expected type `{}` got `{}`", variable_type.node, right_type)),
                self.source.file,
                right.pos
              )
            )
          } else {
            self.current_tab().1.set_type(index, 0, variable_type.to_owned())?;
          }

        } else {
          self.current_tab().1.set_type(index, 0, right_type)?;
        }

        match right.node {
          Function(..) | Block(_) | If(..) => self.visit_expression(right)?,
          _ => (),
        }

      } else {
        self.current_tab().1.set_type(index, 0, variable_type.to_owned())?;
      }

      Ok(())
    } else {
      unreachable!()
    }
  }



  pub fn type_statement(&mut self, statement: &'v Statement<'v>) -> Result<Type<'v>, ()> {
    use self::StatementNode::*;

    let t = match statement.node {
      Expression(ref expression) => self.type_expression(expression)?,
      Return(ref expression)     => if let Some(ref expression) = *expression {
        self.type_expression(expression)?
      } else {
        Type::from(TypeNode::Nil)
      }
      _ => Type::from(TypeNode::Nil)
    };

    Ok(t)
  }



  pub fn type_expression(&mut self, expression: &'v Expression<'v>) -> Result<Type<'v>, ()> {
    use self::ExpressionNode::*;

    let t = match expression.node {
      Identifier(ref name) => if let Some((index, env_index)) = self.current_tab().0.get_name(name) {
        self.current_tab().1.get_type(index, env_index)?.clone()
      } else {
        return Err(
          response!(
            Wrong(format!("no such value `{}` in this scope", name)),
            self.source.file,
            expression.pos
          )
        )
      },

      Unwrap(ref expr) => {
        let t = self.type_expression(&**expr)?;

        if let TypeMode::Splat(ref len) = t.mode {
          Type::new(t.node, TypeMode::Unwrap(len.unwrap()))
        } else {
          unreachable!()
        }
      },

      Module(ref content) => {
        self.visit_expression(content)?;

        let mut content_type = HashMap::new();
        
        let frame = self.tab_frames.last().unwrap();

        let names = frame.0.names.clone();

        for symbol in names.borrow().iter() {
          content_type.insert(symbol.0.clone(), frame.1.get_type(*symbol.1, 0)?.clone());
        }

        Type::from(TypeNode::Module(content_type))
      },

      Empty    => Type::from(TypeNode::Nil),

      Str(_)   => Type::from(TypeNode::Str),
      Char(_)  => Type::from(TypeNode::Char),
      Bool(_)  => Type::from(TypeNode::Bool),
      Int(_)   => Type::from(TypeNode::Int),
      Float(_) => Type::from(TypeNode::Float),

      Call(ref expression, _) => {
        if let TypeNode::Func(_, ref return_type, ..) = self.type_expression(expression)?.node {
          (**return_type).clone()
        } else {
          panic!("accident (submit an issue): called {:#?}", self.type_expression(expression)?.node)
        }
      },

      Index(ref array, _) => if let TypeNode::Array(ref t, _) = self.type_expression(array)?.node {
        (**t).clone()
      } else {
        unreachable!()
      },

      If(_, ref expression, _) => self.type_expression(expression)?,

      Array(ref content) => Type::array(self.type_expression(content.first().unwrap())?, content.len()),

      Cast(_, ref t) => Type::from(t.node.clone()),

      Binary(ref left, ref op, ref right) => {
        use self::Operator::*;

        match (self.type_expression(left)?.node, op, self.type_expression(right)?.node) {
          (ref a, ref op, ref b) => match **op {
            Add | Sub | Mul | Div | Pow | Mod => if [a, b] != [&TypeNode::Nil, &TypeNode::Nil] { // real hack here
              Type::from(if a != &TypeNode::Nil { a.to_owned() } else { b.to_owned() })
            } else {
              return Err(
                response!(
                  Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                  self.source.file,
                  expression.pos
                )
              )
            },

            Concat => if *a == TypeNode::Str {
              match *b {
                TypeNode::Func(..) | TypeNode::Array(..) => return Err(
                  response!(
                    Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                    self.source.file,
                    expression.pos
                  )
                ),

                _ => Type::from(TypeNode::Str)
              }
            } else {
              return Err(
                response!(
                  Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                  self.source.file,
                  expression.pos
                )
              )
            },

            Eq | Lt | Gt | NEq | LtEq | GtEq => if a == b {
              Type::from(TypeNode::Bool)
            } else {
              return Err(
                response!(
                  Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                  self.source.file,
                  expression.pos
                )
              )
            },

            _ => return Err(
              response!(
                Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                self.source.file,
                expression.pos
              )
            )
          },
        }
      },

      Function(ref params, ref return_type, _, ref generics) => {
        let mut param_types = Vec::new();

        for param in params {
          param_types.push(param.1.clone())
        }

        Type::from(TypeNode::Func(param_types, Rc::new(return_type.clone()), generics.clone().unwrap_or(Vec::new()), Some(&expression.node)))
      },

      Block(ref statements) => {
        let flag_backup = self.flag.clone();

        if self.flag.is_none() {
          self.flag = Some(FlagContext::Block(None))
        }

        let block_type = if statements.len() > 0 {
          for element in statements {

            match element.node {
              StatementNode::Expression(ref expression) => match expression.node {
                Block(_) | If(..) => { self.type_expression(expression)?; },

                _ => (),
              },

              StatementNode::Return(ref return_type) => {
                let flag = self.flag.clone();

                if let Some(ref flag) = flag {
                  if let &FlagContext::Block(ref consistent) = flag {

                    let return_type = if let Some(ref return_type) = *return_type {
                      self.type_expression(&return_type)?
                    } else {
                      Type::from(TypeNode::Nil)
                    };

                    if let Some(ref consistent) = *consistent {
                      if return_type != *consistent {
                        return Err(
                          response!(
                            Wrong(format!("mismatched types, expected `{}` found `{}`", consistent, return_type)),
                            self.source.file,
                            expression.pos
                          )
                        )
                      }
                    } else {
                      self.flag = Some(FlagContext::Block(Some(return_type.clone())))
                    }
                  }
                }
              },

              _ => (),
            }
          }

          self.visit_expression(&expression)?;

          self.tabs.push(self.tab_frames.last().unwrap().clone());

          let last          = statements.last().unwrap();
          let implicit_type = self.type_statement(last)?;

          self.tabs.pop();

          if let Some(flag) = self.flag.clone() {
            if let FlagContext::Block(ref consistent) = flag {
              if let Some(ref consistent) = *consistent {
                if implicit_type != *consistent {
                  return Err(
                    response!(
                      Wrong(format!("mismatched types, expected `{}` found `{}`", consistent, implicit_type)),
                      self.source.file,
                      last.pos
                    )
                  )
                }
              } else {
                self.flag = Some(FlagContext::Block(Some(implicit_type.clone())))
              }
            }
          }

          implicit_type

        } else {
          Type::from(TypeNode::Nil)
        };

        self.flag = flag_backup;

        block_type
      },

      _ => Type::from(TypeNode::Nil)
    };

    Ok(t)
  }



  pub fn current_tab(&mut self) -> &mut (SymTab, TypeTab<'v>) {
    let len = self.tabs.len() - 1;

    &mut self.tabs[len]
  }



  pub fn push_scope(&mut self) {
    let local_symtab  = SymTab::new(Rc::new(self.current_tab().0.clone()), &[]);
    let local_typetab = TypeTab::new(Rc::new(self.current_tab().1.clone()), &[], HashMap::new());

    self.tabs.push((local_symtab.clone(), local_typetab.clone()));
  }

  pub fn pop_scope(&mut self) {
    self.tab_frames.push(self.tabs.pop().unwrap());
  }
}
use std::fmt::{ self, Display, Write, Formatter };
use std::rc::Rc;
use std::collections::HashMap;

use super::super::error::Response::Wrong;

use super::*;
use super::TokenElement;

use std::fs;
use std::fs::File;

use std::io::prelude::*;
use std::path::Path;

use std::env;



#[derive(Debug, Clone)]
pub enum TypeNode {
  Int,
  Float,
  Bool,
  Str,
  Char,
  Nil,
  Id(String, Vec<Type>),
  Array(Rc<Type>, usize),
  Func(Vec<Type>, Rc<Type>, Vec<String>, Option<Rc<ExpressionNode>>),
  Module(HashMap<String, Type>),
  Struct(String, HashMap<String, Type>, Vec<String>),
}

impl TypeNode {
  pub fn check_expression(&self, other: &ExpressionNode) -> bool {
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



impl PartialEq for TypeNode {
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
      (&Id(ref a, ref b),                    &Id(ref c, ref d))                    => a == c && b == d,
      (&Func(ref a_params, ref a_retty, ..), &Func(ref b_params, ref b_retty, ..)) => a_params == b_params && a_retty == b_retty,

      (&Struct(_, ref content, ref generics), &Struct(_, ref content_b, ref generics_b)) => {
        let mut false_0 = true;
        let mut false_1 = true;

        for (ref name, ref element) in content.iter() {
          if let Some(ref element_b) = content_b.get(*name) {
            if element != element_b {

              if let TypeNode::Id(ref name, _) = element.node {
                if !generics.contains(name) && !generics_b.contains(name) {
                  false_0 = false
                }
              }
            }
          }
        }

        for (ref name_b, ref element_b) in content_b.iter() {
          if let Some(ref element) = content.get(*name_b) {
            if element != element_b {

              if let TypeNode::Id(ref name_b, _) = element.node {
                if !generics.contains(name_b) && !generics_b.contains(name_b) {
                  false_1 = false
                }
              }
            }
          }
        }

        false_0 || false_1
      },      

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

impl TypeMode {
  pub fn check(&self, other: &TypeMode) -> bool {
    use self::TypeMode::*;

    match (self, other) {
      (&Regular,    &Regular)     => true,
      (&Immutable,  &Immutable)   => true,
      (&Optional,    &Optional)   => true,
      (&Undeclared, &Undeclared)  => true,
      (&Splat(a),      &Splat(b)) => &a == &b,
      (&Unwrap(_),  &Unwrap(_))   => true,
      _                           => false,
    }
  }
}

impl Display for TypeNode {
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

      Id(ref n, ref generics) => {
        write!(f, "{}", n)?;

        if generics.len() > 0 {
          write!(f, "<{}>", generics.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join(", "))
        } else {
          Ok(())
        }
      },

      Module(_) => write!(f, "module"),

      Struct(ref name, _, ref generics) => {
        write!(f, "{}", name)?;

        if generics.len() > 0 {
          write!(f, "<{}>", generics.join(", "))
        } else {
          Ok(())
        }
      },

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
pub struct Type {
  pub node: TypeNode,
  pub mode: TypeMode,
}

impl Type {
  pub fn new(node: TypeNode, mode: TypeMode) -> Self {
    Self {
      node, mode,
    }
  }

  pub fn id(id: &str, generics: Vec<Type>) -> Self {
    Type::new(TypeNode::Id(id.to_owned(), generics), TypeMode::Regular)
  }

  pub fn from(node: TypeNode) -> Type {
    Type::new(node, TypeMode::Regular)
  }

  pub fn array(t: Type, len: usize) -> Type {
    Type::new(TypeNode::Array(Rc::new(t), len), TypeMode::Regular)
  }

  pub fn function(params: Vec<Type>, return_type: Type) -> Self {
    Type::new(TypeNode::Func(params, Rc::new(return_type), Vec::new(), None), TypeMode::Regular)
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}{}", self.mode, self.node)
  }
}



#[derive(Debug, Clone)]
pub enum FlagContext {
  Block(Option<Type>),
  Nothing,
}

#[derive(Debug, Clone)]
pub enum Inside {
  Loop,
  Nothing,
}


pub struct Visitor<'v> {
  pub tabs:       Vec<(SymTab, TypeTab)>,
  pub tab_frames: Vec<(SymTab, TypeTab)>,

  pub source:  &'v Source,
  pub ast:     &'v Vec<Statement>,

  pub flag:   Option<FlagContext>,
  pub inside: Option<Inside>,
}

impl<'v> Visitor<'v> {
  pub fn new(source: &'v Source, ast: &'v Vec<Statement>) -> Self {
    Visitor {
      tabs:       vec!((SymTab::global(), TypeTab::global())),
      tab_frames: Vec::new(), // very intelligent hack

      source,
      ast,

      flag:   None,
      inside: None,
    }
  }

  pub fn visit(&mut self) -> Result<(), ()> {
    for statement in self.ast {
      self.visit_statement(&statement)?
    }

    self.tab_frames.push(self.tabs.last().unwrap().clone());

    Ok(())
  }

  pub fn visit_statement(&mut self, statement: &Statement) -> Result<(), ()> {
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

      Return(ref value) => if let Some(ref expression) = *value {
        self.visit_expression(expression)
      } else {
        Ok(())
      },

      Break => if let Some(Inside::Loop) = self.inside {
        Ok(())
      } else {
        return Err(
          response!(
            Wrong("can't break outside loop"),
            self.source.file,
            statement.pos
          )
        )
      },

      Skip => if let Some(Inside::Loop) = self.inside {
        Ok(())
      } else {
        return Err(
          response!(
            Wrong("can't skip outside loop"),
            self.source.file,
            statement.pos
          )
        )
      },

      Import(ref path, ref specifics) => {
        let my_folder  = Path::new(&self.source.file.0).parent().unwrap();
        let file_path  = format!("{}/{}.wu", my_folder.to_str().unwrap(), path);

        let mut module = Path::new(&file_path);

        let init_path = format!("{}/{}/init.wu", my_folder.to_str().unwrap(), path);

        let module = if !module.exists() {
          let module = Path::new(&init_path);

          if !module.exists() {
            return Err(
              response!(
                Wrong(format!("no such module `{0}`, need `{0}.wu` or `{0}/init.wu`", path)),
                self.source.file,
                statement.pos
              )
            )
          }

          module
        } else {
          module
        };


        let display = module.display();

        let mut file = match File::open(&module) {
          Err(why) => panic!("failed to open {}: {}", display, why),
          Ok(file) => file,
        };

        let mut content = String::new();

        match file.read_to_string(&mut content) {
          Err(why) => panic!("failed to read {}: {}", display, why),
          Ok(_)    => {
            let source = Source::new(module.to_str().unwrap().to_string());
            let lexer = Lexer::default(content.chars().collect(), &source);

            let mut tokens = Vec::new();

            for token_result in lexer {
              if let Ok(token) = token_result {
                tokens.push(token)
              } else {
                panic!()
              }
            }

            let parsed = Parser::new(tokens, self.source).parse()?;

            let mut visitor = Visitor::new(self.source, &parsed);

            visitor.visit()?;

            let mut content_type = HashMap::new();

            let frame = visitor.tab_frames.last().unwrap();

            let names = frame.0.names.clone();

            for symbol in names.borrow().iter() {
              content_type.insert(symbol.0.clone(), frame.1.get_type(*symbol.1, 0)?.clone());
            }

            for name in specifics {
              if let Some(kind) = content_type.get(name) {
                let index = if let Some((index, _)) = self.current_tab().0.get_name(name) {
                  index
                } else {
                  self.current_tab().1.grow();
                  self.current_tab().0.add_name(name)
                };

                self.current_tab().1.set_type(index, 0, kind.clone())?;
              } else {
                return Err(
                  response!(
                    Wrong(format!("no such member `{}`", name)),
                    self.source.file,
                    statement.pos
                  )
                )
              }
            }

            let module_type = Type::from(TypeNode::Module(content_type));

            let index = if let Some((index, _)) = self.current_tab().0.get_name(path) {
              index
            } else {
              self.current_tab().1.grow();
              self.current_tab().0.add_name(path)
            };

            self.current_tab().1.set_type(index, 0, module_type)?;
          }
        }

        Ok(())
      },
    }
  }

  fn ensure_no_implicit(&self, expression: &Expression) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Block(ref statements) => if let Some(statement) = statements.last() {
        if let StatementNode::Expression(ref expression) = statement.node {
          match expression.node {

            Call(..)   => (),
            Block(..)  => { self.ensure_no_implicit(expression)?; }

            If(_, ref expr, _) | While(_, ref expr) => self.ensure_no_implicit(&*expr)?,

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

      If(_, ref expr, _) | While(_, ref expr) => self.ensure_no_implicit(&*expr)?,

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

  fn visit_expression(&mut self, expression: &Expression) -> Result<(), ()> {
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

      While(ref condition, ref body) => {
        self.visit_expression(&*condition)?;

        let condition_type = self.type_expression(&*condition)?.node;

        if condition_type == TypeNode::Bool {
          let inside_backup = self.inside.clone();

          self.inside = Some(Inside::Loop);

          self.push_scope();

          self.visit_expression(body)?;

          let body_type = self.type_expression(body)?;

          if body_type.node != TypeNode::Nil {
            let body_pos = if let Block(ref content) = body.node {
              content.last().unwrap().pos.clone()
            } else {
              unreachable!()
            };
            
            return Err(
              response!(
                Wrong("mismatched types, expected `()`"),
                self.source.file,
                body_pos
              )
            )
          }

          self.pop_scope();

          self.inside = inside_backup;

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
      }

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

          let mut type_buffer: Option<Type> = None; // for unwraps

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

            let mut param_new = param.clone();

            if let TypeNode::Id(ref name, _) = param.node {
              if generics.contains(name) {
                if let Some(kind) = covers.get(name) {
                  if &arg_type == kind {
                    continue
                  } else {
                    return Err(
                      response!(
                        Wrong(format!("mismatched argument, expected `{}` got `{}`", kind, arg_type)),
                        self.source.file,
                        args[index].pos
                      )
                    )
                  }
                }

                covers.insert(name.clone(), arg_type.clone());

                continue
              } else {
                if let TypeNode::Struct(ref struct_name, ref struct_args, _) = arg_type.node {
                  if name == struct_name {
                    if let Some((type_index, env_index)) = self.current_tab().0.get_name(name) {
                      let kind = self.current_tab().1.get_type(type_index, env_index)?;

                      if let TypeNode::Struct(_, ref other_args, ref generics) = kind.node {

                        let mut cover_type = Vec::new();

                        for arg in struct_args.iter() {
                          if let TypeNode::Id(ref name, _) = other_args[arg.0].node {
                            if generics.contains(name) {
                              if let Some(ref kind) = covers.get(name) {
                                if arg.1 != *kind {
                                  return Err(
                                    response!(
                                      Wrong(format!("mismatched argument, expected `{}` got `{}`", kind, arg.1)),
                                      self.source.file,
                                      args[index].pos
                                    )
                                  )
                                }
                              }

                              if covers.get(name).is_none() {
                                covers.insert(name.clone(), arg.1.clone());
                              }

                              cover_type.push(arg.1.clone())
                            }
                          }
                        }

                        param_new = self.degeneralize_struct(name, &cover_type, &expression.pos)?;
                      }
                    }
                  }
                }
              }
            } else {
                
              if let TypeNode::Func(ref params, ..) = arg_type.node {
                if let TypeNode::Func(ref params_expr, ref return_type_expr, ref generics_expr, ref reference) = expression_type {

                  let mut new_params = Vec::new(); 
                  let mut new_retty  = return_type_expr.clone();

                  for (i, other_param) in params_expr.iter().enumerate() {
                    if let TypeNode::Id(ref name, _) = other_param.node {

                      if let Some(cover_kind) = covers.get(name) {
                        new_params.push(cover_kind.clone());

                        if let TypeNode::Id(ref name_ret, ref covers) = return_type_expr.node {
                          if covers.len() == 0 {
                            if name == name_ret {
                              new_retty = Rc::new(cover_kind.clone())
                            }
                          }
                        }
                      } else {
                        new_params.push(params[i - 1].clone());

                        if let TypeNode::Id(ref name_ret, ref covers) = return_type_expr.node {
                          if covers.len() == 0 {
                            if name == name_ret {
                              new_retty = Rc::new(params[i - 1].clone())
                            }
                          }
                        }
                      }

                      if covers.get(name).is_none() {
                        covers.insert(name.clone(), params[i - 1].clone());
                      }
                    }
                  }

                  // the none generic version ;))
                  param_new = Type::from(
                    TypeNode::Func(new_params, new_retty, generics_expr.clone(), reference.clone())
                  )
                }
              }

            }
            
            if (index < args.len() && !param_new.node.check_expression(&args[index].node)) && param_new != arg_type {
              return Err(
                response!(
                  Wrong(format!("mismatched argument, expected `{}` got `{}`", param_new, arg_type)),
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

                if let TypeNode::Id(ref name, _) = last.node {
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
            if let Function(ref params, ref return_type, ref body, ref generics) = *func.clone().unwrap() {
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
        
        match left_type.node {
         TypeNode::Array(_, ref len) => {
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
          },

          TypeNode::Module(ref content) => {
            if let Identifier(ref name) = index.node {
              if !content.contains_key(name) {
                return Err(
                  response!(
                    Wrong(format!("no such module member `{}`", name)),
                    self.source.file,
                    index.pos
                  )
                )
              }
            } else {
              return Err(
                response!(
                  Wrong(format!("module access must be done with an identifier")),
                  self.source.file,
                  index.pos
                )
              )
            }
          },

          _ => return Err(
            response!(
              Wrong(format!("can't index `{}`", left_type)),
              self.source.file,
              left.pos
            )
          )
        }

        Ok(())
      },

      Struct(_, ref params, ref generics) => {
        let mut generics_buffer = Vec::new();
        let mut name_buffer     = Vec::new();

        for generic in generics.iter() {
          if generics_buffer.contains(&generic) {
            return Err(
              response!(
                Wrong(format!("generic `{}` defined more than once", generic)),
                self.source.file,
                expression.pos
              )
            )
          }

          generics_buffer.push(&generic)
        }

        for &(ref name, _) in params.iter() {
          if name_buffer.contains(&name) {
            return Err(
              response!(
                Wrong(format!("field `{}` defined more than once", name)),
                self.source.file,
                expression.pos
              )
            )
          }

          name_buffer.push(&name)
        }

        Ok(())
      },

      Initialization(ref left, ref args) => {
        let struct_type = self.type_expression(&*left)?;

        if let TypeNode::Struct(_, ref content, ref generics) = struct_type.node {
          if struct_type.mode.check(&TypeMode::Undeclared) {

            let mut covers = HashMap::new();

            for arg in args.iter() {
              self.visit_expression(&arg.1)?;

              let arg_type = self.type_expression(&arg.1)?;
              
              if let Some(ref content_type) = content.get(&arg.0) {
                if !content_type.node.check_expression(&Parser::fold_expression(&arg.1)?.node) && arg_type != **content_type {
                  if let TypeNode::Id(ref name, _) = content_type.node {
                    if generics.contains(name) {
                      if let Some(kind) = covers.get(name) {
                        if arg_type != *kind {
                          return Err(
                            response!(
                              Wrong(format!("mismatched types, expected `{}` got `{}`", kind, arg_type)),
                              self.source.file,
                              arg.1.pos
                            )
                          )
                        } else {
                          continue
                        }
                      }

                      covers.insert(name, arg_type);

                      continue
                    }
                  }

                  return Err(
                    response!(
                      Wrong(format!("mismatched types, expected `{}` got `{}`", content_type, arg_type)),
                      self.source.file,
                      expression.pos
                    )
                  )
                }


              } else {
                return Err(
                  response!(
                    Wrong(format!("no such member `{}`", arg.0)),
                    self.source.file,
                    arg.1.pos
                  )
                )
              }
            }

          } else {
            return Err(
              response!(
                Wrong(format!("can't initialize non-struct: `{}`", struct_type.node)),
                self.source.file,
                expression.pos
              )
            )
          }
        } else {
          return Err(
            response!(
              Wrong(format!("can't initialize non-struct: `{}`", struct_type.node)),
              self.source.file,
              expression.pos
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
      pos: TokenElement,
      params: &Vec<(String, Type)>, return_type: &Type,
      body: &Rc<Expression>, generics: &Vec<String>, generic_covers: Option<HashMap<String, Type>>,
      splat_len: usize
  ) -> Result<(), ()> {
    let mut generics_buffer = Vec::new();
    let mut name_buffer     = Vec::new();

    for generic in generics.iter() {
      if generics_buffer.contains(&generic) {
        return Err(
          response!(
            Wrong(format!("can't shadow generic `{}`", generic)),
            self.source.file,
            pos.clone()
          )
        )
      }

      generics_buffer.push(&generic)
    }

    for &(ref name, _) in params.iter() {
      if name_buffer.contains(&name) {
        return Err(
          response!(
            Wrong(format!("can't shadow field `{}`", name)),
            self.source.file,
            pos.clone()
          )
        )
      }

      name_buffer.push(&name)
    }

    let mut param_names = Vec::new();
    let mut param_types = Vec::new();

    let mut return_type = return_type;

    for param in params {
      param_names.push(param.0.clone());

      let kind = {
        if let TypeNode::Id(ref name, _) = return_type.node {
          if generics.contains(name) {
            if let Some(ref covers) = generic_covers {
              return_type = covers.get(name).unwrap()
            }
          }
        }

        if let TypeNode::Id(ref name, ref args) = param.1.node {
          if generics.contains(name) {
            if let Some(ref covers) = generic_covers {
              Type::new(covers.get(name).unwrap().clone().node, param.1.mode.clone())
            } else {
              param.1.clone()
            }
          } else {
            let mut covers_new = Vec::new();

            for arg in args {
              if let TypeNode::Id(ref name, _) = arg.node {
                if generics.contains(name) {
                  if let Some(ref covers) = generic_covers {
                    covers_new.push(Type::new(covers.get(name).unwrap().clone().node, arg.mode.clone()))
                  } else {
                    covers_new.push(arg.clone())
                  }
                }
              }
            }

            self.degeneralize_struct(&name, &covers_new, &pos)?
          }
        } else {
          param.1.clone()
        }
      };

      param_types.push(kind);
    }

    let last_type = param_types.last().unwrap().clone();

    if let TypeMode::Splat(_) = last_type.mode {
      let len = param_types.len();

      param_types[len - 1] = Type::new(last_type.node, TypeMode::Splat(Some(splat_len)))
    }

    if generics.len() > 0 && generic_covers.is_none() {
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



  fn visit_variable(&mut self, variable: &StatementNode) -> Result<(), ()> {
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
          Function(..) | Block(_) | If(..) | While(..) => (),
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
          Function(..) | Block(_) | If(..) | While(..) => self.visit_expression(right)?,
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



  pub fn type_statement(&mut self, statement: &Statement) -> Result<Type, ()> {
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



  pub fn type_expression(&mut self, expression: &Expression) -> Result<Type, ()> {
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

      Initialization(ref left, ref args) => {
        let mut content_type = HashMap::new();

        if let TypeNode::Struct(name, content, generics) = self.type_expression(left)?.node {
          let mut generics_type = Vec::new();

          for arg in args {
            let arg_type = self.type_expression(&arg.1)?;

            if let TypeNode::Id(ref name, _) = content[&arg.0].node {
              if generics.contains(name) {
                generics_type.push(format!("{}", arg_type))
              }
            }

            content_type.insert(arg.0.clone(), arg_type);
          }

          generics_type.dedup();

          Type::from(TypeNode::Struct(name, content_type, generics_type))
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

      Index(ref array, ref index) => match self.type_expression(array)?.node {
        TypeNode::Array(ref t, _) => (**t).clone(),
        TypeNode::Module(ref content) => {
          if let Identifier(ref name) = index.node {
            content.get(name).unwrap().clone()
          } else {
            unreachable!()
          }
        },

        _ => unreachable!(),
      },

      If(_, ref expression, _) | While(_, ref expression) => self.type_expression(expression)?,

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

        Type::from(TypeNode::Func(param_types, Rc::new(return_type.clone()), generics.clone(), Some(Rc::new(expression.node.clone()))))
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
                Block(_) | If(..) | While(..) => { self.type_expression(expression)?; },

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

      Struct(ref name, ref params, ref generics) => {
        let mut param_hash = HashMap::new();

        for param in params {
          param_hash.insert(param.0.clone(), param.1.clone());
        }

        Type::new(TypeNode::Struct(name.to_owned(), param_hash, generics.clone()), TypeMode::Undeclared)
      } 

      _ => Type::from(TypeNode::Nil)
    };

    Ok(t)
  }



  pub fn current_tab(&mut self) -> &mut (SymTab, TypeTab) {
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



  fn degeneralize_struct(&mut self, name: &String, args: &Vec<Type>, pos: &TokenElement) -> Result<Type, ()> {
    if let Some((index, env_index)) = self.current_tab().0.get_name(name) {
      let kind = self.current_tab().1.get_type(index, env_index)?;

      let result = if let TypeNode::Struct(ref struct_name, ref content, ref generics) = kind.node {
        let mut content_type  = HashMap::new();
        let mut generics_type = Vec::new();

        for (ref member_name, ref element) in content.iter() {
          if let TypeNode::Id(ref name, _) = element.node {
            if let Some(index) = generics.iter().position(|ref r| *r == name) {

              if let Some(kind) = args.get(index) {
                generics_type.push(format!("{}", kind)); // the worstest of hacks, like this method :)))
                content_type.insert((*member_name).clone(), kind.clone());
              } else {
                return Err(
                  response!(
                    Wrong(format!("missing generic type `{}` on `{}`", name, struct_name)),
                    self.source.file,
                    pos.clone()
                  )
                )
              }

            } else {
              content_type.insert((*member_name).clone(), (*element).clone());
            }
          } else {
            content_type.insert((*member_name).clone(), (*element).clone());
          }
        }

        generics_type.dedup();

        Type::from(TypeNode::Struct(struct_name.clone(), content_type, generics_type.clone()))
      } else {
        kind
      };

      Ok(result)

    } else {
      return Err(
        response!(
          Wrong(format!("no such type `{}` in this scope", name)),
          self.source.file,
          pos.clone()
        )
      )
    }
  }
}
use super::*;

use std::path::Path;
use std::collections::HashMap;



#[derive(Clone, PartialEq)]
pub enum FlagImplicit {
  Return,
  Global,
  Assign(String),
}

#[derive(Clone, PartialEq)]
pub enum Inside {
  Loop,
  Nothing,
}



pub struct Generator<'g> {
  source: &'g Source,

  flag: Option<FlagImplicit>,
  inside: Option<Inside>,

  loop_depth: usize,

  method_calls: &'g HashMap<Pos, bool>
}

impl<'g> Generator<'g> {
  pub fn new(source: &'g Source, method_calls: &'g HashMap<Pos, bool>) -> Self {
    Generator {
      source,

      flag: None,
      inside: None,

      loop_depth: 0,
    
      method_calls,
    }
  }



  fn get_names(statements: &Vec<Statement>) -> Vec<String> {
    use self::StatementNode::*;

    let mut names = Vec::new();

    for statement in statements {
      match statement.node {
        Variable(_, ref name, ..)     => names.push(name.to_owned()),
        Import(ref name, ref imports) => {
          if imports.len() == 0 {
            names.push(name.to_owned())
          } else {
            names.append(&mut imports.to_owned())
          }
        },
        _ => (),
      }
    }

    names
  }



  pub fn generate(&mut self, ast: &'g Vec<Statement>) -> String {
    let mut result = "return (function()\n".to_string();
    let mut output = String::new();

    for statement in ast.iter() {
      output.push_str(&self.generate_statement(&statement));
      output.push('\n')
    }

    self.push_line(&mut result, &output);

    result.push_str("  return {\n");

    let mut assignments = String::new();

    for name in Self::get_names(ast) {
      assignments.push_str(&format!("  {0} = {0},\n", Self::make_valid(&name)))
    }

    self.push_line(&mut result, &assignments);

    result.push_str("  }");

    result.push_str("\nend)()");

    result
  }



  fn generate_statement(&mut self, statement: &Statement) -> String {
    use self::StatementNode::*;

    let result = match statement.node {
      Expression(ref expression)       => self.generate_expression(expression),
      Variable(_, ref left, ref right) => self.generate_local(left, right),
      Assignment(ref left, ref right)  => self.generate_assignment(left, right),

      Return(ref expr)  => if let Some(ref expr) = *expr {
        use self::ExpressionNode::*;

        let flag_backup = self.flag.clone();

        self.flag = Some(FlagImplicit::Return);

        let line = match expr.node {
          Block(..) | If(..) | While(..) => self.generate_expression(expr),
          _                              => format!("return {}", self.generate_expression(expr)),
        };

        self.flag = flag_backup;

        line
      } else {
        "return\n".to_string()
      },

      Import(ref name, ref specifics) => {
        let my_folder  = Path::new(&self.source.file.0).parent().unwrap();
        let file_path  = format!("{}/{}", my_folder.to_str().unwrap(), name);


        let mut result = format!("local {} = require('{}')\n", name, file_path);

        for specific in specifics {
          result.push_str(&format!("{0} = {1}['{0}']\n", specific, name))
        }

        result.push('\n');

        result
      },

      Break => String::from("break"),
      Skip  => String::from("do break end"),

      Implement(ref name, ref body, _) => if let ExpressionNode::Block(ref content) = body.node {
        let assign = self.generate_expression(name);

        let flag_backup = self.flag.clone();

        let mut result = String::new();

        for element in content {
          if let Variable(_, ref name, ref right) = element.node {
            if let ExpressionNode::Extern(_, ref lua) = right.clone().unwrap().node {
              if let Some(ref lua) = lua {
                let assign = format!("{}['{}']", assign, Self::make_valid(name));

                result.push_str(&format!("{} = {}\n\n", assign, lua))
              }
            } else {
              let assign = format!("{}['{}']", assign, Self::make_valid(name));

              self.flag = Some(FlagImplicit::Assign(assign.clone()));

              let right = self.generate_expression(&right.clone().unwrap());

              result.push_str(&format!("{} = {}\n\n", assign, right))
            }
          }
        }

        self.flag = flag_backup;

        result
      } else {
        unreachable!()
      },
    };

    result
  }



  fn generate_expression(&mut self, expression: &Expression) -> String {
    use self::ExpressionNode::*;
    use std::string;

    let result = match expression.node {
      Binary(ref left, ref op, ref right) => {
        let mut result = string::String::new();

        result.push_str(
          &format!(
            "({} {} {})",
            self.generate_expression(&left),
            self.generate_operator(&op),
            self.generate_expression(&right),
          )
        );

        result
      },

      Call(ref called, ref args) => {
        let flag_backup = self.flag.clone();

        self.flag      = Some(FlagImplicit::Assign("none".to_string()));
        let mut caller = self.generate_expression(called);
        let mut result = format!("{}(", caller);

        let prefix = self.method_calls.get(&called.pos).is_some();

        if let Index(ref left, ..) = called.node {
          caller = self.generate_expression(left)
        }

        if prefix {
          result.push_str(&caller);

          if args.len() > 0 {
            result.push_str(", ")
          }
        }

        for (i, arg) in args.iter().enumerate() {
          result.push_str(&self.generate_expression(arg));

          if i < args.len() - 1 {
            result.push_str(", ")
          }
        }

        result.push(')');

        self.flag = flag_backup;
        
        result
      },

      Module(ref content) => {
        if let Block(ref elements) = content.node {
          let mut result = "(function()\n".to_string();

          let mut body = String::new();

          for element in elements {
            body.push_str(&self.generate_statement(&element))
          }

          body.push_str("\nreturn {\n");

          let mut assignments = String::new();

          for name in Self::get_names(elements) {
            assignments.push_str(&format!("{0} = {0},\n", name))
          }

          self.push_line(&mut body, &assignments);

          body.push('}');

          self.push_line(&mut result, &body);

          result.push_str("end)()");

          result
        } else {
          unreachable!()
        }
      },

      Block(ref content) => {
        let flag_backup = self.flag.clone();

        let flag = self.flag.clone();        

        let mut result = if let Some(ref f) = flag {
          match *f {
            FlagImplicit::Assign(_) => {
              self.flag = Some(FlagImplicit::Return);

              "(function()\n"
            },

            FlagImplicit::Return => "",

            _ => "do\n",
          }
        } else  {
          "do\n"
        }.to_string();

        for (i, element) in content.iter().enumerate() {          
          if i == content.len() - 1 {
            if self.flag.is_some() {
              if let StatementNode::Expression(ref expression) = element.node {
                match expression.node {
                  Block(_) => (),
                  _ => match &self.flag.clone().unwrap() {
                    &FlagImplicit::Return => {
                      let line = match expression.node {
                        Block(..) | If(..) | While(..) => self.generate_expression(expression),
                        _                              => format!("return {}", self.generate_expression(expression)),
                      };

                      result.push_str(&self.make_line(&line));

                      break
                    },

                    _ => ()
                  },
                }
              }
            }
          }

          let line = self.generate_statement(&element);
          result.push_str(&self.make_line(&line));
        }

        self.flag = flag_backup;

        if let Some(ref f) = flag {
          match *f {
            FlagImplicit::Assign(_) => {
              self.flag = Some(FlagImplicit::Return);

              result.push_str("end)()")
            },

            FlagImplicit::Return => (),

            _ => result.push_str("end"),
          }
        } else {
          result.push_str("end")
        }

        result
      },

      Function(ref params, _, ref body, is_method) => {
        let mut result = "function(".to_string();

        if is_method {
          result.push_str("self");

          if params.len() > 0 {
            result.push_str(", ")
          }
        }

        let mut splat = None;

        for (i, param) in params.iter().enumerate() {

          if let TypeMode::Splat(_) = param.1.mode {
            result.push_str("...");

            splat = Some(param.0.clone())
          } else {
            result.push_str(&param.0);
          }

          if i < params.len() - 1 {
            result.push_str(", ")
          }
        }

        result.push_str(")\n");

        if let Some(ref name) = splat {
          result.push_str(&format!("  local {} = {{...}}\n", name))
        }

        let flag_backup = self.flag.clone();

        self.flag = Some(FlagImplicit::Return);

        let line = match body.node {
          Block(..) | If(..) | While(..) => self.generate_expression(body),
          _                              => format!("return {}", self.generate_expression(body)),
        };

        result.push_str(&&line);

        self.flag = flag_backup;

        result.push_str("end");

        result
      },

      Array(ref content) => {        
        let mut result = "({\n".to_string();

        for (i, arg) in content.iter().enumerate() {
          let value    = self.generate_expression(arg);
          let mut line = format!("[{}] = {}", i, value);

          if i < content.len() - 1 {
            line.push(',')
          }

          result.push_str(&self.make_line(&line));
        }

        result.push_str("})");

        result
      },

      Index(ref source, ref index, is_braces) => {
        let source = self.generate_expression(source);

        let index = if let Identifier(ref name) = index.node {
          if is_braces {
            format!("{}", Self::make_valid(name))
          } else {
            format!("'{}'", Self::make_valid(name))
          }
        } else {
          self.generate_expression(index)
        };

        format!("{}[{}]", source, index)
      },

      If(ref condition, ref body, ref elses) => {
        let flag_backup = self.flag.clone();

        let mut result = if let Some(FlagImplicit::Assign(_)) = self.flag {
          self.flag = Some(FlagImplicit::Return);

          "(function()\n"
        } else {
          ""
        }.to_string();

        result.push_str(&format!("if {} then\n", self.generate_expression(condition)));

        let mut body_string = String::new(); // doing this to remove redundant 'do' and 'end'

        if let Block(ref content) = body.node {
          for (i, element) in content.iter().enumerate() {          
            if i == content.len() - 1 {
              if self.flag.is_some() {
                if let StatementNode::Expression(ref expression) = element.node {
                  match expression.node {
                    Block(_) | If(..) | While(..) => (),
                    _ => match &self.flag.clone().unwrap() {
                      &FlagImplicit::Return => {
                        let line = match body.node {
                          Block(..) | If(..) | While(..) => self.generate_expression(body),
                          _                              => format!("return {}", self.generate_expression(body)),
                        };

                        result.push_str(&self.make_line(&line));

                        break
                      },

                      _ => ()
                    },
                  }
                }
              }
            }

            let line = self.generate_statement(&element);
            result.push_str(&self.make_line(&line));
          }
        }

        result.push_str(&self.make_line(&body_string));

        if let &Some(ref elses) = elses {
          for branch in elses {

            if let Some(ref condition) = branch.0 {
              result.push_str(&format!("elseif {} then\n", self.generate_expression(condition)));
            } else {
              result.push_str("else\n")
            }

            body_string = String::new();

            if let Block(ref content) = branch.1.node {
              for (i, element) in content.iter().enumerate() {          
                if i == content.len() - 1 {
                  if self.flag.is_some() {
                    if let StatementNode::Expression(ref expression) = element.node {
                      match expression.node {
                        Block(_) | If(..) | While(..) => (),
                        _ => match &self.flag.clone().unwrap() {
                          &FlagImplicit::Return => {
                            let line = match body.node {
                              Block(..) | If(..) | While(..) => self.generate_expression(&branch.1),
                              _                              => format!("return {}", self.generate_expression(&branch.1)),
                            };

                            result.push_str(&self.make_line(&line));

                            break
                          },

                          _ => ()
                        },
                      }
                    }
                  }
                }

                let line = self.generate_statement(&element);
                result.push_str(&self.make_line(&line));
              }
            }

            result.push_str(&self.make_line(&body_string));
          }
        }

        self.flag = flag_backup;

        if let Some(FlagImplicit::Assign(_)) = self.flag {
          result.push_str("end\nend)()")
        } else {
          result.push_str("end")
        }

        result
      },

      While(ref condition, ref body) => {
        let flag_backup   = self.flag.clone();
        let inside_backup = self.inside.clone();

        if self.inside == Some(Inside::Loop) {
          self.loop_depth += 1
        } else {
          self.loop_depth = 0;
          self.inside     = Some(Inside::Loop)
        }

        let mut result = if let Some(FlagImplicit::Assign(_)) = self.flag {
          self.flag = Some(FlagImplicit::Return);

          "(function()\n"
        } else {
          ""
        }.to_string();

        if self.flag == Some(FlagImplicit::Return) {
          self.flag = None
        }

        let condition = self.generate_expression(condition);

        let mut whole = format!("while {} do\n", condition);

        let mut body_string = "repeat\n".to_string(); // doing this to remove redundant 'do' and 'end'

        if let Block(ref content) = body.node {
          for (i, element) in content.iter().enumerate() {
            if i == content.len() - 1 {              
              if StatementNode::Skip == element.node {
                break
              } else {
                if let StatementNode::Expression(ref expression) = element.node {
                  if Empty == expression.node {
                    break
                  }
                }
              }
            }

            body_string.push_str(&self.generate_statement(&element));
            body_string.push('\n')
          }
        }

        // body_string.push_str(&format!("::__while_{}::\n", self.loop_depth));
        body_string.push_str("until false\n");

        self.push_line(&mut whole, &body_string);

        whole.push_str("end");

        if let Some(FlagImplicit::Assign(_)) = flag_backup {
          self.push_line(&mut result, &whole)
        } else {
          result.push_str(&whole)
        }

        self.flag   = flag_backup;
        self.inside = inside_backup;

        if let Some(FlagImplicit::Assign(_)) = self.flag {
          result.push_str("end)()")
        }

        result
      },

      Initialization(ref name, ref body) => {
        let mut inner = String::new();

        for &(ref name, ref expression) in body.iter() {
          inner.push_str(&format!("{} = {},\n", name, self.generate_expression(expression)))
        }

        format!("setmetatable({{\n{}}}, {{__index={}}})", self.make_line(&inner), self.generate_expression(name))
      },

      Extern(_, ref lua) => if let &Some(ref lua) = lua {
        lua.to_owned()
      } else {
        String::new()
      },

      Int(ref n)        => format!("{}", n),
      Float(ref n)      => format!("{}", n),
      Bool(ref n)       => format!("{}", n),
      Str(ref n)        => format!("\"{}\"", n),
      Char(ref n)       => format!("\"{}\"", n),
      Identifier(ref n) => Self::make_valid(n),

      Cast(ref a, ref t) => {
        use self::TypeNode::*;

        let result = match t.node {
          Float => "tonumber(",
          Str   => "tostring(",
          Int   => "math.floor(tonumber(",
          _     => "(",
        };

        format!("{}{}){}", result, self.generate_expression(a), if t.node == Int { ")" } else { "" })
      }

      UnwrapSplat(ref expression) => format!("table.unpack({})", self.generate_expression(expression)),
      Unwrap(ref expression)      => self.generate_expression(expression),
      Neg(ref n)                  => format!("-{}", self.generate_expression(n)),
      Not(ref n)                  => format!("not {}", self.generate_expression(n)),

      Empty => String::from("nil"),
      _     => String::new()
    };

    result
  }



  fn make_valid(n: &String) -> String {
    let mut result = String::new();

    for a in n.chars() {
      let new_a = match a {
        '?' => "__question_mark__".to_string(),
        '!' => "__exclamation_mark__".to_string(),
        a   => a.to_string(),
      };

      result.push_str(&new_a)
    }

    result
  }



  fn generate_local(&mut self, name: &str, right: &Option<Expression>) -> String {
    let flag_backup = self.flag.clone();

    let name = Self::make_valid(&name.to_string());

    let mut result = {
      let output = if self.flag == Some(FlagImplicit::Global) {
        name.to_owned()
      } else {
        format!("local {}", name)
      };

      self.flag = Some(FlagImplicit::Assign(name.to_string()));

      output
    };

    if let &Some(ref right) = right {
      if let ExpressionNode::Function(..) = right.node {
        result = self.generate_expression(right);
        result = result.replace("function", &format!("function {}", name));
      } else {
        let right_str = match right.node {
          ExpressionNode::Struct(..)                          => "{}".to_string(),
          ExpressionNode::Extern(_, ref lua) if lua.is_none() => return String::new(),
          ExpressionNode::Trait(..)                           => return String::new(),

          _ => self.generate_expression(right)
        };

        result.push_str(&format!(" = {}\n", right_str))
      }
    }

    self.flag = flag_backup;

    format!("{}", result)
  }



  fn generate_assignment<'b>(&mut self, left: &'b Expression, right: &'b Expression) -> String {
    let left_string  = self.generate_expression(left);

    let flag_backup = self.flag.clone();

    self.flag = Some(FlagImplicit::Assign(left_string.clone()));
    
    let right_string = self.generate_expression(right);

    self.flag = flag_backup;

    let result = format!("{} = {}", left_string, right_string);

    result
  }



  fn generate_operator<'b>(&mut self, op: &'b Operator) -> String {
    use self::Operator::*;

    match *op {
      Concat => "..".to_string(),
      NEq    => "~=".to_string(),
      _ => format!("{}", op)
    }
  }



  fn make_line(&mut self, value: &str) -> String {
    let mut output = String::new();

    for line in value.lines() {
      output.push_str("  ");

      output.push_str(&line);
      output.push('\n')
    }

    output
  }

  fn push_line(&mut self, target: &mut String, value: &str) {
    target.push_str(&self.make_line(&value))
  }
}
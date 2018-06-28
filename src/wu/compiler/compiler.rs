use super::*;



#[derive(Debug, Clone)]
pub enum FlagImplicit {
  Return,
  Assign(String)
}



pub struct Generator {
  flag: Option<FlagImplicit>
}

impl<'g> Generator {
  pub fn new() -> Self {
    Generator {
      flag: None,
    }
  }



  pub fn generate(&mut self, ast: &'g Vec<Statement>) -> String {
    let mut output = String::new();

    for statement in ast.iter() {
      output.push_str(&self.generate_statement(&statement));
      output.push('\n')
    }

    output
  }



  fn generate_statement<'b>(&mut self, statement: &'b Statement<'b>) -> String {
    use self::StatementNode::*;

    let result = match statement.node {
      Expression(ref expression)       => self.generate_expression(expression),
      Variable(_, ref left, ref right) => self.generate_local(left, right),
      Assignment(ref left, ref right)  => self.generate_assignment(left, right),

      Return(ref expr)  => if let Some(ref expr) = *expr {
        format!("return {}\n", self.generate_expression(expr))
      } else {
        "return\n".to_string()
      }, 
    };

    result
  }



  fn generate_expression<'b>(&mut self, expression: &'b Expression<'b>) -> String {
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

        self.flag = Some(FlagImplicit::Assign("none".to_string()));

        let mut result = format!("{}(", self.generate_expression(called));

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

      Block(ref content) => {
        let flag_backup = self.flag.clone();

        let flag = self.flag.clone();

        let mut result = if let Some(ref f) = flag {
          match *f {
            FlagImplicit::Assign(_) => {
              self.flag = Some(FlagImplicit::Return);

              "(function()\n"
            },

            FlagImplicit::Return => ""
          }
        } else  {
          "do\n"
        }.to_string();

        for (i, element) in content.iter().enumerate() {
          if i == content.len() - 1 {
            if self.flag.is_some() {
              if let StatementNode::Expression(ref expression) = element.node {
                match expression.node {
                  Block(_) | If(..) => (),
                  _ => match &self.flag.clone().unwrap() {
                    &FlagImplicit::Return => {
                      let line = format!("return {}\n", self.generate_expression(expression));

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

            FlagImplicit::Return => ()
          }
        } else {
          result.push_str("end\n")
        }

        result
      },

      Function(ref params, _, ref body, _) => {
        let mut result = format!("function(");

        for (i, param) in params.iter().enumerate() {
          result.push_str(&param.0);

          if i < params.len() - 1 {
            result.push_str(", ")
          }
        }

        result.push_str(")\n");

        let flag_backup = self.flag.clone();

        self.flag = Some(FlagImplicit::Return);

        let line = if let Block(..) = body.node {
          self.generate_expression(body)
        } else {
          format!("return {}", self.generate_expression(body))
        };

        result.push_str(&self.make_line(&line));

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

      Index(ref source, ref index) => {
        let source = self.generate_expression(source);
        let index  = self.generate_expression(index);

        format!("{}[{}]", source, index)
      }

      If(ref condition, ref body, ref elses) => {
        let flag_backup = self.flag.clone();

        let mut result = if let Some(FlagImplicit::Assign(_)) = self.flag {
          self.flag = Some(FlagImplicit::Return);

          "(function()\n"
        } else {
          ""
        }.to_string();

        result.push_str(&format!("if {} then\n", self.generate_expression(condition)));

        let body = self.generate_expression(body);

        result.push_str(&self.make_line(&body));

        if let &Some(ref elses) = elses {
          for branch in elses {

            if let Some(ref condition) = branch.0 {
              result.push_str(&format!("elseif {} then\n", self.generate_expression(condition)));
            } else {
              result.push_str("else\n")
            }

            let body = self.generate_expression(&branch.1);

            result.push_str(&self.make_line(&body));
          }
        }

        self.flag = flag_backup;

        if let Some(FlagImplicit::Assign(_)) = self.flag {
          result.push_str("end\nend)()\n")
        } else {
          result.push_str("end\n")
        }

        result
      },

      Int(ref n)        => format!("{}", n),
      Float(ref n)      => format!("{}", n),
      Bool(ref n)       => format!("{}", n),
      Str(ref n)        => format!("\"{}\"", n),
      Char(ref n)       => format!("\"{}\"", n),
      Identifier(ref n) => format!("{}", n),
      
      _ => String::new()
    };

    result
  }



  fn generate_local<'b>(&mut self, name: &str, right: &'b Option<Expression<'b>>) -> String {
    use self::ExpressionNode::*;
    use std::string;

    let flag_backup = self.flag.clone();

    let mut result = {
      let output = format!("local {}", name);

      self.flag = Some(FlagImplicit::Assign(name.to_string()));

      output
    };

    if let &Some(ref right) = right {
      let right_str = self.generate_expression(right);

      result.push_str(&format!(" = {}", right_str))
    }

    self.flag = flag_backup;

    format!("{}\n", result)
  }



  fn generate_assignment<'b>(&mut self, left: &'b Expression, right: &'b Expression) -> String {
    let left_string  = self.generate_expression(left);

    let flag_backup = self.flag.clone();

    self.flag = Some(FlagImplicit::Assign(left_string.clone()));
    
    let right_string = self.generate_expression(right);

    self.flag = flag_backup;

    let result = format!("{} = {}\n", left_string, right_string);

    result
  }



  fn generate_operator<'b>(&mut self, op: &'b Operator) -> String {
    use self::Operator::*;

    match *op {
      Concat => "..".to_string(),
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
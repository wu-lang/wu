use super::*;



#[derive(Debug, Clone)]
pub enum FlagImplicit {
  Return,
  Assign(String)
}



pub struct Generator<'g> {
  pub visitor: &'g mut Visitor<'g>,

  flag: Option<FlagImplicit>
}

impl<'g> Generator<'g> {
  pub fn new(visitor: &'g mut Visitor<'g>) -> Self {
    Generator {
      visitor,
      flag: None,
    }
  }



  pub fn generate(&mut self, ast: &'g Vec<Statement>) -> Result<String, ()> {
    let mut output = String::new();

    for statement in ast.iter() {
      output.push_str(&self.generate_statement(&statement)?)
    }

    Ok(output)
  }




  fn generate_statement<'b>(&mut self, statement: &'b Statement<'b>) -> Result<String, ()> {
    use self::StatementNode::*;

    let result = match statement.node {
      Expression(ref expression) => self.generate_expression(expression)?,

      Variable(_, ref left, ref right) => {
        if let ExpressionNode::Set(_) = left.node {
          unimplemented!()
        } else {
          self.generate_local(left, right)?
        }
      },

      Constant(_, ref left, ref right) => {
        if let ExpressionNode::Set(_) = left.node {
          unimplemented!()
        } else {
          self.generate_local(left, &Some(right.clone()))?
        }
      },

      Assignment(ref left, ref right) => self.generate_assignment(left, right)?,

      Break             => "break\n".to_string(),
      Skip              => "continue\n".to_string(),
      Return(ref expr)  => if let Some(ref expr) = *expr {
        format!("return {}\n", self.generate_expression(expr)?)
      } else {
        "return\n".to_string()
      },
    };

    Ok(result)
  }



  fn generate_expression<'b>(&mut self, expression: &'b Expression<'b>) -> Result<String, ()> {
    use self::ExpressionNode::*;
    use std::string;

    let result = match expression.node {
      Binary(ref left, ref op, ref right) => {
        let mut result = string::String::new();

        result.push_str(
          &format!(
            "({} {} {})",
            self.generate_expression(&left)?,
            self.generate_operator(&op),
            self.generate_expression(&right)?,
          )
        );

        result
      },

      Call(ref called, ref args) => {
        let mut result = format!("{}(", self.generate_expression(called)?);

        for (i, arg) in args.iter().enumerate() {
          result.push_str(&self.generate_expression(arg)?);

          if i < args.len() - 1 {
            result.push(',')
          }
        }

        result.push(')');

        result
      },

      Block(ref content) => {
        let mut result = "do\n".to_string();

        for (i, element) in content.iter().enumerate() {
          if i == content.len() - 1 {
            if self.flag.is_some() {
              if let StatementNode::Expression(ref expression) = element.node {
                match expression.node {
                  Block(_) | If(..) => (),
                  _ => match &self.flag.clone().unwrap() {
                    &FlagImplicit::Return => {
                      let line = format!("return {}\n", self.generate_expression(expression)?);

                      result.push_str(&self.make_line(&line));

                      break
                    },

                    &FlagImplicit::Assign(ref name) => {
                      let line = format!("{} = {}\n", name, self.generate_expression(expression)?);

                      result.push_str(&self.make_line(&line));
                      break
                    },
                    _ => ()
                  },
                }
              }
            }
          }

          let line = self.generate_statement(&element)?;
          result.push_str(&self.make_line(&line));
        }

        result.push_str("end\n");

        result
      },

      Function(ref params, _, ref body) => {
        let mut result = format!("function(");

        for param in params {
          match param.node {
            StatementNode::Variable(_, ref left, _) => if let ExpressionNode::Identifier(ref name) = left.node {
              result.push_str(name)
            } else {
              unimplemented!()
            },

            _ => unimplemented!()
          }
        }

        result.push_str(")\n");

        self.flag = Some(FlagImplicit::Return);

        let line = self.generate_expression(body)?;

        result.push_str(&self.make_line(&line));

        self.flag = None;

        result.push_str("end");

        result
      },

      Array(ref content) => {        
        let mut result = "({\n".to_string();

        for (i, arg) in content.iter().enumerate() {
          let value    = self.generate_expression(arg)?;
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
        let source = self.generate_expression(source)?;
        let index  = self.generate_expression(index)?;

        format!("{}[{}]", source, index)
      }

      If(ref condition, ref body, ref elses) => {
        let mut result = format!("if {} then\n", self.generate_expression(condition)?);

        let body = self.generate_expression(body)?;

        result.push_str(&self.make_line(&body));

        if let &Some(ref elses) = elses {
          for branch in elses {

            if let Some(ref condition) = branch.0 {
              result.push_str(&format!("elseif {} then\n", self.generate_expression(condition)?));

            } else {
              result.push_str("else\n")
            }

            let body = self.generate_expression(&branch.1)?;

            result.push_str(&self.make_line(&body));
          }
        }

        result.push_str("end");

        result
      },

      Loop(ref body) => {
        let mut result = format!("while true\n");

        result.push_str(&self.generate_expression(&body)?);

        result
      },

      While(ref condition, ref body) => {
        let mut result = format!("while {}\n", self.generate_expression(&condition)?);

        result.push_str(&self.generate_expression(&body)?);

        result
      },

      Int(ref n)        => format!("{}", n),
      Float(ref n)      => format!("{}", n),
      Bool(ref n)       => format!("{}", n),
      String(ref n)     => format!("\"{}\"", n),
      Char(ref n)       => format!("\"{}\"", n),
      Identifier(ref n) => format!("{}", n),
      _                 => string::String::new()
    };

    Ok(result)
  }



  fn generate_operator<'b>(&mut self, op: &'b Operator) -> String {
    use self::Operator::*;

    match *op {
      Concat => "..".to_string(),
      _ => format!("{}", op)
    }
  }



  fn generate_local<'b>(&mut self, left: &'b Expression<'b>, right: &'b Option<Expression<'b>>) -> Result<String, ()> {
    use self::ExpressionNode::*;
    use std::string;

    let mut result = if let Identifier(ref name) = left.node {
      let output = format!("local {}", name);

      self.flag = Some(FlagImplicit::Assign(name.clone()));

      output
    } else {
      string::String::new()
    };

    if let &Some(ref right) = right {
      let right_str = self.generate_expression(right)?;

      match right.node {
        Block(_) | If(..) | While(..) | Loop(..) => result.push_str(&format!("\n{}", right_str)),
        _ => result.push_str(&format!(" = {}", right_str)),
      }
    }

    self.flag = None;

    Ok(format!("{}\n", result))
  }



  fn generate_assignment<'b>(&mut self, left: &'b Expression, right: &'b Expression) -> Result<String, ()> {
    use self::ExpressionNode::*;

    let left_string  = self.generate_expression(left)?;

    self.flag = Some(FlagImplicit::Assign(left_string.clone()));
    
    let right_string = self.generate_expression(right)?;

    self.flag = None;

    let result = match right.node {
      If(..) | While(..) | Loop(..) | Block(_) => format!("{}", right_string),
      _                                        => format!("{} = {}\n", left_string, right_string)
    };

    Ok(result)
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
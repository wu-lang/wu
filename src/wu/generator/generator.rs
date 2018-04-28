use super::*;



pub struct Generator<'g> {
  pub visitor: &'g mut Visitor<'g>,
}

impl<'g> Generator<'g> {
  pub fn new(visitor: &'g mut Visitor<'g>) -> Self {
    Generator {
      visitor,
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

      Block(ref content) => {
        let mut result = "do\n".to_string();

        for element in content {
          let line = self.generate_statement(&element)?;
          result.push_str(&self.make_line(&line))
        }

        result.push_str("end\n");

        result
      },

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

        result.push_str("end\n");

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



  fn generate_local<'b>(&mut self, left: &'b Expression, right: &'b Option<Expression>) -> Result<String, ()> {
    use self::ExpressionNode::*;
    use std::string;

    let mut result = if let Identifier(ref name) = left.node {
      let left  = self.generate_expression(left)?;
      let output = format!("local {}", left);

      output
    } else {
      string::String::new()
    };

    if let &Some(ref right) = right {
      let right_str = self.generate_expression(right)?;
      result.push_str(&format!(" = {}", right_str));
    }

    Ok(format!("{}\n", result))
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
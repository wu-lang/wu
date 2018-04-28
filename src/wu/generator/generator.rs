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




  fn generate_statement(&mut self, statement: &Statement<'g>) -> Result<String, ()> {
    use self::StatementNode::*;

    let result = match statement.node {
      Expression(ref expression) => self.generate_expression(expression)?,
      _ => String::new()
    };

    Ok(result)
  }



  fn generate_expression(&mut self, expression: &Expression<'g>) -> Result<String, ()> {
    use self::ExpressionNode::*;
    use std::string;

    let result = match expression.node {
      Binary(ref left, ref op, ref right) => {
        let mut result = string::String::new();

        result.push_str(
          &format!(
            "({} {} {})",
            self.generate_expression(&left)?,
            op,
            self.generate_expression(&right)?,
          )
        );

        self.make_line(&result)
      },

      Block(ref content) => {
        let mut result = "do\n".to_string();

        for element in content {
          let line = self.generate_statement(&element)?;
          result.push_str(&self.make_line(&line))
        }

        result.push_str("end");

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
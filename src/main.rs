extern crate colored;

mod wu;

use self::wu::source::*;
use self::wu::lexer::*;
use self::wu::parser::*;
use self::wu::visitor::*;

fn main() {
  let test = r#"
A: struct {}
B: struct {
  a: A
}


x := new B {
  a: new A {}
}
  "#;

  let source = Source::from("<main>", test.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(test.chars().collect(), &source);

  let mut tokens = Vec::new();

  for token_result in lexer {
    if let Ok(token) = token_result {
      tokens.push(token)
    } else {
      return
    }
  }

  let mut parser = Parser::new(tokens, &source);

  match parser.parse() {
    Ok(ref ast) => {
      let mut visitor = Visitor::new(&ast, &source);

      visitor.visit();
    },

    _ => ()
  }
}
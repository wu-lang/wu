extern crate colored;

mod wu;
use wu::source::*;
use wu::lexer::*;
use wu::parser::Parser;
use wu::visitor::Visitor;

fn main() {
  let mut content = r#"(a, b, c)"#;

  let source = Source::from("main.rs/testing.wu", content.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(content.chars().collect(), &source);

  let tokens     = lexer.collect::<Vec<Token>>();
  let tokens_ref = tokens.iter().map(|x| &*x).collect::<Vec<&Token>>();

  let mut parser  = Parser::new(tokens_ref, &source);
  let mut visitor = Visitor::new(&source);

  match parser.parse() {
    Ok(ast) => {
      println!("{:#?}", ast);

      for statement in ast {
        visitor.visit_statement(&statement);
      }
    },

    Err(_) => (),
  }
}
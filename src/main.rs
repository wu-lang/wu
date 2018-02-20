extern crate colored;

mod wu;
use wu::source::*;
use wu::lexer::*;
use wu::parser::Parser;
use wu::visitor::Visitor;

fn main() {
  let mut content = r#"
foobadoo       
  "#;

  let source = Source::from("main.rs/testing.wu", content.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(content.chars().collect(), &source);

  let tokens = lexer.collect::<Vec<Token>>();

  let mut parser  = Parser::new(tokens, &source);
  let mut visitor = Visitor::new(&source);

  for statement in parser.parse().unwrap() {
    visitor.visit_statement(&statement);
  }
}
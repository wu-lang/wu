extern crate colored;

mod wu;
use wu::source::*;
use wu::lexer::*;
use wu::parser::Parser;

fn main() {
  let mut content = r#"

"#;

  let source = Source::from("main.rs/testing.wu", content.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(content.chars().collect(), &source);

  let tokens = lexer.collect::<Vec<Token>>();

  let mut parser = Parser::new(tokens, &source);

  println!("{:#?}", parser.parse());
}
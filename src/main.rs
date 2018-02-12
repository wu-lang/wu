extern crate colored;

mod wu;
use wu::source::*;
use wu::lexer::Lexer;

fn main() {
  let mut content = r#"
jumping?
jump!

jump_height

_123!?!?
"#;

  let source = Source::from("main.rs/testing", content.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(content.chars().collect(), &source);

  for token in lexer {
    println!("{:?}  ({})", token.lexeme, token.token_type)
  }
}
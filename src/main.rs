#![feature(i128)]
#![feature(i128_type)]

#![feature(u128)]
#![feature(u128_type)]

extern crate colored;

mod wu;
use wu::source::*;
use wu::lexer::*;
use wu::parser::{ Parser, ExpressionNode, Expression, };
use wu::visitor::Visitor;
use wu::generator::Generator;

use std::env;

fn run(content: &str) {
  let source = Source::from("main.rs/testing.wu", content.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(content.chars().collect(), &source);

  let mut tokens = Vec::new();

  for token_result in lexer {
    if let Ok(token) = token_result {
      tokens.push(token)
    } else {
      return
    }
  }

  let tokens_ref = tokens.iter().map(|x| &*x).collect::<Vec<&Token>>();

  let mut parser  = Parser::new(tokens_ref, &source);

  match parser.parse() {
    Ok(ast) => {
      println!("{:#?}", ast);      

      let mut visitor = Visitor::new(&source, &ast);      
 
      match visitor.visit() {
        Ok(_) => {
          let mut generator = Generator::new(&mut visitor);

          println!("------\n{}", generator.generate(&ast).unwrap())
        },
        _ => ()
      }
    },

    _ => (),
  }
}

fn main() {
  let test0 = r"
fac :: (a: int, b: int) string -> a + b

a := fac(1, 2) as float
b := fac(3, 4) as float
c := fac(5, 6) as float
  ";

  let test1 = r"
a: [int] = [10, 20, 30, 40]

b := 2 as float

æ := a[b]
ø := a[b]
å := a[b]
z := a[b]
  ";

  let test2 = r"
a: [[int]] = [[1, 2], [3, 4]]
  ";

  let test3 = r"
{
  1

  {
    2

    {
      3
    }
  }
}
  ";

  run(test3)
}
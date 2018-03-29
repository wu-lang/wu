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
use wu::interpreter::*;

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
          let mut compiler = Compiler::new(&mut visitor);

          match compiler.compile(&ast) {
            Ok(_) => {
              let mut vm = VirtualMachine::new();

              vm.execute(compiler.bytecode.as_slice());

              println!();

              println!("stack: {:?}", &vm.compute_stack[..64]);
              println!();
              println!("vars:  {:?}", &vm.var_stack[..256]);
            },

            _ => (),
          }
        }
        _ => ()
      }
    },

    _ => (),
  }
}

fn main() {
  let test0 = r"
fac :: (a: i32, b: i32) i32 -> a + b

a := fac(1, 2) as i8
b := fac(3, 4) as i8
c := fac(5, 6) as i8
  ";

  let test1 = r"
a: [i8; 3] = [ 1, 2, 3 ]
  ";

  run(&test1);
}
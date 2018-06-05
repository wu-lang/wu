#![feature(i128)]
#![feature(i128_type)]

#![feature(u128)]
#![feature(u128_type)]

extern crate colored;
extern crate rustyline;

mod wu;
use wu::source::*;
use wu::lexer::*;
use wu::parser::{ Parser, ExpressionNode, Expression, };
use wu::visitor::Visitor;
use wu::generator::Generator;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use rustyline::error::ReadlineError;



const PROMPT:        &'static str = ">> ";
const PROMPT_INDENT: &'static str = " | ";



fn repl() {
  let mut repl = rustyline::Editor::<()>::new();

  let mut is_indented = false;

  let mut program = String::new();

  loop {
    let line = repl.readline(if is_indented { PROMPT_INDENT } else { PROMPT });

    match line {
      Ok(content) => {
        if content.len() == 0 {
          continue
        }

        is_indented = content.chars().last().unwrap() == '\\';

        if is_indented {
          program.push_str(&content[.. content.len() - 1]);
          program.push('\n')
        } else {
          program.push_str(&content);

          println!();

          run(&program, "<repl>");

          program.push('\n');
        }
      }

      Err(ReadlineError::Interrupted) => {
        println!("<Interrupted>");
        break
      }

      Err(ReadlineError::Eof) => {
        println!("<EOF>");
        break
      }

      Err(err) => {
        println!("<Error>: {:?}", err);
        break
      }
    }
  }
}



fn run(content: &str, file: &str) -> Option<String> {
  let source = Source::from(file, content.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(content.chars().collect(), &source);

  let mut tokens = Vec::new();

  for token_result in lexer {
    if let Ok(token) = token_result {
      tokens.push(token)
    } else {
      return None
    }
  }

  let tokens_ref = tokens.iter().map(|x| &*x).collect::<Vec<&Token>>();

  let mut parser  = Parser::new(tokens_ref, &source);

  match parser.parse() {
    Ok(ast) => {
      let mut visitor = Visitor::new(&source, &ast);

      match visitor.visit() {
        Ok(_) => {
          let mut generator = Generator::new(&mut visitor);

          generator.generate(&ast).ok()
        },
        _ => None
      }
    },

    _ => None,
  }
}

fn test() {
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

  let test3 = r#"
a := 1
b: int = 2

if a == 0 {
  c: string
  d := "hey"
  foo :: 100
} elif b == 2 {
  baz := 1 + 1 * 2 ^ 4
} else {
  bar: int: 0
}
  "#;

  let test4 = r#"
fib :: (i: int) int -> {
  if i < 3 {
    i
  } else {
    fib(i - 1) + fib(i - 2)
  }
}

fib(
  if true {
    10
  } else {
    5
  }
)
  "#;

  let test5 = r#"
i := 0
i = while i < 10 {
  i + 1
}
  "#;

  let test6 = r#"
-- blocks are fancy

{
  something := 100
}

a := 0
a = loop {
  a + 1
}
  "#;
}

fn transpile_file(path: &Path) {
  let mut content = String::new();
  File::open(path).expect("File not found")
                  .read_to_string(&mut content)
                  .expect("Couldn't read the file");

  if let Some(code) = run(&content, path.to_str().unwrap()) {
    let lua_path = path.with_file_name({
      let mut name = path.file_stem()
                         .unwrap_or_else(|| path.file_name().unwrap())
                         .to_os_string();
      name.push(".lua");
      name
    });

    let mut lua_file = File::create(lua_path).expect("Can't create file");
    lua_file.write_all(code.as_bytes()).expect("Can't write to file")
  }
  println!("Transpiled \"{}\"", path.to_string_lossy());
}

fn main() {
  let args = env::args().collect::<Vec<String>>();

  if args.len() > 1 {
    for path in args[1..].iter() {
      let path = Path::new(path);
      if path.exists() {
        if path.is_file() {
          //File
          transpile_file(&path)
        } else if path.is_dir() {
          //Directory
          for file in path.read_dir().unwrap() {
            if let Ok(file) = file {
              if file.path().to_str().map(|s| s.ends_with(".wu")).unwrap_or(false) && file.path().is_file() {
                transpile_file(&file.path())
              }
            }
          }
        }
      } else {
        panic!("File doesn't exist")
      }
    }
  } else {
    repl()
  }
}

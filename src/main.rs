extern crate colored;
extern crate rustyline;

use colored::Colorize;

mod wu;
use wu::source::*;
use wu::lexer::*;
use wu::parser::*;
use wu::visitor::*;
use wu::compiler::*;


use std::fs;
use std::fs::File;
use std::fs::metadata;

use std::env;
use std::path::Path;

use std::io::prelude::*;

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



fn compile_path(path: &str) {
  let meta = match metadata(path) {
    Ok(m)    => m,
    Err(why) => panic!("{}", why),
  };

  if meta.is_file() {
    let split: Vec<&str> = path.split('.').collect();
    let path_lua = format!("{}.lua", split[0 .. split.len() - 1].to_vec().join("."));

    if !Path::new(&path_lua).is_file() {
      if let Some(n) = file_content(path) {
        write(path, &n);
      }
    }
  } else {
    let paths = fs::read_dir(path).unwrap();

    for path in paths {
      let path = format!("{}", path.unwrap().path().display());
      let split: Vec<&str> = path.split('.').collect();

      if Path::new(&path).is_dir() {
        compile_path(&format!("{}", path))
      }

      match split.last() {
        Some(n) if *n == "wu" => {
          let path = format!("{}.lua", split[0 .. split.len() - 1].to_vec().join("."));

          if Path::new(&path).is_file() {
            // miss me with that compiling twice shit
            continue
          }
        },
        _ => continue,
      }

      compile_path(&format!("{}", path))
    }
  }
}



fn file_content(path: &str) -> Option<String> {
  let display = Path::new(path).display();

  let mut file = match File::open(&path) {
      Err(why) => panic!("failed to open {}: {}", display, why),
      Ok(file) => file,
  };

  let mut s = String::new();

  match file.read_to_string(&mut s) {
    Err(why) => panic!("failed to read {}: {}", display, why),
    Ok(_)    => run(&s, path),
  }
}



fn write(path: &str, data: &str) {
  let path = Path::new(path);

  println!("{} {}", "compiled".green().bold(), path.display().to_string().replace("./", ""));

  let split_name = path.file_name().unwrap().to_str().unwrap().split('.');
  let split: Vec<&str> = split_name.collect();

  let path_split = path.to_str().unwrap().split('/').collect::<Vec<&str>>();

  let path_real  = if path_split.len() > 1 {
    format!("{}/{}.lua", path_split[0 .. path_split.len() - 1].join("/"), split[0])
  } else {
    format!("{}.lua", split[0])
  };

  let mut output_file = File::create(&path_real).unwrap();
  match output_file.write_all(data.as_bytes()) {
    Ok(_)    => (),
    Err(why) => println!("{}", why)
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
          let mut generator = Generator::new();

          Some(generator.generate(&ast))
        },

        _ => None
      }
    },

    _ => None,
  }
}



fn clean_path(path: &str) {
  let meta = match metadata(path) {
    Ok(m) => m,
    Err(why) => panic!("{}", why),
  };

  if meta.is_dir() {
    let paths = fs::read_dir(path).unwrap();

    for path in paths {
      let path = path.unwrap().path();
      if path.is_dir() {
        clean_path(&path.display().to_string())
      } else {
        let path = format!("{}", path.display());
        let split: Vec<&str> = path.split('.').collect();

        // removes lua file if wu source exists
        match split.last() {
          Some(n) if *n == "wu" => {
            let path = format!("{}.lua", split[0 .. split.len() - 1].to_vec().join("."));

            if Path::new(&path).is_file() {
              match fs::remove_file(&path) {
                Ok(_) => println!("{} {}", "removed".red().bold(), path.replace("./", "")),
                Err(why) => panic!("{}", why)
              }
            }
          },
          _ => continue,
        }
      }
    }
  } else {
    let split: Vec<&str> = path.split('.').collect();

    let path = format!("{}.lua", split[0 .. split.len() - 1].to_vec().join("."));

    if Path::new(&path).is_file() {
      match fs::remove_file(&path) {
        Ok(_)    => println!("{} {}", "removed".red().bold(), path.replace("./", "")),
        Err(why) => panic!("{}", why)
      }
    }
  }
}



fn main() {
  let args = env::args().collect::<Vec<String>>();

  if args.len() > 1 {
    if args[1] == "clean" {
      if args.len() > 2 {
        clean_path(&args[2])
      }
    } else {
      compile_path(&args[1])    
    }    
  } else {
    repl()
  }
}

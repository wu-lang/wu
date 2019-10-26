extern crate colored;

use self::colored::Colorize;

mod wu;
mod eval;

use eval::Evaluator;

use self::wu::compiler::*;
use self::wu::lexer::*;
use self::wu::parser::*;
use self::wu::source::*;
use self::wu::visitor::*;

use std::fs;
use std::fs::metadata;
use std::fs::File;

use std::env;

use std::io::prelude::*;
use std::path::Path;

const HELP: &'static str = "\
The Wu Compiler
- made by Niels

Usage:
  wu                    # Show this message

  -- Compilation
  wu compile <file>     # Compile .wu file to corresponding .lua file
  wu compile <folder>   # Compile all .wu files in given folder
  wu clean <folder>     # Removes all compiled .lua files in given folder

  -- Evaluation
  wu eval <file>        # Evaluate .wu file
";

// TODO: maybe make some sort of struct such that
// all of these functions are methods on that struct,
// and then the Action is stored as a field in that struct,
// so that it doesn't have to get passed all the way down
// the function calls when only `run` actually needs to use it.
#[derive(Copy, Clone)]
pub enum Action {
    Compile,
    Interpret,
}

fn process_path(path: &str, action: Action) {
    let meta = match metadata(path) {
        Ok(m) => m,
        Err(why) => panic!("{}", why),
    };

    if meta.is_file() {
        let split: Vec<&str> = path.split('.').collect();

        println!(
            "{} {}",
            "compiling".green().bold(),
            path.to_string().replace("./", "")
        );

        if *split.last().unwrap() == "wu" {
            if let Some(n) = file_content(path, action) {
                write(path, &n);
            }
        }
    } else {
        let paths = fs::read_dir(path).unwrap();

        for folder_path in paths {
            let folder_path = format!("{}", folder_path.unwrap().path().display());
            let split: Vec<&str> = folder_path.split('.').collect();

            if Path::new(&folder_path).is_dir() || *split.last().unwrap() == "wu" {
                process_path(&folder_path, action)
            }
        }
    }
}

fn file_content(path: &str, action: Action) -> Option<String> {
    let display = Path::new(path).display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("failed to open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();

    match file.read_to_string(&mut s) {
        Err(why) => panic!("failed to read {}: {}", display, why),
        Ok(_) => run(&s, path, action),
    }
}

fn write(path: &str, data: &str) {
    let path = Path::new(path);

    let split_name = path.file_name().unwrap().to_str().unwrap().split('.');
    let split: Vec<&str> = split_name.collect();

    let path_split = path.to_str().unwrap().split('/').collect::<Vec<&str>>();

    let path_real = if path_split.len() > 1 {
        format!(
            "{}/{}.lua",
            path_split[0..path_split.len() - 1].join("/"),
            split[0]
        )
    } else {
        format!("{}.lua", split[0])
    };

    let mut output_file = File::create(&path_real).unwrap();
    match output_file.write_all(data.as_bytes()) {
        Ok(_) => (),
        Err(why) => println!("{}", why),
    }
}

pub fn run(content: &str, file: &str, action: Action) -> Option<String> {
    let source = Source::from(
        file,
        content.lines().map(|x| x.into()).collect::<Vec<String>>(),
    );
    let lexer = Lexer::default(content.chars().collect(), &source);

    let mut tokens = Vec::new();

    for token_result in lexer {
        if let Ok(token) = token_result {
            tokens.push(token)
        } else {
            return None;
        }
    }

    let mut parser = Parser::new(tokens, &source);

    match parser.parse() {
        Ok(ref ast) => {
            let mut visitor = Visitor::new(ast, &source);

            match visitor.visit() {
                Ok(_) => (),
                _ => return None,
            }

            match action {
                Action::Compile => {
                    let mut generator = Generator::new(&source, &visitor.method_calls);
                    Some(generator.generate(&ast))
                },
                Action::Interpret => {
                    let mut evaluator = Evaluator::new(&source, &visitor.method_calls);
                    evaluator.eval(&ast);
                    None
                }
            }
        }

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
                        let path = format!("{}.lua", split[0..split.len() - 1].to_vec().join("."));

                        if Path::new(&path).is_file() {
                            println!("{} {}", "removing".red().bold(), path.replace("./", ""));

                            match fs::remove_file(&path) {
                                Ok(_) => (),
                                Err(why) => panic!("{}", why),
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
    } else {
        let split: Vec<&str> = path.split('.').collect();

        let path = format!("{}.lua", split[0..split.len() - 1].to_vec().join("."));

        if Path::new(&path).is_file() {
            match fs::remove_file(&path) {
                Ok(_) => println!("{} {}", "removed".red().bold(), path.replace("./", "")),
                Err(why) => panic!("{}", why),
            }
        }
    }
}

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() > 1 {
        match args[1].as_str() {
            "clean" => {
                if args.len() > 2 {
                    clean_path(&args[2])
                }
            }

            action => process_path(&args[2], match action {
                "compile" | "comp" | "c" => Action::Compile,
                "interpret" | "eval" | "evaluate" | "interp" | "i" | "e" => Action::Interpret,
                other => panic!("unknown action: {}", other)
            }),
           
        }
    } else {
        println!("{}", HELP)
    }
}

extern crate colored;
extern crate toml;
extern crate git2;
extern crate rustyline;
extern crate dirs;

use self::colored::Colorize;

mod wu;

use self::wu::compiler::*;
use self::wu::lexer::*;
use self::wu::parser::*;
use self::wu::source::*;
use self::wu::visitor::*;
use self::wu::error::*;
use self::wu::handler;

use std::fs;
use std::fs::metadata;
use std::fs::File;

use std::env;

use std::io::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};


const HELP: &'static str = "\
The Wu Compiler

Usage:
    wu                # Show this message
    wu <file>         # Compile .wu file to corresponding .lua file
    wu <folder>       # Compile all .wu files in given folder
    wu clean <folder> # Removes all compiled .lua files from given folder

Project usage:
    wu new <name>     # Create a new Wu project
    wu sync           # Installs/synchronizes dependencies
    wu build          # Installs dependencies and builds current project
";

fn compile_path(path: &str) {
    let meta = match metadata(path) {
        Ok(m) => m,
        Err(why) => panic!("{}", why),
    };

    if meta.is_file() {
        let split: Vec<&str> = path.split('.').collect();

        print!(
            "{} {}",
            "Compiling".green().bold(),
            path.to_string().replace("./", "")
        );

        if *split.last().unwrap() == "wu" {
            if let Some(n) = file_content(path) {
                write(path, &n);
            }
        }
        println!()
    } else {
        let paths = fs::read_dir(path).unwrap();

        for folder_path in paths {
            let folder_path = format!("{}", folder_path.unwrap().path().display());
            let split: Vec<&str> = folder_path.split('.').collect();

            if Path::new(&folder_path).is_dir() || *split.last().unwrap() == "wu" {
                compile_path(&folder_path)
            }
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
        Ok(_) => run(&s, path),
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

pub fn run(content: &str, file: &str) -> Option<String> {
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
            let mut symtab = SymTab::new();

            let splat_any = Type::new(TypeNode::Any, TypeMode::Splat(None));

            symtab.assign_str(
                "print",
                Type::function(vec!(splat_any.clone()), Type::from(TypeNode::Nil), false)  
            );

            symtab.assign_str(
                "ipairs",
                Type::function(vec!(splat_any.clone()), splat_any.clone(), false)
            );

            symtab.assign_str(
                "pairs",
                Type::function(vec!(splat_any.clone()), splat_any, false)
            );

            let mut visitor = Visitor::from_symtab(ast, &source, symtab);

            match visitor.visit() {
                Ok(_) => (),
                _ => return None,
            }

            let mut generator = Generator::new(&source, &visitor.method_calls);

            Some(generator.generate(&ast))
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
                            println!("{} {}", "Removing".red().bold(), path.replace("./", ""));

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

#[inline]
fn confirm_home() {
    if env::var("WU_HOME").is_err() {
        let dir = if let Some(dir) = dirs::home_dir() {
            format!("{}/.wu/libs/", dir.display())
        } else {
            return response!(
                Response::Weird(format!("missing environment variable `WU_HOME`")),
                Response::Note("failed to find home directory, you can set the variable yourself")
            )
        };

        if !Path::new(&dir).exists() {
            if fs::create_dir_all(dir).is_err() {
                response!(
                    Response::Weird(format!("missing environment variable `WU_HOME`")),
                    Response::Note("run again as super-user to fix this automatically")
                )
            }
        } else {
            env::set_var("WU_HOME", dir)
        }
    }
}

#[inline]
fn main() {
    confirm_home();

    let args = env::args().collect::<Vec<String>>();

    if args.len() > 1 {
        match args[1].as_str() {
            "clean" => {
                if args.len() > 2 {
                    clean_path(&args[2])
                }
            },

            "new" => if args.len() > 2 {
                handler::new(Some(&args[2]))
            } else {
                handler::new(None)
            },

            "build" => {
                handler::get();

                if args.len() > 2 {
                    compile_path(&args[2])
                } else {
                    compile_path(".")
                }
            },

            "sync" => handler::get(),

            file => {
                let now = Instant::now();

                compile_path(&file);

                println!(
                    "{} things in {}ms",
                    "  Finished".green().bold(),
                    now.elapsed().as_millis()
                );
            }
        }
    } else {
        println!("{}", HELP)
    }
}
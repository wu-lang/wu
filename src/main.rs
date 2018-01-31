extern crate colored;
use colored::Colorize;

mod wu;
use wu::lexer::*;
use wu::parser::*;
use wu::visitor::*;
use wu::codegen::*;

use std::fs;
use std::fs::File;
use std::fs::metadata;

use std::env;
use std::path::Path;

use std::io::prelude::*;

pub fn path_ast(path: &str) -> Option<Vec<Statement>> {
    let mut file = match File::open(&path) {
        Err(why) => panic!("failed to open {}: {}", path, why),
        Ok(file) => file,
    };

    let mut source = String::new();

    if let Err(why) = file.read_to_string(&mut source) {
        panic!("failed to read {}: {}", path, why);
    }
    
    compile_path(path);

    let lines: Vec<String> = source.lines().map(|x| x.to_string()).collect();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, path);

    let mut parser = Parser::new(lexer.collect::<Vec<Token>>(), &lines, path);

    match parser.parse() {
        Ok(ast) => Some(ast),
        Err(e)  => {
            e.display(&lines, path);
            None
        }
    }
}

fn compile_path(path: &str) {
    let meta = metadata(path).unwrap();    

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

// removes compiled lua files 
fn clean_path(path: &str) {
    let meta = metadata(path).unwrap();

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
                            fs::remove_file(&path).unwrap()
                        }
                    },
                    _ => continue,
                }
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
        Ok(_)    => compile(&s, path),
    }
}

fn write(path: &str, data: &str) {
    let path = Path::new(path);
    println!("{} {}", "compiled".green().bold(), path.display().to_string().replace("./", ""));

    let split_name = path.file_name().unwrap().to_str().unwrap().split('.');
    let split: Vec<&str> = split_name.collect();

    let path_split = path.to_str().unwrap().split('/').collect::<Vec<&str>>();
    let path_real  = &format!("{}/{}.lua", path_split[0 .. path_split.len() - 1].join("/"), split[0]);

    let mut output_file = File::create(if path_split.len() > 1 { path_real } else { path_split[0] }).unwrap();
    match output_file.write_all(data.as_bytes()) {
        Ok(_)    => (),
        Err(why) => println!("{}", why)
    }
}

fn compile(source: &str, path: &str) -> Option<String> {
    let lines: Vec<String> = source.lines().map(|x| x.to_string()).collect();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, path);

    let mut parser = Parser::new(lexer.collect::<Vec<Token>>(), &lines, path);

    match parser.parse() {
        Ok(ast)       => {
            let mut visitor = Visitor::new(&ast, &lines, path);

            match visitor.validate() {
                Ok(_)         => {
                    let mut codegen = Codegen::new(&ast, &mut visitor);

                    return Some(format!("{}", codegen.generate()))
                },

                Err(response) => response.display(&lines, path),
            }
        },

        Err(response) => response.display(&lines, path),
    }

    None
}

fn main() {
    match env::args().nth(1) {
        Some(a) => match a.as_str() {
            "clean" => if env::args().len() > 2 {
                clean_path(&env::args().nth(2).unwrap())
            } else {
                clean_path(".")
            },

            _ => {
                clean_path(&a);
                compile_path(&a)
            }
        }
        None    => println!("\
wu's transpiler

usage:
    wu <file>           -- compiles file
    wu <folder>         -- recursively compiles every `.wu` file in folder
    wu clean <folder>   -- recursively removes every compiled `.lua` file in folder
        "),
    }
}

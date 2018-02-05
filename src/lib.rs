#![feature(i128)]
#![feature(i128_type)]

extern crate colored;

mod wu;
use std::fs::File;

use std::io::prelude::*;

pub fn path_ast(path: &str) -> Option<Vec<wu::parser::Statement>> {
    use wu::lexer::*;
    use wu::parser::*;

    let mut file = match File::open(&path) {
        Err(why) => panic!("failed to open {}: {}", path, why),
        Ok(file) => file,
    };

    let mut source = String::new();

    match file.read_to_string(&mut source) {
        Err(why) => panic!("failed to read {}: {}", path, why),
        Ok(_)    => (),
    }

    let lines = source.lines().map(|x| x.to_string()).collect::<Vec<String>>();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, &path);

    let mut parser = Parser::new(lexer.collect::<Vec<Token>>(), &lines, &path);

    match parser.parse() {
        Ok(ast) => Some(ast),
        Err(e)  => {
            e.display(&lines, &path);
            None
        }
    }
}

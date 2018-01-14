extern crate colored;

mod wu;
use wu::lexer::*;
use wu::parser::*;

fn main() {
    let source = r#"
foo = .1234
bar = 100
    "#;

    let path = "test.wu";
    
    let lines = source.lines().map(|x| x.to_string()).collect();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, &path);

    let mut parser = Parser::new(lexer.collect(), &lines, &path);

    match parser.parse() {
        Ok(ast)       => println!("{:#?}", ast),
        Err(response) => response.display(&lines, &path),
    }
}

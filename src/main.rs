extern crate colored;

mod wu;
use wu::lexer::*;
use wu::parser::*;
use wu::visitor::*;

fn main() {
    let source = r#"
foo :: 1234

bar := foo + true
    "#;

    let path = "test.wu";

    let lines = source.lines().map(|x| x.to_string()).collect();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, &path);
    

    let mut parser = Parser::new(lexer.collect::<Vec<Token>>(), &lines, &path);

    match parser.parse() {
        Ok(ast)       => {
            println!("{:#?}", ast);

            let mut visitor = Visitor::new(&ast, &lines, &path);
            
            match visitor.validate() {
                Ok(_)         => (),
                Err(response) => response.display(&lines, &path),
            }
        },

        Err(response) => response.display(&lines, &path),
    }
}

extern crate colored;

mod wu;
use wu::lexer::*;
use wu::parser::*;
use wu::visitor::*;
use wu::codegen::*;

fn main() {
    let source = r#"
module test {    
    struct point {
        x: float
        y: float
    }

    module inner_test {
        struct point_inner {
            x: float
            y: float
            z: float
        }
    }
}

test_point := test inner_test point_inner {
    x: 1
    y: 2
    z: 3
}
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
                Ok(_)         => {
                    let codegen = Codegen::new(&ast);

                    println!("```lua\n{}```", codegen)
                },
                Err(response) => response.display(&lines, &path),
            }
        },

        Err(response) => response.display(&lines, &path),
    }
}

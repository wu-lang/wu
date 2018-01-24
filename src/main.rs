extern crate colored;

mod wu;
use wu::lexer::*;
use wu::parser::*;
use wu::visitor::*;
use wu::codegen::*;

fn main() {
    let source = r#"
struct point {
    x: float
    y: float
}

position :: point {
    x: 100
    y: 100
}

new_point :: (x: float, y: float) point -> point {
    x: x
    y: y
}

point_from :: (other: point) point -> point {
    x: other x
    y: other y
}

hmm := point_from(position,)
hm2 := new_point(100, 12.0)
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

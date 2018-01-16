extern crate colored;

mod wu;
use wu::lexer::*;
use wu::parser::*;
use wu::visitor::*;

fn main() {
    let source = r#"
-- functions, haha

foo: int = {
    a :: 100
    b: int = a

    return true
}

apply :: (fun (int) int, a int) int -> {
    ret :: fun(a)
    ret
}

add_ten :: (a int) int -> a + 12

bar := 100
foo := apply(add_ten, bar)
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

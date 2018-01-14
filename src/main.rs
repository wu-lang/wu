extern crate colored;

mod wu;
use wu::lexer::*;

fn main() {
    let source = r#"
foo: float = .1234
bar: int = 100
    "#;
    
    let path = "test.wu";
    
    let lines = source.lines().map(|x| x.to_string()).collect();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, &path);
    
    println!("{:#?}", lexer.collect::<Vec<Token>>());
}

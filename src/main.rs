extern crate colored;

mod wu;
use wu::source::*;

fn main() {
    let mut content = r#"
a: int
a = 100

b: string = "hello wu"
    "#;

    let source = Source::from("main.rs/testing", content.lines().map(|x| x.into()).collect::<Vec<String>>());
}
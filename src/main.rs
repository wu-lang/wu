extern crate colored;

mod wu;

use self::wu::source::*;
use self::wu::lexer::*;
use self::wu::parser::*;
use self::wu::visitor::*;
use self::wu::compiler::*;

fn main() {
  let test = r#"
GraphicsType: struct {
  rectangle: fun(str, float, float, float, float)
} 

LoveType: struct {
  graphics: GraphicsType
  load: fun
}

# hey
love: extern LoveType = "love"

love graphics rectangle("fill", 100, 100, 100, 100)

love load = fun {
  print: extern fun(...)

  print("hey", "and hey")
}

Foo: struct {
  x: int
}

foo: fun(...b: Foo) -> ...Foo {
  a := *b

  x: Foo = b[0]
  
  a
}

a := new Foo { x: 1 }
b := new Foo { x: 2 }
c := new Foo { x: 3 }

foo(a, b, c)
  "#;

  let test = r#"
import foo

fib: fun(a: int) -> int {
  if a < 3 {
    return a
  }
  
  fib(a - 1) + fib(a - 2)
}

# binding lua functions is easy
print: extern fun(...)

print_fibs: fun(numbers: ...int) {
  print(*numbers)
}
  "#;

  let source = Source::from("<main>", test.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(test.chars().collect(), &source);

  let mut tokens = Vec::new();

  for token_result in lexer {
    if let Ok(token) = token_result {
      tokens.push(token)
    } else {
      return
    }
  }

  let mut parser = Parser::new(tokens, &source);

  match parser.parse() {
    Ok(ref ast) => {
      let mut visitor = Visitor::new(&ast, &source);

      match visitor.visit() {
        Ok(_) => {
          let mut compiler = Generator::new(&source, &visitor.method_calls);

          println!("input:\n```\n{}\n```\n\noutput:\n```lua\n{}\n```", test, compiler.generate(ast))
        },

        _ => ()
      }
    },

    _ => ()
  }
}
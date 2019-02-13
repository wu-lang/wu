<img align="right" width="30%" height="30%" src="https://preview.ibb.co/ePa1eH/wu_dragon.png" alt="wu_dragon">

# Wu

[![Foo](https://user-images.githubusercontent.com/7288322/34429152-141689f8-ecb9-11e7-8003-b5a10a5fcb29.png)](https://discord.gg/qm92sPP)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/wu-lang/wu/blob/master/LICENSE)

An expression oriented, gradually typed and mission-critical programming language.

## Syntax

A full walk-through of the language can be found over at the [wu-lang documentation](https://wu-lang.gitbook.io/guide/).

### Motivation

Apart from being a strong and exquisite hipster language, Wu strives to be a decently useful, control-focused high-level language for use in game development as well as general purpose development. Its syntax is highly inspired by Rust's strong *explicit* syntax, combined with concepts from Jonathan Blow's Jai syntax and the sugar of MoonScript and the functional language family.

The language is meant and designed to be a solid alternative to MoonScript, while being superior on control and maintainability.

### Teaser

#### Structs

```
Point: struct {
  x: float
  y: float
}

implement Point {
  length: fun(self) -> float {
    (self x^2 + self y^2)^.5
  }

  normalize!: fun(self) {
    len := self length()

    self x = self x / len
    self y = self y / len
  }
}

pos := new Point {
  x: 100
  y: 100
}

pos normalize!()
```

#### Splats

```
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
```

## Roadmap

- [x] Minimum viable product
- [x] Trait system
- [x] Fix modules
- [ ] Nilable/Optional types for better Lua interop
- [ ] Binding if-let for safe Optional unwrapping
- [ ] Multiple returns for better Lua interop
- [ ] `Extern module` for easier wrapping
- [ ] Lua STD wrapper
- [ ] For-loops and ranges
- [ ] Iterator library
- [ ] Lexical macros
- [ ] A custom, super fast virtual machine

## Disclaimer

Wu is built by a minimal team of people, all of which are basically kids working on the compiler when bored in class. The whole thing is currently in very early stages. That said, it's probably fine, so go use it in production.

## Contributers

- [nilq](https://github.com/nilq)

- [fuzzylitchi](https://github.com/fuzzylitchi)

- [evolbug](https://github.com/evolbug)

### License

[MIT License](https://github.com/wu-lang/wu/blob/master/LICENSE)

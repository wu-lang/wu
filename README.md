<img align="right" width="30%" height="30%" src="https://preview.ibb.co/ePa1eH/wu_dragon.png" alt="wu_dragon">

# Wu

[![Foo](https://user-images.githubusercontent.com/7288322/34429152-141689f8-ecb9-11e7-8003-b5a10a5fcb29.png)](https://discord.gg/qm92sPP)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/wu-lang/wu/blob/master/LICENSE)

An expression oriented, strongly typed, sweet, and mission-critical programming language.

## Syntax

A full walk-through of the language can be found over at the [wu-lang documentation](https://wu-lang.gitbook.io/guide/).

### Motivation

Apart from being a strong and exquisite hipster language, Wu strives to be a decently useful, control-focused high-level language for use in game development as well as general purpose development. Its syntax is highly inspired by Rust's strong *explicit* syntax, combined with concepts from Jonathan Blow's Jai language and the sugar of MoonScript and the functional language family.

The language is meant and designed to be a solid alternative to MoonScript, and even superior on control and scalability.

### Teaser

#### Structs

```
point: type<T> {
  x: T
  y: T
}

position := point {
  x: 100, y: 100
}

copy_point: def<T>(a: point<T>) -> point<T> {
  a clone()
}
```

#### Splats

```
fib: def(a: int) -> int {
  if a < 3 {
    return a
  }
  
  fib(a - 1) + fib(a - 2)
}

print_fibs: def(..numbers: int) {
  for n in numbers {
    print(fib(n))
  }
}
```

## Disclaimer

Wu is built by a minimal team of people, all of which are basically kids working on the compiler when bored in class. The whole thing is currently in very early stages. That said, it's probably fine, so go use it in production.

## Contributers

- [nilq](https://github.com/nilq)

- [fuzzylitchi](https://github.com/fuzzylitchi)

- [evolbug](https://github.com/evolbug)

### License

[MIT License](https://github.com/wu-lang/wu/blob/master/LICENSE)

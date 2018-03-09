# Wu

[![Foo](https://user-images.githubusercontent.com/7288322/34429152-141689f8-ecb9-11e7-8003-b5a10a5fcb29.png)](https://discord.gg/qm92sPP)

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/wu-lang/wu/blob/master/LICENSE)

A good purpose, high-control, high-level language.

## Syntax

A full walk-through of the language can be found over at the [wu-lang documentation](https://wu-lang.github.io/wu.html).


### A decent language

The language is generally structured around the opinion, that the control and cool features Rust provide are too nice for a language to miss out on.
Wu is thus highly inspired by Rust, while grabbing a lot of concepts from MoonScript, Jonathan Blow's Jai and the general functional family.

### Taster

```
module functions {
  fib :: (a: i128) i128 -> match a {
    | 0 -> 0
    | 1 -> 1
    | _ -> fib(a - 1) + fib(a - 2)
  }
  
  fac :: (a: i128) i128 -> match a {
    | 0 or 1 -> a
    | _      -> fac(a - 1) * a
  }
}

foo: i128 = functions fib(1000)
bar      := functions fac(100)
```

## Disclaimer

Wu is built by a minimal team of people, all of which are basically kids working on the compiler when bored in school. The whole thing is currently in its very early stages, but is propably fine, go use it in production.

### Contributers

- [nilq](https://github.com/nilq)
- [fuzzylitchi](https://github.com/fuzzylitchi)
- [evolbug](https://githubc.om/evolbug)

### License

[MIT License](https://github.com/wu-lang/wu/blob/master/LICENSE)
